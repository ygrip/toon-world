# Testing toon-world

CI is intentionally disabled during the initial milestone chain. Local verification is the source of truth until CI is re-enabled.

The suite targets behavioral contracts and silent-data-loss risks rather than chasing a decorative line-coverage percentage.

## Required local verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

A milestone is not locally verified until all four commands succeed.

## Inherited coverage

Milestone 0.3 includes the complete query/output coverage from 0.1 and structured-adapter coverage from 0.2:

- query identity, filtering, projection, errors, ordered/empty streams;
- CLI defaults, stdin/file routing, missing-file errors;
- TOON output semantic round-trip and JSON/text output boundaries;
- NDJSON ordering;
- CSV text preservation, quoting/CRLF, invalid headers;
- YAML native types, aliases, and multi-documents;
- nested TOML;
- XML attributes, ordered children, mixed content, multi-root input;
- format detection, overrides, malformed input, UTF-8 errors.

## TOON input coverage introduced in 0.3

| Contract | Covered |
| --- | --- |
| `.toon` extension detection | yes |
| case-insensitive TOON extension | yes |
| explicit `--from toon` | yes |
| explicit TOON stdin | yes |
| explicit TOON override over misleading extension | yes |
| standard tabular TOON query | yes |
| multiple query results preserve order | yes |
| JSON -> TOON -> decoder -> JSON semantic round-trip | yes |
| absent property vs `null` vs empty string | yes |
| nested objects/arrays | yes |
| heterogeneous nested values | yes |
| empty object/array preservation | yes |
| malformed TOON error category | yes |
| invalid UTF-8 error category | yes |

## Round-trip principle

TOON tests prefer semantic equivalence over exact serialized text:

```text
JSON-compatible value
  -> toon-format encoder
  -> toon-world TOON decoder
  -> compact JSON output
  -> equivalent JSON-compatible value
```

This proves more than asserting commas, indentation, or quoting from a particular encoder version.

The hard-coded TOON table fixture remains intentionally small. Its purpose is to prove toon-world can consume ordinary TOON syntax without requiring its own encoder to have produced the data first.

## Compatibility note

Encoding and decoding both use `toon-format` 0.5.x in this milestone. Tests therefore validate the compatibility contract actually shipped by the dependency, not a newer TOON revision the binary does not yet implement.

## Adding tests

Any TOON input bug involving information loss should gain a semantic round-trip regression. Syntax-only parsing bugs should use the smallest hand-written TOON fixture that reproduces the issue.
