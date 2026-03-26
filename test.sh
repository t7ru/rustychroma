#!/bin/bash
set -e

echo "Testing..."
cargo test

echo "Testing but parallel..."
cargo test --features parallel

echo "Running benchmarks..."
cargo bench

echo "Looks good! I think."
