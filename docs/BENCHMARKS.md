# Compact sparse-TOON benchmarks

Compact mode is an experimental, lossless codec for root arrays of objects. It flattens nested object leaves into JSON-Pointer columns, keeps arrays as values, and uses `~` only for absent fields. Standard TOON remains the fallback for every other result shape.

The benchmark answers one narrow question: for the checked-in data, how much smaller is sparse-TOON than the current standard TOON encoder? It does not claim that the same reduction applies to all production data or every model tokenizer.

Run the reproducible benchmark:

```bash
./scripts/benchmark.sh
```

The command first runs the benchmark regression tests, then prints the table. Use it after encoder changes and before recording new numbers.

## Fixtures and metrics

| Fixture | Shape being measured | Why it matters |
| --- | --- | --- |
| `bench_sparse.json` | Optional fields across five nested records | Primary sparse-table use case; must reduce tokens |
| `bench_dense.json` | Same nested fields on every record | Shows whether column normalization still helps uniform data |
| `bench_nested.json` | Deeper objects plus array leaves | Exercises flattening without changing array semantics |

`bytes` is the exact UTF-8 output length. `tokens` is the `cl100k_base` estimate from `tiktoken-rs`, chosen as a reproducible OpenAI-oriented baseline. Reduction is calculated as `(standard - compact) / standard × 100`.

Dense and nested rows are reported, not gated: sparse-TOON may expand some real-world shapes. The automated gate requires only `bench_sparse.json` to use fewer `cl100k_base` tokens than standard TOON.

## Results

| fixture | normal bytes | normal tokens | compact bytes | compact tokens | byte reduction | token reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| bench_sparse.json | 462 | 153 | 301 | 106 | 34.85% | 30.72% |
| bench_dense.json | 305 | 101 | 192 | 65 | 37.05% | 35.64% |
| bench_nested.json | 352 | 105 | 227 | 71 | 35.51% | 32.38% |

For these fixtures, sparse-TOON reduced token estimates by **30.72% to 35.64%**. The primary sparse fixture reduced output from 153 to 106 tokens (30.72%). The results are stable because the fixtures, encoder, tokenizer, and dependency lockfile are committed.

## Read results carefully

- These are serialization measurements, not end-to-end latency or model-quality benchmarks.
- `cl100k_base` is an estimate for one tokenizer. A different model may split text differently.
- More optional fields and repeated nested keys usually favor sparse-TOON. Small arrays, scalar values, or irregular nested arrays may not.
- Compact output is lossless only when decoded through `--from sparse-toon`; use standard TOON when sharing data with a generic TOON consumer.

See [`USAGE.md`](USAGE.md) for a compact encode/decode walkthrough.
