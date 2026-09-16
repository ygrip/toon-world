use std::io::Write;

use assert_cmd::Command;
use tempfile::NamedTempFile;

fn input_file(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    file
}

#[test]
fn converts_json_file_to_toon_by_default() {
    let file = input_file(r#"{"name":"Ada","active":true}"#);
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .arg(file.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("name: Ada"));
    assert!(stdout.contains("active: true"));
}

#[test]
fn queries_and_projects_json() {
    let file = input_file(
        r#"{"users":[{"id":1,"name":"Ada","active":true},{"id":2,"name":"Bob","active":false}]}"#,
    );
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "-q",
            ".users[] | select(.active) | {id,name}",
            "--to",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "{\"id\":1,\"name\":\"Ada\"}\n");
}

#[test]
fn reads_json_from_stdin() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["-q", ".name", "--to", "text"])
        .write_stdin(r#"{"name":"Ada"}"#)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}

#[test]
fn emits_multiple_text_results_line_by_line() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["-q", ".[]", "--to", "text"])
        .write_stdin(r#"["Ada","Bob"]"#)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\nBob\n");
}

#[test]
fn reports_malformed_json() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command.write_stdin("{").output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[parse:json]"));
}

#[test]
fn reports_malformed_query() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["-q", ".[", "--to", "json"])
        .write_stdin("{}")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[query]"));
}
