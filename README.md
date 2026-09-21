# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Milestone 0.2:** structured input adapters. This branch stacks on the JSON/query core from 0.1.

## Why

Conversion alone is not the product. `toon-world` exists so different formats can be read through one interface, filtered before they enter an agent's context, and emitted in a compact representation.

```text
JSON ───┐
NDJSON ─┤
CSV ────┤
YAML ───┤
TOML ───┤
XML ────┤──> adapter ──> normalized value ──> jq-compatible query ──> TOON / JSON / text
TOON ───┤
MD ─────┤
HTML ───┘
```

`toon-world` embeds [`jaq`](https://github.com/01mf02/jaq), so every adapter feeds the same query engine.

## Implemented through 0.2

- JSON, NDJSON/JSONL, CSV, YAML, TOML, and XML input
- input format inference from known extensions
- explicit `--from <format>` override
- file input, implicit stdin, explicit `-` stdin, and raw `--data`
- raw multiline CSV/YAML/TOML/XML/NDJSON with explicit `--from`
- jq-compatible selection/filter/projection
- TOON default output, compact JSON, and scalar text output
- format-specific parse error categories
- warning diagnostics on stderr
- unknown-extension fallback warns before assuming JSON
- `--quiet` suppresses warnings
- `--warnings-as-errors` turns warnings into non-zero failures before result output

## Input contracts

| Input | Normalization |
| --- | --- |
| JSON | JSON value unchanged |
| NDJSON / JSONL | ordered array of parsed JSON values |
| CSV | header row becomes object keys; every cell remains a string |
| YAML | native scalar/container types; multiple documents become an ordered array |
| TOML | tables/arrays/scalars mapped into the common value model |
| XML | structural representation preserving tags (`t`), attributes (`a`), and ordered children (`c`) |

### CSV is intentionally text-first

```text
001   -> "001"
true  -> "true"
      -> ""
```

No type guessing occurs. Duplicate or empty headers are rejected rather than silently losing fields.

### XML remains structural

For:

```xml
<p>Hello <strong>world</strong>!</p>
```

the child sequence remains logically equivalent to:

```text
text "Hello "
element strong -> text "world"
text "!"
```

That ordering matters, so the tests defend it explicitly.

## Input modes

All formats support file/stdin, and raw values use `--data`:

```bash
# file
toon-world users.csv

# JSON Lines file (.jsonl is inferred as NDJSON)
toon-world events.jsonl -q '.[] | select(.level == "warn")'

# stdin
cat users.csv | toon-world --from csv

# raw data
toon-world --from csv --data $'id,name\n1,Ada\n2,Bob\n'
toon-world --from yaml --data $'name: Ada\nactive: true\n'
toon-world --from xml --data '<user id="1"><name>Ada</name></user>'
```

`FILE` and `--data` are mutually exclusive. When there is no filename to infer from, raw data/stdin defaults to JSON unless `--from` is supplied.

## CLI

```bash
# infer format from extension
toon-world users.csv -q '.[] | select(.role == "admin")'
toon-world config.yaml -q '.services[] | {name,url}'
toon-world config.toml -q '.database.host' --to text

# stdin defaults to JSON, override for another format
cat users.csv | toon-world --from csv -q '.[].name' --to text

# raw data
ntoon-world --from yaml --data $'name: Ada\nactive: true\n' -q '.name' --to text

# explicit override wins over extension
toon-world payload.json --from yaml -q '.name'
```

## Warnings and errors

Unknown filename extensions fall back to JSON but emit a warning to stderr:

```text
warning[input]: unknown extension '.txt'; assuming JSON
```

The result stream stays clean:

```text
stdout -> query result only
stderr -> warnings/errors only
```

Use:

```bash
toon-world payload.txt --quiet
toon-world payload.txt --warnings-as-errors
```

Supplying `--from` explicitly avoids the fallback warning because the format is no longer ambiguous.

## TOON compatibility

Output still uses `toon-format` 0.5.x, whose published documentation declares TOON specification **v3.0** compatibility. TOON input is intentionally deferred to milestone 0.3 so that compatibility boundary stays explicit.

## Testing and local verification

CI is intentionally disabled for the initial milestone chain. This branch inherits the complete 0.1 query/output suite and adds adapter coverage for ordering, malformed input, raw multiline data, warning behavior, stderr/stdout isolation, CSV quoting/CRLF and string preservation, YAML aliases/multi-documents, nested TOML, XML mixed content, UTF-8 errors, extension detection, overrides, and stdin/file routing.

See [`docs/TESTING.md`](docs/TESTING.md) for the detailed matrix.

Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Smoke checks:

```bash
cargo run --quiet -- --from csv --data $'id,name,active\n001,Ada,true\n002,Bob,false\n' \
  -q '.[] | select(.active == "true") | {id,name}' --to json
# {"id":"001","name":"Ada"}

cargo run --quiet -- --from yaml --data $'name: Ada\nactive: true\ncount: 2\n' \
  -q '{name,active,count}' --to json
# {"name":"Ada","active":true,"count":2}
```

## Project docs

- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestone direction
- [`docs/MILESTONES.md`](docs/MILESTONES.md): stacked PR contract/status
- [`docs/TESTING.md`](docs/TESTING.md): local verification and coverage matrix
- [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md): architecture/design

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)
