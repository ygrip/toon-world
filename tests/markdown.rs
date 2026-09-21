use std::io::Write;

use assert_cmd::Command;
use tempfile::Builder;
use toon_world::cli::InputFormat;
use toon_world::{input, markdown, query};

fn one(value: jaq_json::Val, expression: &str) -> String {
    let values = query::execute(expression, value).unwrap();
    assert_eq!(values.len(), 1, "query: {expression}");
    values[0].to_string()
}

fn sample() -> &'static str {
    r#"---
title: toon-world
owner: ygrip
---

# toon-world

A fast **query** tool with [GitHub](https://github.com/ygrip/toon-world).

## Installation

Install it locally.

```bash
cargo install --path .
```

## Features

- JSON
- CSV
- TOON

| Format | Queryable |
| --- | --- |
| JSON | yes |
| TOON | yes |
"#
}

#[test]
fn detects_markdown_extensions_case_insensitively() {
    for path in ["README.md", "guide.markdown", "NOTES.MD"] {
        assert_eq!(
            input::detect_format(std::path::Path::new(path)),
            Some(InputFormat::Markdown)
        );
    }
}

#[test]
fn normalizes_title_sections_and_paragraphs() {
    let value = markdown::parse(sample()).unwrap();

    assert_eq!(one(value.clone(), ".type"), r#""markdown""#);
    assert_eq!(one(value.clone(), ".title"), r#""toon-world""#);
    assert_eq!(
        one(value.clone(), ".sections[0].heading"),
        r#""toon-world""#
    );
    assert_eq!(one(value.clone(), ".sections[0].level"), "1");
    assert_eq!(one(value, ".sections[1].heading"), r#""Installation""#);
}

#[test]
fn keeps_preamble_before_first_heading_as_section() {
    let value = markdown::parse("Intro paragraph.\n\n# Title\n\nBody.\n").unwrap();

    assert_eq!(one(value.clone(), ".sections[0].heading"), "null");
    assert_eq!(one(value.clone(), ".sections[0].level"), "0");
    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].text"),
        r#""Intro paragraph.""#
    );
    assert_eq!(one(value, ".sections[1].heading"), r#""Title""#);
}

#[test]
fn title_is_null_when_document_has_no_h1() {
    let value = markdown::parse("## Usage\n\nRun it.\n").unwrap();

    assert_eq!(one(value, ".title"), "null");
}

#[test]
fn captures_frontmatter_without_interpreting_its_schema() {
    let value = markdown::parse(sample()).unwrap();
    let frontmatter = one(value, ".frontmatter");

    assert!(frontmatter.contains("title: toon-world"));
    assert!(frontmatter.contains("owner: ygrip"));
}

#[test]
fn section_helper_returns_matching_section() {
    let value = markdown::parse(sample()).unwrap();

    assert_eq!(
        one(value, r#"section("Installation") | .heading"#),
        r#""Installation""#
    );
}

#[test]
fn section_helper_returns_all_duplicate_headings_in_order() {
    let value = markdown::parse("## Note\n\nFirst.\n\n## Note\n\nSecond.\n").unwrap();
    let values = query::execute(r#"section("Note") | .blocks[0].text"#, value).unwrap();

    assert_eq!(values.len(), 2);
    assert_eq!(values[0].to_string(), r#""First.""#);
    assert_eq!(values[1].to_string(), r#""Second.""#);
}

#[test]
fn missing_section_helper_returns_empty_stream() {
    let value = markdown::parse(sample()).unwrap();
    let values = query::execute(r#"section("Missing")"#, value).unwrap();

    assert!(values.is_empty());
}

#[test]
fn code_helper_filters_by_language() {
    let value = markdown::parse(sample()).unwrap();

    assert_eq!(
        one(value, r#"section("Installation") | code("bash") | .text"#),
        r#""cargo install --path .""#
    );
}

#[test]
fn code_block_without_language_uses_null_language() {
    let value = markdown::parse("# Demo\n\n```\necho hi\n```\n").unwrap();

    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].type"),
        r#""code""#
    );
    assert_eq!(one(value.clone(), ".sections[0].blocks[0].lang"), "null");
    assert_eq!(one(value, ".sections[0].blocks[0].text"), r#""echo hi""#);
}

#[test]
fn collects_document_links_and_titles() {
    let value =
        markdown::parse("# Demo\n\n[GitHub](https://github.com/ygrip/toon-world \"Source\")\n")
            .unwrap();

    assert_eq!(one(value.clone(), ".links | length"), "1");
    assert_eq!(one(value.clone(), ".links[0].text"), r#""GitHub""#);
    assert_eq!(
        one(value.clone(), ".links[0].href"),
        r#""https://github.com/ygrip/toon-world""#
    );
    assert_eq!(one(value, ".links[0].title"), r#""Source""#);
}

#[test]
fn inline_formatting_is_flattened_to_text_meaning() {
    let value = markdown::parse("# Demo\n\nUse **bold**, *italic*, and `code`.\n").unwrap();

    assert_eq!(
        one(value, ".sections[0].blocks[0].text"),
        r#""Use bold, italic, and code.""#
    );
}

#[test]
fn normalizes_lists_and_tables_as_typed_blocks() {
    let value = markdown::parse(sample()).unwrap();

    assert_eq!(
        one(
            value.clone(),
            r#"section("Features") | .blocks[] | select(.type == "list") | .items[1]"#,
        ),
        r#""CSV""#
    );
    assert_eq!(
        one(
            value,
            r#"section("Features") | .blocks[] | select(.type == "table") | .rows[1][0]"#,
        ),
        r#""TOON""#
    );
}

#[test]
fn preserves_task_list_markers() {
    let value = markdown::parse("# Tasks\n\n- [x] done\n- [ ] pending\n").unwrap();

    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].items[0]"),
        r#""[x] done""#
    );
    assert_eq!(
        one(value, ".sections[0].blocks[0].items[1]"),
        r#""[ ] pending""#
    );
}

#[test]
fn normalizes_blockquote_rule_and_raw_html_blocks() {
    let value =
        markdown::parse("# Demo\n\n> quoted text\n\n---\n\n<div data-x=\"1\">raw</div>\n").unwrap();

    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].type"),
        r#""blockquote""#
    );
    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].text"),
        r#""quoted text""#
    );
    assert_eq!(
        one(value.clone(), ".sections[0].blocks[1].type"),
        r#""rule""#
    );
    assert_eq!(
        one(value.clone(), ".sections[0].blocks[2].type"),
        r#""html""#
    );
    assert!(one(value, ".sections[0].blocks[2].text").contains("data-x"));
}

#[test]
fn preserves_section_order_and_heading_levels() {
    let value = markdown::parse("# A\n\ntext\n\n### C\n\nmore\n\n## B\n").unwrap();

    assert_eq!(
        one(value.clone(), ".sections | map(.heading)"),
        r#"["A","C","B"]"#
    );
    assert_eq!(one(value, ".sections | map(.level)"), "[1,3,2]");
}

#[test]
fn markdown_file_extension_selects_adapter() {
    let mut file = Builder::new().suffix(".md").tempfile().unwrap();
    file.write_all(b"# Demo\n\n## Usage\n\nRun it.\n").unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "-q",
            r#"section("Usage") | .heading"#,
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Usage\n");
}

#[test]
fn explicit_markdown_input_works_from_stdin() {
    let mut command = Command::cargo_bin("toon-world").unwrap();
    let output = command
        .args(["--from", "markdown", "-q", ".title", "--to", "text"])
        .write_stdin("# Demo\n")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Demo\n");
}

#[test]
fn explicit_markdown_input_overrides_misleading_extension() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(b"# Demo\n").unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--from",
            "markdown",
            "-q",
            ".title",
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Demo\n");
}

#[test]
fn invalid_utf8_reports_markdown_parse_error() {
    let error = input::parse_bytes(&[0xff, 0xfe], InputFormat::Markdown)
        .expect_err("invalid UTF-8 must fail")
        .to_string();

    assert!(error.contains("error[parse:markdown]"));
}
