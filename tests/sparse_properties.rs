use proptest::prelude::*;
use serde_json::{Map, Value};
use toon_world::sparse;

fn json_strategy() -> BoxedStrategy<Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>()
            .prop_map(serde_json::Number::from)
            .prop_map(Value::Number),
        any::<String>().prop_map(Value::String),
    ]
    .prop_recursive(5, 128, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..5).prop_map(Value::Array),
            prop::collection::vec((any::<String>(), inner), 0..6)
                .prop_map(|entries| Value::Object(Map::from_iter(entries))),
        ]
    })
    .boxed()
}

fn object_array_strategy() -> BoxedStrategy<Value> {
    prop::collection::vec(
        prop::collection::vec((any::<String>(), json_strategy()), 0..8)
            .prop_map(|entries| Value::Object(Map::from_iter(entries))),
        1..8,
    )
    .prop_map(Value::Array)
    .boxed()
}

proptest! {
    #[test]
    fn accepted_sparse_values_round_trip(value in object_array_strategy()) {
        if let Some(encoded) = sparse::encode(&value).expect("encoding must not fail") {
            let decoded = sparse::decode(&encoded).unwrap_or_else(|error| {
                panic!("decoding must not fail: {error}\nencoded:\n{encoded}")
            });
            prop_assert_eq!(decoded, Some(value));
        }
    }
}
