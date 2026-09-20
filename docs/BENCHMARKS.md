# Compact sparse-TOON benchmarks

Compact mode is an experimental, lossless, headerless sparse-TOON codec. It folds nested objects into TOON-style field groups, keeps row leaves flat, recursively writes arrays as indented child tables or standard TOON arrays, and uses `~` only for absent fields. Standard TOON remains the fallback whenever it has fewer tokens.

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
| `bench_cucumber.json` | Nested tags, hooks, steps, rows, arguments, and embeddings | Guards Cucumber-style complex report output |

`bytes` is the exact UTF-8 output length. `tokens` is the `cl100k_base` estimate from `tiktoken-rs`, chosen as a reproducible OpenAI-oriented baseline. Reduction is calculated as `(standard - compact) / standard × 100`.

Every fixture is gated against token regression. Compact mode may fall back to standard TOON, so its output must use no more `cl100k_base` tokens than normal TOON.

## Results

| fixture | normal bytes | normal tokens | compact bytes | compact tokens | byte reduction | token reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| bench_sparse.json | 462 | 153 | 252 | 99 | 45.45% | 35.29% |
| bench_dense.json | 305 | 101 | 142 | 52 | 53.44% | 48.51% |
| bench_nested.json | 352 | 105 | 185 | 68 | 47.44% | 35.24% |
| bench_cucumber.json | 1465 | 325 | 1073 | 269 | 26.76% | 17.23% |

For these fixtures, sparse-TOON reduced token estimates by **17.23% to 48.51%**. The Cucumber fixture now uses headerless sparse output with recursively indented child tables, reducing 325 tokens to 269 without embedding nested arrays as JSON cells. The primary sparse fixture reduced output from 153 to 99 tokens (35.29%). The results are stable because the fixtures, encoder, tokenizer, and dependency lockfile are committed.

## Read results carefully

- These are serialization measurements, not end-to-end latency or model-quality benchmarks.
- `cl100k_base` is an estimate for one tokenizer. A different model may split text differently.
- More optional fields and repeated nested keys usually favor sparse-TOON. Small arrays, scalar values, or irregular nested arrays may not.
- Compact output is lossless only when decoded through `--from sparse-toon`; use standard TOON when sharing data with a generic TOON consumer.

See [`USAGE.md`](USAGE.md) for a compact encode/decode walkthrough.
