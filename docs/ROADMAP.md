# toon-world roadmap

This roadmap is intentionally staged. The project should prove that it is useful before growing a private dialect of TOON.

## Phase 0 — baseline and measurement

Goal: establish a trustworthy comparison point.

- Pin the supported TOON specification version.
- Define the canonical value model used internally.
- Build a benchmark corpus containing:
  - uniform JSON arrays;
  - heterogeneous JSON arrays;
  - nested objects;
  - CSV tables;
  - XML with attributes/repeated nodes/mixed content;
  - representative HTML pages;
  - NDJSON streams.
- Measure:
  - input bytes;
  - output bytes;
  - conversion throughput;
  - peak memory;
  - optional tokenizer-specific token counts.

Exit condition: every later optimization can be compared against reproducible fixtures instead of vibes.

## Phase 1 — standard TOON transformer

Goal: ship a small, predictable single binary.

Recommended implementation: **Rust**, primarily for a small standalone binary, explicit memory control, mature streaming parsers, and predictable performance. This remains an implementation choice rather than part of the file format.

Initial inputs:

1. JSON
2. NDJSON
3. CSV
4. YAML
5. TOML

Initial output:

- standard TOON only.

CLI baseline:

```bash
toon-world FILE
toon-world --from json --to toon FILE
cat FILE | toon-world
toon-world FILE --stats
```

Requirements:

- stdin/stdout first-class;
- format auto-detection when reliable;
- explicit `--from` override;
- deterministic output;
- useful parse errors with input location;
- semantic round-trip tests through the canonical value model;
- no automatic lossy transforms.

## Phase 2 — XML and HTML adapters

Goal: handle document-shaped formats without pretending they are JSON with angle brackets.

### XML

Add a structural mapping that distinguishes:

- element names;
- attributes;
- text nodes;
- repeated child elements;
- mixed content;
- namespaces where present.

Add a separate `--semantic` mode only after structural behavior is stable.

### HTML

Add document-aware conversion with two explicit behaviors:

- structural mode for DOM-like preservation;
- semantic mode for LLM context extraction.

Semantic HTML may retain headings, paragraphs, links, lists, tables, forms, metadata, and useful image text while discarding scripts, styles, tracking attributes, and layout-only wrappers.

## Phase 3 — context-oriented controls

Goal: make the tool useful in agent pipelines without weakening default guarantees.

Potential explicit options:

```bash
--keep <selector>
--drop <selector>
--drop-null
--semantic
--stats
--tokenizer <name>
```

These operations are opt-in because some are intentionally lossy.

## Phase 4 — sparse heterogeneous table experiment

Goal: test whether non-uniform object arrays can be encoded more efficiently than standard TOON list form while remaining reversible.

Example candidate:

```text
[3]{type,repo,pr,key}:
  github,punakawan,34,~
  jira,~,~,ABC-1
  github,mom,12,~
```

Proposed semantics:

- `~` means the property was absent;
- `null` remains JSON null;
- `""` remains an empty string;
- original array order is preserved;
- no regrouping or hoisting occurs.

This must remain an explicit toon-world extension unless accepted by the TOON specification.

Before implementing it as a stable feature, compare candidate encodings by:

- bytes;
- tokenizer-specific tokens;
- encode/decode speed;
- model comprehension/retrieval accuracy;
- density of present cells.

The encoder should never assume sparse tabular is smaller. Standard list form remains the baseline.

## Phase 5 — optimization policy

Only after real benchmark data exists, consider an `--auto` context mode that chooses among safe representations or transformations.

Do **not** add numeric key aliases, constant hoisting, row regrouping, or source-shape-changing compression unless a concrete benchmark demonstrates enough gain to justify new decoding metadata and complexity.

## Release shape

A sensible release sequence is:

- `0.1`: JSON / NDJSON / CSV → standard TOON + stats
- `0.2`: YAML / TOML + stronger streaming and benchmarks
- `0.3`: XML structural adapter
- `0.4`: HTML structural + semantic adapter
- `0.x-experimental`: sparse heterogeneous tables behind an explicit flag
- `1.0`: stable CLI, documented conversion contracts, reproducible benchmarks

The version numbers are milestones, not promises. Features should move only when their contracts are stable and measured.
