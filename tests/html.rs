use std::io::Write;

use assert_cmd::Command;
use tempfile::Builder;
use toon_world::cli::InputFormat;
use toon_world::{html, input, query};

fn one(value: jaq_json::Val, expression: &str) -> String {
    let values = query::execute(expression, value).unwrap();
    assert_eq!(values.len(), 1, "query: {expression}");
    values[0].to_string()
}

fn sample() -> &'static str {
    r#"<!doctype html>
<html lang="en">
<head>
  <title>toon-world Demo</title>
  <meta name="description" content="Query anything">
  <meta property="og:type" content="website">
  <style>.hidden { display:none }</style>
</head>
<body>
  <nav><a href="/home">Home</a></nav>
  <main>
    <h1>toon-world</h1>
    <p>A <strong>fast</strong> query tool.</p>
    <h2>Usage</h2>
    <pre><code class="language-bash">cargo run -- --help</code></pre>
    <ul><li>JSON</li><li>TOON</li></ul>
    <ol start="3"><li>third</li><li>fourth</li></ol>
    <table>
      <tr><th>Format</th><th>Queryable</th></tr>
      <tr><td>JSON</td><td>yes</td></tr>
      <tr><td>TOON</td><td>yes</td></tr>
    </table>
    <a href="https://github.com/ygrip/toon-world" title="Source">GitHub</a>
    <img src="logo.png" alt="toon-world logo" title="Logo">
    <form action="/search" method="post">
      <label for="q">Query</label>
      <input id="q" name="q" type="text" placeholder="Search">
    </form>
    <script>window.tracking = true;</script>
  </main>
</body>
</html>"#
}

#[test]
fn detects_html_extensions_case_insensitively() {
    for path in ["index.html", "page.htm", "INDEX.HTML"] {
        assert_eq!(
            input::detect_format(std::path::Path::new(path)),
            Some(InputFormat::Html)
        );
    }
}

