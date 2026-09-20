#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use toon_world::sparse;

fuzz_target!(|input: &[u8]| {
    let Ok(value) = serde_json::from_slice::<Value>(input) else {
        return;
    };
    let Value::Array(rows) = &value else {
        return;
    };
    if rows.is_empty() || !rows.iter().all(Value::is_object) {
        return;
    }
    if let Ok(Some(encoded)) = sparse::encode(&value) {
        assert_eq!(sparse::decode(&encoded).unwrap(), Some(value));
    }
});
