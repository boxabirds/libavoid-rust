#!/bin/bash
# Build and copy WASM files for the web example

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building WASM package..."
cd "$PROJECT_ROOT"
wasm-pack build --target web --features wasm

echo "Copying pkg to examples/web..."
rm -rf "$SCRIPT_DIR/pkg"
cp -r "$PROJECT_ROOT/pkg" "$SCRIPT_DIR/pkg"

echo "Done! Run 'python3 -m http.server 8000' from examples/web/ to test."
