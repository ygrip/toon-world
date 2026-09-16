use jaq_json::Val;
use toon_world::cli::OutputFormat;
use toon_world::output;

fn json(input: &str) -> Val {
    jaq_json::read::parse_single(input.as_bytes()).expect("valid test JSON")
}

#[test]
fn encodes_uniform_data_as_toon() {
    let value = json(r#"{"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}]}"#);
    let rendered = output::encode_results(&[value], OutputFormat::Toon).unwrap();

    assert!(rendered.contains("users[2]{id,name}:"));
    assert!(rendered.contains("1,Ada"));
    assert!(rendered.contains("2,Bob"));
}

#[test]
fn encodes_compact_json() {
    let value = json(r#"{"name":"Ada","active":true}"#);
    let rendered = output::encode_results(&[value], OutputFormat::Json).unwrap();

    assert_eq!(rendered, r#"{"name":"Ada","active":true}"#);
}

#[test]
fn text_output_emits_scalar_results_line_by_line() {
    let values = vec![json(r#""Ada""#), json("42"), json("true"), json("null")];
    let rendered = output::encode_results(&values, OutputFormat::Text).unwrap();

    assert_eq!(rendered, "Ada\n42\ntrue\nnull");
}

#[test]
fn text_output_rejects_structured_values() {
    let error = output::encode_results(&[json(r#"{"name":"Ada"}"#)], OutputFormat::Text)
        .expect_err("objects must not be rendered as raw text")
        .to_string();

    assert!(error.contains("text output requires scalar"));
}

#[test]
fn structured_output_collects_multiple_query_results() {
    let rendered = output::encode_results(&[json("1"), json("2")], OutputFormat::Json).unwrap();

    assert_eq!(rendered, "[1,2]");
}
