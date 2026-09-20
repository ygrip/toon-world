use std::fs;
use std::path::PathBuf;

use jaq_json::Val;
use serde_json::Value;
use tiktoken_rs::cl100k_base;
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

fn tokens(value: &str) -> usize {
    cl100k_base()
        .unwrap()
        .encode_with_special_tokens(value)
        .len()
}

#[test]
fn compact_benchmark_report() {
    for name in ["bench_sparse.json", "bench_dense.json", "bench_nested.json"] {
        let value = fixture(name);
        let normal = render(&value, false);
        let compact = render(&value, true);
        println!(
            "| {name} | {} | {} | {} | {} |",
            normal.len(),
            tokens(&normal),
            compact.len(),
            tokens(&compact)
        );
        assert!(compact.starts_with("@toon-world/sparse-v1"));
    }
}

#[test]
fn compact_reduces_cl100k_tokens_for_sparse_fixture() {
    let value = fixture("bench_sparse.json");
    let normal = render(&value, false);
    let compact = render(&value, true);

    assert!(tokens(&compact) < tokens(&normal));
}
