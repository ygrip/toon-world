use std::fs;
use std::path::PathBuf;

use toon_world::cli::InputFormat;
use toon_world::{input, query};

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name),
    )
    .unwrap()
}

fn one(bytes: &[u8], format: InputFormat, expression: &str) -> String {
    let value = input::parse_bytes(bytes, format).unwrap();
    let values = query::execute(expression, value).unwrap();
    assert_eq!(values.len(), 1, "query: {expression}");
    values[0].to_string()
}

#[test]
fn parses_checked_in_structured_samples() {
    assert_eq!(
        one(&fixture("users.json"), InputFormat::Json, ".users[0].name"),
        r#""Ada Lovelace""#
    );
    assert_eq!(
        one(
            &fixture("events.ndjson"),
            InputFormat::Ndjson,
            ".[2].success"
        ),
        "false"
    );
    assert_eq!(
        one(&fixture("users.csv"), InputFormat::Csv, ".[0].id"),
        r#""001""#
    );
    assert_eq!(
        one(
            &fixture("services.yaml"),
            InputFormat::Yaml,
            ".service.replicas"
        ),
        "3"
    );
    assert_eq!(
        one(&fixture("package.toml"), InputFormat::Toml, ".package.name"),
        r#""toon-world-sample""#
    );
    assert_eq!(
        one(&fixture("fragment.xml"), InputFormat::Xml, ".[1].a.id"),
        r#""102""#
    );
    assert_eq!(
        one(&fixture("users.toon"), InputFormat::Toon, ".users[1].name"),
        r#""Bob""#
    );
}

#[test]
fn parses_checked_in_document_samples() {
    assert_eq!(
        one(&fixture("guide.md"), InputFormat::Markdown, ".title"),
        r#""Sample guide""#
    );
    let html =
        input::parse_bytes_with_options(&fixture("page.html"), InputFormat::Html, true).unwrap();
    assert_eq!(
        query::execute(".title", html).unwrap()[0].to_string(),
        r#""Sample page""#
    );
}
