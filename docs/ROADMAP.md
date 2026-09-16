# toon-world roadmap

`toon-world` is a universal query/transform bridge whose default structured output is TOON. The goal is one fast executable that can parse different formats, query only the needed data, and emit compact context.

## 0.1 — query core

Status: implemented on this branch, local verification required.

- Rust single binary
- JSON file/stdin input
- embedded jaq query engine
- identity query by default
- TOON / compact JSON / scalar text output
- deterministic error categories
- high-value query/output/CLI tests

## 0.2 — structured adapters

Planned as a separate stacked PR:

- NDJSON / JSONL
- CSV
- YAML
- TOML
- structural XML

The adapter boundary must remain independent from the query engine.

## 0.3 — TOON input

Add TOON decoding as first-class input, with the compatibility claim pinned to the TOON version actually supported by the selected Rust implementation.

## 0.4 — Markdown

Normalize Markdown into a retrieval-oriented document model with ordered sections, typed blocks, frontmatter, links, and jaq helpers such as `section()` and `code()`.

## 0.5 — HTML

Support two explicit contracts:

- structural DOM model by default;
- content-oriented `--semantic` mode for agents.

## 0.6 — measurement

Add `--stats` for raw input bytes, rendered output bytes, absolute delta, and percentage reduction/increase. Tokenizer-specific estimates can remain optional future work if they would burden the default binary.

Do not add `--keep`, `--drop`, or `--drop-null` as parallel interfaces for operations jq already expresses.

## Experimental — sparse heterogeneous tables

Research a reversible extension for overlapping object shapes:

```text
[3]{type,repo,pr,key}:
  github,punakawan,34,~
  jira,~,~,ABC-1
  github,mom,12,~
```

Candidate semantics:

- `~` = property absent;
- `null` = JSON null;
- `""` = empty string;
- row order preserved.

Only stabilize this if representative benchmarks prove material context savings and reliable decoding/comprehension.

## Release shape

- `0.1`: JSON query core
- `0.2`: structured adapters
- `0.3`: TOON input
- `0.4`: Markdown
- `0.5`: HTML
- `0.6`: byte-size stats
- `0.x-experimental`: sparse tables
- `1.0`: stable CLI + documented contracts + reproducible benchmarks

See [`MILESTONES.md`](MILESTONES.md) for the stacked PR chain and [`TESTING.md`](TESTING.md) for verification/coverage policy.
