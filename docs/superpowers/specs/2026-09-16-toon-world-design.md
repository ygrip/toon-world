# toon-world initial design

Date: 2026-09-16
Status: initial design

## 1. Problem

TOON is compact when the source already fits the JSON data model well, especially uniform arrays of objects. Real context sources arrive as JSON, NDJSON, CSV, YAML, TOML, XML, and HTML, and they vary widely in shape.

`toon-world` should provide one small executable that accepts these inputs and produces compact TOON without silently changing the source meaning.

The project is not primarily a new serialization format. It is a format adapter and context-normalization tool whose default target is standard TOON.

## 2. Product contract

### Default contract

For formats that map directly into a JSON-compatible value tree:

```text
decode_toon(encode_toon(normalize(source))) == normalize(source)
```

The equality is structural/semantic, not byte-identical. Whitespace, quoting style, comments, and source-specific formatting do not have to round-trip unless explicitly supported.

Default conversion must preserve:

- object keys and values;
- array ordering;
- null versus empty string;
- booleans and numbers;
- nested structure;
- property absence.

Default conversion must not:

- drop fields;
- reorder heterogeneous records for compression;
- hoist repeated values into a new parent structure;
- create key aliases;
- emit a toon-world extension while claiming standard TOON.

### Lossy / semantic contract

Operations that intentionally discard source detail must be explicit, for example:

```bash
--semantic
--drop <selector>
--keep <selector>
--drop-null
```

A semantic conversion is optimized for useful LLM context, not reconstruction of the original source.

## 3. Standards boundary

The current TOON specification is the authority for standard output. The initial implementation should pin the supported spec version in code and documentation rather than silently changing behavior when the upstream draft changes.

As of this design, TOON 4.1 requires arrays to share the same key set and compatible column shapes before tabular form is used. Heterogeneous object arrays therefore use list form.

`toon-world` may research extensions, but an extension needs:

1. an explicit mode or target;
2. versioned semantics;
3. reversible decoding;
4. benchmark evidence;
5. clear separation from standard `.toon` output.

## 4. Architecture

```text
                  ┌──────────────┐
file / stdin ────>│ input detect │
                  └──────┬───────┘
                         │
                  ┌──────▼───────┐
                  │ format parser │
                  └──────┬───────┘
                         │ events / values
                  ┌──────▼────────┐
                  │ canonical IR   │
                  └──────┬────────┘
                         │
                  ┌──────▼────────┐
                  │ transform mode │
                  │ lossless /     │
                  │ semantic       │
                  └──────┬────────┘
                         │
                  ┌──────▼────────┐
                  │ TOON encoder   │
                  └──────┬────────┘
                         │
                  stdout / file
```

### 4.1 Input detector

Responsibilities:

- honor explicit `--from` first;
- use extension when trustworthy;
- sniff stdin only when detection is unambiguous;
- fail with a useful message rather than guessing aggressively.

### 4.2 Format adapters

Each adapter has one job: parse its source into canonical events/values while retaining source distinctions required by that adapter's contract.

Initial adapters:

- JSON
- NDJSON
- CSV
- YAML
- TOML

Later adapters:

- XML
- HTML

### 4.3 Canonical representation

For JSON-like formats, use a small value model equivalent to:

```text
Null
Bool
Number
String
Array<Value>
Object<ordered key, Value>
```

Object encounter order should be retained because TOON field order is observable in output even though JSON object semantics are not inherently ordered.

XML/HTML need an adapter-specific representation before deciding whether they collapse into the JSON-compatible value model. Mixed content cannot be represented faithfully by pretending every element is a plain object.

### 4.4 Encoder

The standard encoder should follow TOON detection rules rather than invent heuristics:

- primitive arrays -> inline form;
- uniform object arrays -> tabular form;
- compatible uniform nested objects -> nested field groups;
- non-uniform arrays -> list form;
- keyed uniform objects -> keyed tabular form where supported.

## 5. Streaming strategy

Streaming is a product goal, not a requirement that every format be processed with O(1) memory from the first release.

Use three levels:

### Level A — naturally streaming

- NDJSON
- CSV

Process rows incrementally where TOON output can be determined without buffering the full document.

### Level B — bounded lookahead

For cases where field shape must be known before a table header is emitted, buffer the minimum necessary region or spool rows when worthwhile.

### Level C — tree-backed

Nested JSON/YAML/TOML may initially use a full value tree for correctness. Replace hot paths with event-based encoding only after benchmarks show that memory is a real problem.

This avoids making the first release dramatically more complex merely so a README can contain the word "streaming" several additional times.

## 6. Input mappings

### 6.1 JSON

Direct mapping to the canonical value model.

### 6.2 NDJSON

Treat each line as one JSON value. A homogeneous object stream may be emitted as a TOON table when its shape is known and buffering policy permits it; otherwise use a standard list-compatible representation.

### 6.3 CSV

CSV headers become field names and records become rows.

Important distinctions:

- quoted delimiters must remain part of the value;
- empty cell is an empty string unless configured otherwise;
- type inference must be conservative and documented;
- an option may disable inference and preserve all cells as strings.

### 6.4 YAML

