use std::io::Write;

use assert_cmd::Command;
use tempfile::Builder;
use toon_world::diagnostics::{resolve_warnings, Warning};

#[test]
fn warning_has_stable_category_format() {
    let warning = Warning::new("input", "unknown extension '.txt'; assuming JSON");

    assert_eq!(
        warning.to_string(),
        "warning[input]: unknown extension '.txt'; assuming JSON"
    );
}

#[test]
fn quiet_suppresses_and_warnings_as_errors_escalates() {
    let warnings = vec![Warning::new("input", "fallback")];

    assert_eq!(resolve_warnings(&warnings, true, false).unwrap(), "");
    assert!(resolve_warnings(&warnings, false, true)
        .expect_err("warning must be escalated")
        .to_string()
        .contains("error[warning-as-error]"));
}

#[test]
fn unknown_extension_warns_but_keeps_stdout_clean() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(br#"{"name":"Ada"}"#).unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([file.path().to_str().unwrap(), "-q", ".name", "--to", "text"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("warning[input]"));
    assert!(stderr.contains("assuming JSON"));
}

#[test]
fn quiet_suppresses_unknown_extension_warning() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(br#"{"name":"Ada"}"#).unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--quiet",
            "-q",
            ".name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn warnings_as_errors_rejects_unknown_extension_before_query_output() {
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
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[warning-as-error]"));
    assert!(stderr.contains("warning[input]"));
}

#[test]
fn explicit_from_avoids_extension_warning() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(b"name: Ada\n").unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--from",
            "yaml",
            "-q",
            ".name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
    assert!(output.stderr.is_empty());
}
