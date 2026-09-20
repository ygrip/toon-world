# Testing toon-world

CI is intentionally disabled during the initial milestone chain. Local verification remains authoritative.

## Required local verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

A branch is not considered locally verified until all four commands succeed.

## Checked-in sample files

`tests/fixtures/` contains representative JSON, NDJSON, CSV, YAML, TOML, XML fragment, TOON, Markdown, and HTML files. `sample_files.rs` parses and queries each file, including XML fragment order and semantic HTML. This prevents regressions from tests that only exercise inline or temporary strings.

`--stats` coverage asserts exact input/output byte accounting and stderr-only output for a checked-in JSON sample.

## Inherited coverage

Later milestones inherit the structured/query coverage from earlier milestones, including:

- jq query and output boundaries;
- file/stdin/CLI error behavior;
- JSON, NDJSON, CSV, YAML, TOML, XML adapters;
- TOON decode and semantic round-trips.
- checked-in `.jsonl` file inference and query routing through the CLI.

## Markdown coverage introduced in 0.4

| Contract | Covered |
| --- | --- |
| `.md` / `.markdown` detection | yes |
| case-insensitive Markdown extension | yes |
| explicit `--from markdown` | yes |
| explicit override over misleading extension | yes |
| title from first H1 | yes |
| no-H1 title remains null | yes |
| preamble before first heading retained | yes |
| section order | yes |
| heading levels | yes |
| YAML-style frontmatter retained raw | yes |
| paragraph normalization | yes |
| inline emphasis/code flattened to text meaning | yes |
| duplicate heading lookup preserves order | yes |
| missing `section()` returns empty stream | yes |
| fenced code language | yes |
| code block without language uses null | yes |
| `code("lang")` helper | yes |
| list normalization | yes |
| task-list markers | yes |
| table headers/rows | yes |
| blockquotes | yes |
| horizontal rules | yes |
| raw HTML block retention | yes |
| link text/href/title index | yes |
| invalid UTF-8 error category | yes |

## Document test philosophy

Markdown normalization is retrieval-oriented, not byte-identical reconstruction. Tests therefore defend **document meaning and queryability**, not which Markdown spelling produced it.

For example, emphasis nodes are intentionally flattened:

```md
Use **bold**, *italic*, and `code`.
```

becomes semantic paragraph text equivalent to:

```text
Use bold, italic, and code.
```

Likewise, `section("Name")` is tested as a jaq helper over the normalized `sections` array. Duplicate headings are allowed and must return multiple values in source order.

## Regression rule

When a Markdown bug is reported, prefer a minimal source fragment plus an assertion on the normalized value or helper output. Avoid snapshotting the entire document unless the whole schema is under test; enormous snapshots mostly prove that enormous snapshots can be reviewed poorly.

## Compact sparse-TOON experiment

The sparse-TOON suite covers lossless encode/decode, nested-object flattening, JSON-Pointer escaping, array leaves, absent/null/empty distinction, standard-TOON fallback, and CLI routing. `compact_benchmarks.rs` measures exact bytes and `cl100k_base` tokens for checked-in sparse, dense, and nested fixtures; only the sparse fixture has a required token reduction gate.
