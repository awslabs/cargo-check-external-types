/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use cargo_check_external_types::cargo::CargoRustDocJson;
use cargo_check_external_types::config::Config;
use cargo_check_external_types::error::{ErrorPrinter, ValidationError, ValidationErrors};
use cargo_check_external_types::here;
use cargo_check_external_types::visitor::Visitor;
use cargo_metadata::camino::Utf8Path;
use cargo_metadata::{CargoOpt, Metadata, Package, TargetKind};
use clap::Parser;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug, Eq, PartialEq)]
enum OutputFormat {
    Errors,
    MarkdownTable,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Errors => "errors",
            Self::MarkdownTable => "markdown-table",
        })
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "errors" => Ok(OutputFormat::Errors),
            "markdown-table" => Ok(OutputFormat::MarkdownTable),
            _ => Err(anyhow!(
                "invalid output format: {}. Expected `errors` or `markdown-table`.",
                s
            )),
        }
    }
}

#[derive(clap::Args, Debug, Eq, PartialEq)]
struct CheckExternalTypesArgs {
    /// Enables all crate features
    #[arg(long, conflicts_with = "no_default_features")]
    all_features: bool,
    /// Disables default features
    #[arg(long, conflicts_with = "all_features")]
    no_default_features: bool,
    /// Comma delimited list of features to enable in the crate
    #[arg(long, value_delimiter = ',')]
    features: Option<Vec<String>>,
    /// Path to the Cargo manifest
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    /// Target triple
    #[arg(long)]
    target: Option<String>,

    /// Path to config toml to read
    #[arg(long)]
    config: Option<PathBuf>,
    /// Enable verbose output for debugging
    #[arg(short, long)]
    verbose: bool,
    /// Format to output results in
    #[arg(long, default_value_t = OutputFormat::Errors)]
    output_format: OutputFormat,
    /// Skip unsupported package types (binary-only, proc-macro) instead of erroring
    #[arg(long)]
    skip_unsupported: bool,
}

#[derive(Parser, Debug, Eq, PartialEq)]
#[command(author, version, about, bin_name = "cargo")]
enum Args {
    CheckExternalTypes(CheckExternalTypesArgs),
}

enum Error {
    ValidationErrors,
    Failure(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Failure(err)
    }
}

/// Reason why a package was skipped during checking.
#[derive(Debug, Clone)]
enum SkipReason {
    /// Package has no lib target (e.g., binary-only crate)
    NoLibTarget,
    /// Package is a proc-macro crate
    ProcMacro,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::NoLibTarget => write!(f, "package has no lib target"),
            SkipReason::ProcMacro => write!(f, "package is a proc-macro crate"),
        }
    }
}

/// Outcome of checking a single package.
enum PackageOutcome {
    /// Package was checked, may have validation errors
    Checked { errors: ValidationErrors },
    /// Package was skipped for a specific reason
    Skipped(SkipReason),
}

fn main() {
    process::exit(match run_main() {
        Ok(_) => 0,
        Err(Error::ValidationErrors) => 1,
        Err(Error::Failure(err)) => {
            println!("{:#}", dbg!(err));
            2
        }
    })
}

