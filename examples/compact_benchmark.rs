use tiktoken_rs::cl100k_base;
use toon_world::sparse;

#[path = "../benches/support/mod.rs"]
mod support;

fn main() {
    let encoder = cl100k_base().expect("cl100k_base must initialize");
    println!("<!-- BEGIN GENERATED COMPACT BENCHMARKS -->");
    println!("### Output size");
    println!("| fixture | input JSON bytes | standard TOON bytes | compact TOON bytes | compact vs JSON | compact vs standard TOON |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    for fixture in support::fixtures() {
        let input = serde_json::to_string(&fixture.value).expect("input JSON must encode");
        let normal =
            toon_format::encode_default(&fixture.value).expect("standard TOON must encode");
        let sparse = sparse::encode(&fixture.value).expect("sparse encoding must not fail");
        let compact = match sparse {
            Some(sparse) if sparse.len() < normal.len() => sparse,
            _ => normal.clone(),
        };
        println!(
            "| {} | {} | {} | {} | {:.2}% | {:.2}% |",
            fixture.name,
            input.len(),
            normal.len(),
            compact.len(),
            reduction(input.len(), compact.len()),
            reduction(normal.len(), compact.len()),
        );
    }
    println!();
    println!("### `cl100k_base` reference");
    println!("| fixture | input JSON tokens | standard TOON tokens | compact TOON tokens | compact vs JSON | compact vs standard TOON |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    for fixture in support::fixtures() {
        let input = serde_json::to_string(&fixture.value).expect("input JSON must encode");
        let normal =
            toon_format::encode_default(&fixture.value).expect("standard TOON must encode");
        let sparse = sparse::encode(&fixture.value).expect("sparse encoding must not fail");
        let compact = match sparse {
            Some(sparse) if sparse.len() < normal.len() => sparse,
            _ => normal.clone(),
        };
        let input_tokens = encoder.encode_with_special_tokens(&input).len();
        let normal_tokens = encoder.encode_with_special_tokens(&normal).len();
        let compact_tokens = encoder.encode_with_special_tokens(&compact).len();
        println!(
            "| {} | {input_tokens} | {normal_tokens} | {compact_tokens} | {:.2}% | {:.2}% |",
            fixture.name,
            reduction(input_tokens, compact_tokens),
            reduction(normal_tokens, compact_tokens),
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
