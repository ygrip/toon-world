# Compact sparse-TOON benchmarks

Compact mode is an experimental, lossless, headerless sparse-TOON codec. It folds nested objects into TOON-style field groups, keeps row leaves flat, recursively writes arrays as indented child tables or standard TOON arrays, and uses `~` only for absent fields. Standard TOON remains the fallback whenever it has fewer tokens.

The benchmark answers two separate questions:

1. How much smaller is the **raw sparse candidate** than the current standard TOON encoder?
2. Does production `--compact` select sparse only when that candidate has fewer `cl100k_base` tokens?

Keeping those measurements separate avoids a self-validating benchmark where the fallback logic makes compact output look non-regressing by construction.

Run the reproducible benchmark:

```bash
./scripts/benchmark.sh
```

The command first runs the benchmark regression tests, then prints the raw standard-vs-sparse comparison and the format selected by production compact mode.

## Fixtures and metrics

| Fixture | Shape being measured | Why it matters |
| --- | --- | --- |
| `bench_sparse.json` | Optional fields across five nested records | Primary sparse-table use case; must reduce tokens |
| `bench_dense.json` | Same nested fields on every record | Shows whether column normalization still helps uniform data |
| `bench_nested.json` | Deeper objects plus array leaves | Exercises flattening without changing array semantics |
| `bench_cucumber.json` | Nested tags, hooks, steps, rows, arguments, and embeddings | Guards Cucumber-style complex report output |

`bytes` is the exact UTF-8 output length. `tokens` is the `cl100k_base` estimate from `tiktoken-rs`, chosen as a reproducible OpenAI-oriented baseline. Reduction is calculated as `(standard - sparse) / standard × 100`.

The benchmark calls `sparse::encode()` directly for the sparse column, so those numbers are not affected by the production fallback. A separate assertion checks that `--compact` returns whichever representation has fewer tokens. A small equal-or-worse candidate also exercises the standard-TOON fallback.

## Results

| fixture | standard bytes | standard tokens | sparse bytes | sparse tokens | byte reduction | token reduction | selected |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| bench_sparse.json | 462 | 153 | 252 | 99 | 45.45% | 35.29% | sparse |
| bench_dense.json | 305 | 101 | 142 | 52 | 53.44% | 48.51% | sparse |
| bench_nested.json | 352 | 105 | 185 | 68 | 47.44% | 35.24% | sparse |
| bench_cucumber.json | 1465 | 325 | 1073 | 269 | 26.76% | 17.23% | sparse |

For these fixtures, the **raw sparse codec itself** reduces token estimates by **17.23% to 48.51%**. The Cucumber fixture reduces 325 tokens to 269 with recursively indented child tables, while the primary sparse fixture reduces 153 tokens to 99. Production `--compact` selects sparse for all four because the raw candidate wins in each case.

## Read results carefully

- These are serialization measurements, not end-to-end latency, CPU, memory, or model-quality benchmarks.
- `cl100k_base` is one tokenizer. A different model or provider may split the same output differently.
- The checked-in fixtures are intentionally small and deterministic; they are regression fixtures, not a representative production corpus.
- More optional fields and repeated nested keys usually favor sparse-TOON. Small arrays, scalar values, or irregular nested arrays may not.
- Compact output is lossless only when decoded through `--from sparse-toon`; use standard TOON when sharing data with a generic TOON consumer.

See [`USAGE.md`](USAGE.md) for a compact encode/decode walkthrough.
