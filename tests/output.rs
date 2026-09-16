use jaq_json::Val;
use serde_json::json;
use toon_world::cli::OutputFormat;
use toon_world::output;

fn value(input: &str) -> Val {
    jaq_json::read::parse_single(input.as_bytes()).expect("valid test JSON")
}

#[test]
fn toon_output_round_trips_uniform_data() {
    let input = value(r#"{"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}]}"#);
    let rendered = output::encode_results(&[input], OutputFormat::Toon).unwrap();
    let decoded: serde_json::Value = toon_format::decode_default(&rendered).unwrap();

    assert_eq!(
        decoded,
        json!({"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}]})
    );
    assert!(rendered.contains("users[2]{id,name}:"));
}

#[test]
fn json_output_is_compact_and_preserves_key_order() {
    let input = value(r#"{"name":"Ada","active":true}"#);
    let rendered = output::encode_results(&[input], OutputFormat::Json).unwrap();

    assert_eq!(rendered, r#"{"name":"Ada","active":true}"#);
}

#[test]
fn structured_output_collects_multiple_query_results() {
    let rendered = output::encode_results(&[value("1"), value("2")], OutputFormat::Json).unwrap();

    assert_eq!(rendered, "[1,2]");
}

#[test]
fn structured_output_represents_no_results_as_empty_array() {
    let json_rendered = output::encode_results(&[], OutputFormat::Json).unwrap();
    let toon_rendered = output::encode_results(&[], OutputFormat::Toon).unwrap();
    let toon_decoded: serde_json::Value = toon_format::decode_default(&toon_rendered).unwrap();

    assert_eq!(json_rendered, "[]");
    assert_eq!(toon_decoded, json!([]));
}

#[test]
fn text_output_emits_scalar_results_line_by_line() {
    let values = vec![value(r#""Ada""#), value("42"), value("true"), value("null")];
    let rendered = output::encode_results(&values, OutputFormat::Text).unwrap();

    assert_eq!(rendered, "Ada\n42\ntrue\nnull");
}

#[test]
fn text_output_keeps_empty_string_distinct_from_null() {
    let values = vec![value(r#"""#), value("null")];
    let rendered = output::encode_results(&values, OutputFormat::Text).unwrap();

    assert_eq!(rendered, "\nnull");
}

#[test]
fn text_output_rejects_structured_values() {
    for input in [r#"{"name":"Ada"}"#, "[1,2]"] {
        let error = output::encode_results(&[value(input)], OutputFormat::Text)
            .expect_err("structured values must not be rendered as raw text")
            .to_string();

        assert!(error.contains("text output requires scalar"));
    }
}
