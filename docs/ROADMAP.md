# toon-world roadmap

`toon-world` is a universal query/transform bridge whose default structured output is TOON. The value is not conversion by itself; it is being able to parse different formats, query only the needed data, and emit a compact result through one executable.

## Milestone 0.1 — query core

Status: implemented in `feat/query-core`, local verification required.

- Rust single binary
- JSON file/stdin input
- embedded jaq query engine
- TOON / compact JSON / scalar text output
- deterministic error categories
- high-value query/output/CLI tests

## Milestone 0.2 — structured adapters

Status: implemented on this branch, local verification required.

Inputs:

1. NDJSON / JSONL
2. CSV
3. YAML
4. TOML
5. XML

The adapter boundary stays independent from the query engine. Queries operate on normalized values, not source syntax.

Normalization decisions:

- NDJSON becomes an ordered array;
- CSV headers become keys and cells remain strings;
- YAML/TOML retain native scalar/container types where representable;
- XML remains structural and preserves ordered mixed content.

## Milestone 0.3 — TOON input

Add TOON decoding as first-class queryable input. Compatibility must remain explicitly pinned to the TOON version supported by the selected Rust implementation.

## Milestone 0.4 — Markdown

Normalize Markdown into a retrieval-oriented document model:

- title/frontmatter;
- ordered sections and heading levels;
- paragraphs, code, lists, tables, blockquotes, rules;
- document links.

Add jq helpers such as:

```bash
toon-world README.md -q 'section("Installation")'
toon-world README.md -q 'section("Usage") | code("bash")'
```

Helpers are jaq definitions over the normalized model, not another DSL.

## Milestone 0.5 — HTML

Support two explicit contracts:

- default structural DOM model;
- `--semantic` content-oriented model for agents.

Semantic mode extracts title, metadata, sections, text blocks, code, lists, tables, links, images, and forms while excluding script/style payloads.

## Milestone 0.6 — measurement

Add `--stats` so optimization claims are measurable rather than decorative.

Initial stats should report at least:

- raw input bytes;
- rendered output bytes;
- absolute byte change;
- percentage reduction/increase.

Tokenizer-specific counts can come later if they do not burden the default binary.

### Explicit non-goal

Do **not** add `--keep`, `--drop`, or `--drop-null` merely as aliases for operations jq already expresses. One query language is enough trouble for civilized society.

## Experimental — sparse heterogeneous tables

Research a reversible extension for arrays whose objects overlap but do not share identical fields:

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

The experiment only earns a stable format commitment if it beats standard TOON on representative size/token benchmarks **and** remains reliably decodable/comprehensible.

## Release shape

- `0.1`: JSON + jq-compatible query core
- `0.2`: NDJSON/CSV/YAML/TOML/XML adapters
- `0.3`: TOON input with pinned compatibility
- `0.4`: Markdown normalized model + helpers
- `0.5`: HTML structural/semantic model + helpers
- `0.6`: byte-size statistics
- `0.x-experimental`: sparse heterogeneous-table mode
- `1.0`: stable CLI, documented format contracts, reproducible performance/context benchmarks

See [`MILESTONES.md`](MILESTONES.md) for the stacked PR chain and [`TESTING.md`](TESTING.md) for local verification and coverage expectations.
