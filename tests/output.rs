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
fn toon_output_round_trips_nested_and_escaped_values() {
    let input = value(
        r#"{"name":"toon, world","nested":{"quote":"say \"hi\"","empty":"","none":null},"items":[1,true,"x"]}"#,
    );
    let rendered = output::encode_results(&[input], OutputFormat::Toon).unwrap();
    let decoded: serde_json::Value = toon_format::decode_default(&rendered).unwrap();

    assert_eq!(
        decoded,
        json!({
            "name": "toon, world",
            "nested": {"quote": "say \"hi\"", "empty": "", "none": null},
            "items": [1, true, "x"]
        })
    );
}

#[test]
fn json_output_is_compact_and_preserves_key_order() {
    let input = value(r#"{"name":"Ada","active":true}"#);
    let rendered = output::encode_results(&[input], OutputFormat::Json).unwrap();

    assert_eq!(rendered, r#"{"name":"Ada","active":true}"#);
}

#[test]
fn compact_selection_uses_sparse_only_when_smaller() {
    let sparse = output::select_compact_toon("standard".to_owned(), Some("small".to_owned()));
    let standard = output::select_compact_toon("small".to_owned(), Some("standard".to_owned()));

    assert_eq!(sparse.label(), "sparse");
    assert_eq!(sparse.rendered(), "small");
    assert_eq!(standard.label(), "standard");
    assert_eq!(standard.rendered(), "small");
}

#[test]
fn json_output_preserves_large_integer_text() {
    let input = value(r#"{"id":123456789012345678901234567890}"#);
    let rendered = output::encode_results(&[input], OutputFormat::Json).unwrap();

    assert_eq!(rendered, r#"{"id":123456789012345678901234567890}"#);
}

#[test]
fn structured_output_collects_multiple_query_results_in_order() {
    let rendered = output::encode_results(
        &[
            value(r#""first""#),
            value(r#""second""#),
            value(r#""third""#),
        ],
        OutputFormat::Json,
    )
    .unwrap();

    assert_eq!(rendered, r#"["first","second","third"]"#);
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
    let values = vec![value("\"\""), value("null")];
    let rendered = output::encode_results(&values, OutputFormat::Text).unwrap();

    assert_eq!(rendered, "\nnull");
}

#[test]
fn text_output_of_empty_result_stream_is_empty() {
    let rendered = output::encode_results(&[], OutputFormat::Text).unwrap();

    assert!(rendered.is_empty());
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
