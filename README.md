# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Milestone 0.3:** TOON is now a first-class queryable input format, stacked on the structured adapters from 0.2.

## Why

Conversion alone is not the product. `toon-world` exists so different formats can be read through one interface, filtered before they enter an agent's context, and emitted compactly.

```text
JSON ───┐
NDJSON ─┤
CSV ────┤
YAML ───┤
TOML ───┤
XML ────┤
TOON ───┤──> adapter ──> normalized value ──> jq-compatible query ──> TOON / JSON / text
MD ─────┤
HTML ───┘
```

`toon-world` embeds [`jaq`](https://github.com/01mf02/jaq), so TOON input uses the same query engine as every other format.

## Implemented through 0.3

- JSON, NDJSON/JSONL, CSV, YAML, TOML, XML, and TOON input
- extension inference plus explicit `--from`
- file, implicit stdin, and explicit `-` stdin
- jq-compatible selection/filter/projection
- TOON default output, compact JSON, scalar text output
- TOON encode/decode semantic round-trip through the common value model

## Input contracts

| Input | Normalization |
| --- | --- |
| JSON | JSON value unchanged |
| NDJSON / JSONL | ordered array of parsed JSON values |
| CSV | header row becomes object keys; every cell remains a string |
| YAML | native scalar/container types; multiple documents become an ordered array |
| TOML | tables/arrays/scalars mapped into the common value model |
| XML | structural `t` / `a` / `c` representation preserving ordered children |
| TOON | decoded JSON-compatible value, then queried like any other structured input |

## JSON Lines input

`.jsonl` and `.ndjson` are inferred as NDJSON and normalized as an ordered array:

```bash
toon-world events.jsonl -q '.[] | select(.level == "warn")'
```

## TOON input

```bash
# infer from .toon
toon-world data.toon -q '.users[] | {id,name}'

# stdin needs an explicit format
toon-world --from toon -q '.users[0].name' --to text < data.toon

# explicit format wins over extension
toon-world payload.json --from toon -q '.name'
```

A TOON query can emit multiple results just like JSON input. Structured output preserves result order; text output remains line-oriented.

## Round-trip guarantee tested by this milestone

Tests exercise semantic equivalence through:

```text
JSON-compatible value
  -> TOON encode
  -> toon-world TOON decode
  -> JSON encode
  -> equivalent JSON-compatible value
```

Coverage includes:

- absent property vs `null` vs empty string;
- nested objects and arrays;
- heterogeneous values;
- empty objects/arrays;
- ordered query results;
- malformed TOON and invalid UTF-8;
- file detection, stdin, and explicit overrides.

## TOON compatibility

This milestone uses `toon-format` 0.5.x for both encoding and decoding. Its published documentation declares TOON specification **v3.0** compatibility, so toon-world currently makes that same compatibility claim and no newer one.

Keeping TOON behind one adapter/encoder boundary lets the dependency be upgraded without changing the query architecture.

## Testing and local verification

CI is intentionally disabled during the initial milestone chain. See [`docs/TESTING.md`](docs/TESTING.md) for the complete inherited and TOON-specific coverage matrix.

Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Smoke checks:

```bash
printf '%s' $'users[2]{id,name}:\n  1,Ada\n  2,Bob' \
  | cargo run --quiet -- --from toon -q '.users[1].name' --to text
# Bob

printf '%s' $'users[2]{id,name}:\n  1,Ada\n  2,Bob' \
  | cargo run --quiet -- --from toon -q '.users' --to json
# [{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}]
```

## Project docs

- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestone direction
- [`docs/MILESTONES.md`](docs/MILESTONES.md): stacked PR contract/status
- [`docs/TESTING.md`](docs/TESTING.md): verification and coverage matrix
- [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md): architecture/design

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)
