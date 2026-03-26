#!/bin/bash
set -e

echo "Wiping dist..."
rm -rf dist
mkdir -p dist/web dist/native

echo "Building..."
wasm-pack build --target web --features wasm
cargo build --release --features c-api

echo "Organizing..."
cp target/release/rustychroma.dll dist/native/
[ -f "rustychroma.h" ] && mv rustychroma.h dist/native/

echo "Heeho! All done!"
