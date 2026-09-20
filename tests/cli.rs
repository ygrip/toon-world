use std::io::Write;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::NamedTempFile;

fn input_file(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    file
}

fn run_stdin(args: &[&str], input: &str) -> std::process::Output {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    command.args(args).write_stdin(input).output().unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn queries_a_checked_in_json_sample_file() {
    let file = fixture("users.json");
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.to_str().unwrap(),
            "-q",
            ".users[] | select(.active) | .name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Ada Lovelace\nLinus Torvalds\n"
    );
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
fn queries_filters_and_projects_json() {
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
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "{\"id\":1,\"name\":\"Ada\"}\n"
    );
}

#[test]
fn reads_json_from_raw_data_argument() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--data",
            r#"{"name":"Ada","active":true}"#,
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
fn raw_data_supports_full_query_pipeline() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--data",
            r#"{"users":[{"id":1,"active":true},{"id":2,"active":false}]}"#,
            "-q",
            ".users[] | select(.active) | .id",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "1\n");
}

#[test]
fn malformed_raw_data_reports_json_parse_error() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args(["--data", "{"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[parse:json]"));
}

#[test]
fn reads_json_from_stdin_when_file_is_omitted() {
    let output = run_stdin(&["-q", ".name", "--to", "text"], r#"{"name":"Ada"}"#);

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}

#[test]
fn dash_explicitly_selects_stdin() {
    let output = run_stdin(&["-", "-q", ".id", "--to", "text"], r#"{"id":42}"#);

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn emits_multiple_text_results_line_by_line() {
    let output = run_stdin(&["-q", ".[]", "--to", "text"], r#"["Ada","Bob"]"#);

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\nBob\n");
}

#[test]
fn empty_text_result_stream_emits_no_output() {
    let output = run_stdin(&["-q", ".[] | select(. > 10)", "--to", "text"], "[1,2,3]");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn emits_empty_array_for_empty_structured_result_stream() {
    let output = run_stdin(&["-q", ".[] | select(. > 10)", "--to", "json"], "[1,2,3]");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[]\n");
}

#[test]
fn rejects_structured_values_in_text_mode() {
    let output = run_stdin(&["--to", "text"], r#"{"name":"Ada"}"#);

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("text output requires scalar"));
}

#[test]
fn preserves_string_content_in_text_mode() {
    let output = run_stdin(
        &["-q", ".message", "--to", "text"],
        r#"{"message":"hello, world: \"toon\""}"#,
    );

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "hello, world: \"toon\"\n"
    );
}

#[test]
fn reports_missing_input_file() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .arg("/definitely/missing/toon-world-input.json")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[input]"));
}

#[test]
fn reports_malformed_json() {
    let output = run_stdin(&[], "{");

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[parse:json]"));
}

#[test]
fn reports_malformed_query() {
    let output = run_stdin(&["-q", ".[", "--to", "json"], "{}");

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[query]"));
}

#[test]
fn reports_runtime_query_error() {
    let output = run_stdin(&["-q", ".missing[]", "--to", "json"], "{}");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[query]"));
    assert!(stderr.contains("runtime"));
}