Normalize YAML values that fit the supported canonical model. YAML-specific features such as anchors, aliases, custom tags, duplicate keys, or non-string mapping keys need explicit rejection or documented normalization rules rather than silent corruption.

### 6.5 TOML

Map tables, arrays, strings, booleans, and numbers into the canonical model. Date/time values require an explicit representation rule because JSON/TOON do not have an intrinsic datetime primitive.

### 6.6 XML

Structural XML must distinguish at minimum:

- elements;
- attributes;
- text nodes;
- child ordering;
- repeated elements;
- mixed content;
- namespaces where retained.

Example mixed content:

```xml
<p>Hello <strong>world</strong>!</p>
```

cannot safely become `p: Hello world!` in structural mode because element boundaries and text ordering disappear.

Semantic XML may intentionally simplify this structure behind `--semantic`.

### 6.7 HTML

HTML should not be implemented as a trivial XML alias.

Structural mode retains meaningful DOM structure.

Semantic mode may retain:

- title and document metadata useful to an LLM;
- headings;
- paragraphs and text;
- links and destinations;
- lists;
- tables;
- forms and labels;
- image alt text.

It may discard:

- scripts;
- styles;
- tracking attributes;
- purely presentational wrappers;
- framework hydration payloads unless explicitly requested.

## 7. Sparse heterogeneous table research

### Problem

Standard TOON cannot tabularize:

```json
[
  {"type":"github","repo":"punakawan","pr":34},
  {"type":"jira","key":"ABC-1"},
  {"type":"github","repo":"mom","pr":12}
]
```

because rows do not share the same field set.

### Candidate extension

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
- row order remains unchanged;
- header is the stable union of encountered fields.

### Why this is only an experiment

A sparse table can be worse than standard list form when row density is low. It also introduces syntax not defined by standard TOON.

The experiment succeeds only if it demonstrates meaningful improvement on representative data without degrading model comprehension.

Compare candidate output with standard TOON using:

```text
encoded bytes
encoded tokens per tokenizer
encode throughput
decode throughput
peak memory
model extraction / retrieval accuracy
```

Do not use a fixed density threshold until measurements justify one. The simplest candidate policy is to encode both representations for a bounded sample and choose the cheaper representation only in explicit extension mode.

## 8. CLI design

Baseline:

```bash
toon-world [FILE]
toon-world [FILE] --from <format>
toon-world [FILE] --to toon
toon-world [FILE] --stats
cat input.json | toon-world
```

Later explicit transformations:

```bash
toon-world page.html --semantic
toon-world input.json --keep id,name,status
toon-world input.json --drop metadata.debug
toon-world input.json --drop-null
```

Possible experimental surface:

```bash
toon-world input.json --experimental-sparse-table
```

Avoid a large flag taxonomy until actual use cases require it.

## 9. Performance goals

The project should optimize for:

- fast startup;
- single self-contained binary;
- low idle/runtime overhead;
- linear-time parsing/encoding for ordinary inputs;
- bounded copying;
- stdout-friendly operation in shell and agent pipelines.

Recommended implementation language: Rust.

Reasons:

- strong standalone binary story;
- explicit ownership and allocation control;
- mature parsers for the target formats;
- good streaming I/O primitives;
- straightforward cross-platform release artifacts.

The design should not rely on unsafe code for ordinary parsing/encoding paths unless profiling later proves a concrete need.

## 10. Error handling

Errors should identify:

- input format;
- byte/line/column when available;
- offending construct;
- whether the failure is parse-time, normalization-time, or encode-time;
- a corrective hint when one is obvious.

Examples:

```text
error[xml]: mixed content cannot be represented by the selected mapping at line 18
hint: use structural XML mode or --semantic
```

```text
error[input]: could not reliably detect stdin format
hint: pass --from json|yaml|csv|...
```

## 11. Verification

Every format adapter needs fixture-based tests covering:

- primitives;
- empty structures;
- nested structures;
- escaping;
- null / empty / absent distinctions;
- malformed input;
- Unicode;
- ordering behavior.

Core invariants:

1. JSON-compatible lossless inputs structurally round-trip through TOON.
2. Default output is valid standard TOON for the pinned spec.
3. Semantic/lossy behavior only occurs behind explicit options.
4. Benchmark fixtures are version-controlled and reproducible.
5. Experimental sparse encoding has decode fixtures proving absent/null/empty distinctions.

## 12. Scope for the first implementation cycle

Keep the first cycle smaller than the whole roadmap:

- Rust project scaffold;
- JSON input;
- standard TOON encoder/decoder integration or implementation strategy;
- stdin/stdout;
- `--from json` plus JSON auto-detection;
- `--stats` for bytes;
- fixture tests and a small benchmark harness.

CSV and NDJSON should be the next adapters because they exercise the tabular and streaming paths without dragging XML's centuries of accumulated personality into the first milestone.

## 13. Open decisions before implementation

These should be resolved before code commits begin:

1. Reuse an existing Rust TOON crate versus implement the pinned subset directly.
2. Exact XML structural mapping contract.
3. Exact HTML semantic retention policy.
4. Tokenizer support strategy for `--stats`.
5. Name and wire format, if any, for the sparse-table extension.

None of these block the initial repository bootstrap or the JSON-first implementation plan.
