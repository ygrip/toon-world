# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Status:** early implementation

## Why

Converting data to TOON is useful, but conversion alone is not the product. The useful workflow is being able to read different formats through one interface, select only the data an agent or shell pipeline needs, and emit a compact result.

```text
JSON ───┐
YAML ───┤
TOML ───┤
CSV ────┤
NDJSON ─┤
XML ────┤──> parse / normalize ──> jq-like query ──> TOON / JSON / text
HTML ───┤
MD ─────┤
TOON ───┘
```

`toon-world` embeds a jq-compatible query engine rather than inventing another query language. Format-specific helpers such as Markdown `section()` are planned as sugar over the same query model.

## Principles

1. **Query first** — parse once, query the normalized value, encode only the result.
2. **Lossless by default** — conversion must not silently drop, regroup, or hoist data.
3. **Standard TOON first** — extensions are explicit, versioned, benchmarked, and opt-in.
4. **One executable** — no runtime dependency on jq, Node, Python, or companion binaries.
5. **stdin/stdout first** — shell and agent pipelines are primary use cases.
6. **Measure, do not guess** — optimizations need reproducible byte/token/performance evidence.

## CLI direction

Conversion uses the identity query by default:

```bash
toon-world data.json
cat data.json | toon-world
```

Query using jq syntax:

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

The query is deliberately a flag instead of an ambiguous positional argument: file conversion stays trivial while stdin remains predictable.

## Query engine

The Rust implementation uses [`jaq`](https://github.com/01mf02/jaq) as the embedded jq-compatible engine. `jaq` already provides a mature Rust parser/compiler/interpreter and multi-format foundations, so toon-world does not grow a home-made jq dialect merely for the character-building experience.

## Document adapters

Markdown and HTML are document-shaped rather than plain data formats. They will expose predictable normalized trees and later semantic helpers.

Examples of planned helpers:

```bash
toon-world README.md -q 'section("Installation")'
toon-world README.md -q 'section("Usage") | code("bash")'
toon-world page.html -q 'links()'
```

Helpers compile to the common query engine; they are not a second query language.

## Sparse heterogeneous tables

A separate experiment may encode heterogeneous arrays with an explicit absent-value sentinel while preserving row order and the distinction between absent, `null`, and empty string. It is not standard TOON and will not be emitted by default.

## Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md) and [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md).

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)