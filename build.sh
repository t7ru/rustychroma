#!/bin/bash
set -e

echo "Wiping dist..."
rm -rf dist
mkdir -p dist/web dist/native

echo "Building..."
if [[ "$BUILD_TARGET" == "wasm" ]]; then
	cargo build --target wasm32-unknown-unknown -r --features wasm
	wasm-bindgen --target web --out-dir dist/web --no-typescript target/wasm32-unknown-unknown/release/rustychroma.wasm

elif [[ "$BUILD_TARGET" == "native" ]]; then
	cargo build --release --features c-api,parallel

	[[ "$GEN_HEADER" == "true" ]] && cbindgen -o dist/native/rustychroma.h

	cp target/release/rustychroma.dll dist/native/ 2>/dev/null || true
	cp target/release/librustychroma.so dist/native/ 2>/dev/null || true
	cp target/release/librustychroma.dylib dist/native/ 2>/dev/null || true
fi

echo "Organizing..."
cp target/release/rustychroma.dll dist/native/ 2>/dev/null ||
cp target/release/librustychroma.so dist/native/ 2>/dev/null ||
cp target/release/librustychroma.dylib dist/native/ 2>/dev/null || true

if [ -z "$(ls -A dist/native | grep -E '\.(dll|so|dylib)$')" ]; then
    echo "Uh oh, no native library found in dist!"
    exit 1
fi

echo "Heeho! All done!"
