#!/bin/bash

# Build script for WASM target

echo "Building WASM target for debashc..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build the WASM target
echo "Compiling to WASM..."
wasm-pack build --target web --out-dir www/pkg

# Create www directory if it doesn't exist
mkdir -p www

echo "WASM build complete!"
echo "To run the demo:"
echo "1. cd www"
echo "2. python3 -m http.server 8000"
echo "3. Open http://localhost:8000 in your browser"
