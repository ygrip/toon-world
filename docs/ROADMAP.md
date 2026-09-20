# toon-world roadmap

`toon-world` is a universal query/transform bridge whose default structured output is TOON. Conversion exists to make many formats queryable through one compact agent- and shell-friendly pipeline.

## 0.1 — query core

Implemented in `feat/query-core`:

- JSON file/stdin input;
- embedded jaq;
- TOON / compact JSON / scalar text output;
- deterministic error categories and focused tests.

## 0.2 — structured adapters

Implemented in `feat/structured-adapters`:

- NDJSON / JSONL;
- CSV as header-keyed string data;
- YAML;
- TOML;
- structural XML.

All adapters feed the same common value/query layer.

## 0.3 — TOON input

Status: implemented on this branch, local verification required.

- `.toon` detection;
- `--from toon`;
- TOON decode into the common value model;
- direct jaq querying;
- semantic round-trip coverage;
- compatibility claim pinned to the selected `toon-format` version.

## 0.4 — Markdown

Normalize Markdown for retrieval:

- title/frontmatter;
- ordered sections + heading levels;
- paragraphs, code, lists, tables, blockquotes, rules;
- links.

Planned helper examples:

```bash
toon-world README.md -q 'section("Installation")'
toon-world README.md -q 'section("Usage") | code("bash")'
```

Helpers remain jaq definitions over the normalized model.

## 0.5 — HTML

Support default structural DOM and explicit `--semantic` content extraction. Semantic mode keeps title, metadata, sections, text blocks, code, lists, tables, links, images, and forms while excluding scripts/styles.

## 0.6 — measurement

Add `--stats` for raw input bytes vs rendered output bytes, absolute change, and percentage reduction/increase. Tokenizer-specific estimates remain optional future work.

Do not add redundant `--keep`, `--drop`, or `--drop-null` flags while jq already expresses those transformations.

## Experimental — sparse heterogeneous tables

Research a reversible extension such as:

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

Only advance this if representative benchmarks show material context savings without sacrificing reliable decoding/comprehension.

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
