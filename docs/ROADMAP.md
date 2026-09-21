# toon-world roadmap

`toon-world` is a universal query/transform bridge whose default structured output is TOON. The goal is one compact query layer across structured and document formats.

## 0.1 — query core

Implemented: JSON file/stdin, embedded jaq, TOON/JSON/text output, deterministic errors, focused tests.

## 0.2 — structured adapters

Implemented: NDJSON/JSONL, CSV, YAML, TOML, structural XML.

Key contracts:

- NDJSON preserves value order;
- CSV remains text-first;
- YAML/TOML retain native scalar/container types where representable;
- XML preserves ordered mixed content.

## 0.3 — TOON input

Implemented: `.toon` detection, `--from toon`, decode into the common model, semantic round-trip coverage, compatibility pinned to the selected `toon-format` release.

## 0.4 — Markdown

Status: implemented on this branch, local verification required.

Normalized model:

- first H1 as title;
- raw YAML-style frontmatter;
- ordered sections + heading levels;
- paragraph/code/list/table/blockquote/rule/raw-HTML blocks;
- link index.

Helpers:

```bash
toon-world README.md -q 'section("Installation")'
toon-world README.md -q 'section("Usage") | code("bash")'
toon-world README.md -q 'links'
```

Helpers are jaq definitions, not a second query language.

## 0.5 — HTML

Add default structural DOM normalization and explicit `--semantic` content extraction. Semantic mode should expose title, metadata, sections, code, lists, tables, links, images, and forms while excluding script/style payloads.

## 0.6 — measurement

Add `--stats` for raw input bytes, rendered output bytes, absolute delta, and percentage reduction/increase. Tokenizer-specific estimates remain optional future work.

Do not add `--keep`, `--drop`, or `--drop-null`: jq already expresses those transformations and duplicating them would create competing interfaces for the same operation.

## Experimental — sparse heterogeneous tables

Research a reversible extension that distinguishes missing, null, and empty values while preserving row order. Only stabilize it if representative size/token benchmarks show material gains and decoding remains reliable.

## Release shape

- `0.1`: JSON query core
- `0.2`: structured adapters
- `0.3`: TOON input
- `0.4`: Markdown
- `0.5`: HTML
- `0.6`: byte-size stats
- `0.x-experimental`: sparse tables
- `1.0`: stable CLI + documented contracts + reproducible benchmarks

See [`MILESTONES.md`](MILESTONES.md) and [`TESTING.md`](TESTING.md).
