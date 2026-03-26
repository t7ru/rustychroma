#!/bin/bash
set -e

echo "Wiping dist..."
rm -rf dist
mkdir -p dist/web dist/native

echo "Building..."
cargo build --target wasm32-unknown-unknown -r --features wasm
wasm-bindgen --target web --out-dir dist/web --no-typescript target/wasm32-unknown-unknown/release/rustychroma.wasm
cargo build --release --features c-api,parallel
cbindgen -o dist/native/rustychroma.h

echo "Organizing..."
cp target/release/rustychroma.dll dist/native/ 2>/dev/null ||
cp target/release/librustychroma.so dist/native/ 2>/dev/null ||
cp target/release/librustychroma.dylib dist/native/ 2>/dev/null || true

echo "Heeho! All done!"
