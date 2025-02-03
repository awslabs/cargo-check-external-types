/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use cargo_check_external_types::cargo::CargoRustDocJson;
use cargo_check_external_types::config::Config;
use cargo_check_external_types::error::{ErrorPrinter, ValidationError};
use cargo_check_external_types::here;
use cargo_check_external_types::visitor::Visitor;
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
    if let Some(features) = args.features {
        cargo_metadata_cmd.features(CargoOpt::SomeFeatures(features));
    }
    let crate_path = if let Some(manifest_path) = args.manifest_path {
        cargo_metadata_cmd.manifest_path(&manifest_path);
        manifest_path
            .canonicalize()
            .context(here!())?
            .parent()
            .expect("parent path")
            .to_path_buf()
    } else {
        std::env::current_dir()
            .context(here!())?
            .canonicalize()
            .context(here!())?
    };
    let cargo_metadata = cargo_metadata_cmd.exec().context(here!())?;

    let config = if let Some(config_path) = &args.config {
        let contents = fs::read_to_string(config_path).context("failed to read config file")?;
        toml::from_str(&contents).context("failed to parse config file")?
    } else {
        resolve_config(&cargo_metadata)
            .context("failed to parse config from Cargo.toml metadata")?
    };

    let cargo_features = resolve_features(&cargo_metadata)?;
    let cargo_lib_name = resolve_lib_name(&cargo_metadata)?;

    eprintln!("Running rustdoc to produce json doc output...");
    let package = CargoRustDocJson::new(
        cargo_lib_name,
        crate_path,
        &cargo_metadata.target_directory,
        cargo_features,
        args.target.clone(),
    )
    .run()
    .context(here!())?;

    eprintln!("Examining all public types...");
    let errors = Visitor::new(config, package)?.visit_all()?;
    match args.output_format {
        OutputFormat::Errors => {
            ErrorPrinter::new(&cargo_metadata.workspace_root).pretty_print_errors(&errors);
            if errors.error_count() > 0 {
                return Err(Error::ValidationErrors);
            }
        }
        OutputFormat::MarkdownTable => {
            println!("| Crate | Type | Used In |");
            println!("| ---   | ---  | ---     |");
            let mut rows = Vec::new();
            for error in errors.iter() {
                if let ValidationError::UnapprovedExternalTypeRef { .. } = error {
                    let type_name = error.type_name();
                    let crate_name = &type_name[0..type_name.find("::").unwrap_or(type_name.len())];
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
            rows.into_iter().for_each(|row| println!("{}", row));
        }
    }

    Ok(())
}

fn resolve_config(metadata: &Metadata) -> Result<Config> {
    let crate_metadata = match serde_json::from_value::<HashMap<String, serde_json::Value>>(
        resolve_root_package(metadata)?.metadata.clone(),
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

fn resolve_features(metadata: &Metadata) -> Result<Vec<String>> {
    let root_package = resolve_root_package(metadata)?;
    if let Some(resolve) = &metadata.resolve {
        let root_node = resolve
            .nodes
            .iter()
            .find(|&n| n.id == root_package.id)
            .ok_or_else(|| anyhow!("Failed to find node for root package"))?;
        Ok(root_node.features.clone())
    } else {
        bail!("Cargo metadata didn't have resolved nodes");
    }
}

fn resolve_lib_name(metadata: &Metadata) -> Result<String> {
    let lib_targets = resolve_root_package(metadata)?
        .targets
        .iter()
        .filter(|t| t.kind.iter().any(|k| *k == TargetKind::Lib))
        .collect::<Vec<_>>();
    if lib_targets.len() != 1 {
        bail!(
            "Expected crate to define 1 lib target, found {}",
            lib_targets.len()
        );
    }
    Ok(lib_targets.first().unwrap().name.clone())
}

fn resolve_root_package(metadata: &Metadata) -> Result<&Package> {
    metadata
        .root_package()
        .ok_or_else(|| {
            let workspace_members = metadata.workspace_members.as_slice().iter().map(|id| id.to_string()).collect::<Vec<_>>().join("\n");
            if !workspace_members.is_empty() {
                anyhow!("it appears you're trying to run `cargo-check-external-types` on a workspace Cargo.toml; Instead, run it on one of the workspace member Cargo.tomls directly:\n{workspace_members}")
            } else {
                anyhow!("No root package found")
            }
        })
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
}
