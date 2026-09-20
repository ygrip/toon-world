use std::fs;
use std::path::PathBuf;

use jaq_json::Val;
use serde_json::Value;
use tiktoken_rs::cl100k_base;
use toon_world::cli::OutputFormat;
use toon_world::{output, sparse};

fn render(value: &Value, compact: bool) -> String {
    let value: Val = serde_json::from_value(value.clone()).unwrap();
    output::encode_results_with_options(&[value], OutputFormat::Toon, compact).unwrap()
}

fn main() {
    let encoder = cl100k_base().expect("cl100k_base must initialize");
    println!("| fixture | standard bytes | standard tokens | sparse bytes | sparse tokens | byte reduction | token reduction | selected |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |");
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

        let standard = render(&value, false);
        let sparse = sparse::encode(&value)
            .expect("sparse encoding must succeed")
            .expect("benchmark fixture must have a sparse candidate");
        let selected = render(&value, true);

        let standard_tokens = encoder.encode_with_special_tokens(&standard).len();
        let sparse_tokens = encoder.encode_with_special_tokens(&sparse).len();
        let bytes = reduction(standard.len(), sparse.len());
        let tokens = reduction(standard_tokens, sparse_tokens);
        let selected_format = if selected == sparse {
            "sparse"
        } else if selected == standard {
            "standard"
        } else {
            "unexpected"
        };

        println!(
            "| {name} | {} | {standard_tokens} | {} | {sparse_tokens} | {bytes:.2}% | {tokens:.2}% | {selected_format} |",
            standard.len(),
            sparse.len()
        );
    }
}

fn reduction(standard: usize, sparse: usize) -> f64 {
    (1.0 - sparse as f64 / standard as f64) * 100.0
}
