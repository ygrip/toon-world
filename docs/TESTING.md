# Testing toon-world

CI is intentionally disabled during the initial milestone chain. Local verification is the source of truth until CI is re-enabled.

The test suite favors **behavior and data-integrity boundaries** over a raw coverage percentage. A line can be executed without proving anything useful; humans have spent decades perfecting that particular achievement.

## Required local verification

Run all four commands before accepting a milestone:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

A milestone PR is not considered locally verified until all four commands succeed.

## Test layers

### Library behavior

Small tests call public modules directly and validate one contract at a time:

- query execution and error categorization;
- output normalization and serialization;
- input adapter normalization in later milestones;
- document helpers and semantic models in later milestones.

These tests should assert semantic values, not incidental formatting, unless formatting itself is the contract.

### CLI contract

CLI tests cover the boundaries that module tests cannot:

- argument defaults and validation;
- file vs stdin routing;
- exit status and stderr category;
- stdout newline behavior;
- end-to-end query/output composition.

### Round-trip tests

Where a format contract is reversible, prefer semantic round-trip checks:

```text
source value -> encoder -> decoder -> equivalent value
```

Do not assert serializer punctuation when decoding can prove the stronger property.

## Milestone 0.1 coverage matrix

| Contract | Covered |
| --- | --- |
| default identity query | yes |
| field selection | yes |
| array filtering | yes |
| object projection | yes |
| ordered multiple query results | yes |
| empty query result stream | yes |
| query parse/compile error category | yes |
| query runtime error category | yes |
| file input | yes |
| implicit stdin | yes |
| explicit `-` stdin | yes |
| missing file error category | yes |
| malformed JSON error category | yes |
| CLI defaults and enum validation | yes |
| TOON default output | yes |
| TOON semantic round-trip | yes |
| nested/escaped TOON values | yes |
| compact JSON output | yes |
| object key order at JSON boundary | yes |
| very large integer preservation | yes |
| scalar text output | yes |
| empty text result stream | yes |
| null vs empty string | yes |
| structured value rejection in text mode | yes |

## Adding tests

A new behavior should normally add the smallest test that would fail if that behavior regressed. Prefer adding to an existing focused test file rather than creating a new file for every edge case.

Adapter milestones should always cover, where applicable:

1. normal input;
2. malformed input;
3. ordering;
4. null/empty/absent distinctions;
5. source-specific type preservation;
6. extension detection and explicit override;
7. one representative CLI query;
8. round-trip behavior when supported.

Document-format milestones should additionally cover:

- heading/section order;
- block extraction;
- links and metadata;
- semantic filtering rules;
- helper functions over the normalized model.
