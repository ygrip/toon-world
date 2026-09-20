# Compact sparse-TOON benchmarks

`--compact` is experimental and lossless. It selects sparse-TOON only when its UTF-8 output is smaller than standard TOON. This runtime choice is provider-neutral; it does not load a model tokenizer.

Every fixture compares same source data in three forms: compact JSON input, standard TOON, and selected compact TOON. Matrix varies 10, 100, and 1,000 rows with 10%, 30%, and 70% missing optional fields.

| Fixture prefix | Data shape |
| --- | --- |
| `flat` | Rows with an `id` plus 12 optional scalar columns |
| `nested` | Rows with optional `profile.identity` and `profile.location` leaves |
| `child-tables` | Rows with three repeated `events` child objects, each with optional scalar attributes |

`r100-missing30pct` means 100 rows with 30% of optional fields absent.

Run size/token reporting and regression tests:

```bash
./scripts/benchmark.sh
./scripts/benchmark.sh --check
```

`--check` fails if this generated block is stale. Refresh it after intentional encoder changes with `./scripts/benchmark.sh --update`.

## Generated format comparison

`Input JSON` is the canonical compact JSON serialization of each generated source value, not pretty-printed JSON. `Compact TOON` is the value selected by `--compact` after its byte comparison.

<!-- BEGIN GENERATED COMPACT BENCHMARKS -->
### Output size
| fixture | input JSON bytes | standard TOON bytes | compact TOON bytes | compact vs JSON | compact vs standard TOON |
| --- | ---: | ---: | ---: | ---: | ---: |
| flat-r10-missing10pct | 2525 | 2866 | 1278 | 49.39% | 55.41% |
| nested-r10-missing10pct | 2003 | 2598 | 846 | 57.76% | 67.44% |
| child-tables-r10-missing10pct | 4573 | 5375 | 2786 | 39.08% | 48.17% |
| flat-r10-missing30pct | 2011 | 2283 | 1090 | 45.80% | 52.26% |
| nested-r10-missing30pct | 1689 | 2179 | 734 | 56.54% | 66.31% |
| child-tables-r10-missing30pct | 3803 | 5074 | 2576 | 32.26% | 49.23% |
| flat-r10-missing70pct | 895 | 1017 | 682 | 23.80% | 32.94% |
| nested-r10-missing70pct | 936 | 1179 | 479 | 48.82% | 59.37% |
| child-tables-r10-missing70pct | 2153 | 2899 | 2054 | 4.60% | 29.15% |
| flat-r100-missing10pct | 26082 | 29427 | 12790 | 50.96% | 56.54% |
| nested-r100-missing10pct | 20558 | 26403 | 8237 | 59.93% | 68.80% |
| child-tables-r100-missing10pct | 46385 | 56946 | 29075 | 37.32% | 48.94% |
| flat-r100-missing30pct | 20505 | 23130 | 10613 | 48.24% | 54.12% |
| nested-r100-missing30pct | 17053 | 21778 | 6892 | 59.58% | 68.35% |
| child-tables-r100-missing30pct | 38140 | 50465 | 26590 | 30.28% | 47.31% |
| flat-r100-missing70pct | 9355 | 10540 | 6263 | 33.05% | 40.58% |
| nested-r100-missing70pct | 9553 | 11924 | 4206 | 55.97% | 64.73% |
| child-tables-r100-missing70pct | 21655 | 28940 | 20545 | 5.13% | 29.01% |
| flat-r1000-missing10pct | 272502 | 305908 | 138611 | 49.13% | 54.69% |
| nested-r1000-missing10pct | 213698 | 272104 | 89598 | 58.07% | 67.07% |
| child-tables-r1000-missing10pct | 480905 | 586471 | 307716 | 36.01% | 47.53% |
| flat-r1000-missing30pct | 214365 | 240571 | 114474 | 46.60% | 52.42% |
| nested-r1000-missing30pct | 177073 | 224279 | 74573 | 57.89% | 66.75% |
| child-tables-r1000-missing30pct | 394900 | 518106 | 279311 | 29.27% | 46.09% |
| flat-r1000-missing70pct | 98095 | 109901 | 66204 | 32.51% | 39.76% |
| nested-r1000-missing70pct | 98887 | 122553 | 44527 | 54.97% | 63.67% |
| child-tables-r1000-missing70pct | 222895 | 295701 | 211706 | 5.02% | 28.41% |

