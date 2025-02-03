/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::bug;
use anyhow::{Context, Result};
use pest::Position;
use rustdoc_types::Span;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use wildmatch::WildMatch;

/// Where the error occurred relative to the [`Path`](crate::path::Path).
///
/// For example, if the path is a path to a function, then this could point to something
/// specific about that function, such as a specific function argument that is in error.
///
/// There is overlap in this enum with [`ComponentType`](crate::path::ComponentType) since
/// some paths are specific enough to locate the external type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorLocation {
    AssocType,
    ArgumentNamed(String),
    ClosureInput,
    ClosureOutput,
    ConstGeneric,
    Constant,
    DynTrait,
    EnumTupleEntry,
    GenericArg,
    GenericDefaultBinding,
    ImplementedTrait,
    QualifiedSelfType,
    QualifiedSelfTypeAsTrait,
    ReExport,
    ReturnValue,
    Static,
    StructField,
    TraitBound,
    TypeAlias,
    WhereBound,
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::AssocType => "associated type",
            Self::ArgumentNamed(name) => return write!(f, "argument named `{}` of", name),
            Self::ClosureInput => "closure input of",
            Self::ClosureOutput => "closure output of",
            Self::ConstGeneric => "const generic of",
            Self::Constant => "constant",
            Self::DynTrait => "dyn trait of",
            Self::EnumTupleEntry => "enum tuple entry of",
            Self::GenericArg => "generic arg of",
            Self::GenericDefaultBinding => "generic default binding of",
            Self::ImplementedTrait => "implemented trait of",
            Self::QualifiedSelfType => "qualified self type",
            Self::QualifiedSelfTypeAsTrait => "qualified type `as` trait",
            Self::ReExport => "re-export named",
            Self::ReturnValue => "return value of",
            Self::Static => "static value",
            Self::StructField => "struct field of",
            Self::TraitBound => "trait bound of",
            Self::TypeAlias => "type alias of",
            Self::WhereBound => "where bound of",
        };
        write!(f, "{}", s)
    }
}

