use std::fs;
use std::path::PathBuf;

use jaq_json::Val;
use serde_json::{json, Value};
use tiktoken_rs::cl100k_base;
use toon_world::cli::OutputFormat;
use toon_world::{output, sparse};

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

fn sparse_candidate(value: &Value) -> String {
    sparse::encode(value)
        .expect("sparse encoding must succeed")
        .expect("benchmark value must have a sparse candidate")
}

fn tokens(value: &str) -> usize {
    cl100k_base()
        .unwrap()
        .encode_with_special_tokens(value)
        .len()
}

#[test]
fn compact_benchmark_report_compares_raw_sparse_candidate() {
    for name in [
        "bench_sparse.json",
        "bench_dense.json",
        "bench_nested.json",
        "bench_cucumber.json",
    ] {
        let value = fixture(name);
        let standard = render(&value, false);
        let sparse = sparse_candidate(&value);
        let selected = render(&value, true);

        println!(
            "| {name} | {} | {} | {} | {} | {} |",
            standard.len(),
            tokens(&standard),
            sparse.len(),
            tokens(&sparse),
            if selected == sparse { "sparse" } else { "standard" }
        );

        let expected = if tokens(&sparse) < tokens(&standard) {
            &sparse
        } else {
            &standard
        };
        assert_eq!(&selected, expected);
    }
}

#[test]
fn raw_sparse_candidate_reduces_tokens_for_all_benchmark_fixtures() {
    for name in [
        "bench_sparse.json",
        "bench_dense.json",
        "bench_nested.json",
        "bench_cucumber.json",
    ] {
        let value = fixture(name);
        let standard = render(&value, false);
        let sparse = sparse_candidate(&value);

        assert!(
            tokens(&sparse) < tokens(&standard),
            "{name}: raw sparse candidate must beat standard TOON"
        );
    }
}

#[test]
fn compact_keeps_standard_when_sparse_candidate_is_not_cheaper() {
    let value = json!([{"id": 1}]);
    let standard = render(&value, false);
    let sparse = sparse_candidate(&value);
    let selected = render(&value, true);

    assert!(
        tokens(&sparse) >= tokens(&standard),
        "fixture must exercise the standard fallback"
    );
    assert_eq!(selected, standard);
}

#[test]
fn cucumber_raw_sparse_candidate_is_headerless_and_smaller() {
    let value = fixture("bench_cucumber.json");
    let standard = render(&value, false);
    let sparse = sparse_candidate(&value);

    assert!(sparse.starts_with("[1]{uri,keyword}:"));
    assert!(!sparse.contains("@toon-world/sparse-v1"));
    assert!(tokens(&sparse) < tokens(&standard));
}
