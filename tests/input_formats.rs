use std::io::Write;
use std::path::Path;

use assert_cmd::Command;
use tempfile::Builder;
use toon_world::cli::InputFormat;
use toon_world::{input, query};

fn query_one(value: jaq_json::Val, expression: &str) -> String {
    let values = query::execute(expression, value).unwrap();
    assert_eq!(values.len(), 1, "query: {expression}");
    values[0].to_string()
}

#[test]
fn detects_known_extensions_case_insensitively() {
    let cases = [
        ("data.json", InputFormat::Json),
        ("data.NDJSON", InputFormat::Ndjson),
        ("data.jsonl", InputFormat::Ndjson),
        ("data.csv", InputFormat::Csv),
        ("data.yml", InputFormat::Yaml),
        ("data.yaml", InputFormat::Yaml),
        ("data.toml", InputFormat::Toml),
        ("data.xml", InputFormat::Xml),
        ("data.xhtml", InputFormat::Xml),
    ];

    for (path, expected) in cases {
        assert_eq!(input::detect_format(Path::new(path)), Some(expected));
    }
    assert_eq!(input::detect_format(Path::new("data.unknown")), None);
}

#[test]
fn parses_ndjson_as_ordered_array() {
    let value = input::parse_bytes(
        b"{\"id\":1,\"name\":\"Ada\"}\n{\"id\":2,\"name\":\"Bob\"}\n",
        InputFormat::Ndjson,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), "length"), "2");
    assert_eq!(query_one(value, ".[1].name"), r#""Bob""#);
}

#[test]
fn ndjson_accepts_surrounding_whitespace_without_reordering() {
    let value = input::parse_bytes(
        b"  {\"id\":1}\n\n{\"id\":2}  \n",
        InputFormat::Ndjson,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".[0].id"), "1");
    assert_eq!(query_one(value, ".[1].id"), "2");
}

