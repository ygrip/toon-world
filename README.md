# toon-world

A fast, lightweight, single-binary transformer that turns structured data into compact, LLM-friendly [TOON](https://github.com/toon-format/spec).

> **Status:** design / bootstrap

## Why

TOON is excellent when data already matches its strongest shape: uniform arrays of objects. Real inputs are messier. APIs return irregular JSON, CSV is already tabular, XML carries attributes and repeated elements, and HTML contains far more presentation syntax than useful context.

`toon-world` aims to be the thin normalization layer between those formats and TOON without quietly changing the meaning of the source.

```text
JSON ───┐
YAML ───┤
TOML ───┤
CSV ────┤
NDJSON ─┤──> normalize ──> TOON
XML ────┤
HTML ───┘
```

## Principles

1. **Lossless by default**
   - Preserve the logical value tree and source ordering where the target model supports it.
   - A default conversion must not silently drop fields, regroup records, or hoist values.

2. **Standard TOON first**
   - Default output should conform to the current TOON specification.
   - Any toon-world-only extension must be explicit, versioned, benchmarked, and opt-in.

3. **Optimize structure before syntax tricks**
   - Do not add alias dictionaries or custom metadata unless measured savings justify the complexity.
   - Prefer the natural TOON representation for the existing data shape.

4. **Streaming where practical**
   - stdin/stdout should be first-class.
   - Large inputs should not require multiple full copies in memory.

5. **Measure, do not guess**
   - Report bytes and, when a tokenizer is selected, estimated token counts.
   - Experimental encodings should only win when they are actually smaller.

## Initial CLI shape

```bash
# auto-detect input, emit standard TOON
toon-world input.json

toon-world input.yaml
toon-world input.csv
cat response.ndjson | toon-world

# explicit formats
toon-world input.xml --from xml --to toon

# inspect savings
toon-world input.json --stats

# intentionally semantic / lossy transforms
toon-world page.html --semantic
```

The default path is boring on purpose: parse, normalize, encode. Cleverness belongs behind explicit flags until it has earned the privilege.

## Format direction

### JSON / YAML / TOML / NDJSON

Normalize into the JSON-compatible value model, then encode as standard TOON.

### CSV

Map the header to a TOON tabular field list and rows to TOON rows. CSV is already close to TOON's ideal shape.

### XML

Provide two eventual modes:

- **structural**: preserve elements, attributes, repeated nodes, and mixed content faithfully enough to reconstruct the logical XML tree.
- **semantic**: intentionally collapse XML ceremony when exact reconstruction is not required.

### HTML

Treat HTML separately from generic XML:

- default structural conversion preserves meaningful DOM structure;
- `--semantic` extracts useful document content such as headings, text, links, lists, tables, and form metadata while dropping presentation noise.

## Sparse heterogeneous tables

A promising experiment is allowing heterogeneous object arrays to use a sparse tabular representation.

Input:

```json
[
  {"type":"github","repo":"punakawan","pr":34},
  {"type":"jira","key":"ABC-1"},
  {"type":"github","repo":"mom","pr":12}
]
```

Standard TOON uses list form because the objects do not share one field set.

A toon-world extension could experimentally encode the union of fields and use an explicit **absent** sentinel distinct from JSON `null` and the empty string:

```text
[3]{type,repo,pr,key}:
  github,punakawan,34,~
  jira,~,~,ABC-1
  github,mom,12,~
```

This is **not standard TOON today** and therefore must not be emitted by the default encoder. It is a research direction to benchmark against standard list form before any format commitment.

## Non-goals for the first release

- inventing a replacement for TOON;
- numeric key aliases;
- schema inference systems;
- lossy field filtering by default;
- preserving byte-identical source formatting;
- turning every source format into one giant AST in memory.

## Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the phased plan and [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md) for the initial design.

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
