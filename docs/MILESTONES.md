# Milestone PR chain

Each implementation milestone is developed as a separate, stacked pull request so it can be reviewed and locally validated in isolation.

| Milestone | Branch | Scope |
| --- | --- | --- |
| 0.1 | `feat/query-core` | JSON + jq-compatible query core + TOON/JSON/text output |
| 0.2 | `feat/structured-adapters` | NDJSON, CSV, YAML, TOML, XML input adapters |
| 0.3 | `feat/toon-input` | TOON as a queryable input format |
| 0.4 | `feat/markdown-adapter` | Markdown normalized document model + helpers |
| 0.5 | `feat/html-adapter` | HTML structural/semantic normalized models + helpers |
| experimental | `feat/sparse-tables` | reversible sparse heterogeneous-table experiment |

## Verification policy

CI is intentionally disabled during these initial milestones. Every PR must include:

- exact local formatting/lint/test/build commands;
- focused tests for the behavior introduced by that milestone;
- smoke commands with expected output;
- an explicit statement that the PR has not been runtime-verified by the implementation agent when no Rust toolchain is available.

## Test policy

Prefer high-value boundary coverage over mechanically testing every function:

- parse success + malformed input per adapter;
- format detection and explicit overrides;
- stdin/file parity where relevant;
- preservation of ordering and important type distinctions;
- representative jq filtering/projection on normalized output;
- round-trip checks when the source/target contract supports them;
- edge cases that would cause silent data loss.
