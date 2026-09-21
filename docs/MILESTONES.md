# Milestone PR chain

Each implementation milestone is developed as a separate, stacked pull request so it can be reviewed and locally validated in isolation.

| Milestone | Branch | Scope | Status on this branch |
| --- | --- | --- | --- |
| 0.1 | `feat/query-core` | JSON + jq-compatible query core + TOON/JSON/text output | implemented, local verification required |
| 0.2 | `feat/structured-adapters` | NDJSON, CSV, YAML, TOML, XML input adapters | later stacked PR |
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
- updated `README.md`, roadmap/milestone documentation, and [`TESTING.md`](TESTING.md);
- an explicit statement that the PR has not been runtime-verified by the implementation agent when no Rust toolchain is available.

The authoritative local verification sequence is:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

## Test policy

Prefer high-value boundary coverage over mechanically testing every function:

- parse success + malformed input per adapter;
- format detection and explicit overrides;
- stdin/file parity where relevant;
- preservation of ordering and important type distinctions;
- representative jq filtering/projection on normalized output;
- round-trip checks when the source/target contract supports them;
- edge cases that would cause silent data loss;
- CLI defaults, error categories, and stdout behavior.

Milestone 0.1 specifically covers the query/output spine, including empty result streams, large integer preservation, nested TOON round-trips, and scalar-vs-structured text behavior. See [`TESTING.md`](TESTING.md) for the full matrix.

## Stacked review rule

Each later PR uses the previous milestone branch as its base. Review each PR against that base, not against `main`; otherwise GitHub will faithfully present every previous milestone again, because apparently humans needed another way to create avoidable review noise.
