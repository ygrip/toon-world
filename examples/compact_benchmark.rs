use tiktoken_rs::cl100k_base;
use toon_world::{output, sparse};

#[path = "../benches/support/mod.rs"]
mod support;

struct Comparison {
    input: String,
    standard: String,
    sparse: Option<String>,
    selected: output::CompactToonSelection,
}

fn compare(value: &serde_json::Value) -> Comparison {
    let input = serde_json::to_string(value).expect("input JSON must encode");
    let standard = toon_format::encode_default(value).expect("standard TOON must encode");
    let sparse = sparse::encode(value).expect("sparse encoding must not fail");
    let selected = output::select_compact_toon(standard.clone(), sparse.clone());
    Comparison {
        input,
        standard,
        sparse,
        selected,
    }
}

fn main() {
    let encoder = cl100k_base().expect("cl100k_base must initialize");
    println!("<!-- BEGIN GENERATED COMPACT BENCHMARKS -->");
    println!("### Output size");
    println!("| fixture | input JSON bytes | standard TOON bytes | raw sparse bytes | selected bytes | selected | raw sparse vs standard | selected vs standard |");
    println!("| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |");
    for fixture in support::fixtures() {
        let comparison = compare(&fixture.value);
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {:.2}% |",
            fixture.name,
            comparison.input.len(),
            comparison.standard.len(),
            display_len(comparison.sparse.as_ref()),
            comparison.selected.rendered().len(),
            comparison.selected.label(),
            reduction_option(
                comparison.sparse.as_ref().map(String::len),
                comparison.standard.len()
            ),
            reduction(
                comparison.standard.len(),
                comparison.selected.rendered().len()
            ),
        );
    }
    println!();
    println!("### `cl100k_base` reference");
    println!("| fixture | input JSON tokens | standard TOON tokens | raw sparse tokens | selected tokens | selected | raw sparse vs standard | selected vs standard |");
    println!("| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |");
    for fixture in support::fixtures() {
        let comparison = compare(&fixture.value);
        let input_tokens = encoder.encode_with_special_tokens(&comparison.input).len();
        let standard_tokens = encoder
            .encode_with_special_tokens(&comparison.standard)
            .len();
        let sparse_tokens = comparison
            .sparse
            .as_ref()
            .map(|sparse| encoder.encode_with_special_tokens(sparse).len());
        let selected_tokens = encoder
            .encode_with_special_tokens(comparison.selected.rendered())
            .len();
        println!(
            "| {} | {input_tokens} | {standard_tokens} | {} | {selected_tokens} | {} | {} | {:.2}% |",
            fixture.name,
            display_number(sparse_tokens),
            comparison.selected.label(),
            reduction_option(sparse_tokens, standard_tokens),
            reduction(standard_tokens, selected_tokens),
        );
    }
    println!("<!-- END GENERATED COMPACT BENCHMARKS -->");
}

fn reduction(normal: usize, compact: usize) -> f64 {
    if normal == 0 {
        0.0
    } else {
        (1.0 - compact as f64 / normal as f64) * 100.0
    }
}

fn reduction_option(candidate: Option<usize>, standard: usize) -> String {
    candidate
        .map(|candidate| format!("{:.2}%", reduction(standard, candidate)))
        .unwrap_or_else(|| "—".to_owned())
}

fn display_len(value: Option<&String>) -> String {
    display_number(value.map(String::len))
}

fn display_number(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_owned())
}
