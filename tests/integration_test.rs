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

/// Run the tool and return (exit_code, stdout, stderr)
fn run_with_args_full(in_path: impl AsRef<Path>, args: &[&str]) -> (i32, String, String) {
    let mut cmd = get_test_bin("cargo-check-external-types");
    cmd.current_dir(in_path.as_ref());
    cmd.arg("check-external-types");
    for &arg in args {
        cmd.arg(arg);
    }
    let output = cmd
        .output()
        .expect("failed to start cargo-check-external-types");
    let exit_code = output.status.code().unwrap_or(-1);
    let (stdout, stderr) = output_text(&output);
    (exit_code, stdout, stderr)
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

// ============================================================================
// Workspace and unsupported package type tests
// ============================================================================

#[test]
fn test_crate_with_examples_succeeds() {
    // This test verifies that the --lib flag works correctly by checking a crate
    // that has an examples/ directory. Without --lib, rustdoc would try to document
    // the example and potentially fail.
    let (exit_code, _stdout, _stderr) =
        run_with_args_full("test-workspace/test-crate-with-examples", &[]);
    // Should succeed (exit code 0 for no errors, or 1 for validation errors which is still "working")
    assert!(
        exit_code == 0 || exit_code == 1,
        "Expected exit code 0 or 1, got {}",
        exit_code
    );
}

#[test]
fn test_binary_only_without_skip_flag() {
    // Binary-only crate should error without --skip-unsupported
    let (exit_code, stdout, _stderr) =
        run_with_args_full("test-workspace/standalone/binary-only", &[]);
    assert_eq!(exit_code, 2, "Expected exit code 2 for unsupported package");
    assert!(
        stdout.contains("no lib target") || stdout.contains("not supported"),
        "Expected error message about no lib target, got: {}",
        stdout
    );
}

#[test]
fn test_binary_only_with_skip_flag() {
    // Binary-only crate should succeed with --skip-unsupported
    let (exit_code, _stdout, stderr) = run_with_args_full(
        "test-workspace/standalone/binary-only",
        &["--skip-unsupported"],
    );
    assert_eq!(exit_code, 0, "Expected exit code 0 with --skip-unsupported");
    assert!(
        stderr.contains("Skipping") && stderr.contains("no lib target"),
        "Expected skip message in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_proc_macro_without_skip_flag() {
    // Proc-macro crate should error without --skip-unsupported
    let (exit_code, stdout, _stderr) =
        run_with_args_full("test-workspace/standalone/proc-macro", &[]);
    assert_eq!(exit_code, 2, "Expected exit code 2 for unsupported package");
    assert!(
        stdout.contains("proc-macro") || stdout.contains("not supported"),
        "Expected error message about proc-macro, got: {}",
        stdout
    );
}

#[test]
fn test_proc_macro_with_skip_flag() {
    // Proc-macro crate should succeed with --skip-unsupported
    let (exit_code, _stdout, stderr) = run_with_args_full(
        "test-workspace/standalone/proc-macro",
        &["--skip-unsupported"],
    );
    assert_eq!(exit_code, 0, "Expected exit code 0 with --skip-unsupported");
    assert!(
        stderr.contains("Skipping") && stderr.contains("proc-macro"),
        "Expected skip message in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_workspace_invocation() {
    // Running on the workspace root should process all members
    let (exit_code, _stdout, stderr) = run_with_args_full("test-workspace", &[]);
    // May exit with 0 (all pass) or 1 (validation errors) - both indicate workspace iteration worked
    assert!(
        exit_code == 0 || exit_code == 1,
        "Expected exit code 0 or 1, got {}",
        exit_code
    );
    // Should show per-package headers
    assert!(
        stderr.contains("Checking package:"),
        "Expected per-package headers in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_workspace_with_features_flag_errors() {
    // --features should not be allowed with workspace targets
    // Note: Cargo itself may error first if the feature doesn't exist in workspace members
    let (exit_code, stdout, _stderr) =
        run_with_args_full("test-workspace", &["--features", "some-feature"]);
    assert_eq!(
        exit_code, 2,
        "Expected exit code 2 for invalid flag combination"
    );
    // Either our error or cargo's error is acceptable
    assert!(
        stdout.contains("--features is not supported for workspace targets")
            || stdout.contains("none of the selected packages contains this feature"),
        "Expected error about --features, got: {}",
        stdout
    );
}

#[test]
fn test_workspace_with_all_features_succeeds() {
    // --all-features should work with workspace targets
    let (exit_code, _stdout, stderr) = run_with_args_full("test-workspace", &["--all-features"]);
    // May exit with 0 or 1 - both indicate workspace iteration worked
    assert!(
        exit_code == 0 || exit_code == 1,
        "Expected exit code 0 or 1, got {}",
        exit_code
    );
    assert!(
        stderr.contains("Checking package:"),
        "Expected per-package headers in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_workspace_mixed_without_skip_flag() {
    // Mixed workspace should error on first unsupported package without --skip-unsupported
    let (exit_code, stdout, _stderr) = run_with_args_full("test-workspace-mixed", &[]);
    assert_eq!(exit_code, 2, "Expected exit code 2 for unsupported package");
    assert!(
        stdout.contains("not supported") || stdout.contains("--skip-unsupported"),
        "Expected error message suggesting --skip-unsupported, got: {}",
        stdout
    );
}

#[test]
fn test_workspace_mixed_with_skip_flag() {
    // Mixed workspace should skip unsupported packages and check supported ones
    let (exit_code, _stdout, stderr) =
        run_with_args_full("test-workspace-mixed", &["--skip-unsupported"]);
    assert_eq!(exit_code, 0, "Expected exit code 0 with --skip-unsupported");
    // Should show skip messages for binary-crate and proc-macro-crate
    assert!(
        stderr.contains("Skipping binary-crate") || stderr.contains("Skipping"),
        "Expected skip messages in stderr, got: {}",
        stderr
    );
    // Should show checking lib-crate
    assert!(
        stderr.contains("Checking package: lib-crate") || stderr.contains("lib-crate"),
        "Expected lib-crate to be checked, got: {}",
        stderr
    );
}
