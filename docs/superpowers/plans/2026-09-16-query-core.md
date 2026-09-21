# Query Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first useful `toon-world` vertical slice: JSON from file/stdin, jq-compatible querying, and TOON/JSON/text output in one Rust binary.

**Architecture:** Keep CLI, input parsing, query execution, and output encoding behind focused modules. Embed `jaq` for jq semantics. Use the upstream Rust TOON encoder only behind an adapter so its spec-version lag can be upgraded without touching query/input code.

**Tech Stack:** Rust 2021, clap, anyhow/thiserror, serde_json, jaq-core/jaq-std/jaq-json, toon-format.

**Spec:** `docs/superpowers/specs/2026-09-16-toon-world-design.md`

## Global Constraints

- Single native executable; no runtime jq/Node/Python dependency.
- JSON is the only required input format in this cycle.
- Query is optional; omitted query means `.`.
- Default structured output is TOON.
- `--to json` emits compact JSON.
- `--to text` accepts scalar output only.
- Keep the query engine isolated from CLI and output modules.
- Do not claim TOON 4.1 conformance while the selected Rust dependency does not provide it.

---

### Task 1: CLI and JSON input boundary

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/cli.rs`
- Create: `src/input.rs`
- Create: `src/main.rs`
- Create: `tests/cli.rs`

**Interfaces:**
- Produces: `cli::Args`, `input::read_json(path: Option<&Path>) -> Result<jaq_json::Val>`.

- [ ] Write failing CLI/input tests for file input, stdin input, default query `.`, and malformed JSON.
- [ ] Run the focused tests and confirm the expected missing-code failures.
- [ ] Add the minimal clap/input implementation.
- [ ] Re-run focused tests, then `cargo test`.

### Task 2: jq-compatible query engine

**Files:**
- Create: `src/query.rs`
- Create: `tests/query.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `jaq_json::Val`.
- Produces: `query::execute(query: &str, input: jaq_json::Val) -> Result<Vec<jaq_json::Val>>`.

- [ ] Write failing tests for `.name`, `.users[] | select(.active)`, projection `{id,name}`, multiple results, and malformed filters.
- [ ] Confirm failures before implementation.
- [ ] Embed jaq parser/compiler/runtime with the standard/json function sets.
- [ ] Re-run tests and keep query compile/runtime errors distinguishable.

### Task 3: output encoders

**Files:**
- Create: `src/output.rs`
- Create: `tests/output.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `OutputFormat::{Toon,Json,Text}` and `output::encode(value: &jaq_json::Val, format: OutputFormat) -> Result<String>`.

- [ ] Write failing tests for TOON table output, compact JSON, raw string/number/bool/null text, and rejection of arrays/objects in text mode.
- [ ] Confirm failures.
- [ ] Enable jaq-json serde support and encode through serde boundaries.
- [ ] Keep the TOON dependency isolated in this module and document its supported upstream spec version.
- [ ] Re-run output tests and full suite.

### Task 4: wire the end-to-end command

**Files:**
- Modify: `src/main.rs`
- Modify: `tests/cli.rs`

**Interfaces:**
- Flow: `Args -> read_json -> execute -> encode -> stdout`.

- [ ] Add failing integration tests for identity conversion, query selection, filtering, projection, JSON output, text output, and stdin.
- [ ] Confirm failures.
- [ ] Wire the modules with deterministic newline-separated result streaming.
- [ ] Re-run integration tests and full suite.

### Task 5: quality and release baseline

**Files:**
- Create: `.gitignore`
- Create: `.github/workflows/ci.yml`
- Modify: `README.md`

- [ ] Add usage examples that exactly match implemented flags.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test`.
- [ ] Run `cargo build --release` and record binary size as a baseline rather than a target claim.
- [ ] Review dependency/spec caveats in README before opening the PR.

## Deferred

NDJSON, CSV, YAML, TOML, XML, Markdown, HTML, TOON input, semantic document helpers, stats/tokenizers, and sparse heterogeneous tables are separate implementation slices after this core path is stable.