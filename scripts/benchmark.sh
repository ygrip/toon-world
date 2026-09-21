#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

case "${1:-}" in
  ""|--check|--update) ;;
  *)
    echo "usage: $0 [--check|--update]" >&2
    exit 2
    ;;
esac

cargo test --test compact_benchmarks --test compact_matrix --test sparse_properties
generated=$(mktemp)
trap 'rm -f "$generated"' EXIT
cargo run --quiet --example compact_benchmark > "$generated"

case "${1:-}" in
  --update)
    updated=$(mktemp)
    trap 'rm -f "$generated" "$updated"' EXIT
    awk -v generated="$generated" '
      /<!-- BEGIN GENERATED COMPACT BENCHMARKS -->/ {
        while ((getline line < generated) > 0) print line
        close(generated)
        skipping = 1
        next
      }
      /<!-- END GENERATED COMPACT BENCHMARKS -->/ && skipping {
        skipping = 0
        next
      }
      !skipping { print }
    ' docs/BENCHMARKS.md > "$updated"
    mv "$updated" docs/BENCHMARKS.md
    ;;
  --check)
    documented=$(mktemp)
    trap 'rm -f "$generated" "$documented"' EXIT
    awk '
      /<!-- BEGIN GENERATED COMPACT BENCHMARKS -->/,/<!-- END GENERATED COMPACT BENCHMARKS -->/ { print }
    ' docs/BENCHMARKS.md > "$documented"
    diff -u "$documented" "$generated"
    ;;
  "")
    cat "$generated"
    ;;
esac
