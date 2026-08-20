/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::here;
use anyhow::{bail, Context, Result};
use rustdoc_types::{Crate, FORMAT_VERSION};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

#[derive(Deserialize)]
struct CrateFormatVersion {
    format_version: u32,
}

/// Cargo's [manifest options], forwarded verbatim to every `cargo` invocation this tool
/// makes.
///
/// [manifest options]: https://doc.rust-lang.org/cargo/commands/cargo.html#manifest-options
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CargoManifestOpts {
    /// Prevents Cargo from accessing the network for any reason. Without this flag, Cargo
    /// will stop with an error if it needs to access the network and the network is not
    /// available. With this flag, Cargo will attempt to proceed without the network if
    /// possible.
    ///
    /// Beware that this may result in different dependency resolution than online mode.
    /// Cargo will restrict itself to crates that are downloaded locally, even if there
    /// might be a newer version as indicated in the local copy of the index.
    pub offline: bool,
    /// Asserts that the exact same dependencies and versions are used as when the existing
    /// `Cargo.lock` file was originally generated. Cargo will exit with an error when
    /// either of the following scenarios arises:
    ///
    /// * The lock file is missing.
    /// * Cargo attempted to change the lock file due to a different dependency resolution.
    ///
    /// It may be used in environments where deterministic builds are desired, such as in
    /// CI pipelines.
    pub locked: bool,
    /// Equivalent to specifying both `--locked` and `--offline`.
    pub frozen: bool,
}

impl CargoManifestOpts {
    /// Renders these options as `cargo` arguments.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.offline {
            args.push("--offline".to_string());
        }
        if self.locked {
            args.push("--locked".to_string());
        }
        if self.frozen {
            args.push("--frozen".to_string());
        }
        args
    }
}

/// Runs the `cargo rustdoc` command required to produce Rustdoc's JSON output with a nightly compiler.
pub struct CargoRustDocJson {
    /// Name of the lib target, by default this is the crate name but may be customized in the
    /// `[lib]` section of the Cargo.toml file.
    lib_name: String,
    /// Path of the crate to examine
    crate_path: PathBuf,
    /// Expected `target/` directory where the output will be
    target_path: PathBuf,
    /// Features to enable
    features: Vec<String>,
    /// Target triple
    target: Option<String>,
    /// Flags restricting network access and `Cargo.lock` mutation
    manifest_opts: CargoManifestOpts,
}

impl CargoRustDocJson {
    pub fn new(
        lib_name: impl Into<String>,
        crate_path: impl Into<PathBuf>,
        target_path: impl Into<PathBuf>,
        features: Vec<String>,
        target: Option<String>,
        manifest_opts: CargoManifestOpts,
    ) -> Self {
        CargoRustDocJson {
            lib_name: lib_name.into(),
            crate_path: crate_path.into(),
            target_path: target_path.into(),
            features,
            target,
            manifest_opts,
        }
    }

    pub fn run(&self) -> Result<Crate> {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

        let mut command = Command::new(cargo);
        command.current_dir(&self.crate_path).arg("rustdoc");
        command.arg("--lib");
        if !self.features.is_empty() {
            command.arg("--no-default-features").arg("--features");
            command.arg(self.features.join(","));
        }
        if let Some(target) = &self.target {
            command.arg("--target").arg(target);
        }
        // Must come before the `--` separator so that cargo, rather than rustdoc, receives them.
        command.args(self.manifest_opts.to_args());
        command
            .arg("--")
            .arg("--document-private-items")
            .arg("-Z")
            .arg("unstable-options")
            .arg("--output-format")
            .arg("json");
        let output = command
            .output()
            .context(here!("failed to run nightly rustdoc"))?;
        handle_failure("rustdoc", &output)?;

        let mut output_file_name = self
            .target_path
            .canonicalize()
            .context(here!("failed to canonicalize {:?}", self.target_path))?;
        if let Some(target) = &self.target {
            output_file_name.push(target);
        }
        output_file_name.push(format!("doc/{}.json", self.lib_name.replace('-', "_")));

        let json = fs::read_to_string(output_file_name).context(here!())?;
        let format_version: CrateFormatVersion = serde_json::from_str(&json)
            .context("Failed to find `format_version` in rustdoc JSON output.")
            .context(here!())?;
        if format_version.format_version != FORMAT_VERSION {
            bail!(
                "The version of rustdoc being used produces JSON format version {0}, but \
                this tool requires format version {1}. This can happen if the locally \
                installed version of rustdoc doesn't match the rustdoc JSON types from \
                the `rustdoc-types` crate.\n\n\
                If this occurs with the latest Rust nightly and the latest version of this \
                tool, then this is a bug, and the tool needs to be upgraded to the latest \
                format version.\n\n\
                Otherwise, you'll need to determine a Rust nightly version that matches \
                this tool's supported format version (or vice versa).",
                format_version.format_version,
                FORMAT_VERSION
            );
        }
        let package: Crate = serde_json::from_str(&json)
            .context("Failed to parse rustdoc output.")
            .context(here!())?;
        Ok(package)
    }
}

pub fn handle_failure(operation_name: &str, output: &Output) -> Result<(), anyhow::Error> {
    if !output.status.success() {
        return Err(capture_error(operation_name, output));
    }
    Ok(())
}

pub fn capture_error(operation_name: &str, output: &Output) -> anyhow::Error {
    let message = format!(
        "Failed to {name}:\nStatus: {status}\nStdout: {stdout}\nStderr: {stderr}\n",
        name = operation_name,
        status = if let Some(code) = output.status.code() {
            format!("{code}")
        } else {
            "Killed by signal".to_string()
        },
        stdout = String::from_utf8_lossy(&output.stdout),
        stderr = String::from_utf8_lossy(&output.stderr)
    );
    anyhow::Error::msg(message)
}

#[cfg(test)]
mod tests {
    use super::CargoManifestOpts;

    #[test]
    fn manifest_opts_to_args() {
        assert!(CargoManifestOpts::default().to_args().is_empty());
        assert_eq!(
            vec!["--offline"],
            CargoManifestOpts {
                offline: true,
                ..Default::default()
            }
            .to_args()
        );
        assert_eq!(
            vec!["--locked"],
            CargoManifestOpts {
                locked: true,
                ..Default::default()
            }
            .to_args()
        );
        assert_eq!(
            vec!["--frozen"],
            CargoManifestOpts {
                frozen: true,
                ..Default::default()
            }
            .to_args()
        );
        // `cargo` itself accepts these together, so the tool doesn't reject the combination.
        assert_eq!(
            vec!["--offline", "--locked", "--frozen"],
            CargoManifestOpts {
                offline: true,
                locked: true,
                frozen: true,
            }
            .to_args()
        );
    }
}
