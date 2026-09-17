# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Milestone 0.1:** JSON query core. This branch intentionally contains only the execution spine; additional input adapters are separate stacked PRs.

## Why

Conversion alone is not the product. `toon-world` exists so data can be filtered before it enters an agent's context, using one jq-compatible query layer and a compact default output.

Milestone 0.1 implements:

```text
JSON file / stdin / --data -> jaq value -> jq-compatible query -> TOON / JSON / text
```

The planned full architecture is:

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

`toon-world` embeds [`jaq`](https://github.com/01mf02/jaq) rather than inventing another query language.

## Implemented in 0.1

- JSON file input
- JSON stdin input, implicit or explicit `-`
- raw JSON input via `--data`
- `FILE` and `--data` are mutually exclusive
- jq-compatible queries through embedded `jaq`
- identity query when `-q` is omitted
- TOON output by default
- compact JSON output via `--to json`
- scalar text output via `--to text`
- multiple query results collected for structured output
- explicit input / parse / query / encoding error categories
- reusable warning diagnostics with `warning[category]: ...`
- `--quiet` to suppress warnings
- `--warnings-as-errors` to fail on warnings
- warning output is stderr-only and never contaminates stdout data
- preservation of object order and arbitrary-precision JSON numbers at the serialization boundary

## Input modes

Exactly one source is used:

```bash
# file
toon-world data.json

# stdin
cat data.json | toon-world
printf '%s' '{"name":"Ada"}' | toon-world

# raw argument
toon-world --data '{"name":"Ada","active":true}'
```

`FILE` and `--data` cannot be combined. Raw data is parsed directly from the argument; toon-world does not create a temporary file.

## CLI

Convert using the identity query:

```bash
toon-world data.json
cat data.json | toon-world
toon-world --data '{"name":"Ada"}'
```

Filter or project using jq syntax:

```bash
toon-world data.json -q '.users[] | select(.active == true)'
toon-world data.json -q '.users[] | {id,name}'
toon-world --data '{"users":[{"id":1,"active":true}]}' -q '.users[] | select(.active)'
```

Choose output:

```bash
toon-world data.json -q '.users' --to toon
toon-world data.json -q '.users' --to json
toon-world package.json -q '.version' --to text
```

Structured output collects multiple jq results into one array. `--to text` emits scalar results one per line and rejects arrays/objects so shell output is not ambiguously serialized.

## Warnings and errors

Diagnostics are intentionally separate from data output:

```text
stdout -> result data only
stderr -> warnings and errors only
```

Errors use stable categories such as:

```text
error[input]: ...
error[parse:json]: ...
error[query]: ...
error[encode:toon]: ...
```

Warnings use:

```text
warning[input]: ...
warning[format]: ...
```

Use `--quiet` to suppress warnings or `--warnings-as-errors` to turn any warning into a non-zero failure. Those two flags are mutually exclusive.

Milestone 0.1 provides the warning infrastructure. Later adapters introduce concrete warning sources such as unknown-extension fallback.

## Query and output contracts

- `.` is the default query.
- Zero structured query results encode as `[]`.
- Zero text query results emit no bytes.
- A single structured result remains that value rather than being wrapped in an array.
- Multiple structured results preserve query order inside one array.
- JSON output is compact.
- TOON output is decoded in tests to verify semantic round-trip rather than punctuation.
- Text output accepts null, booleans, numbers, and strings; arrays/objects are rejected.

## TOON compatibility

The current Rust integration uses `toon-format` 0.5.x, whose published documentation declares TOON specification **v3.0** compatibility. This milestone does **not** claim compatibility with newer TOON spec revisions.

The encoder is isolated behind `output` so a future TOON implementation upgrade does not disturb input/query architecture.

## Principles

1. **Query first**: parse once, query the normalized value, encode only the result.
2. **Lossless by default**: conversion must not silently drop, regroup, or hoist data.
3. **Standard TOON first**: extensions are explicit, versioned, benchmarked, and opt-in.
4. **One executable**: no runtime dependency on jq, Node, Python, or companion binaries.
5. **stdin/stdout first**: shell and agent pipelines are primary use cases.
6. **Measure, do not guess**: optimizations need reproducible byte/token/performance evidence.

## Testing and local verification

CI is intentionally disabled during the initial milestone chain. The test suite focuses on behavior boundaries and silent-data-loss risks, including raw data/file conflicts, warning policy, query failures, empty streams, nested/escaped TOON values, large integers, CLI argument defaults, file/stdin routing, and null-vs-empty-string behavior.

See [`docs/TESTING.md`](docs/TESTING.md) for the coverage matrix and test policy.

Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Smoke checks:

```bash
cargo run --quiet -- --data '{"users":[{"id":1,"name":"Ada","active":true},{"id":2,"name":"Bob","active":false}]}' \
  -q '.users[] | select(.active) | {id,name}' --to json
# {"id":1,"name":"Ada"}

printf '%s' '{"version":"0.1.0"}' \
  | cargo run --quiet -- -q '.version' --to text
# 0.1.0
```

## Project docs

- [`docs/ROADMAP.md`](docs/ROADMAP.md): planned milestone sequence
- [`docs/MILESTONES.md`](docs/MILESTONES.md): stacked PR contract and status
- [`docs/TESTING.md`](docs/TESTING.md): verification policy and coverage matrix
- [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md): architecture/design
- [`docs/superpowers/plans/2026-09-16-query-core.md`](docs/superpowers/plans/2026-09-16-query-core.md): 0.1 implementation plan

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)
