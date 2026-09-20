use std::fs;
use std::path::PathBuf;

use jaq_json::Val;
use serde_json::Value;
use toon_world::cli::OutputFormat;
use toon_world::output;

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn render(value: &Value, compact: bool) -> String {
    let input: Val = serde_json::from_value(value.clone()).unwrap();
    output::encode_results_with_options(&[input], OutputFormat::Toon, compact).unwrap()
}

#[test]
fn compact_benchmark_report() {
    for name in [
        "bench_sparse.json",
        "bench_dense.json",
        "bench_nested.json",
        "bench_cucumber.json",
    ] {
        let value = fixture(name);
        let normal = render(&value, false);
        let compact = render(&value, true);
        println!(
            "| {name} | {} | {} | {} |",
            normal.len(),
            compact.len(),
            normal.len().saturating_sub(compact.len())
        );
        assert!(compact.len() <= normal.len());
    }
}