fn run_main() -> Result<(), Error> {
    let Args::CheckExternalTypes(args) = Args::parse();
    if args.verbose {
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("debug"))
            .unwrap();
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(true)
            .with_level(true)
            .with_target(false)
            .pretty();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }

    let mut cargo_metadata_cmd = cargo_metadata::MetadataCommand::new();
    if args.all_features {
        cargo_metadata_cmd.features(CargoOpt::AllFeatures);
    }
    if args.no_default_features {
        cargo_metadata_cmd.features(CargoOpt::NoDefaultFeatures);
    }
    if let Some(features) = &args.features {
        cargo_metadata_cmd.features(CargoOpt::SomeFeatures(features.clone()));
    }
    if let Some(manifest_path) = &args.manifest_path {
        cargo_metadata_cmd.manifest_path(manifest_path);
    }
    let cargo_metadata = cargo_metadata_cmd.exec().context(here!())?;

    // Determine if we're in workspace mode or single-package mode
    let is_workspace_mode = cargo_metadata.root_package().is_none();

    // Validate --features is not used with workspace mode
    if is_workspace_mode && args.features.is_some() {
        return Err(Error::Failure(anyhow!("--features is not supported for workspace targets. Use --all-features instead, or run on individual packages.")));
    }

    // Get the list of packages to check
    let packages: Vec<&Package> = if is_workspace_mode {
        let mut pkgs: Vec<_> = cargo_metadata.workspace_packages();
        pkgs.sort_by(|a, b| a.name.cmp(&b.name));
        pkgs
    } else {
        vec![cargo_metadata.root_package().unwrap()]
    };

    let mut had_validation_errors = false;

    for package in packages {
        if is_workspace_mode {
            eprintln!("Checking package: {}...", package.name);
        }

        // Resolve config: --config flag takes precedence, then per-package metadata
        let config = if let Some(config_path) = &args.config {
            let contents = fs::read_to_string(config_path).context("failed to read config file")?;
            toml::from_str(&contents).context("failed to parse config file")?
        } else {
            resolve_config_for_package(package)
                .context("failed to parse config from Cargo.toml metadata")?
        };

        // Resolve the crate path for this package
        let crate_path = package
            .manifest_path
            .parent()
            .expect("manifest should have parent")
            .as_std_path()
            .to_path_buf();

        // Resolve features for this package
        let cargo_features = if let Some(features) = &args.features {
            features.clone()
        } else {
            resolve_features_for_package(&cargo_metadata, package)?
        };

        // Check the package
        let outcome = check_package(
            package,
            config,
            &crate_path,
            &cargo_metadata.target_directory,
            cargo_features,
            args.target.clone(),
        )?;

        match outcome {
            PackageOutcome::Skipped(reason) => {
                if args.skip_unsupported {
                    eprintln!("Skipping {}: {}", package.name, reason);
                } else {
                    return Err(Error::Failure(anyhow!(
                        "Package '{}' is not supported: {}. Use --skip-unsupported to skip.",
                        package.name,
                        reason
                    )));
                }
            }
            PackageOutcome::Checked { errors } => match args.output_format {
                OutputFormat::Errors => {
                    ErrorPrinter::new(&cargo_metadata.workspace_root).pretty_print_errors(&errors);
                    if errors.error_count() > 0 {
                        had_validation_errors = true;
                    }
                }
                OutputFormat::MarkdownTable => {
                    if is_workspace_mode {
                        println!("## {}", package.name);
                        println!();
                    }
                    println!("| Crate | Type | Used In |");
                    println!("| ---   | ---  | ---     |");
                    let mut rows = Vec::new();
                    for error in errors.iter() {
                        if let ValidationError::UnapprovedExternalTypeRef { .. } = error {
                            let type_name = error.type_name();
                            let crate_name =
                                &type_name[0..type_name.find("::").unwrap_or(type_name.len())];
                            let location = error.location().unwrap();
                            rows.push(format!(
                                "| {} | {} | {}:{}:{} |",
                                crate_name,
                                type_name,
                                location.filename.to_string_lossy(),
                                location.begin.0,
                                location.begin.1
                            ));
                        }
                    }
                    rows.sort();
                    rows.into_iter().for_each(|row| println!("{row}"));
                    if is_workspace_mode {
                        println!();
                    }
                }
            },
        }
    }

    if had_validation_errors {
        Err(Error::ValidationErrors)
    } else {
        Ok(())
    }
}

/// Check a single package and return the outcome.
fn check_package(
    package: &Package,
    config: Config,
    crate_path: &PathBuf,
    target_directory: &Utf8Path,
    features: Vec<String>,
    target: Option<String>,
) -> Result<PackageOutcome> {
    // Check if package is a proc-macro crate
    let is_proc_macro = package
        .targets
        .iter()
        .any(|t| t.kind.contains(&TargetKind::ProcMacro));
    if is_proc_macro {
        return Ok(PackageOutcome::Skipped(SkipReason::ProcMacro));
    }

    // Check if package has a lib target
    let lib_name = match resolve_lib_name_for_package(package) {
        Some(name) => name,
        None => return Ok(PackageOutcome::Skipped(SkipReason::NoLibTarget)),
    };

    eprintln!("Running rustdoc to produce json doc output...");
    let crate_data =
        CargoRustDocJson::new(lib_name, crate_path, target_directory, features, target)
            .run()
            .context(here!())?;

    eprintln!("Examining all public types...");
    let errors = Visitor::new(config, crate_data)?.visit_all()?;
    Ok(PackageOutcome::Checked { errors })
}