### `cl100k_base` reference
| fixture | input JSON tokens | standard TOON tokens | compact TOON tokens | compact vs JSON | compact vs standard TOON |
| --- | ---: | ---: | ---: | ---: | ---: |
| flat-r10-missing10pct | 1133 | 1381 | 659 | 41.84% | 52.28% |
| nested-r10-missing10pct | 842 | 1039 | 490 | 41.81% | 52.84% |
| child-tables-r10-missing10pct | 1982 | 2275 | 1553 | 21.64% | 31.74% |
| flat-r10-missing30pct | 903 | 1105 | 580 | 35.77% | 47.51% |
| nested-r10-missing30pct | 692 | 859 | 428 | 38.15% | 50.17% |
| child-tables-r10-missing30pct | 1632 | 2055 | 1395 | 14.52% | 32.12% |
| flat-r10-missing70pct | 403 | 505 | 375 | 6.95% | 25.74% |
| nested-r10-missing70pct | 346 | 442 | 278 | 19.65% | 37.10% |
| child-tables-r10-missing70pct | 882 | 1155 | 988 | -12.02% | 14.46% |
| flat-r100-missing10pct | 11203 | 13663 | 6076 | 45.76% | 55.53% |
| nested-r100-missing10pct | 8302 | 10243 | 4510 | 45.68% | 55.97% |
| child-tables-r100-missing10pct | 19402 | 23043 | 15321 | 21.03% | 33.51% |
| flat-r100-missing30pct | 8803 | 10783 | 5208 | 40.84% | 51.70% |
| nested-r100-missing30pct | 6702 | 8323 | 3824 | 42.94% | 54.06% |
| child-tables-r100-missing30pct | 15802 | 19923 | 13636 | 13.71% | 31.56% |
| flat-r100-missing70pct | 4003 | 5023 | 3299 | 17.59% | 34.32% |
| nested-r100-missing70pct | 3426 | 4369 | 2427 | 29.16% | 44.45% |
| child-tables-r100-missing70pct | 8602 | 11283 | 9586 | -11.44% | 15.04% |
| flat-r1000-missing10pct | 112003 | 136604 | 60275 | 46.18% | 55.88% |
| nested-r1000-missing10pct | 83002 | 102404 | 44741 | 46.10% | 56.31% |
| child-tables-r1000-missing10pct | 194002 | 230404 | 153157 | 21.05% | 33.53% |
| flat-r1000-missing30pct | 88003 | 107804 | 51595 | 41.37% | 52.14% |
| nested-r1000-missing30pct | 67002 | 83204 | 37881 | 43.46% | 54.47% |
| child-tables-r1000-missing30pct | 158002 | 199204 | 136307 | 13.73% | 31.57% |
| flat-r1000-missing70pct | 40003 | 50204 | 32505 | 18.74% | 35.25% |
| nested-r1000-missing70pct | 34242 | 43664 | 23911 | 30.17% | 45.24% |
| child-tables-r1000-missing70pct | 86002 | 112804 | 95807 | -11.40% | 15.07% |
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

### Snapshot results

Measured on macOS 26.6.2 arm64 with Rust 1.93.0. CPU uses Criterion’s median from 10 samples with 10 ms warmup/measurement; allocations use release mode. CPU results vary by hardware and load.

| Fixture | Standard TOON CPU | Sparse candidate CPU | Complete compact CPU | Allocations: standard / sparse / compact | Bytes allocated: standard / sparse / compact |
| --- | ---: | ---: | ---: | ---: | ---: |
| `flat-r100-missing30pct` | 398 µs | 126 µs | 526 µs | 8,535 / 2,145 / 10,680 | 732,485 / 81,536 / 814,021 |
| `nested-r100-missing30pct` | 486 µs | 193 µs | 672 µs | 11,802 / 3,884 / 15,686 | 717,994 / 242,777 / 960,771 |
| `child-tables-r100-missing30pct` | 774 µs | 383 µs | 1.15 ms | 19,908 / 8,807 / 28,715 | 1,362,478 / 493,783 / 1,856,261 |

Complete compact selection includes standard TOON encoding, sparse candidate encoding, and byte comparison. It is intentionally more expensive than either single encoder.

## Interpretation

- `cl100k_base` is benchmark-only. Different model tokenizers can split output differently.
- CPU timing depends on host load. Allocation counts and bytes describe serializer work, not process RSS.
- Sparse output is only selected when bytes decrease. It may fall back to standard TOON.
- Decode sparse output with `--from sparse-toon`; use standard TOON for generic TOON consumers.
