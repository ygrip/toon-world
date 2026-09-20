use std::fs;
use std::path::PathBuf;

use jaq_json::Val;
use serde_json::Value;
use tiktoken_rs::cl100k_base;
use toon_world::cli::OutputFormat;
use toon_world::output;

fn render(value: &Value, compact: bool) -> String {
    let value: Val = serde_json::from_value(value.clone()).unwrap();
    output::encode_results_with_options(&[value], OutputFormat::Toon, compact).unwrap()
}

fn main() {
    let encoder = cl100k_base().expect("cl100k_base must initialize");
    println!("| fixture | normal bytes | normal tokens | compact bytes | compact tokens | byte reduction | token reduction |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for name in [
        "bench_sparse.json",
        "bench_dense.json",
        "bench_nested.json",
        "bench_cucumber.json",
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let normal = render(&value, false);
        let compact = render(&value, true);
        let normal_tokens = encoder.encode_with_special_tokens(&normal).len();
        let compact_tokens = encoder.encode_with_special_tokens(&compact).len();
        let bytes = reduction(normal.len(), compact.len());
        let tokens = reduction(normal_tokens, compact_tokens);
        println!(
            "| {name} | {} | {normal_tokens} | {} | {compact_tokens} | {bytes:.2}% | {tokens:.2}% |",
            normal.len(), compact.len()
        );
    }
}

fn reduction(normal: usize, compact: usize) -> f64 {
    (1.0 - compact as f64 / normal as f64) * 100.0
}
