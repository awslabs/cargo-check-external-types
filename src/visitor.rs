/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::config::{AllowedTypeError, AllowedTypeMatch, Config};
use crate::error::{ErrorLocation, ValidationError, ValidationErrors};
use crate::path::{ComponentType, Path};
use crate::{bug_panic, here};
use anyhow::{anyhow, Context, Result};
use rustdoc_types::{
    Crate, FunctionSignature, GenericArgs, GenericBound, GenericParamDef, GenericParamDefKind,
    Generics, Id, Item, ItemEnum, ItemSummary, Path as RustDocPath, Struct, StructKind, Term,
    Trait, Type, Union, Variant, VariantKind, Visibility, WherePredicate,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tracing::{debug, instrument, warn};
use wildmatch::WildMatch;

macro_rules! unstable_rust_feature {
    ($name:expr, $documentation_uri:expr) => {
        panic!(
            "unstable Rust feature '{}' (see {}) is not supported by cargo-check-external-types",
            $name, $documentation_uri
        )
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum VisibilityCheck {
    /// Check to make sure the item is public before visiting it
    Default,
    /// Assume the item is public and examine it.
    /// This is useful for visiting private items that are publicly re-exported
    AssumePublic,
}

pub(crate) type Index = HashMap<Id, Item>;
pub(crate) type Paths = HashMap<Id, ItemSummary>;

/// Visits all items in the Rustdoc JSON output to discover external types in public APIs
/// and track them as validation errors if the [`Config`] doesn't allow them.
pub struct Visitor {
    /// Parsed config file from the user, or the defaults if none was provided
    config: Config,
    /// The integer ID of the crate being visited that was assigned by rustdoc
    root_crate_id: u32,
    /// Name of the crate being visited
    root_crate_name: String,
    /// Map of rustdoc [`Id`] to rustdoc [`Item`]
    index: Index,
    /// Map of rustdoc [`Id`] to rustdoc [`ItemSummary`]
    paths: Paths,

    /// Set of errors
    ///
    /// The visitor adds errors to this set while it visits each item in the rustdoc
    /// output.
    errors: RefCell<ValidationErrors>,

    /// A set of approved crate patterns.
    ///
    /// The visitor removes a pattern from this set if at least one match is found.
    /// Any remaining patterns at the end of processing are treated as unused
    /// and added to the validation errors.
    unused_approve: RefCell<HashSet<String>>,
}

impl Visitor {
    pub fn new(config: Config, package: Crate) -> Result<Self> {
        let unused_approve = RefCell::new(
            config
                .allowed_external_types
                .iter()
                .map(|glob| glob.to_string())
                .collect(),
        );
        Ok(Visitor {
            config,
            root_crate_id: Self::root_crate_id(&package)?,
            root_crate_name: Self::root_crate_name(&package)?,
            index: package.index,
            paths: package.paths,
            errors: RefCell::new(ValidationErrors::new()),
            unused_approve,
        })
    }

    /// This is the entry point for visiting the entire Rustdoc JSON tree, starting
    /// from the root module (the only module where `is_crate` is true).
    pub fn visit_all(self) -> Result<ValidationErrors> {
        let root_path = Path::new(&self.root_crate_name);
        let root_module = self
            .index
            .values()
            .filter_map(|item| {
                if let ItemEnum::Module(module) = &item.inner {
                    Some(module)
                } else {
                    None
                }
            })
            .find(|module| module.is_crate)
            .ok_or_else(|| anyhow!("failed to find crate root module"))?;

        for id in &root_module.items {
            let item = self.item(id).context(here!())?;
            self.visit_item(&root_path, item, VisibilityCheck::Default)?;
        }

        self.unused_approve
            .take()
            .into_iter()
            .for_each(|pattern| self.add_error(ValidationError::unused_approval_pattern(pattern)));

        Ok(self.errors.take())
    }

    /// Returns true if the given item is public. In some cases, this must be determined
    /// by examining the surrounding context. For example, enum variants are public if the
    /// enum is public, even if their visibility is set to `Visibility::Default`.
    fn is_public(path: &Path, item: &Item) -> bool {
        match item.visibility {
            Visibility::Public => true,
            // This code is much clearer with a match statement
            #[allow(clippy::match_like_matches_macro)]
            Visibility::Default => match (&item.inner, path.last_type()) {
                // Enum variants are public if the enum is public
                (ItemEnum::Variant(_), Some(ComponentType::Enum)) => true,
                // Struct fields inside of enum variants are public if the enum is public
                (ItemEnum::StructField(_), Some(ComponentType::EnumVariant)) => true,
                // When an `AssocType` is visited, it is for the impl of a public trait. Impls of private traits are skipped
                (ItemEnum::AssocType { .. }, Some(_)) => true,
                // Trait items are public if the trait is public
                (_, Some(ComponentType::Trait)) => true,
                _ => false,
            },
            _ => false,
        }
    }

    #[instrument(level = "debug", skip(self, path, item), fields(path = %path, name = ?item.name, id = %item.id.0))]
    fn visit_item(
        &self,
        path: &Path,
        item: &Item,
        visibility_check: VisibilityCheck,
    ) -> Result<()> {
        if visibility_check == VisibilityCheck::Default && !Self::is_public(path, item) {
            return Ok(());
        }

        let mut path = path.clone();
        match &item.inner {
            ItemEnum::AssocConst { type_, .. } => {
                path.push(ComponentType::AssocConst, item);
                self.visit_type(&path, &ErrorLocation::StructField, type_)
                    .context(here!())?;
            }
            ItemEnum::AssocType {
                bounds,
                type_,
                generics,
            } => {
                path.push(ComponentType::AssocType, item);
                if let Some(typ) = type_ {
                    self.visit_type(&path, &ErrorLocation::AssocType, typ)
                        .context(here!())?;
                }
                self.visit_generic_bounds(&path, bounds).context(here!())?;
                self.visit_generics(&path, generics).context(here!())?;
            }
            ItemEnum::Constant { type_, .. } => {
                path.push(ComponentType::Constant, item);
                self.visit_type(&path, &ErrorLocation::Constant, type_)
                    .context(here!())?;
            }
            ItemEnum::Enum(enm) => {
                path.push(ComponentType::Enum, item);
                self.visit_generics(&path, &enm.generics).context(here!())?;
                self.visit_impls(&path, &enm.impls).context(here!())?;
                for id in &enm.variants {
                    self.visit_item(
                        &path,
                        self.item(id).context(here!())?,
                        VisibilityCheck::Default,
                    )
                    .context(here!())?;
                }
            }
            ItemEnum::ExternType => unstable_rust_feature!(
                "extern_types",
                "https://doc.rust-lang.org/beta/unstable-book/language-features/extern-types.html"
            ),
            ItemEnum::Function(function) => {
                path.push(ComponentType::Function, item);
                self.visit_fn_sig(&path, &function.sig).context(here!())?;
                self.visit_generics(&path, &function.generics)
                    .context(here!())?;
            }
            ItemEnum::Use(use_) => {
                path.push_raw(ComponentType::ReExport, &use_.name, item.span.as_ref());
                // look at the type the `use` statement is referencing
                if let Some(target_id) = &use_.id {
                    // if the item is in the index, check to see if it's in the
                    // root crate.
                    if let Ok(item) = self.item(target_id).context(here!()) {
                        if self.in_root_crate(target_id) {
                            // If yes, then visit it.
                            self.visit_item(&path, item, VisibilityCheck::AssumePublic)?
                        }
                    } else {
                        // If the item isn't in the index, then it's an external
                        // type. Check if it's allowed by the config. If it's
                        // not referenced in `paths` then it's assumed to be an
                        // external hidden module.
                        if let Ok(type_name) = self.type_name(target_id) {
                            self.check_allow_type(&path, &ErrorLocation::ReExport, type_name);
                        } else {
                            let first_hidden_module_in_path =
                                infer_first_hidden_module_in_import_source(
                                    &use_.source,
                                    &self.index,
                                );
                            self.add_error(ValidationError::hidden_module(
                                use_.name.clone(),
                                &ErrorLocation::ReExport,
                                path.to_string(),
                                path.last_span(),
                                first_hidden_module_in_path,
                            ));
                        }
                    }
                }
            }
            ItemEnum::Module(module) => {
                if !module.is_crate {
                    path.push(ComponentType::Module, item);
                }
                for id in &module.items {
                    let module_item = self.item(id).context(here!())?;
                    // Re-exports show up twice in the doc json: once as an `ItemEnum::Import`,
                    // and once as the type as if it were originating from the root crate (but
                    // with a different crate ID). We only want to examine the `ItemEnum::Import`
                    // for re-exports since it includes the correct span where the re-export occurs,
                    // and we don't want to examine the innards of the re-export.
                    if module_item.crate_id == self.root_crate_id {
                        self.visit_item(&path, module_item, VisibilityCheck::Default)
                            .context(here!())?;
                    }
                }
            }
            // ItemEnum::OpaqueTy(_) => unstable_rust_feature!("type_alias_impl_trait", "https://doc.rust-lang.org/beta/unstable-book/language-features/type-alias-impl-trait.html"),
            ItemEnum::Static(sttc) => {
                path.push(ComponentType::Static, item);
                self.visit_type(&path, &ErrorLocation::Static, &sttc.type_)
                    .context(here!())?;
            }
            ItemEnum::Struct(strct) => {
                path.push(ComponentType::Struct, item);
                self.visit_struct(&path, strct).context(here!())?;
            }
            ItemEnum::StructField(typ) => {
                path.push(ComponentType::StructField, item);
                self.visit_type(&path, &ErrorLocation::StructField, typ)
                    .context(here!())?;
            }
            ItemEnum::Trait(trt) => {
                path.push(ComponentType::Trait, item);
                self.visit_trait(&path, trt).context(here!())?;
            }
            ItemEnum::TypeAlias(alias) => {
                path.push(ComponentType::TypeAlias, item);
                self.visit_type(&path, &ErrorLocation::TypeAlias, &alias.type_)
                    .context(here!())?;
                self.visit_generics(&path, &alias.generics)
                    .context(here!())?;
            }
            ItemEnum::TraitAlias(_) => unstable_rust_feature!(
                "trait_alias",
                "https://doc.rust-lang.org/beta/unstable-book/language-features/trait-alias.html"
            ),
            ItemEnum::Union(unn) => {
                path.push(ComponentType::Union, item);
                self.visit_union(&path, unn).context(here!())?;
            }
            ItemEnum::Variant(variant) => {
                path.push(ComponentType::EnumVariant, item);
                self.visit_variant(&path, variant).context(here!())?;
            }
            ItemEnum::ExternCrate { .. }
            | ItemEnum::Impl(_)
            | ItemEnum::Macro(_)
            | ItemEnum::Primitive(_)
            | ItemEnum::ProcMacro(_) => {}
        }
        Ok(())
    }

    fn visit_impls(&self, path: &Path, impl_ids: &[Id]) -> Result<()> {
        for id in impl_ids {
            let impl_item = self.item(id).context(here!())?;
            let mut impl_path = path.clone();
            impl_path.push_raw(
                ComponentType::Impl,
                "",
                impl_item.span.as_ref().or_else(|| path.last_span()),
            );
            self.visit_impl(&impl_path, impl_item).context(here!())?;
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, strct), fields(path = %path))]
    fn visit_struct(&self, path: &Path, strct: &Struct) -> Result<()> {
        self.visit_generics(path, &strct.generics)?;
        let field_ids = match &strct.kind {
            StructKind::Unit => {
                // Unit structs don't have fields
                Vec::new()
            }
            StructKind::Tuple(members) => members.iter().flatten().cloned().collect(),
            StructKind::Plain {
                fields,
                has_stripped_fields,
            } => {
                if *has_stripped_fields {
                    self.add_error(ValidationError::fields_stripped(path));
                }
                fields.clone()
            }
        };
        for id in &field_ids {
            let field = self.item(id).context(here!())?;
            self.visit_item(path, field, VisibilityCheck::Default)?;
        }
        self.visit_impls(path, &strct.impls).context(here!())?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, unn), fields(path = %path))]
    fn visit_union(&self, path: &Path, unn: &Union) -> Result<()> {
        self.visit_generics(path, &unn.generics)?;
        for id in &unn.fields {
            let field = self.item(id).context(here!())?;
            self.visit_item(path, field, VisibilityCheck::Default)?;
        }
        self.visit_impls(path, &unn.impls).context(here!())?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, trt), fields(path = %path))]
    fn visit_trait(&self, path: &Path, trt: &Trait) -> Result<()> {
        self.visit_generics(path, &trt.generics)?;
        self.visit_generic_bounds(path, &trt.bounds)?;
        for id in &trt.items {
            let item = self.item(id).context(here!())?;
            self.visit_item(path, item, VisibilityCheck::Default)?;
        }
        Ok(())
    }

    /// Visits an `impl` block
    #[instrument(level = "debug", skip(self, path, item), fields(path = %path, id = %item.id.0))]
    fn visit_impl(&self, path: &Path, item: &Item) -> Result<()> {
        if let ItemEnum::Impl(imp) = &item.inner {
            // Ignore blanket implementations
            if imp.blanket_impl.is_some() {
                return Ok(());
            }
            // Does the `impl` implement a trait?
            if let Some(trait_) = &imp.trait_ {
                if let Ok(trait_item) = self.item(&trait_.id) {
                    // Don't look for exposure in impls of private traits
                    if !Self::is_public(path, trait_item) {
                        return Ok(());
                    }

                    if let Some(_generic_args) = &trait_.args {
                        // The `trait_` can have generic `args`, but we don't need to visit them
                        // since they are on the trait itself. If the trait is part of the root crate,
                        // it will be visited and checked for external types. If the trait is external,
                        // then what it references doesn't matter for the purposes of this impl that is
                        // being visited.
                    }
                }

                self.check_rustdoc_path(path, &ErrorLocation::ImplementedTrait, trait_)
                    .context(here!())?;
            }

            self.visit_generics(path, &imp.generics)?;
            for id in &imp.items {
                self.visit_item(
                    path,
                    self.item(id).context(here!())?,
                    VisibilityCheck::Default,
                )?;
            }
        } else {
            unreachable!("should be passed an Impl item");
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, decl), fields(path = %path))]
    fn visit_fn_sig(&self, path: &Path, decl: &FunctionSignature) -> Result<()> {
        for (index, (name, typ)) in decl.inputs.iter().enumerate() {
            if index == 0 && name == "self" {
                continue;
            }
            self.visit_type(path, &ErrorLocation::ArgumentNamed(name.into()), typ)
                .context(here!())?;
        }
        if let Some(output) = &decl.output {
            self.visit_type(path, &ErrorLocation::ReturnValue, output)
                .context(here!())?;
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, typ), fields(path = %path))]
    fn visit_type(&self, path: &Path, what: &ErrorLocation, typ: &Type) -> Result<()> {
        match typ {
            Type::ResolvedPath(resolved_path) => {
                self.check_rustdoc_path(path, what, resolved_path)
                    .context(here!())?;
                if let Some(args) = &resolved_path.args {
                    self.visit_generic_args(path, args.as_ref())?;
                }
            }
            Type::Generic(_) => {}
            Type::Primitive(_) => {}
            Type::Pat { .. } => {
                panic!(
                    "Pattern types are unstable and rustc internal rust-lang#120131. \
                      They are unsuported by cargo-check-external-types."
                )
            }
            Type::FunctionPointer(fp) => {
                self.visit_fn_sig(path, &fp.sig)?;
                self.visit_generic_param_defs(path, &fp.generic_params)?;
            }
            Type::Tuple(types) => {
                for typ in types {
                    self.visit_type(path, &ErrorLocation::EnumTupleEntry, typ)?;
                }
            }
            Type::Slice(typ) => self.visit_type(path, what, typ).context(here!())?,
            Type::Array { type_, .. } => self.visit_type(path, what, type_).context(here!())?,
            Type::DynTrait(dyn_trait) => {
                for trait_ in &dyn_trait.traits {
                    self.check_rustdoc_path(path, &ErrorLocation::DynTrait, &trait_.trait_)
                        .context(here!())?;
                    self.visit_generic_param_defs(path, &trait_.generic_params)
                        .context(here!())?;
                }
            }
            Type::ImplTrait(impl_trait) => {
                for bound in impl_trait {
                    match bound {
                        GenericBound::TraitBound {
                            trait_,
                            generic_params,
                            ..
                        } => {
                            self.check_rustdoc_path(path, what, trait_)?;
                            self.visit_generic_param_defs(path, generic_params)?;
                        }
                        GenericBound::Use(_) => {}
                        GenericBound::Outlives(_) => {}
                    }
                }
            }
            Type::Infer => {
                // Don't know what Rust code translates into `Type::Infer`
                bug_panic!("This is a bug (visit_type for Type::Infer).");
            }
            Type::RawPointer { type_, .. } => {
                self.visit_type(path, what, type_).context(here!())?
            }
            Type::BorrowedRef { type_, .. } => {
                self.visit_type(path, what, type_).context(here!())?
            }
            Type::QualifiedPath {
                self_type, trait_, ..
            } => {
                self.visit_type(path, &ErrorLocation::QualifiedSelfType, self_type)?;
                if let Some(trait_) = trait_ {
                    self.check_rustdoc_path(
                        path,
                        &ErrorLocation::QualifiedSelfTypeAsTrait,
                        trait_,
                    )?;
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, args), fields(path = %path))]
    fn visit_generic_args(&self, path: &Path, args: &GenericArgs) -> Result<()> {
        match args {
            GenericArgs::AngleBracketed { args, constraints } => {
                for arg in args {
                    match arg {
                        rustdoc_types::GenericArg::Type(typ) => {
                            self.visit_type(path, &ErrorLocation::GenericArg, typ)?
                        }
                        rustdoc_types::GenericArg::Lifetime(_)
                        | rustdoc_types::GenericArg::Const(_)
                        | rustdoc_types::GenericArg::Infer => {}
                    }
                }
                for constraint in constraints {
                    match &constraint.binding {
                        rustdoc_types::AssocItemConstraintKind::Equality(term) => {
                            if let Term::Type(typ) = term {
                                self.visit_type(path, &ErrorLocation::GenericDefaultBinding, typ)
                                    .context(here!())?;
                            }
                        }
                        rustdoc_types::AssocItemConstraintKind::Constraint(bounds) => {
                            self.visit_generic_bounds(path, bounds)?;
                        }
                    }
                }
            }
            GenericArgs::Parenthesized { inputs, output } => {
                for input in inputs {
                    self.visit_type(path, &ErrorLocation::ClosureInput, input)
                        .context(here!())?;
                }
                if let Some(output) = output {
                    self.visit_type(path, &ErrorLocation::ClosureOutput, output)
                        .context(here!())?;
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, bounds), fields(path = %path))]
    fn visit_generic_bounds(&self, path: &Path, bounds: &[GenericBound]) -> Result<()> {
        for bound in bounds {
            if let GenericBound::TraitBound {
                trait_,
                generic_params,
                ..
            } = bound
            {
                self.check_rustdoc_path(path, &ErrorLocation::TraitBound, trait_)
                    .context(here!())?;
                self.visit_generic_param_defs(path, generic_params)?;
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, params), fields(path = %path))]
    fn visit_generic_param_defs(&self, path: &Path, params: &[GenericParamDef]) -> Result<()> {
        for param in params {
            match &param.kind {
                GenericParamDefKind::Type {
                    bounds, default, ..
                } => {
                    self.visit_generic_bounds(path, bounds)?;
                    if let Some(typ) = default {
                        self.visit_type(path, &ErrorLocation::GenericDefaultBinding, typ)
                            .context(here!())?;
                    }
                }
                GenericParamDefKind::Const { type_, .. } => {
                    self.visit_type(path, &ErrorLocation::ConstGeneric, type_)
                        .context(here!())?;
                }
                GenericParamDefKind::Lifetime { .. } => {
                    // Lifetimes don't have types to check
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, generics), fields(path = %path))]
    fn visit_generics(&self, path: &Path, generics: &Generics) -> Result<()> {
        self.visit_generic_param_defs(path, &generics.params)?;
        for where_pred in &generics.where_predicates {
            match where_pred {
                WherePredicate::BoundPredicate {
                    type_: _,
                    bounds,
                    generic_params,
                } => {
                    // https://github.com/taiki-e/pin-project-lite/issues/86#issuecomment-2438300474
                    // self.visit_type(path, &ErrorLocation::WhereBound, type_)
                    //     .context(here!())?;
                    self.visit_generic_bounds(path, bounds)?;
                    self.visit_generic_param_defs(path, generic_params)?;
                }
                WherePredicate::LifetimePredicate { outlives, .. } => {
                    let bounds: Vec<_> = outlives
                        .iter()
                        .map(|it| GenericBound::Outlives(it.clone()))
                        .collect();
                    self.visit_generic_bounds(path, &bounds)?;
                }
                WherePredicate::EqPredicate { lhs, .. } => {
                    self.visit_type(path, &ErrorLocation::WhereBound, lhs)
                        .context(here!())?;
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, variant), fields(path = %path))]
    fn visit_variant(&self, path: &Path, variant: &Variant) -> Result<()> {
        match &variant.kind {
            VariantKind::Plain => {}
            VariantKind::Tuple(types) => {
                for type_id in types.iter().flatten() {
                    // The type ID isn't the ID of the type being referenced, but rather, the ID
                    // of the tuple entry (for example `0` or `1`). The actual type needs to be further
                    // probed out of this (hence calling `visit_item` instead of `check_external`).
                    let tuple_entry_item = self.item(type_id).context(here!())?;
                    self.visit_item(path, tuple_entry_item, VisibilityCheck::Default)?;
                }
            }
            VariantKind::Struct {
                fields,
                has_stripped_fields,
            } => {
                assert!(!has_stripped_fields, "rustdoc is instructed to document private items, so `fields_stripped` should always be `false`");
                for id in fields {
                    self.visit_item(
                        path,
                        self.item(id).context(here!())?,
                        VisibilityCheck::Default,
                    )?;
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, path, rustdoc_path), fields(path = %path))]
    fn check_rustdoc_path(
        &self,
        path: &Path,
        what: &ErrorLocation,
        rustdoc_path: &RustDocPath,
    ) -> Result<()> {
        self.check_external(path, what, &rustdoc_path.id)
            .context(here!())?;
        if let Some(generic_args) = &rustdoc_path.args {
            self.visit_generic_args(path, generic_args.as_ref())
                .context(here!())?;
        }
        Ok(())
    }

    fn check_external(&self, path: &Path, what: &ErrorLocation, id: &Id) -> Result<()> {
        if let Ok(type_name) = self.type_name(id) {
            self.check_allow_type(path, what, type_name);
        } else if !self.in_root_crate(id) {
            self.add_error(ValidationError::hidden_item(
                what,
                path.to_string(),
                path.last_span(),
            ));
        }
        Ok(())
    }

    fn check_allow_type(&self, path: &Path, what: &ErrorLocation, type_name: String) {
        match self.config.allows_type(&self.root_crate_name, &type_name) {
            Ok(AllowedTypeMatch::RootMatch) | Ok(AllowedTypeMatch::StandardLibrary(_)) => {}
            Ok(AllowedTypeMatch::WildcardMatch(pattern)) => {
                self.remove_unused_approval_pattern(pattern)
            }
            Err(AllowedTypeError::StandardLibraryNotAllowed(_))
            | Err(AllowedTypeError::NoMatchFound) => {
                self.add_error(ValidationError::unapproved_external_type_ref(
                    type_name,
                    what,
                    path.to_string(),
                    path.last_span(),
                ))
            }
            Err(AllowedTypeError::DuplicateMatches(duplicated_approve)) => {
                for approved in duplicated_approve.iter() {
                    self.remove_unused_approval_pattern(approved);
                }
                self.add_error(ValidationError::duplicate_approved(
                    type_name,
                    what,
                    path.to_string(),
                    path.last_span(),
                    duplicated_approve,
                ))
            }
        }
    }

    fn add_error(&self, error: ValidationError) {
        debug!("detected error {:?}", error);
        self.errors.borrow_mut().add(error);
    }

    fn remove_unused_approval_pattern(&self, pattern: &WildMatch) {
        self.unused_approve
            .borrow_mut()
            .remove(&pattern.to_string());
    }

    fn item(&self, id: &Id) -> Result<&Item> {
        self.index
             .get(id)
             .ok_or_else(|| {
                 if let Some(item_summary) = self.paths.get(id) {
                     anyhow!("Failed to find item in index for ID {:?} but did find an item summary: {item_summary:?}", id)
                 } else {
                     anyhow!("Failed to find item in index for ID {:?}", id)
                 }
             })
             .context(here!())
    }

    fn item_summary(&self, id: &Id) -> Option<&ItemSummary> {
        self.paths.get(id)
    }

    fn type_name(&self, id: &Id) -> Result<String> {
        Ok(self.item_summary(id).context(here!())?.path.join("::"))
    }

    fn root_crate_id(package: &Crate) -> Result<u32> {
        Ok(Self::root(package)?.crate_id)
    }

    /// Returns `true` if the given `id` belongs to the root crate.
    ///
    /// Checks index for info on containing crate. If the item is not found in
    /// the index, it is assumed to be external.
    fn in_root_crate(&self, id: &Id) -> bool {
        if let Ok(item) = self.item(id) {
            item.crate_id == self.root_crate_id
        } else {
            false
        }
    }

    fn root_crate_name(package: &Crate) -> Result<String> {
        Ok(Self::root(package)?
            .name
            .as_ref()
            .expect("root should always have a name")
            .clone())
    }

    fn root(package: &Crate) -> Result<&Item> {
        package
            .index
            .get(&package.root)
            .ok_or_else(|| anyhow!("root not found in index"))
            .context(here!())
    }
}

/// Check each segment of a module path against the index. If a segment isn't present in the index,
/// assume that it's the hidden module and return it. Because the path
fn infer_first_hidden_module_in_import_source(
    import_source: &str,
    index: &Index,
) -> Option<String> {
    import_source.split("::").find_map(|part| {
        // When the path part is included in the index, we skip it. If it's not indexed, then it's likely hidden.
        let part_is_not_indexed = !index
            .values()
            .any(|v| v.name.as_ref().map(|name| name == part).unwrap_or_default());

        part_is_not_indexed.then_some(part.to_owned())
    })
}
