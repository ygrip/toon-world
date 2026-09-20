# Compact sparse-TOON benchmarks

`--compact` is experimental and lossless. It selects sparse-TOON only when its UTF-8 output is smaller than standard TOON. This runtime choice is provider-neutral; it does not load a model tokenizer.

Every fixture compares same source data in four forms: compact JSON input, standard TOON, raw sparse candidate, and selected output. Matrix varies 10, 100, and 1,000 rows with 10%, 30%, and 70% target sparsity.

| Fixture prefix | Data shape |
| --- | --- |
| `flat` | Rows with an `id` plus 12 optional scalar columns |
| `nested` | Rows with optional `profile.identity` and `profile.location` leaves |
| `child-tables` | Rows with three repeated `events` child objects, each with optional scalar attributes |

`r100-missing30pct` means 100 rows with 30% target missing optional fields. Small fixtures approximate this target; 100- and 1,000-row fixtures reach it exactly.

Run size/token reporting and regression tests:

```bash
./scripts/benchmark.sh
./scripts/benchmark.sh --check
```

`--check` fails if this generated block is stale. Refresh it after intentional encoder changes with `./scripts/benchmark.sh --update`.

## Generated format comparison

`Input JSON` is the canonical compact JSON serialization of each generated source value, not pretty-printed JSON. `Raw sparse` exposes codec output even if it loses. `Selected output` is the value `--compact` emits after its byte comparison.

<!-- BEGIN GENERATED COMPACT BENCHMARKS -->
### Output size
| fixture | input JSON bytes | standard TOON bytes | raw sparse bytes | selected bytes | selected | raw sparse vs standard | selected vs standard |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| flat-r10-missing10pct | 2525 | 2866 | 1278 | 1278 | sparse | 55.41% | 55.41% |
| nested-r10-missing10pct | 2003 | 2598 | 846 | 846 | sparse | 67.44% | 67.44% |
| child-tables-r10-missing10pct | 4573 | 5375 | 2786 | 2786 | sparse | 48.17% | 48.17% |
| flat-r10-missing30pct | 2011 | 2283 | 1090 | 1090 | sparse | 52.26% | 52.26% |
| nested-r10-missing30pct | 1689 | 2179 | 734 | 734 | sparse | 66.31% | 66.31% |
| child-tables-r10-missing30pct | 3803 | 5074 | 2576 | 2576 | sparse | 49.23% | 49.23% |
| flat-r10-missing70pct | 895 | 1017 | 682 | 682 | sparse | 32.94% | 32.94% |
| nested-r10-missing70pct | 936 | 1179 | 479 | 479 | sparse | 59.37% | 59.37% |
| child-tables-r10-missing70pct | 2153 | 2899 | 2054 | 2054 | sparse | 29.15% | 29.15% |
| flat-r100-missing10pct | 26082 | 29427 | 12790 | 12790 | sparse | 56.54% | 56.54% |
| nested-r100-missing10pct | 20558 | 26403 | 8237 | 8237 | sparse | 68.80% | 68.80% |
| child-tables-r100-missing10pct | 46385 | 56946 | 29075 | 29075 | sparse | 48.94% | 48.94% |
| flat-r100-missing30pct | 20505 | 23130 | 10613 | 10613 | sparse | 54.12% | 54.12% |
| nested-r100-missing30pct | 17053 | 21778 | 6892 | 6892 | sparse | 68.35% | 68.35% |
| child-tables-r100-missing30pct | 38140 | 50465 | 26590 | 26590 | sparse | 47.31% | 47.31% |
| flat-r100-missing70pct | 9355 | 10540 | 6263 | 6263 | sparse | 40.58% | 40.58% |
| nested-r100-missing70pct | 9553 | 11924 | 4206 | 4206 | sparse | 64.73% | 64.73% |
| child-tables-r100-missing70pct | 21655 | 28940 | 20545 | 20545 | sparse | 29.01% | 29.01% |
| flat-r1000-missing10pct | 272502 | 305908 | 138611 | 138611 | sparse | 54.69% | 54.69% |
| nested-r1000-missing10pct | 213698 | 272104 | 89598 | 89598 | sparse | 67.07% | 67.07% |
| child-tables-r1000-missing10pct | 480905 | 586471 | 307716 | 307716 | sparse | 47.53% | 47.53% |
| flat-r1000-missing30pct | 214365 | 240571 | 114474 | 114474 | sparse | 52.42% | 52.42% |
| nested-r1000-missing30pct | 177073 | 224279 | 74573 | 74573 | sparse | 66.75% | 66.75% |
| child-tables-r1000-missing30pct | 394900 | 518106 | 279311 | 279311 | sparse | 46.09% | 46.09% |
| flat-r1000-missing70pct | 98095 | 109901 | 66204 | 66204 | sparse | 39.76% | 39.76% |
| nested-r1000-missing70pct | 98887 | 122553 | 44527 | 44527 | sparse | 63.67% | 63.67% |
| child-tables-r1000-missing70pct | 222895 | 295701 | 211706 | 211706 | sparse | 28.41% | 28.41% |

