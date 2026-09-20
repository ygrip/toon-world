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

    assert!(rendered.starts_with("[3]{id,profile{name,team},active}:\n"));
    assert!(rendered.contains("    roles[1]: admin"));
    assert!(!rendered.contains("@toon-world/sparse-v1"));
    assert!(!rendered.contains("root=^") && !rendered.contains("^0="));
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
        {"a/b": {"til~de": "value"}, "items": [{"name": "Ada"}]},
        {"a/b": {"til~de": "other"}, "items": []}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("[2]{\"a/b\"{\"til~de\"},items[]{name}}:\n"));
    assert!(rendered.contains("items[1]:"));
    assert!(rendered.contains("items[0]:"));
    assert!(!rendered.contains("items[1]{name}"));
    assert!(!rendered.contains("root=^") && !rendered.contains("^0="));
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

    assert!(rendered.starts_with(
        "[1]{uri,tags[]{name,location{line,column}},elements[]{"
    ));
    assert!(!rendered.contains("tags[1]{name"));
    assert!(rendered.contains("    tags[1]:\n"));
    assert!(rendered.contains("    elements[1]:"));
    assert!(!rendered.contains("@toon-world/sparse-v1"));
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
fn legacy_recursive_sparse_rejects_oversized_row_count_without_preallocating() {
    let rendered = "@toon-world/sparse-v1\nroot=^0\n^0=[1000000000]{id}:\n";
    let error = input::parse_bytes(rendered.as_bytes(), InputFormat::SparseToon)
        .expect_err("oversized recursive table must fail without huge allocation")
        .to_string();

    assert!(error.contains("missing recursive table row"));
}

