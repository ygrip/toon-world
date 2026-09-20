# toon-world

A fast, lightweight, single-binary query and transformation tool for structured and document data, with compact [TOON](https://github.com/toon-format/spec) output by default.

> **Milestone 0.6:** HTML adapters and machine-readable byte statistics complete the current stack.

## Why

Conversion alone is not the product. `toon-world` exists so different formats can be parsed into one queryable model, filtered before they enter an agent's context, and emitted compactly.

```text
JSON ───┐
NDJSON ─┤
CSV ────┤
YAML ───┤
TOML ───┤
XML ────┤
TOON ───┤
MD ─────┤──> adapter ──> normalized value ──> jq-compatible query ──> TOON / JSON / text
HTML ───┘
```

`toon-world` embeds [`jaq`](https://github.com/01mf02/jaq); Markdown helpers are jaq definitions over the normalized document model, not a second query language.

## Implemented through 0.6

- JSON, NDJSON/JSONL, CSV, YAML, TOML, XML, TOON, Markdown, and HTML input
- extension inference plus explicit `--from`
- jq-compatible selection/filter/projection
- TOON default output, compact JSON, scalar text output
- Markdown normalized into ordered sections and typed blocks
- document helpers: `section("...")`, `code("...")`, and `links`
- structural and semantic HTML document models
- `--stats` JSON byte statistics on stderr

## Installation

Build and install from a checkout:

```bash
# macOS / Linux
./scripts/install.sh

# Windows PowerShell
.\scripts\install.ps1
```

Both installers use an existing Rust toolchain or install it with official rustup, build a release binary, and copy it to the user-local bin directory. They print PATH guidance instead of modifying shell profiles.

## Markdown model

Markdown is normalized for retrieval rather than byte-identical reconstruction:

```json
{
  "type": "markdown",
  "title": "toon-world",
  "frontmatter": "title: toon-world",
  "sections": [
    {
      "heading": "Installation",
      "level": 2,
      "blocks": [
        {"type":"paragraph","text":"Install it locally."},
        {"type":"code","lang":"bash","text":"cargo install --path ."}
      ]
    }
  ],
  "links": [
    {"text":"GitHub","href":"https://github.com/ygrip/toon-world","title":null}
  ]
}
```

### Section semantics

- content before the first heading is retained as a preamble section with `heading: null` and `level: 0`;
- the first H1 becomes `title`;
- a document without H1 has `title: null`;
- duplicate headings are allowed and `section("Name")` returns each match in source order;
- heading levels and section order are preserved.

### Block semantics

Supported normalized blocks:

- paragraph;
- fenced/indented code with optional language;
- ordered/unordered list;
- task-list markers inside list text;
- table headers + rows;
- blockquote;
- horizontal rule;
- raw HTML block.

Inline emphasis, strong, strikethrough, and inline code contribute their text meaning rather than creating formatting-only AST nodes.

YAML-style frontmatter is kept as raw metadata text. It is not silently reinterpreted into a second schema.

## Document queries

```bash
# exact section heading; duplicate headings produce multiple results
toon-world README.md -q 'section("Installation")'

# bash code in one section
toon-world README.md -q 'section("Usage") | code("bash")'

# all document links
toon-world README.md -q 'links'

# ordinary jq remains available
toon-world README.md -q '.sections[] | {heading,level}'
```

A missing `section()` produces an empty jq result stream, following normal query semantics rather than inventing a Markdown-specific error.

## Structured input contracts

| Input | Normalization |
| --- | --- |
| JSON | JSON value unchanged |
| NDJSON / JSONL | ordered array of line values |
| CSV | header row becomes object keys; every cell remains a string |
| YAML | native scalar/container types; multiple documents become an ordered array |
| TOML | tables/arrays/scalars mapped into the common value model |
| XML | structural representation preserving tags (`t`), attributes (`a`), and ordered children (`c`) |
| TOON | decoded JSON-compatible value |
| Markdown | title/frontmatter, ordered sections, typed blocks, link index |

## TOON compatibility

The current Rust integration uses `toon-format` 0.5.x for encoding and decoding. Its published documentation declares TOON specification **v3.0** compatibility, so toon-world currently makes that same compatibility claim and no newer one.

## Testing and local verification

CI is intentionally disabled during the initial milestone chain. Markdown tests cover preambles, missing/duplicate headings, frontmatter, block types, code with and without language, task lists, link titles, section/code helpers, extension/override routing, and invalid UTF-8, in addition to the inherited structured/TOON suite.

Checked-in realistic sample files cover every supported input format. `--stats` reports JSON only on stderr, leaving normal stdout byte-for-byte unchanged.

See [`docs/TESTING.md`](docs/TESTING.md) for the detailed matrix.

Run locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Smoke checks:

```bash
printf '%s' $'# Demo\n\n## Usage\n\n```bash\ncargo run\n```\n' \
  | cargo run --quiet -- --from markdown -q 'section("Usage") | code("bash") | .text' --to text
# cargo run

printf '%s' $'Intro.\n\n# Demo\n' \
  | cargo run --quiet -- --from markdown -q '.sections[0].blocks[0].text' --to text
# Intro.
```

## Project docs

- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestone direction
- [`docs/MILESTONES.md`](docs/MILESTONES.md): stacked PR contract/status
- [`docs/TESTING.md`](docs/TESTING.md): verification and Markdown coverage matrix
- [`docs/superpowers/specs/2026-09-16-toon-world-design.md`](docs/superpowers/specs/2026-09-16-toon-world-design.md): architecture/design

## Reference

- [TOON specification](https://github.com/toon-format/spec)
- [TOON reference implementation](https://github.com/toon-format/toon)
- [jaq](https://github.com/01mf02/jaq)
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark)