#[derive(Default)]
pub struct ValidationErrors {
    errors: BTreeSet<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn error_count(&self) -> usize {
        self.errors
            .iter()
            .map(ValidationError::level)
            .filter(|&l| l == ErrorLevel::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.errors
            .iter()
            .map(ValidationError::level)
            .filter(|&l| l == ErrorLevel::Warning)
            .count()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.insert(error);
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidationError> {
        self.errors.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ErrorLevel {
    Error,
    Warning,
}

/// Error type for validation errors that get displayed to the user on the CLI.
#[derive(Debug)]
pub enum ValidationError {
    UnapprovedExternalTypeRef {
        type_name: String,
        what: ErrorLocation,
        in_what_type: String,
        location: Option<Span>,
        sort_key: String,
    },
    FieldsStripped {
        type_name: String,
    },
    HiddenModule {
        type_name: String,
        what: ErrorLocation,
        in_what_type: String,
        location: Option<Span>,
        hidden_module: Option<String>,
    },
    HiddenItem {
        what: ErrorLocation,
        in_what_type: String,
        location: Option<Span>,
        sort_key: String,
    },
    UnusedApprovalPattern {
        type_name: String,
    },
    DuplicateApproved {
        type_name: String,
        what: ErrorLocation,
        in_what_type: String,
        location: Option<Span>,
        duplicate: Vec<String>,
        sort_key: String,
    },
}

impl ValidationError {
    pub fn unapproved_external_type_ref(
        type_name: impl Into<String>,
        what: &ErrorLocation,
        in_what_type: impl Into<String>,
        location: Option<&Span>,
    ) -> Self {
        let type_name = type_name.into();
        let in_what_type = in_what_type.into();
        let sort_key = format!(
            "{}:{type_name}:{what}:{in_what_type}",
            location_sort_key(location)
        );
        if location.is_none() {
            bug!("An error is missing a span and will be printed without context, file name, and line number.");
        }
        Self::UnapprovedExternalTypeRef {
            type_name,
            what: what.clone(),
            in_what_type,
            location: location.cloned(),
            sort_key,
        }
    }

    pub fn level(&self) -> ErrorLevel {
        match self {
            Self::UnapprovedExternalTypeRef { .. } => ErrorLevel::Error,
            Self::HiddenModule { .. }
            | Self::HiddenItem { .. }
            | Self::FieldsStripped { .. }
            | Self::UnusedApprovalPattern { .. }
            | Self::DuplicateApproved { .. } => ErrorLevel::Warning,
        }
    }

    pub fn fields_stripped(path: &crate::path::Path) -> Self {
        Self::FieldsStripped {
            type_name: path.to_string(),
        }
    }

    pub fn hidden_module(
        type_name: impl Into<String>,
        what: &ErrorLocation,
        in_what_type: impl Into<String>,
        location: Option<&Span>,
        hidden_module: Option<String>,
    ) -> Self {
        if location.is_none() {
            bug!("A warning is missing a span and will be printed without context, file name, and line number.");
        }
        Self::HiddenModule {
            type_name: type_name.into(),
            what: what.clone(),
            in_what_type: in_what_type.into(),
            location: location.cloned(),
            hidden_module,
        }
    }

    pub fn hidden_item(
        what: &ErrorLocation,
        in_what_type: impl Into<String>,
        location: Option<&Span>,
    ) -> Self {
        if location.is_none() {
            bug!("A warning is missing a span and will be printed without context, file name, and line number.");
        }
        Self::HiddenItem {
            what: what.clone(),
            in_what_type: in_what_type.into(),
            location: location.cloned(),
            sort_key: location_sort_key(location),
        }
    }

    pub fn unused_approval_pattern(type_name: impl Into<String>) -> Self {
        Self::UnusedApprovalPattern {
            type_name: type_name.into(),
        }
    }

    pub fn duplicate_approved(
        type_name: impl Into<String>,
        what: &ErrorLocation,
        in_what_type: impl Into<String>,
        location: Option<&Span>,
        duplicate: Vec<&WildMatch>,
    ) -> Self {
        if location.is_none() {
            bug!("A warning is missing a span and will be printed without context, file name, and line number.");
        }
        let type_name = type_name.into();
        let in_what_type = in_what_type.into();
        let duplicate = duplicate
            .iter()
            .map(|pattern| pattern.to_string())
            .collect();
        let sort_key = format!(
            "{}:{type_name}:{what}:{in_what_type}",
            location_sort_key(location)
        );
        Self::DuplicateApproved {
            type_name,
            what: what.clone(),
            in_what_type,
            location: location.cloned(),
            duplicate,
            sort_key,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Self::UnapprovedExternalTypeRef { type_name, .. }
            | Self::HiddenModule { type_name, .. }
            | Self::FieldsStripped { type_name }
            | Self::UnusedApprovalPattern { type_name }
            | Self::DuplicateApproved { type_name, .. } => type_name,
            Self::HiddenItem { .. } => "N/A",
        }
    }

    pub fn location(&self) -> Option<&Span> {
        match self {
            Self::UnapprovedExternalTypeRef { location, .. }
            | Self::HiddenModule { location, .. }
            | Self::HiddenItem { location, .. }
            | Self::DuplicateApproved { location, .. } => location.as_ref(),
            Self::FieldsStripped { .. } | Self::UnusedApprovalPattern { .. } => None,
        }
    }

    fn sort_key(&self) -> &str {
        match self {
            Self::UnapprovedExternalTypeRef { sort_key, .. }
            | Self::DuplicateApproved { sort_key, .. } => sort_key.as_ref(),
            Self::FieldsStripped { type_name }
            | Self::HiddenModule { type_name, .. }
            | Self::UnusedApprovalPattern { type_name } => type_name.as_ref(),
            Self::HiddenItem { sort_key, .. } => sort_key.as_ref(),
        }
    }

    pub fn fmt_headline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnapprovedExternalTypeRef { type_name, .. } => {
                write!(
                    f,
                    "Unapproved external type `{type_name}` referenced in public API"
                )
            }
            Self::HiddenModule {
                type_name,
                hidden_module,
                ..
            } => {
                let hidden_module = hidden_module.as_deref().unwrap_or("???");
                write!(
                     f,
                     "Module path for reexported type `{type_name}` contains a `#[doc(hidden)]` module \"{hidden_module}\". Types declared in this module cannot be checked for external types"
                 )
            }
            Self::HiddenItem {
                what, in_what_type, ..
            } => {
                write!(
                     f,
                     "{what} {in_what_type} references a hidden item. Items marked `#[doc(hidden)]` cannot be checked for external types"
                 )
            }
            Self::FieldsStripped { type_name } => {
                write!(
                     f,
                     "Fields on `{type_name}` marked `#[doc(hidden)]` cannot be checked for external types"
                 )
            }
            Self::UnusedApprovalPattern { type_name } => {
                write!(
                    f,
                    "Approved external type `{type_name}` wasn't referenced in public API"
                )
            }
            Self::DuplicateApproved {
                type_name,
                duplicate,
                ..
            } => {
                write!(
                    f,
                    "External type `{type_name}` is allowed multiple times:\n Allowed patterns:{}",
                    duplicate
                        .iter()
                        .map(|glob| format!("\n    - {}", glob))
                        .fold(String::new(), |acc, f| acc + &f)
                )
            }
        }
    }

    pub fn subtext(&self) -> Cow<'static, str> {
        match self {
            Self::UnapprovedExternalTypeRef {
                what, in_what_type, ..
            } => format!("in {} `{}`", what, in_what_type).into(),
            Self::FieldsStripped { .. } | Self::UnusedApprovalPattern { .. } => "".into(),
            Self::HiddenModule {
                what, in_what_type, ..
            }
            | Self::HiddenItem {
                what, in_what_type, ..
            }
            | Self::DuplicateApproved {
                what, in_what_type, ..
            } => format!("in {} `{}`", what, in_what_type).into(),
        }
    }
}

fn location_sort_key(location: Option<&Span>) -> String {
    if let Some(location) = location {
        format!(
            "{}:{:07}:{:07}",
            location.filename.to_string_lossy(),
            location.begin.0,
            location.begin.1
        )
    } else {
        "none".into()
    }
}

impl Ord for ValidationError {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(other.sort_key())
    }
}

