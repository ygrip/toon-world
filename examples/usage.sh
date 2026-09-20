#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Query a checked-in JSON sample.
cargo run --quiet -- tests/fixtures/users.json -q '.users[] | select(.active) | .name' --to text

# Infer JSON Lines from its filename.
cargo run --quiet -- tests/fixtures/events.jsonl -q '.[] | select(.level == "warn") | .message' --to text

# Extract content from semantic HTML and report byte statistics on stderr.
cargo run --quiet -- tests/fixtures/page.html --semantic -q '.title' --to text --stats

# Emit experimental sparse-TOON, then decode it back to JSON.
compact_file=$(mktemp)
trap 'rm -f "$compact_file"' EXIT
cargo run --quiet -- tests/fixtures/bench_sparse.json --compact > "$compact_file"
cargo run --quiet -- "$compact_file" --from sparse-toon --to json
