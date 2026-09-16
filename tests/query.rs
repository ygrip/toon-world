use jaq_json::Val;
use toon_world::query;

fn json(input: &str) -> Val {
    jaq_json::read::parse_single(input.as_bytes()).expect("valid test JSON")
}

#[test]
fn selects_a_field() {
    let values = query::execute(".name", json(r#"{"name":"Ada","age":36}"#)).unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_string(), r#""Ada""#);
}

#[test]
fn filters_array_items() {
    let values = query::execute(
        ".users[] | select(.active)",
        json(r#"{"users":[{"name":"Ada","active":true},{"name":"Bob","active":false}]}"#),
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert!(values[0].to_string().contains("Ada"));
}

#[test]
fn projects_object_fields() {
    let values = query::execute(
        ".users[] | {id,name}",
        json(r#"{"users":[{"id":1,"name":"Ada","active":true}]}"#),
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_string(), r#"{"id":1,"name":"Ada"}"#);
}

#[test]
fn preserves_multiple_results() {
    let values = query::execute(".[]", json("[1,2,3]")).unwrap();

    assert_eq!(values.len(), 3);
    assert_eq!(values[0].to_string(), "1");
    assert_eq!(values[2].to_string(), "3");
}

#[test]
fn rejects_malformed_query() {
    let error = query::execute(".[", json("{}"))
        .expect_err("invalid query must fail")
        .to_string();

    assert!(error.contains("error[query]"));
}
