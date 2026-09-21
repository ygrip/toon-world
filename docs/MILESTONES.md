# Milestone PR chain

Each implementation milestone is developed as a separate, stacked pull request so it can be reviewed and locally validated in isolation.

| Milestone | Branch | Scope | Status on this branch |
| --- | --- | --- | --- |
| 0.1 | `feat/query-core` | JSON + jq-compatible query core + TOON/JSON/text output | inherited |
| 0.2 | `feat/structured-adapters` | NDJSON, CSV, YAML, TOML, XML input adapters | implemented, local verification required |
| 0.3 | `feat/toon-input` | TOON as a queryable input format | later stacked PR |
| 0.4 | `feat/markdown-adapter` | Markdown normalized document model + helpers | later stacked PR |
| 0.5 | `feat/html-adapter` | HTML structural/semantic normalized models + helpers | later stacked PR |
| 0.6 | `feat/context-stats` | byte-size statistics for input and rendered output | later stacked PR |
| experimental | `feat/sparse-tables` | reversible sparse heterogeneous-table experiment | research milestone |

## Verification policy

CI is intentionally disabled during these initial milestones. Every PR must include:

- exact local formatting/lint/test/build commands;
- focused tests for the behavior introduced by that milestone;
- smoke commands with expected output;
- updated README, roadmap/milestone docs, and [`TESTING.md`](TESTING.md);
- an explicit statement that runtime verification still belongs to local validation while this environment lacks Rust.

Use:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

## 0.2 normalization contract

The structured adapter milestone deliberately keeps one common query model while preserving source semantics:

| Format | Contract |
| --- | --- |
| JSON | unchanged JSON value |
| NDJSON / JSONL | ordered array of parsed JSON values |
| CSV | header row becomes object keys; all cells remain strings |
| YAML | native scalar/container types; multiple documents become an ordered array |
| TOML | tables/arrays/scalars map into the common value model |
| XML | structural `t`/`a`/`c` representation preserving ordered children |

No adapter is allowed to perform silent convenience coercions merely because `001` looked lonely and wanted to become the number `1`.

## Test policy

The 0.2 branch inherits the complete 0.1 query/output suite and adds adapter-focused coverage for:

- extension detection and explicit overrides;
- malformed input categories;
- ordering;
- source type preservation;
- CSV quoted/CRLF edge cases and invalid headers;
- YAML aliases;
- nested TOML;
- XML mixed content;
- invalid UTF-8 on text formats;
- representative file and stdin CLI routes.

See [`TESTING.md`](TESTING.md) for the detailed matrix.

## Stacked review rule

PR 0.2 is based on `feat/query-core`. Later milestones base on this branch in turn. Review each PR against its immediate base branch so the diff contains only that milestone.
