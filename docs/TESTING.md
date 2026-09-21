# Testing toon-world

CI is intentionally disabled during the initial milestone chain. Local verification is the source of truth until CI is re-enabled.

The suite optimizes for **behavioral confidence per test**, not a cosmetic coverage percentage. Each adapter is tested where silent coercion, reordering, or data loss would hurt the most.

## Required local verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

A milestone is not locally verified until all four commands succeed.

## Test layers

### Query/output core inherited from 0.1

Coverage includes:

- identity, selection, filtering, projection;
- ordered and empty result streams;
- query parse/runtime errors;
- file/stdin routing and missing-file errors;
- CLI defaults and enum validation;
- TOON semantic round-trip, nested/escaped values;
- compact JSON, key order, and very large integers;
- scalar text output, empty output, null vs empty string, and structured-value rejection.

### Adapter tests introduced in 0.2

Adapter tests call `input::parse_bytes` directly so a parser regression is distinguishable from CLI wiring.

| Adapter contract | Covered |
| --- | --- |
| extension detection, case-insensitive | yes |
| explicit `--from` override | yes |
| unknown extension JSON fallback | yes |
| non-JSON stdin with `--from` | yes |
| NDJSON/JSONL preserves line order | yes |
| checked-in `.jsonl` file is inferred and queried through the CLI | yes |
| NDJSON tolerates surrounding whitespace | yes |
| malformed NDJSON labeled correctly | yes |
| CSV header-to-object mapping | yes |
| CSV cells remain strings | yes |
| CSV preserves leading zeros | yes |
| CSV preserves empty string | yes |
| CSV quoted comma/quote handling | yes |
| CSV CRLF input | yes |
| duplicate/empty CSV headers rejected | yes |
| malformed-width CSV labeled correctly | yes |
| YAML bool/number/null types | yes |
| YAML anchors/aliases | yes |
| multi-document YAML order | yes |
| TOML scalar/array mapping | yes |
| TOML nested tables | yes |
| XML tag/attribute/child mapping | yes |
| XML mixed-content order | yes |
| multiple XML roots preserve order | yes |
| invalid UTF-8 labeled per text format | yes |

## Normalization invariants

These invariants are intentionally tested because they are easy to violate while trying to be clever:

### CSV

CSV is text-first. The adapter does **not** infer types.

```text
001   -> "001"
true  -> "true"
      -> ""
```

If callers want numbers or booleans, they can convert explicitly in jq.

### NDJSON

Each parsed JSON value becomes one ordered array element. Input line order is preserved.

### YAML / TOML

Native scalar types are retained where the parser exposes them through the common value model.

### XML

The structural model preserves element tags, attributes, and ordered child content. Mixed text/element sequences are tested because flattening them would destroy meaning.

## Adding tests

For each new adapter or behavior, prefer the smallest test that proves one invariant. Add CLI tests only where command routing itself is the behavior; parser semantics belong in direct adapter tests.

Round-trip tests should decode and compare semantic values instead of asserting serializer punctuation whenever possible.
