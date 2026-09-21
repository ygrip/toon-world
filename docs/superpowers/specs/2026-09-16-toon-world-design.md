# toon-world initial design

Date: 2026-09-16
Status: approved for initial implementation

## 1. Problem

The value of TOON is not merely converting punctuation from one serialization into another. Agent and shell workflows usually need only a slice of a large input. `toon-world` therefore exists to read multiple structured/document formats through one interface, query or transform the normalized data, and emit a compact result.

The product is a **universal query and transformation CLI with TOON as the default structured output**, not a new serialization format and not a replacement query language.

## 2. Product contract

### Default flow

```text
source -> parser/adapter -> normalized value -> jq-compatible query -> output encoder
```

The identity query is used when the caller supplies no query.

For JSON-compatible inputs, conversion without an explicit lossy option must preserve the logical value tree. Equality is structural/semantic, not byte-identical.

Default conversion must preserve:

- object keys and values;
- array ordering;
- null versus empty string;
- booleans and numbers;
- nested structure;
- property absence.

Default conversion must not silently:

- drop fields;
- reorder heterogeneous records;
- hoist repeated values into a new parent structure;
- create key aliases;
- emit a toon-world extension while claiming standard TOON.

A query may intentionally return only part of the source. That is not considered an implicit lossy conversion because the selection is explicitly requested by the caller.

## 3. CLI contract

The initial CLI keeps conversion trivial and avoids ambiguous positional query/file parsing:

```bash
toon-world [FILE]
toon-world [FILE] -q '<jq filter>'
cat input.json | toon-world -q '<jq filter>'
toon-world [FILE] -q '<jq filter>' --to toon|json|text
```

Rules:

- omitted `FILE` reads stdin;
- omitted `-q` means identity filter `.`;
- default structured output is `toon`;
- `--to json` emits compact JSON initially;
- `--to text` requires scalar/string-like output and prints its raw value;
- multiple jq results are emitted as a result stream; the initial implementation may encode each result independently rather than implicitly collecting them.

## 4. Architecture

```text
JSON ───┐
YAML ───┤
TOML ───┤
CSV ────┤
NDJSON ─┤
XML ────┤
HTML ───┤──> adapters -> normalized values -> jaq -> output encoder
MD ─────┤
TOON ───┘
```

### 4.1 Query engine

Embed `jaq` rather than implementing a new jq dialect. `jaq-core` provides parsing/compilation/execution and the jaq ecosystem already supports several structured formats. toon-world owns the stable CLI and format contracts around it.

The query engine must be isolated behind a small interface so jaq API changes do not leak through the rest of the codebase.

Conceptual interface:

```rust
pub fn execute(query: &str, input: Value) -> Result<Vec<Value>, QueryError>;
```

The concrete internal value may initially use `jaq_json::Val`; boundaries should prevent it from infecting adapters and encoders unnecessarily.

### 4.2 Input adapters

Each adapter converts source syntax into normalized queryable values. Explicit `--from` wins; extension-based detection follows; stdin sniffing is conservative.

Initial implementation: JSON only.

Next structured adapters should prefer jaq's existing format support where the mapping contract matches toon-world requirements: NDJSON, CSV, YAML, TOML, XML.

Markdown and HTML remain toon-world-specific document adapters.

### 4.3 Output encoders

Initial outputs:

- `toon` — default for structured values;
- `json` — compact JSON;
- `text` — scalar/raw shell output.

TOON encoding is an adapter boundary of its own. The project must explicitly document which upstream TOON spec/version the selected Rust encoder supports. The current TOON specification is 4.1 while the published official Rust crate may lag; toon-world must not claim unsupported conformance.

## 5. Markdown model

Markdown should be parsed to an AST and normalized into a stable document model, not treated as plain text with regular expressions.

The model should make these concepts queryable:

- title;
- frontmatter;
- headings and levels;
- sections;
- paragraphs;
- lists;
- links;
- code blocks and language;
- tables;
- blockquotes.

Planned sugar such as:

```jq
section("Installation")
code("rust")
```

must compile to or register as functions in the same query engine. They are not a second DSL.

## 6. HTML model

HTML is document-shaped and must not be a trivial XML alias. Structural mode preserves meaningful DOM ordering and attributes needed by the contract. Semantic mode may intentionally retain headings, text, links, lists, tables, forms, metadata, and alt text while removing scripts, styles, tracking attributes, hydration data, and layout-only wrappers.

Planned helper:

```jq
links()
```

## 7. Streaming

Streaming remains a product goal, but query execution often requires materialized values depending on the filter. Do not advertise O(1) memory for arbitrary jq programs.

Use streaming where it naturally survives the query boundary:

- NDJSON/CSV records when the query can be evaluated per record;
- direct conversion paths with identity/simple projections;
- output result streaming.

Correctness comes before heroic streaming machinery.

## 8. Sparse heterogeneous tables

Standard TOON uses list form for heterogeneous object arrays. A possible toon-world extension may use the stable union of fields plus an explicit absent sentinel:

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

This remains experimental and opt-in until byte/token savings, encode/decode cost, and model comprehension are benchmarked. Numeric alias dictionaries, regrouping, and constant hoisting are out of scope until evidence says otherwise.

## 9. Performance and implementation

Implementation language: Rust.

Goals:

- one self-contained executable;
- fast startup;
- low allocation/copying overhead;
- deterministic output;
- stdin/stdout friendliness;
- release binaries for Linux/macOS/Windows;
- safe Rust for ordinary paths.

Use mature libraries instead of rebuilding foundations. In particular, embed jaq for jq semantics.

## 10. Error contract

Errors identify the stage:

```text
error[input]: ...
error[parse:json]: ...
error[query]: ...
error[encode:toon]: ...
```

When locations are available, include line/column or query span. Invalid filters must fail before data execution when possible.

## 11. Verification

Initial tests must cover:

- JSON identity conversion;
- field selection;
- array iteration and `select`;
- object projection;
- multiple jq results;
- stdin and file input;
- malformed JSON;
- malformed jq filter;
- structured TOON output;
- compact JSON output;
- scalar text output and rejection of non-scalar text output.

Later adapters add fixture-based tests for escaping, Unicode, empty structures, format-specific edge cases, and normalized tree contracts.

## 12. Initial implementation scope

The first implementation cycle deliberately stops at a useful vertical slice:

1. Rust CLI scaffold.
2. JSON file/stdin input.
3. Embedded jaq query execution.
4. Identity query by default.
5. TOON/JSON/text output.
6. Integration/unit tests.
7. Basic byte statistics may follow after the core path is stable.

CSV/YAML/XML/Markdown/HTML are subsequent slices, not excuses to keep v0.1 hypothetical.