### `cl100k_base` reference
| fixture | input JSON tokens | standard TOON tokens | raw sparse tokens | selected tokens | selected | raw sparse vs standard | selected vs standard |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| flat-r10-missing10pct | 1133 | 1381 | 659 | 659 | sparse | 52.28% | 52.28% |
| nested-r10-missing10pct | 842 | 1039 | 490 | 490 | sparse | 52.84% | 52.84% |
| child-tables-r10-missing10pct | 1982 | 2275 | 1553 | 1553 | sparse | 31.74% | 31.74% |
| flat-r10-missing30pct | 903 | 1105 | 580 | 580 | sparse | 47.51% | 47.51% |
| nested-r10-missing30pct | 692 | 859 | 428 | 428 | sparse | 50.17% | 50.17% |
| child-tables-r10-missing30pct | 1632 | 2055 | 1395 | 1395 | sparse | 32.12% | 32.12% |
| flat-r10-missing70pct | 403 | 505 | 375 | 375 | sparse | 25.74% | 25.74% |
| nested-r10-missing70pct | 346 | 442 | 278 | 278 | sparse | 37.10% | 37.10% |
| child-tables-r10-missing70pct | 882 | 1155 | 988 | 988 | sparse | 14.46% | 14.46% |
| flat-r100-missing10pct | 11203 | 13663 | 6076 | 6076 | sparse | 55.53% | 55.53% |
| nested-r100-missing10pct | 8302 | 10243 | 4510 | 4510 | sparse | 55.97% | 55.97% |
| child-tables-r100-missing10pct | 19402 | 23043 | 15321 | 15321 | sparse | 33.51% | 33.51% |
| flat-r100-missing30pct | 8803 | 10783 | 5208 | 5208 | sparse | 51.70% | 51.70% |
| nested-r100-missing30pct | 6702 | 8323 | 3824 | 3824 | sparse | 54.06% | 54.06% |
| child-tables-r100-missing30pct | 15802 | 19923 | 13636 | 13636 | sparse | 31.56% | 31.56% |
| flat-r100-missing70pct | 4003 | 5023 | 3299 | 3299 | sparse | 34.32% | 34.32% |
| nested-r100-missing70pct | 3426 | 4369 | 2427 | 2427 | sparse | 44.45% | 44.45% |
| child-tables-r100-missing70pct | 8602 | 11283 | 9586 | 9586 | sparse | 15.04% | 15.04% |
| flat-r1000-missing10pct | 112003 | 136604 | 60275 | 60275 | sparse | 55.88% | 55.88% |
| nested-r1000-missing10pct | 83002 | 102404 | 44741 | 44741 | sparse | 56.31% | 56.31% |
| child-tables-r1000-missing10pct | 194002 | 230404 | 153157 | 153157 | sparse | 33.53% | 33.53% |
| flat-r1000-missing30pct | 88003 | 107804 | 51595 | 51595 | sparse | 52.14% | 52.14% |
| nested-r1000-missing30pct | 67002 | 83204 | 37881 | 37881 | sparse | 54.47% | 54.47% |
| child-tables-r1000-missing30pct | 158002 | 199204 | 136307 | 136307 | sparse | 31.57% | 31.57% |
| flat-r1000-missing70pct | 40003 | 50204 | 32505 | 32505 | sparse | 35.25% | 35.25% |
| nested-r1000-missing70pct | 34242 | 43664 | 23911 | 23911 | sparse | 45.24% | 45.24% |
| child-tables-r1000-missing70pct | 86002 | 112804 | 95807 | 95807 | sparse | 15.07% | 15.07% |
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
