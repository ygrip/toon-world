# Compact sparse-TOON benchmarks

`--compact` is experimental and lossless. It selects sparse-TOON only when its UTF-8 output is smaller than standard TOON. This runtime choice is provider-neutral; it does not load a model tokenizer.

The generated matrix covers 10, 100, and 1,000 rows; 10%, 30%, and 70% missing optional fields; shallow rows, deep object columns, and repeated child-object arrays. It measures serialized bytes and preserves `cl100k_base` as a reproducible benchmark-only reference.

Run size/token reporting and regression tests:

```bash
./scripts/benchmark.sh
./scripts/benchmark.sh --check
```

`--check` fails if this generated block is stale. Refresh it after intentional encoder changes with `./scripts/benchmark.sh --update`.

## Generated size and tokenizer reference

<!-- BEGIN GENERATED COMPACT BENCHMARKS -->
| fixture | normal bytes | compact bytes | byte reduction | normal cl100k | compact cl100k | cl100k reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shallow-10-10 | 2866 | 1278 | 55.41% | 1381 | 659 | 52.28% |
| deep-10-10 | 2598 | 846 | 67.44% | 1039 | 490 | 52.84% |
| children-10-10 | 5375 | 2786 | 48.17% | 2275 | 1553 | 31.74% |
| shallow-10-30 | 2283 | 1090 | 52.26% | 1105 | 580 | 47.51% |
| deep-10-30 | 2179 | 734 | 66.31% | 859 | 428 | 50.17% |
| children-10-30 | 5074 | 2576 | 49.23% | 2055 | 1395 | 32.12% |
| shallow-10-70 | 1017 | 682 | 32.94% | 505 | 375 | 25.74% |
| deep-10-70 | 1179 | 479 | 59.37% | 442 | 278 | 37.10% |
| children-10-70 | 2899 | 2054 | 29.15% | 1155 | 988 | 14.46% |
| shallow-100-10 | 29427 | 12790 | 56.54% | 13663 | 6076 | 55.53% |
| deep-100-10 | 26403 | 8237 | 68.80% | 10243 | 4510 | 55.97% |
| children-100-10 | 56946 | 29075 | 48.94% | 23043 | 15321 | 33.51% |
| shallow-100-30 | 23130 | 10613 | 54.12% | 10783 | 5208 | 51.70% |
| deep-100-30 | 21778 | 6892 | 68.35% | 8323 | 3824 | 54.06% |
| children-100-30 | 50465 | 26590 | 47.31% | 19923 | 13636 | 31.56% |
| shallow-100-70 | 10540 | 6263 | 40.58% | 5023 | 3299 | 34.32% |
| deep-100-70 | 11924 | 4206 | 64.73% | 4369 | 2427 | 44.45% |
| children-100-70 | 28940 | 20545 | 29.01% | 11283 | 9586 | 15.04% |
| shallow-1000-10 | 305908 | 138611 | 54.69% | 136604 | 60275 | 55.88% |
| deep-1000-10 | 272104 | 89598 | 67.07% | 102404 | 44741 | 56.31% |
| children-1000-10 | 586471 | 307716 | 47.53% | 230404 | 153157 | 33.53% |
| shallow-1000-30 | 240571 | 114474 | 52.42% | 107804 | 51595 | 52.14% |
| deep-1000-30 | 224279 | 74573 | 66.75% | 83204 | 37881 | 54.47% |
| children-1000-30 | 518106 | 279311 | 46.09% | 199204 | 136307 | 31.57% |
| shallow-1000-70 | 109901 | 66204 | 39.76% | 50204 | 32505 | 35.25% |
| deep-1000-70 | 122553 | 44527 | 63.67% | 43664 | 23911 | 45.24% |
| children-1000-70 | 295701 | 211706 | 28.41% | 112804 | 95807 | 15.07% |
<!-- END GENERATED COMPACT BENCHMARKS -->

## CPU and memory

CPU timing uses Criterion and reports standard TOON, sparse-candidate, and complete compact-selection paths independently:

```bash
cargo bench --bench compact
```

Allocation reporting uses an instrumented system allocator and reports allocation count plus allocated bytes separately from timing:

```bash
cargo run --release --example compact_memory_benchmark
```

## Interpretation

- `cl100k_base` is benchmark-only. Different model tokenizers can split output differently.
- CPU timing depends on host load. Allocation counts and bytes describe serializer work, not process RSS.
- Sparse output is only selected when bytes decrease. It may fall back to standard TOON.
- Decode sparse output with `--from sparse-toon`; use standard TOON for generic TOON consumers.