impl PartialOrd for ValidationError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ValidationError {}

impl PartialEq for ValidationError {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key() == other.sort_key()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_headline(f)
    }
}

/// Pretty printer for error context.
///
/// This makes validation errors look similar to the compiler errors from rustc.
pub struct ErrorPrinter {
    workspace_root: PathBuf,
    file_cache: HashMap<PathBuf, String>,
}

impl ErrorPrinter {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            file_cache: HashMap::new(),
        }
    }

    fn get_file_contents(&mut self, path: &Path) -> Result<&str> {
        if !self.file_cache.contains_key(path) {
            let full_file_name = self.workspace_root.join(path).canonicalize()?;
            let contents = std::fs::read_to_string(&full_file_name)
                .context("failed to load source file for error context")
                .context(full_file_name.to_string_lossy().to_string())?;
            self.file_cache.insert(path.to_path_buf(), contents);
        }
        Ok(self.file_cache.get(path).unwrap())
    }

    fn print_error_level(level: ErrorLevel) {
        use owo_colors::{OwoColorize, Stream};
        match level {
            ErrorLevel::Error => {
                print!(
                    "{}",
                    "error: "
                        .if_supports_color(Stream::Stdout, |text| text.red())
                        .if_supports_color(Stream::Stdout, |text| text.bold())
                );
            }
            ErrorLevel::Warning => {
                print!(
                    "{}",
                    "warning: "
                        .if_supports_color(Stream::Stdout, |text| text.yellow())
                        .if_supports_color(Stream::Stdout, |text| text.bold())
                );
            }
        }
    }

    /// Outputs a human readable error with file location context
    ///
    /// # Example output
    ///
    /// ```text
    /// error: Unapproved external type `external_lib::SomeStruct` referenced in public API
    ///    --> test-crate/src/lib.rs:38:1
    ///    |
    /// 38 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
    ///    | ^-----------------------------------------------------------------------^
    ///    |
    ///    = in argument named `_one` of `test_crate::external_in_fn_input`
    /// ```
    pub fn pretty_print_error_context(&mut self, location: &Span, subtext: &str) {
        match self.get_file_contents(&location.filename) {
            Ok(file_contents) => {
                let begin = Self::position_from_line_col(file_contents, location.begin);
                let end = Self::position_from_line_col(file_contents, location.end);

                // HACK: Using Pest to do the pretty error context formatting for lack of
                // knowledge of a smaller library tailored to this use-case
                let variant = pest::error::ErrorVariant::<()>::CustomError {
                    message: subtext.into(),
                };
                let err_context = match (begin, end) {
                    (Some(b), Some(e)) => {
                        Some(pest::error::Error::new_from_span(variant, b.span(&e)))
                    }
                    (Some(b), None) => Some(pest::error::Error::new_from_pos(variant, b)),
                    _ => None,
                };
                if let Some(err_context) = err_context {
                    println!(
                        "{}\n",
                        err_context.with_path(&location.filename.to_string_lossy())
                    );
                }
            }
            Err(err) => {
                Self::print_error_level(ErrorLevel::Error);
                println!("{subtext}");
                println!(
                    "  --> {}:{}:{}",
                    location.filename.to_string_lossy(),
                    location.begin.0,
                    location.begin.1 + 1
                );
                println!("   | Failed to load {:?}", location.filename);
                println!("   | relative to {:?}", self.workspace_root);
                println!("   | to provide error message context.");
                println!("   | Cause: {err:?}");
            }
        }
    }

    fn position_from_line_col(contents: &str, (line, col): (usize, usize)) -> Option<Position> {
        let (mut cl, mut cc) = (1, 1);
        let content_bytes = contents.as_bytes();
        for (index, &byte) in content_bytes.iter().enumerate() {
            if cl == line && cc == col {
                return Position::new(contents, index);
            }

            cc += 1;
            if byte == b'\n' {
                cl += 1;
                cc = 0;
            }
        }
        None
    }

    pub fn pretty_print_errors(&mut self, errors: &ValidationErrors) {
        for error in errors.iter() {
            Self::print_error_level(error.level());
            println!("{}", error);
            if let Some(location) = error.location() {
                self.pretty_print_error_context(location, error.subtext().as_ref())
            }
        }
        if !errors.is_empty() {
            use owo_colors::{OwoColorize, Stream};
            let (error_count, warning_count) = (errors.error_count(), errors.warning_count());
            println!(
                "{error_count} {errors}, {warning_count} {warnings} emitted",
                errors = "errors".if_supports_color(Stream::Stdout, |text| text.red()),
                warnings = "warnings".if_supports_color(Stream::Stdout, |text| text.yellow())
            );
        }
    }
}
