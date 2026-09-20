#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo '== Active users from JSON =='
cargo run --quiet -- tests/fixtures/users.json -q '.users[] | select(.active) | .name' --to text

echo '== Warning event from JSONL =='
cargo run --quiet -- tests/fixtures/events.jsonl -q '.[] | select(.level == "warn") | .message' --to text

echo '== Semantic HTML title and byte statistics =='
cargo run --quiet -- tests/fixtures/page.html --semantic -q '.title' --to text --stats

echo '== Sparse-TOON round trip =='
compact_file=$(mktemp)
trap 'rm -f "$compact_file"' EXIT
cargo run --quiet -- tests/fixtures/bench_sparse.json --compact > "$compact_file"
sed -n '1,3p' "$compact_file"
cargo run --quiet -- "$compact_file" --from sparse-toon --to json
