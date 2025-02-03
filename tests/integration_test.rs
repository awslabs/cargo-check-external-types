/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use cargo_check_external_types::cargo::handle_failure;
use pretty_assertions::assert_str_eq;
use std::fs;
use std::path::Path;
use std::process::Output;
use test_bin::get_test_bin;

/// Returns (stdout, stderr)
pub fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_with_args(in_path: impl AsRef<Path>, args: &[&str]) -> String {
    let mut cmd = get_test_bin("cargo-check-external-types");
    cmd.current_dir(in_path.as_ref());
    cmd.arg("check-external-types");
    for &arg in args {
        cmd.arg(arg);
    }
    let output = cmd
        .output()
        .expect("failed to start cargo-check-external-types");
    match output.status.code() {
        Some(1) => { /* expected */ }
        _ => handle_failure("cargo-check-external-types", &output).unwrap(),
    }
    let (stdout, _) = output_text(&output);
    stdout
}

#[test]
fn with_default_config() {
    let expected_output = fs::read_to_string("tests/default-config-expected-output.md").unwrap();
    let actual_output = run_with_args("test-workspace/test-crate", &[]);
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn with_custom_lib_name() {
    let expected_output =
        fs::read_to_string("tests/default-config-custom-lib-name-expected-output.md").unwrap();
    let actual_output = run_with_args("test-workspace/test-crate-custom-lib-name", &[]);
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn with_some_allowed_types() {
    let expected_output = fs::read_to_string("tests/allow-some-types-expected-output.md").unwrap();
    let actual_output = run_with_args(
        "test-workspace/test-crate",
        &["--config", "../../tests/allow-some-types.toml"],
    );
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn with_some_allowed_types_in_metadata() {
    let expected_output =
        fs::read_to_string("tests/allow-some-types-metadata-expected-output.md").unwrap();
    let actual_output = run_with_args(
        "test-workspace/test-crate-metadata-config",
        &[], // We provide no config here so the crate's Cargo.toml metadata is used.
    );
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn with_some_allowed_types_explicit_config_file() {
    let actual_output = run_with_args(
        "test-workspace/test-crate-metadata-config",
        // Because we provide an explicit config file, we expect it to take precedence over
        // the Cargo.toml metadata.
        &["--config", "../../tests/allow-some-types.toml"],
    );
    // The config file allows all of the types, so we expect no output.
    assert_str_eq!("", actual_output);
}

#[test]
fn with_output_format_markdown_table() {
    let expected_output =
        fs::read_to_string("tests/output-format-markdown-table-expected-output.md").unwrap();
    let actual_output = run_with_args(
        "test-workspace/test-crate",
        &["--output-format", "markdown-table"],
    );
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn test_unused_allowed_external_types() {
    let expected_output = fs::read_to_string("tests/allow-types-unused.md").unwrap();
    let actual_output = run_with_args(
        "test-workspace/test-crate",
        &["--config", "../../tests/allow-types-unused.toml"],
    );
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn test_multiple_allowed_external_types() {
    let expected_output = fs::read_to_string("tests/allow-types-multiple-times.md").unwrap();
    let actual_output = run_with_args(
        "test-workspace/test-crate",
        &["--config", "../../tests/allow-types-multiple-times.toml"],
    );
    assert_str_eq!(expected_output, actual_output);
}

// Make sure that the visitor doesn't attempt to visit the inner items of re-exported external types.
// Rustdoc doesn't include these inner items in its JSON output, which leads to obtuse crashes if they're
// referenced. It's also just the wrong behavior to look into the type being re-exported, since if it's
// approved, then it doesn't matter what it referenced. If it's not approved, then the re-export itself
// is the violation.
#[test]
fn test_reexports() {
    let expected_output = fs::read_to_string("tests/test-reexports-expected-output.md").unwrap();
    let actual_output = run_with_args("test-workspace/test-reexports-crate", &[]);
    assert_str_eq!(expected_output, actual_output);
}

#[test]
fn test_type_exported_from_hidden_module() {
    let expected_output =
        fs::read_to_string("tests/test-type-exported-from-hidden-module.md").unwrap();
    let actual_output = run_with_args("test-workspace/test-type-exported-from-hidden-module", &[]);
    assert_str_eq!(expected_output, actual_output);
}
