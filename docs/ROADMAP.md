# toon-world roadmap

`toon-world` is a universal query/transform bridge whose default structured output is TOON. Conversion exists so data from different formats can be filtered through one jq-compatible interface before it reaches an agent or another shell command.

## Phase 0 — baseline and measurement

- Pin Rust/MSRV and dependency versions.
- Define the normalized value contract.
- Establish fixtures for uniform, heterogeneous, nested, and scalar data.
- Measure startup time, throughput, output bytes, and peak memory.
- Track TOON spec/version support explicitly.

## Phase 1 — JSON + jq-compatible query core

Goal: the first genuinely useful binary.

```bash
toon-world data.json
toon-world data.json -q '.users[] | select(.active)'
cat data.json | toon-world -q '.users[] | {id,name}'
toon-world package.json -q '.version' --to text
```

Deliver:

- Rust single binary;
- file or stdin input;
- JSON input;
- embedded `jaq` query compilation/execution;
- identity query when `-q` is omitted;
- TOON default output;
- JSON and text output;
- deterministic errors and exit codes;
- integration tests for conversion, selection, filtering, projection, scalar output, stdin, and malformed filters.

## Phase 2 — structured format adapters

Reuse jaq's existing format ecosystem where its mapping contracts fit toon-world:

1. NDJSON
2. CSV
3. YAML
4. TOML
5. XML

The adapter boundary must remain independent from the query engine. Query expressions operate on normalized values, not source syntax.

## Phase 3 — TOON input

Add TOON decoding as a first-class input once the selected Rust implementation is compatible with the TOON specification version toon-world claims to support. Do not silently claim current-spec conformance while embedding an older implementation.

## Phase 4 — Markdown and HTML

Document formats get explicit normalized schemas rather than being treated as awkward JSON.

### Markdown

Expose headings, sections, paragraphs, lists, links, code blocks, tables, and frontmatter. Structural mode preserves document semantics; semantic mode may collapse formatting noise.

Planned query sugar:

```bash
toon-world README.md -q 'section("Installation")'
toon-world README.md -q 'section("Usage") | code("bash")'
```

### HTML

Expose meaningful document content such as title, headings, sections, links, lists, tables, forms, and image alt text. Semantic mode may remove scripts, styles, tracking attributes, hydration payloads, and layout-only wrappers.

```bash
toon-world page.html -q 'links()'
```

Document helpers compile into the same query engine; there is no second DSL.

## Phase 5 — context-oriented controls

Potential explicit transforms:

```bash
--keep <selector>
--drop <selector>
--drop-null
--semantic
--stats
--tokenizer <name>
```

These remain opt-in when they can change information content.

## Phase 6 — sparse heterogeneous table experiment

Research a reversible extension for arrays whose objects have overlapping but non-identical fields:

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

The extension must beat standard TOON on representative byte/token benchmarks and preserve model comprehension before it earns a stable format commitment.

## Release shape

- `0.1`: JSON + jq-compatible queries + TOON/JSON/text output
- `0.2`: NDJSON/CSV/YAML/TOML/XML adapters
- `0.3`: TOON input with explicitly pinned spec compatibility
- `0.4`: Markdown structural adapter + section/code helpers
- `0.5`: HTML structural/semantic adapter + document helpers
- `0.x-experimental`: sparse heterogeneous table mode
- `1.0`: stable CLI, documented format contracts, reproducible performance/token benchmarks

The version numbers are milestones, not promises. The tool should remain small enough that an agent can invoke it casually without summoning an ecosystem.