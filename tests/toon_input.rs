use std::io::Write;

use assert_cmd::Command;
use serde_json::json;
use tempfile::Builder;
use toon_world::cli::{InputFormat, OutputFormat};
use toon_world::{input, output, query};

fn round_trip_through_toon(original: serde_json::Value) -> serde_json::Value {
    let encoded = toon_format::encode_default(&original).unwrap();
    let parsed = input::parse_bytes(encoded.as_bytes(), InputFormat::Toon).unwrap();
    let rendered = output::encode_results(&[parsed], OutputFormat::Json).unwrap();
    serde_json::from_str(&rendered).unwrap()
}

#[test]
fn detects_toon_extension_case_insensitively() {
    for path in ["data.toon", "DATA.TOON"] {
        assert_eq!(
            input::detect_format(std::path::Path::new(path)),
            Some(InputFormat::Toon)
        );
    }
}

#[test]
fn parses_toon_table_into_queryable_values() {
    let value = input::parse_bytes(
        b"users[2]{id,name}:\n  1,Ada\n  2,Bob",
        InputFormat::Toon,
    )
    .unwrap();
    let result = query::execute(".users[1].name", value).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].to_string(), r#""Bob""#);
}

#[test]
fn toon_query_can_emit_multiple_results_in_order() {
    let original = json!({"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"},{"id":3,"name":"Cid"}]});
    let encoded = toon_format::encode_default(&original).unwrap();
    let value = input::parse_bytes(encoded.as_bytes(), InputFormat::Toon).unwrap();
    let results = query::execute(".users[] | .name", value).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), r#""Ada""#);
    assert_eq!(results[1].to_string(), r#""Bob""#);
    assert_eq!(results[2].to_string(), r#""Cid""#);
}

#[test]
fn toon_round_trip_preserves_absent_null_and_empty_string() {
    let original = json!({"items":[{"id":1,"note":null,"empty":""},{"id":2,"empty":""}],"active":true});
    assert_eq!(round_trip_through_toon(original.clone()), original);
}

#[test]
fn toon_round_trip_preserves_nested_and_heterogeneous_values() {
    let original = json!({"meta":{"name":"toon, world","enabled":true},"items":[{"type":"github","repo":"punakawan","pr":34},{"type":"jira","key":"ABC-1"},[1,2,3],"tail"]});
    assert_eq!(round_trip_through_toon(original.clone()), original);
}

#[test]
fn toon_round_trip_preserves_empty_containers() {
    let original = json!({"object":{},"array":[],"nested":{"items":[]}});
    assert_eq!(round_trip_through_toon(original.clone()), original);
}

#[test]
fn malformed_toon_reports_toon_parse_error() {
    let error = input::parse_bytes(b"items[2]: one", InputFormat::Toon)
        .expect_err("declared length mismatch must fail")
        .to_string();
    assert!(error.contains("error[parse:toon]"));
}

#[test]
fn invalid_utf8_reports_toon_parse_error() {
    let error = input::parse_bytes(&[0xff, 0xfe], InputFormat::Toon)
        .expect_err("invalid UTF-8 must fail")
        .to_string();
    assert!(error.contains("error[parse:toon]"));
}

#[test]
fn toon_file_extension_selects_decoder() {
    let mut file = Builder::new().suffix(".toon").tempfile().unwrap();
    file.write_all(b"name: Ada\nactive: true").unwrap();
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([file.path().to_str().unwrap(), "-q", ".name", "--to", "text"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}

#[test]
fn explicit_toon_input_works_from_stdin() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["--from", "toon", "-q", ".active", "--to", "text"])
        .write_stdin("name: Ada\nactive: true")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "true\n");
}

#[test]
fn raw_toon_input_is_queryable_without_a_temp_file() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "toon",
            "--data",
            "users[2]{id,name}:\n  1,Ada\n  2,Bob",
            "-q",
            ".users[1].name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Bob\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_toon_input_overrides_misleading_extension() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(b"name: Ada\nactive: true").unwrap();
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([file.path().to_str().unwrap(), "--from", "toon", "-q", ".name", "--to", "text"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}