fn resolve_config_for_package(package: &Package) -> Result<Config> {
    let crate_metadata = match serde_json::from_value::<HashMap<String, serde_json::Value>>(
        package.metadata.clone(),
    ) {
        Ok(m) => m,
        // We avoid using ? on the serde_json::from_value because when the metadata is not provided
        // this will err trying to unmarshal a null value into a map. In this instance we want to
        // use the default config.
        Err(_) => return Ok(Default::default()),
    };

    Ok(
        if let Some(our_metadata) = crate_metadata.get(env!("CARGO_CRATE_NAME")) {
            // Here we do use ? to propagate the error from the unmarshal - it would indicate
            // the metadata config is present, but invalid.
            serde_json::from_value(our_metadata.clone())?
        } else {
            Default::default()
        },
    )
}

fn resolve_features_for_package(metadata: &Metadata, package: &Package) -> Result<Vec<String>> {
    if let Some(resolve) = &metadata.resolve {
        let root_node = resolve
            .nodes
            .iter()
            .find(|&n| n.id == package.id)
            .ok_or_else(|| anyhow!("Failed to find node for package {}", package.name))?;
        Ok(root_node.features.clone())
    } else {
        bail!("Cargo metadata didn't have resolved nodes");
    }
}

fn resolve_lib_name_for_package(package: &Package) -> Option<String> {
    let lib_targets: Vec<_> = package
        .targets
        .iter()
        .filter(|t| t.kind.contains(&TargetKind::Lib))
        .collect();
    if lib_targets.len() == 1 {
        Some(lib_targets.first().unwrap().name.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Args::command().debug_assert();
    }
}

#[cfg(test)]
mod arg_parse_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from(["cargo", "check-external-types"]).unwrap()
        );
    }

    #[test]
    fn all_features() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: true,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from(["cargo", "check-external-types", "--all-features"]).unwrap()
        );
    }

    #[test]
    fn no_default_features() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: true,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from(["cargo", "check-external-types", "--no-default-features"])
                .unwrap()
        );
    }

    #[test]
    fn feature_list() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: Some(vec!["foo".into(), "bar".into()]),
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from(["cargo", "check-external-types", "--features", "foo,bar"])
                .unwrap()
        );
    }

    #[test]
    fn manifest_path() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: Some("test-path".into()),
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from([
                "cargo",
                "check-external-types",
                "--manifest-path",
                "test-path"
            ])
            .unwrap()
        );
    }

    #[test]
    fn target() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: Some("x86_64-unknown-linux-gnu".into()),
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from([
                "cargo",
                "check-external-types",
                "--target",
                "x86_64-unknown-linux-gnu"
            ])
            .unwrap()
        );
    }

    #[test]
    fn verbose() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: true,
                output_format: OutputFormat::Errors,
                skip_unsupported: false,
            }),
            Args::try_parse_from(["cargo", "check-external-types", "--verbose"]).unwrap()
        );
    }

    #[test]
    fn output_format_markdown_table() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::MarkdownTable,
                skip_unsupported: false,
            }),
            Args::try_parse_from([
                "cargo",
                "check-external-types",
                "--output-format",
                "markdown-table"
            ])
            .unwrap()
        );
    }

    #[test]
    fn conflict_all_features_no_default_features() {
        // Check `--all-features` and `--no-default-features` conflict
        assert!(Args::try_parse_from([
            "cargo",
            "check-external-types",
            "--all-features",
            "--no-default-features"
        ])
        .is_err());
    }

    #[test]
    fn skip_unsupported() {
        assert_eq!(
            Args::CheckExternalTypes(CheckExternalTypesArgs {
                all_features: false,
                no_default_features: false,
                features: None,
                manifest_path: None,
                target: None,
                config: None,
                verbose: false,
                output_format: OutputFormat::Errors,
                skip_unsupported: true,
            }),
            Args::try_parse_from(["cargo", "check-external-types", "--skip-unsupported"]).unwrap()
        );
    }
}
