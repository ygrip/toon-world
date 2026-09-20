use jaq_json::Val;
use serde_json::{json, Value};
use toon_world::cli::{InputFormat, OutputFormat};
use toon_world::{input, output, sparse};

fn value(input: &str) -> Val {
    jaq_json::read::parse_single(input.as_bytes()).expect("valid test JSON")
}

fn decoded(rendered: &str) -> Value {
    let value = input::parse_bytes(rendered.as_bytes(), InputFormat::SparseToon).unwrap();
    serde_json::from_str(&value.to_string()).unwrap()
}

#[test]
fn compact_sparse_toon_round_trips_nested_sparse_rows() {
    let original = json!([
        {"id": 1, "profile": {"name": "Ada", "team": "platform"}, "roles": ["admin"]},
        {"id": 2, "profile": {"name": "Grace"}, "active": null},
        {"id": 3, "profile": {"name": "Linus", "team": "kernel"}, "active": false}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("@toon-world/sparse-v1\n"));
    assert!(rendered.contains("\"/profile/name\""));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_distinguishes_absent_null_empty_and_literal_marker() {
    let original = json!([
        {"id": 1, "note": null, "empty": "", "marker": "~"},
        {"id": 2, "empty": ""}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.contains("null"));
    assert!(rendered.contains("\"~\""));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_escapes_pointer_keys_and_references_nested_arrays() {
    let original = json!([
        {"a/b": {"til~de": "value"}, "items": [1, {"name": "Ada"}]},
        {"a/b": {"til~de": "other"}, "items": []}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.contains("/a~1b/til~0de"));
    assert!(rendered.contains("\"/items\""));
    assert!(rendered.contains("root=^0"));
    assert!(!rendered.contains("[1,{\"name\":\"Ada\"}]"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_can_fall_back_to_standard_toon_when_standard_is_smaller() {
    let original = json!({"name": "Ada"});
    let rendered = output::encode_results_with_options(
        &[value(&original.to_string())],
        OutputFormat::Toon,
        true,
    )
    .unwrap();

    assert!(!rendered.starts_with("@toon-world/sparse-v1"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn recursive_v1_round_trips_cucumber_shaped_nested_data() {
    let original = json!([{
        "uri": "classpath:features/Affiliate.feature",
        "tags": [{"name":"@Regression","location":{"line":1,"column":1}}],
        "elements": [{
            "before": [{"result":{"duration":12,"status":"passed"},"match":{"location":"before()"}}],
            "steps": [{
                "after": [{"embeddings":[{"mime_type":"image/png","data":"abc","name":"shot"}]}],
                "rows": [{"cells":["username","password"]}],
                "match": {"arguments":[{"val":"user","offset":0}]}
            }]
        }]
    }]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("@toon-world/sparse-v1\nroot=^0\n"));
    assert!(rendered.contains("\"/elements\""));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn legacy_v1_table_still_decodes() {
    let rendered = "@toon-world/sparse-v1\n[2]{\"/id\",\"/name\"}:\n1,\"Ada\"\n2,~\n";
    assert_eq!(
        decoded(rendered),
        json!([{"id": 1, "name": "Ada"}, {"id": 2}])
    );
}

#[test]
fn compact_requires_toon_output() {
    let error = output::encode_results_with_options(&[value("1")], OutputFormat::Json, true)
        .expect_err("compact JSON output must fail")
        .to_string();

    assert!(error.contains("--compact requires --to toon"));
}

#[test]
fn malformed_sparse_toon_reports_its_format() {
    let error = input::parse_bytes(
        b"@toon-world/sparse-v1\n[1]{\"/id\"}:\n1,2\n",
        InputFormat::SparseToon,
    )
    .expect_err("invalid sparse rows must fail")
    .to_string();

    assert!(error.contains("error[parse:sparse-toon]"));
}
