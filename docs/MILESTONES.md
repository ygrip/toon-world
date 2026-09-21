# Milestone PR chain

Each implementation milestone is developed as a separate, stacked pull request so it can be reviewed and locally validated in isolation.

| Milestone | Branch | Scope | Status on this branch |
| --- | --- | --- | --- |
| 0.1 | `feat/query-core` | JSON + jq-compatible query core + TOON/JSON/text output | inherited |
| 0.2 | `feat/structured-adapters` | NDJSON, CSV, YAML, TOML, XML input adapters | inherited |
| 0.3 | `feat/toon-input` | TOON as a queryable input format | implemented, local verification required |
| 0.4 | `feat/markdown-adapter` | Markdown normalized document model + helpers | later stacked PR |
| 0.5 | `feat/html-adapter` | HTML structural/semantic normalized models + helpers | later stacked PR |
| 0.6 | `feat/context-stats` | byte-size statistics for input and rendered output | later stacked PR |
| experimental | `feat/sparse-tables` | reversible sparse heterogeneous-table experiment | research milestone |

## Verification policy

CI is intentionally disabled during these initial milestones. Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Every milestone PR carries milestone-specific tests, smoke commands, and updated README/roadmap/testing documentation.

## 0.3 TOON contract

TOON is decoded into a JSON-compatible value and then enters the same jaq query pipeline as every other structured input:

```text
TOON -> decode -> common value -> jaq -> TOON / JSON / text
```

This branch deliberately uses the same `toon-format` dependency for encode and decode. The compatibility claim is therefore limited to the TOON version that dependency documents, currently v3.0 for the selected 0.5.x release.

Tests defend:

- hand-written tabular TOON consumption;
- semantic round-trip of nested and heterogeneous values;
- absent vs null vs empty-string distinctions;
- empty containers;
- query result ordering;
- malformed/invalid UTF-8 errors;
- extension detection and explicit overrides.

See [`TESTING.md`](TESTING.md) for the complete inherited + 0.3 matrix.

## Stacked review rule

PR 0.3 is based on `feat/structured-adapters`. Review against that branch so the PR shows the TOON-input delta instead of reenacting milestones 0.1 and 0.2 for sport.
