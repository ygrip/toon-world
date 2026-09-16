# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Status:** early implementation. The current milestone implements the JSON query core; additional input adapters are delivered in later milestone PRs.

## Why

Conversion alone is not the product. `toon-world` exists so structured and document formats can be read through one interface, filtered before they enter an agent's context, and emitted in a compact representation.

```text
JSON ───┐
YAML ───┤
TOML ───┤
CSV ────┤
NDJSON ─┤
XML ────┤──> adapter ──> normalized value ──> jq-compatible query ──> TOON / JSON / text
HTML ───┤
MD ─────┤
TOON ───┘
```

`toon-world` embeds [`jaq`](https://github.com/01mf02/jaq) rather than inventing another query language. Format-specific helpers such as Markdown `section()` are sugar over the same query model.

## Implemented in the 0.1 milestone

- JSON file input
- JSON stdin input
- jq-compatible queries through embedded `jaq`
- identity query when `-q` is omitted
- TOON output by default
- compact JSON output
- scalar text output for shell pipelines
- explicit input / parse / query / encoding error categories

## CLI

Convert using the identity query:

```bash
toon-world data.json
cat data.json | toon-world
```

Filter or project using jq syntax:

```bash
toon-world data.json -q '.users[] | select(.active == true)'
toon-world data.json -q '.users[] | {id,name}'
cat response.json | toon-world -q '.repositories[] | {name,url}'
```

Choose output:

```bash
toon-world data.json -q '.users' --to toon
toon-world data.json -q '.users' --to json
toon-world package.json -q '.version' --to text
```

When a jq program yields multiple results, structured output collects them into one array. `--to text` writes scalar results one per line and rejects arrays/objects so shell pipelines do not receive ambiguous serialization.

## Query engine

The Rust implementation uses `jaq` as the embedded jq-compatible engine. Parsing, querying, and output encoding are isolated so future adapters do not need their own query implementation.

Query results are normalized at the output boundary before JSON/TOON encoding. Object order and arbitrary-precision JSON numbers are preserved where the JSON model supports them; jaq-only values such as binary strings are rejected rather than silently corrupted.

## TOON compatibility

The current Rust integration uses `toon-format` 0.5.x, whose published documentation declares TOON specification **v3.0** compatibility. The upstream TOON specification has advanced beyond that version, so this early implementation does **not** claim newer-spec conformance yet.

The encoder is isolated behind the output module so it can be upgraded or replaced without changing the query/input architecture.

## Principles

1. **Query first**: parse once, query the normalized value, encode only the result.
2. **Lossless by default**: conversion must not silently drop, regroup, or hoist data.
3. **Standard TOON first**: extensions are explicit, versioned, benchmarked, and opt-in.
4. **One executable**: no runtime dependency on jq, Node, Python, or companion binaries.
5. **stdin/stdout first**: shell and agent pipelines are primary use cases.
6. **Measure, do not guess**: optimizations need reproducible byte/token/performance evidence.

## Verification

CI is intentionally disabled during the initial implementation milestones. Validate locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Useful smoke checks:

```bash
printf '%s' '{"users":[{"id":1,"name":"Ada","active":true},{"id":2,"name":"Bob","active":false}]}' \
  | cargo run --quiet -- -q '.users[] | select(.active) | {id,name}' --to json
# {"id":1,"name":"Ada"}

printf '%s' '{"version":"0.1.0"}' \
  | cargo run --quiet -- -q '.version' --to text
# 0.1.0
```

## Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md), the [design](docs/superpowers/specs/2026-09-16-toon-world-design.md), and the [query-core implementation plan](docs/superpowers/plans/2026-09-16-query-core.md).

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)