#[test]
fn structural_mode_preserves_dom_shape_and_attributes() {
    let value = html::parse(sample(), false).unwrap();

    assert_eq!(one(value.clone(), ".type"), r#""html""#);
    assert_eq!(one(value.clone(), ".mode"), r#""structural""#);
    assert_eq!(one(value.clone(), ".root.tag"), r#""html""#);
    assert_eq!(one(value, ".root.attrs.lang"), r#""en""#);
}

#[test]
fn structural_mode_preserves_text_comment_and_element_order() {
    let value = html::parse(
        "<html><body><!--marker-->hello<span data-x=\"1\">world</span>tail</body></html>",
        false,
    )
    .unwrap();
    let body = r#".root.children[] | select(.tag == "body")"#;

    assert_eq!(
        one(value.clone(), &format!("{body} | .children[0].comment")),
        r#""marker""#
    );
    assert_eq!(
        one(value.clone(), &format!("{body} | .children[1].text")),
        r#""hello""#
    );
    assert_eq!(
        one(value.clone(), &format!("{body} | .children[2].tag")),
        r#""span""#
    );
    assert_eq!(
        one(
            value.clone(),
            &format!("{body} | .children[2].attrs.\"data-x\"")
        ),
        r#""1""#
    );
    assert_eq!(
        one(value, &format!("{body} | .children[3].text")),
        r#""tail""#
    );
}

#[test]
fn structural_mode_keeps_script_and_style_elements() {
    let value = html::parse(sample(), false).unwrap();
    let script = query::execute(
        r#".root.children[] | select(.tag == "body") | .children[] | select(.tag == "main") | .children[] | select(.tag == "script") | .children[0].text"#,
        value.clone(),
    )
    .unwrap();
    let style = query::execute(
        r#".root.children[] | select(.tag == "head") | .children[] | select(.tag == "style") | .children[0].text"#,
        value,
    )
    .unwrap();

    assert_eq!(script.len(), 1);
    assert!(script[0].to_string().contains("window.tracking"));
    assert_eq!(style.len(), 1);
    assert!(style[0].to_string().contains("display:none"));
}

#[test]
fn semantic_mode_extracts_title_metadata_and_sections() {
    let value = html::parse(sample(), true).unwrap();

    assert_eq!(one(value.clone(), ".mode"), r#""semantic""#);
    assert_eq!(one(value.clone(), ".title"), r#""toon-world Demo""#);
    assert_eq!(one(value.clone(), ".metadata[0].key"), r#""description""#);
    assert_eq!(one(value.clone(), ".metadata[1].key"), r#""og:type""#);
    assert_eq!(
        one(value.clone(), ".sections[0].heading"),
        r#""toon-world""#
    );
    assert_eq!(one(value, ".sections[1].heading"), r#""Usage""#);
}

#[test]
fn semantic_mode_keeps_preamble_and_heading_levels() {
    let value = html::parse(
        "<html><body><p>Intro.</p><h1>Title</h1><h3>Deep</h3><p>Body.</p></body></html>",
        true,
    )
    .unwrap();

    assert_eq!(one(value.clone(), ".sections[0].heading"), "null");
    assert_eq!(one(value.clone(), ".sections[0].level"), "0");
    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].text"),
        r#""Intro.""#
    );
    assert_eq!(one(value.clone(), ".sections[1].level"), "1");
    assert_eq!(one(value, ".sections[2].level"), "3");
}

#[test]
fn semantic_mode_uses_null_title_when_missing() {
    let value = html::parse("<html><body><h1>Body heading</h1></body></html>", true).unwrap();

    assert_eq!(one(value, ".title"), "null");
}

#[test]
fn semantic_mode_discards_script_and_style_payloads() {
    let value = html::parse(sample(), true).unwrap();
    let rendered = value.to_string();

    assert!(!rendered.contains("window.tracking"));
    assert!(!rendered.contains("display:none"));
}

#[test]
fn section_and_code_helpers_work_for_semantic_html() {
    let value = html::parse(sample(), true).unwrap();

    assert_eq!(
        one(value, r#"section("Usage") | code("bash") | .text"#),
        r#""cargo run -- --help""#
    );
}

#[test]
fn semantic_code_without_language_uses_null() {
    let value = html::parse(
        "<html><body><h1>Demo</h1><pre><code>echo hi</code></pre></body></html>",
        true,
    )
    .unwrap();

    assert_eq!(
        one(value.clone(), ".sections[0].blocks[0].type"),
        r#""code""#
    );
    assert_eq!(one(value.clone(), ".sections[0].blocks[0].lang"), "null");
    assert_eq!(one(value, ".sections[0].blocks[0].text"), r#""echo hi""#);
}

#[test]
fn semantic_mode_normalizes_lists_and_tables() {
    let value = html::parse(sample(), true).unwrap();

    assert_eq!(
        one(
            value.clone(),
            r#"section("Usage") | .blocks[] | select(.type == "list" and .ordered == false) | .items[1]"#,
        ),
        r#""TOON""#
    );
    assert_eq!(
        one(
            value.clone(),
            r#"section("Usage") | .blocks[] | select(.type == "list" and .ordered == true) | .start"#,
        ),
        "3"
    );
    assert_eq!(
        one(
            value,
            r#"section("Usage") | .blocks[] | select(.type == "table") | .rows[1][0]"#,
        ),
        r#""TOON""#
    );
}

#[test]
fn semantic_mode_indexes_links_images_and_forms() {
    let value = html::parse(sample(), true).unwrap();

    assert_eq!(one(value.clone(), ".links | length"), "2");
    assert_eq!(
        one(value.clone(), ".links[1].href"),
        r#""https://github.com/ygrip/toon-world""#
    );
    assert_eq!(one(value.clone(), ".links[1].title"), r#""Source""#);
    assert_eq!(one(value.clone(), ".images[0].alt"), r#""toon-world logo""#);
    assert_eq!(one(value.clone(), ".images[0].title"), r#""Logo""#);
    assert_eq!(one(value.clone(), ".forms[0].method"), r#""post""#);
    assert_eq!(one(value.clone(), ".forms[0].fields[0].name"), r#""q""#);
    assert_eq!(one(value, ".forms[0].labels[0].text"), r#""Query""#);
}

#[test]
fn semantic_forms_default_to_get_and_capture_multiple_field_types() {
    let value = html::parse(
        "<html><body><form><select name=\"kind\"><option>A</option></select><textarea name=\"body\"></textarea><button name=\"go\">Go</button></form></body></html>",
        true,
    )
    .unwrap();

    assert_eq!(one(value.clone(), ".forms[0].method"), r#""get""#);
    assert_eq!(one(value.clone(), ".forms[0].fields[0].tag"), r#""select""#);
    assert_eq!(
        one(value.clone(), ".forms[0].fields[1].tag"),
        r#""textarea""#
    );
    assert_eq!(one(value, ".forms[0].fields[2].tag"), r#""button""#);
}

#[test]
fn links_helper_works_for_semantic_html() {
    let value = html::parse(sample(), true).unwrap();
    let values = query::execute(r#"links | .href"#, value).unwrap();

    assert_eq!(values.len(), 2);
    assert_eq!(values[0].to_string(), r#""/home""#);
    assert_eq!(
        values[1].to_string(),
        r#""https://github.com/ygrip/toon-world""#
    );
}

#[test]
fn cli_defaults_html_to_structural_mode() {
    let mut file = Builder::new().suffix(".html").tempfile().unwrap();
    file.write_all(b"<html><body><p>Hello</p></body></html>")
        .unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([file.path().to_str().unwrap(), "-q", ".mode", "--to", "text"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "structural\n");
}

#[test]
fn semantic_flag_is_applied_through_cli() {
    let mut file = Builder::new().suffix(".html").tempfile().unwrap();
    file.write_all(sample().as_bytes()).unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--semantic",
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
fn explicit_html_input_overrides_misleading_extension() {
    let mut file = Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(b"<html><head><title>Demo</title></head><body></body></html>")
        .unwrap();

    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            file.path().to_str().unwrap(),
            "--from",
            "html",
            "--semantic",
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
fn invalid_utf8_reports_html_parse_error() {
    let error = input::parse_bytes(&[0xff, 0xfe], InputFormat::Html)
        .expect_err("invalid UTF-8 must fail")
        .to_string();

    assert!(error.contains("error[parse:html]"));
}
