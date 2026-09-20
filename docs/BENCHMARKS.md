# Compact sparse-TOON benchmarks

Compact mode is an experimental, lossless codec for root arrays of objects. It flattens nested object leaves into JSON-Pointer columns, keeps arrays as values, and uses `~` only for absent fields. Standard TOON remains the fallback for every other result shape.

Run the reproducible benchmark:

```bash
./scripts/benchmark.sh
```

Tokens use `cl100k_base`; bytes are exact UTF-8 output lengths. Dense data is reported rather than required to improve. Sparse data must use fewer cl100k tokens than standard TOON.

| fixture | normal bytes | normal tokens | compact bytes | compact tokens | byte reduction | token reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| bench_sparse.json | 462 | 153 | 301 | 106 | 34.85% | 30.72% |
| bench_dense.json | 305 | 101 | 192 | 65 | 37.05% | 35.64% |
| bench_nested.json | 352 | 105 | 227 | 71 | 35.51% | 32.38% |
