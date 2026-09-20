#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."
cargo test --test compact_benchmarks
cargo run --quiet --example compact_benchmark
