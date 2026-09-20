# Using toon-world

`toon-world` reads one structured or document input, runs a jq-compatible query, and writes the selected result. TOON is the default output because it is compact; use JSON when another program will consume the result and text for scalar shell output.

## Start here

```bash
# Show available flags and formats.
toon-world --help

# Convert a JSON file to TOON.
toon-world users.json

# Select only the data an agent or script needs.
toon-world users.json -q '.users[] | select(.active) | {id,name}'

# Emit scalars one per line for shell use.
toon-world users.json -q '.users[].name' --to text
```

When no file is supplied, input comes from stdin and defaults to JSON:

```bash
printf '%s' '{"name":"Ada"}' | toon-world -q '.name' --to text
```

## Choose an input format

Known file extensions select the parser automatically. Use `--from` when reading stdin, raw data, or a misleading filename.

```bash
# Filename inference.
toon-world events.jsonl -q '.[].event'
toon-world config.yaml -q '.service.replicas' --to text
toon-world page.html --semantic -q '.title' --to text

# Stdin and raw data need an explicit non-JSON format.
cat users.csv | toon-world --from csv -q '.[].name' --to text
toon-world --from yaml --data $'service:\n  replicas: 3' -q '.service.replicas' --to text
```

Supported extensions are JSON, `.ndjson`/`.jsonl`, CSV, YAML, TOML, XML, TOON, Markdown, and HTML. An unknown extension falls back to JSON with a warning; pass `--quiet` to suppress it or `--warnings-as-errors` to fail instead.

## Pick an output

| Need | Command | Result |
| --- | --- | --- |
| Compact context | `toon-world users.json` | Standard TOON (default) |
| Program input | `toon-world users.json --to json` | One compact JSON value |
| Shell values | `toon-world users.json -q '.users[].name' --to text` | One scalar per line |
| Size measurement | `toon-world users.json --stats` | Output on stdout; JSON byte statistics on stderr |

`--to text` accepts only scalar query results. Use `--to json` or TOON for objects and arrays.

## Query documents

Markdown and semantic HTML expose sections, code blocks, and links as normal queryable data.

```bash
# Markdown: find a named section and its Bash snippets.
toon-world README.md -q 'section("Installation") | code("bash") | .text' --to text

# HTML: ignore page chrome/scripts and retrieve visible title content.
toon-world page.html --semantic -q '.title' --to text

# Both document formats: inspect the normalized sections directly.
toon-world README.md -q '.sections[] | {heading,level}' --to json
```

Use structural HTML (the default) when tags, attributes, comments, or exact child order matter. Use `--semantic` when you want content-oriented sections, lists, tables, links, images, and forms.

## Experimental compact sparse-TOON

Use `--compact` for nested or heterogeneous structured results. It writes experimental sparse-TOON v1 only when it beats ordinary TOON under `cl100k_base`; otherwise it emits ordinary TOON:

```bash
toon-world records.json --compact > records.stoon
head -3 records.stoon
# @toon-world/sparse-v1
# root=^0
# ^0=[2]{"/id","/profile/name","/profile/team"}:
# 1,"Ada","platform"

# Restore the exact JSON-compatible value.
toon-world records.stoon --from sparse-toon --to json
```

Nested objects become JSON-Pointer columns. Nested arrays become referenced sparse tables or lists, so hooks, steps, tags, rows, arguments, and embeddings do not expand into a JSON cell. `~` means a property was absent; `null`, `""`, and `"~"` retain their normal JSON meanings. Existing root-table v1 files remain valid. `--compact` cannot be combined with `--to json` or `--to text`.

For a Cucumber report, compact mode recursively normalizes repeated child collections:

```bash
toon-world cucumber.json --compact > cucumber.stoon
toon-world cucumber.stoon --from sparse-toon --to json > cucumber.roundtrip.json
```

Sparse-TOON is a toon-world experimental codec. Standard `--to toon` remains the interoperable TOON path.

## Run the examples

```bash
# Exercises JSON, JSONL, semantic HTML/statistics, and sparse round-trip.
./examples/usage.sh

# Runs token/byte regression checks and prints the benchmark table.
./scripts/benchmark.sh
```

Both scripts run against checked-in fixtures, so they are safe to use as a quick post-install check.
