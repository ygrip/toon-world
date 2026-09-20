# Milestone PR chain

Each implementation milestone is developed as a separate, stacked pull request so it can be reviewed and locally validated in isolation.

| Milestone | Branch | Scope | Status on this branch |
| --- | --- | --- | --- |
| 0.1 | `feat/query-core` | JSON + jq-compatible query core + TOON/JSON/text output | inherited |
| 0.2 | `feat/structured-adapters` | NDJSON, CSV, YAML, TOML, XML input adapters | inherited |
| 0.3 | `feat/toon-input` | TOON as a queryable input format | inherited |
| 0.4 | `feat/markdown-adapter` | Markdown normalized document model + helpers | implemented, local verification required |
| 0.5 | `feat/html-adapter` | HTML structural/semantic normalized models + helpers | later stacked PR |
| 0.6 | `feat/context-stats` | byte-size statistics for input and rendered output | later stacked PR |
| experimental | `feat/sparse-tables` | reversible sparse heterogeneous-table experiment | research milestone |

## Verification policy

CI is intentionally disabled. Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Every milestone must keep README, roadmap, milestone status, testing matrix, and PR-local smoke commands synchronized with the actual branch behavior.

## 0.4 Markdown contract

Markdown is normalized into a compact retrieval-oriented model:

```text
type: markdown
title: first H1 or null
frontmatter: raw metadata text or null
sections[]:
  heading
  level
  blocks[]
links[]
```

Typed blocks cover paragraphs, code, lists, tables, blockquotes, rules, and raw HTML blocks. Inline formatting contributes text meaning rather than creating a formatting AST.

Helpers are ordinary jaq definitions over the model:

```jq
section("Installation")
section("Usage") | code("bash")
links
```

Tests cover preambles, missing/duplicate headings, code with and without language, task lists, tables, blockquotes, rules, raw HTML, link metadata, extension/override routing, and invalid UTF-8. See [`TESTING.md`](TESTING.md).

## Stacked review rule

PR 0.4 is based on `feat/toon-input`. Review against that branch so Markdown remains one comprehensible milestone rather than a four-season recap of every earlier PR.