#[test]
fn compact_uses_bare_columns_and_quotes_only_ambiguous_paths() {
    let original = json!([
        {"uri": "one", "start_timestamp": "now", "profile": {"name": "Ada"}, "a.b": 1},
        {"uri": "two", "start_timestamp": "later", "profile": {"name": "Grace"}, "a.b": 2}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.contains("{uri,start_timestamp,profile{name},\"a.b\"}"));
    assert!(!rendered.contains("\"uri\""));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_renders_nested_object_columns_and_child_tables() {
    let original = json!([
        {
            "uri": "a.feature",
            "keyword": "Feature",
            "elements": [{"id": 1, "name": "Login"}, {"id": 2, "name": "Logout"}]
        },
        {"uri": "b.feature", "elements": []}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert_eq!(
        rendered,
        concat!(
            "[2]{uri,keyword,elements[]{id,name}}:\n",
            "  a.feature,Feature\n",
            "    elements[2]:\n",
            "      1,Login\n",
            "      2,Logout\n",
            "  b.feature,~\n",
            "    elements[0]:\n"
        )
    );
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_hoists_shared_item_array_schema_once() {
    let original = json!([
        {"id": "D-1", "items": [{"sku": "A1", "qty": 2}, {"sku": "A2", "qty": 1}]},
        {"id": "D-2", "items": [{"sku": "B1", "qty": 1}]},
        {"id": "D-3", "items": []}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("[3]{id,items[]{sku,qty}}:\n"));
    assert_eq!(rendered.matches("sku,qty").count(), 1);
    assert!(rendered.contains("    items[2]:\n"));
    assert!(rendered.contains("    items[1]:\n"));
    assert!(rendered.contains("    items[0]:\n"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_hoists_table_nested_inside_table() {
    let original = json!([
        {
            "id": 1,
            "groups": [{"name": "g1", "members": [{"m": "a"}, {"m": "b"}]}]
        },
        {"id": 2, "groups": []}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("[2]{id,groups[]{name,members[]{m}}}:\n"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_hoists_table_nested_inside_object() {
    let original = json!([
        {"a": {"tags": [{"k": "x", "v": 1}]}},
        {"a": {}}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("[2]{a{tags[]{k,v}}}:\n"));
    // The row has no leaf cells at all (its only field is a nested table),
    // so it must not render as a blank line (parse_lines drops those).
    assert!(rendered.contains("  ~\n"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_declines_table_vs_object_conflict() {
    let original = json!([{"value": {"x": 1}}, {"value": [{"x": 1}]}]);

    assert!(sparse::encode(&original).unwrap().is_none());
}

#[test]
fn compact_encodes_root_object_with_item_array() {
    let original = json!({
        "items": [
            {"id": "d1", "status": "cancelled", "usage": {"tokens": 0}},
            {"id": "d2", "status": "completed", "usage": {"tokens": 100}}
        ],
        "scope_counts": {"active": 7, "completed": 20}
    });
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("{items[]{id,status,usage{tokens}},scope_counts{active,completed}}:\n"));
    assert!(rendered.contains("    items[2]:\n"));
    assert!(!rendered.contains("items[2]{id"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_encodes_root_object_without_arrays() {
    let original = json!({"name": "Ada", "profile": {"team": "platform"}});
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert_eq!(rendered, "{name,profile{team}}:\n  Ada,platform\n");
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_declines_trivial_root_objects() {
    assert!(sparse::encode(&json!({})).unwrap().is_none());
    assert!(sparse::encode(&json!({"extra": {}})).unwrap().is_none());
}

#[test]
fn compact_root_object_with_scalar_array_still_round_trips_as_attachment() {
    let original = json!({"id": 1, "roles": ["admin", "owner"]});
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("{id}:\n"));
    assert!(rendered.contains("roles[2]: admin,owner"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_preserves_empty_and_absent_nested_objects() {
    let original = json!([
        {"id": 1, "profile": {}},
        {"id": 2, "profile": {"name": "Ada"}},
        {"id": 3}
    ]);
    let rendered = sparse::encode(&original).unwrap().unwrap();

    assert!(rendered.starts_with("[3]{id,profile{name}}:\n"));
    assert!(rendered.contains("  1,~\n    profile:\n"));
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_requires_toon_output() {
    let error = output::encode_results_with_options(&[value("1")], OutputFormat::Json, true)
        .expect_err("compact JSON output must fail")
        .to_string();

    assert!(error.contains("--compact requires --to toon"));
}

#[test]
fn compact_declines_conflicting_column_shapes() {
    let original = json!([{"value": 1}, {"value": [1, 2]}]);

    assert!(sparse::encode(&original).unwrap().is_none());
}

#[test]
fn compact_round_trips_child_arrays_of_empty_objects() {
    let original = json!([
        {"id": 1, "items": [{}, {}]},
        {"id": 2, "items": [{}]}
    ]);

    let sparse = sparse::encode(&original).unwrap().unwrap();
    assert!(sparse.contains("items[2]{}:"));
    assert_eq!(decoded(&sparse), original);
    let rendered = output::encode_results_with_options(
        &[value(&original.to_string())],
        OutputFormat::Toon,
        true,
    )
    .unwrap();
    assert_eq!(decoded(&rendered), original);
}

#[test]
fn compact_declines_schemas_deeper_than_its_decoder_limit() {
    let mut nested = json!({"leaf": 1});
    for _ in 0..64 {
        nested = json!({"next": nested});
    }

    assert!(sparse::encode(&json!([nested])).unwrap().is_none());
}

#[test]
fn compact_round_trips_forty_seven_nested_objects() {
    let mut nested = json!({"leaf": "value"});
    for _ in 0..47 {
        nested = json!({"next": nested});
    }
    let value = json!([{"tree": nested}]);

    let rendered = sparse::encode(&value).unwrap().unwrap();
    assert_eq!(decoded(&rendered), value);
}

#[test]
fn compact_preserves_first_seen_order_for_wide_schemas() {
    let rows = (0..100)
        .map(|row| {
            let mut object = serde_json::Map::new();
            object.insert("id".to_owned(), json!(row));
            for column in 0..100 {
                if (row + column) % 2 == 0 {
                    object.insert(format!("column_{column}"), json!(row + column));
                }
            }
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    let rendered = sparse::encode(&Value::Array(rows)).unwrap().unwrap();

    assert!(rendered.starts_with("[100]{id,column_0,column_2,column_4"));
    assert!(rendered.contains(",column_1,column_3,column_5"));
}

#[test]
fn compact_declines_empty_keys() {
    let value = json!([{"": [[], "nested"], "profile": {"": null}}]);

    assert!(sparse::encode(&value).unwrap().is_none());
}

#[test]
fn compact_declines_non_object_child_arrays() {
    let value = json!([{"items": ["value", {"name": "Ada"}]}]);

    assert!(sparse::encode(&value).unwrap().is_none());
}

#[test]
fn sparse_detection_does_not_validate_standard_toon_indentation() {
    let standard = format!("root:\n{}leaf: 1\n", " ".repeat(130));

    assert!(sparse::decode(&standard).unwrap().is_none());
}

#[test]
fn malformed_headerless_sparse_toon_reports_its_format() {
    for rendered in [
        "[2]{id,name}:\n  1,Ada\n",
        "[1]{id,id}:\n  1,2\n",
        "[1]{id}:\n    1\n",
        "[1]{id}:\n  1\n      tags[0]:\n",
    ] {
        let error = input::parse_bytes(rendered.as_bytes(), InputFormat::SparseToon)
            .expect_err("malformed sparse table must fail")
            .to_string();

        assert!(error.contains("error[parse:sparse-toon]"), "{error}");
    }
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
