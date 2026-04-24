#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM_OUT="$REPO_ROOT/web/public/wasm"

command -v wasm-pack >/dev/null || { echo "wasm-pack not found. Install: cargo install wasm-pack"; exit 1; }

echo "Building WASM from bible-web crate..."
wasm-pack build "$REPO_ROOT/bible-web" --target web --no-pack --out-dir "$WASM_OUT"
rm -f "$WASM_OUT/.gitignore"

echo "Done: $(du -sh "$WASM_OUT/bible_web_bg.wasm" | cut -f1) WASM binary"
