use std::io::Write;

use assert_cmd::Command;
use tempfile::Builder;
use toon_world::diagnostics::{resolve_warnings, Warning};

#[test]
fn warning_policy_supports_suppress_and_escalate() {
    let warnings = vec![Warning::new("input", "fallback")];
    assert_eq!(resolve_warnings(&warnings, true, false).unwrap(), "");
    assert!(resolve_warnings(&warnings, false, true).is_err());
}

#[test]
fn unknown_extension_warning_does_not_pollute_stdout() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(br#"{"name":"Ada"}"#).unwrap();
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([file.path().to_str().unwrap(), "-q", ".name", "--to", "text"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("warning[input]"));
}

#[test]
fn warnings_as_errors_blocks_result_output() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(br#"{"name":"Ada"}"#).unwrap();
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--warnings-as-errors",
            "-q",
            ".name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[warning-as-error]"));
}