#[test]
fn csv_headers_become_fields_and_cells_remain_strings() {
    let value = input::parse_bytes(
        b"id,name,active,note\n001,Ada,true,\n002,Bob,false,hello\n",
        InputFormat::Csv,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".[0].id"), r#""001""#);
    assert_eq!(query_one(value.clone(), ".[0].active"), r#""true""#);
    assert_eq!(query_one(value.clone(), ".[0].note"), "\"\"");
    assert_eq!(query_one(value, ".[1].name"), r#""Bob""#);
}

#[test]
fn csv_preserves_quotes_commas_and_crlf_content() {
    let value = input::parse_bytes(
        b"id,note\r\n1,\"hello, \"\"toon\"\"\"\r\n",
        InputFormat::Csv,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".[0].id"), r#""1""#);
    assert_eq!(query_one(value, ".[0].note"), r#""hello, \"toon\"""#);
}

#[test]
fn csv_rejects_duplicate_or_empty_headers() {
    for bytes in [
        b"id,id\n1,2\n".as_slice(),
        b"id,\n1,2\n".as_slice(),
    ] {
        let error = input::parse_bytes(bytes, InputFormat::Csv)
            .expect_err("invalid CSV headers must fail")
            .to_string();
        assert!(error.contains("error[parse:csv]"));
    }
}

#[test]
fn yaml_preserves_native_scalar_types() {
    let value = input::parse_bytes(
        b"name: Ada\nactive: true\ncount: 2\nempty: null\n",
        InputFormat::Yaml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".name"), r#""Ada""#);
    assert_eq!(query_one(value.clone(), ".active"), "true");
    assert_eq!(query_one(value.clone(), ".count"), "2");
    assert_eq!(query_one(value, ".empty"), "null");
}

#[test]
fn yaml_resolves_anchors_and_aliases() {
    let value = input::parse_bytes(
        b"defaults: &defaults\n  active: true\n  retries: 3\ncopy: *defaults\n",
        InputFormat::Yaml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".copy.active"), "true");
    assert_eq!(query_one(value, ".copy.retries"), "3");
}

#[test]
fn multiple_yaml_documents_become_an_ordered_array() {
    let value = input::parse_bytes(b"---\nid: 1\n---\nid: 2\n", InputFormat::Yaml).unwrap();

    assert_eq!(query_one(value.clone(), "length"), "2");
    assert_eq!(query_one(value, ".[1].id"), "2");
}

#[test]
fn parses_toml_tables_and_arrays() {
    let value = input::parse_bytes(
        b"name = \"Ada\"\nactive = true\ntags = [\"rust\", \"toon\"]\n",
        InputFormat::Toml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".name"), r#""Ada""#);
    assert_eq!(query_one(value.clone(), ".active"), "true");
    assert_eq!(query_one(value, ".tags[1]"), r#""toon""#);
}

#[test]
fn toml_preserves_nested_table_structure() {
    let value = input::parse_bytes(
        b"[database]\nhost = \"localhost\"\nports = [5432, 5433]\n",
        InputFormat::Toml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".database.host"), r#""localhost""#);
    assert_eq!(query_one(value, ".database.ports[1]"), "5433");
}

#[test]
fn xml_preserves_tag_attribute_and_child_structure() {
    let value = input::parse_bytes(
        br#"<user id="1"><name>Ada</name></user>"#,
        InputFormat::Xml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".t"), r#""user""#);
    assert_eq!(query_one(value.clone(), ".a.id"), r#""1""#);
    assert_eq!(query_one(value.clone(), ".c[0].t"), r#""name""#);
    assert_eq!(query_one(value, ".c[0].c[0]"), r#""Ada""#);
}

#[test]
fn xml_preserves_mixed_content_order() {
    let value = input::parse_bytes(
        br#"<p>Hello <strong>world</strong>!</p>"#,
        InputFormat::Xml,
    )
    .unwrap();

    assert_eq!(query_one(value.clone(), ".c[0]"), r#""Hello ""#);
    assert_eq!(query_one(value.clone(), ".c[1].t"), r#""strong""#);
    assert_eq!(query_one(value.clone(), ".c[1].c[0]"), r#""world""#);
    assert_eq!(query_one(value, ".c[2]"), r#""!""#);
}

#[test]
fn multiple_xml_roots_become_an_ordered_array() {
    let value = input::parse_bytes(br#"<a>1</a><b>2</b>"#, InputFormat::Xml).unwrap();

    assert_eq!(query_one(value.clone(), "length"), "2");
    assert_eq!(query_one(value.clone(), ".[0].t"), r#""a""#);
    assert_eq!(query_one(value, ".[1].t"), r#""b""#);
}

#[test]
fn malformed_inputs_report_their_format() {
    let cases = [
        (InputFormat::Ndjson, b"{}\n{".as_slice(), "ndjson"),
        (InputFormat::Csv, b"a,b\n1\n".as_slice(), "csv"),
        (InputFormat::Yaml, b"a: [1".as_slice(), "yaml"),
        (InputFormat::Toml, b"a = [".as_slice(), "toml"),
        (InputFormat::Xml, b"<a></b>".as_slice(), "xml"),
    ];

    for (format, bytes, name) in cases {
        let error = input::parse_bytes(bytes, format)
            .expect_err("malformed input must fail")
            .to_string();
        assert!(error.contains(&format!("error[parse:{name}]")), "{error}");
    }
}

#[test]
fn text_formats_reject_invalid_utf8_with_format_error() {
    for (format, name) in [
        (InputFormat::Yaml, "yaml"),
        (InputFormat::Toml, "toml"),
        (InputFormat::Xml, "xml"),
    ] {
        let error = input::parse_bytes(&[0xff, 0xfe], format)
            .expect_err("invalid UTF-8 must fail")
            .to_string();
        assert!(error.contains(&format!("error[parse:{name}]")), "{error}");
    }
}

#[test]
fn file_extension_selects_yaml_adapter() {
    let mut file = Builder::new().suffix(".yaml").tempfile().unwrap();
    file.write_all(b"name: Ada\n").unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "-q",
            ".name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}

#[test]
fn unknown_extension_falls_back_to_json() {
    let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
    file.write_all(br#"{"name":"Ada"}"#).unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "-q",
            ".name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("warning[input]"));
}

#[test]
fn explicit_from_overrides_file_extension() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
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
}

#[test]
fn explicit_from_parses_csv_on_stdin() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["--from", "csv", "-q", ".[1].name", "--to", "text"])
        .write_stdin("id,name\n1,Ada\n2,Bob\n")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Bob\n");
}

#[test]
fn raw_csv_data_is_queryable_without_a_temp_file() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "csv",
            "--data",
            "id,name\n1,Ada\n2,Bob\n",
            "-q",
            ".[1].name",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Bob\n");
}

#[test]
fn raw_multiline_yaml_data_is_queryable() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "yaml",
            "--data",
            "name: Ada\nactive: true\n",
            "-q",
            ".active",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "true\n");
}

#[test]
fn raw_xml_data_is_queryable() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "xml",
            "--data",
            "<user id=\"1\"><name>Ada</name></user>",
            "-q",
            ".c[0].c[0]",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Ada\n");
}
