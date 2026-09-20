use tiktoken_rs::cl100k_base;
use toon_world::sparse;

#[path = "../benches/support/mod.rs"]
mod support;

fn main() {
    let encoder = cl100k_base().expect("cl100k_base must initialize");
    println!("<!-- BEGIN GENERATED COMPACT BENCHMARKS -->");
    println!("| fixture | normal bytes | compact bytes | byte reduction | normal cl100k | compact cl100k | cl100k reduction |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for fixture in support::fixtures() {
        let normal =
            toon_format::encode_default(&fixture.value).expect("standard TOON must encode");
        let sparse = sparse::encode(&fixture.value).expect("sparse encoding must not fail");
        let compact = match sparse {
            Some(sparse) if sparse.len() < normal.len() => sparse,
            _ => normal.clone(),
        };
        let normal_tokens = encoder.encode_with_special_tokens(&normal).len();
        let compact_tokens = encoder.encode_with_special_tokens(&compact).len();
        println!(
            "| {} | {} | {} | {:.2}% | {normal_tokens} | {compact_tokens} | {:.2}% |",
            fixture.name,
            normal.len(),
            compact.len(),
            reduction(normal.len(), compact.len()),
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
