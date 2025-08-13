#!/bin/bash

# Build script for WASM target with incremental builds

cargo run --bin convert_examples

echo "Checking if WASM rebuild is needed..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Check if www/pkg directory exists and has the expected files
WASM_DIR="www/pkg"
WASM_FILES=("$WASM_DIR/debashl_bg.wasm" "$WASM_DIR/debashl.js" "$WASM_DIR/debashl.d.ts")

# Function to check if any source files are newer than WASM files
needs_rebuild() {
    if [ ! -d "$WASM_DIR" ]; then
        echo "WASM directory doesn't exist, rebuild needed"
        return 0
    fi
    
    # Check if all expected WASM files exist
    for file in "${WASM_FILES[@]}"; do
        if [ ! -f "$file" ]; then
            echo "Missing WASM file: $file, rebuild needed"
            return 0
        fi
    done
    
    # Find the newest WASM file timestamp
    NEWEST_WASM=$(find "$WASM_DIR" -name "*.wasm" -o -name "*.js" -o -name "*.d.ts" -printf '%T@\n' | sort -n | tail -1)
    
    # Find the newest source file timestamp
    NEWEST_SOURCE=$(find src/ -name "*.rs" -printf '%T@\n' | sort -n | tail -1)
    
    # Compare timestamps (WASM timestamp is in seconds, source timestamp is in seconds)
    if [ "$NEWEST_SOURCE" \> "$NEWEST_WASM" ]; then
        echo "Source files are newer than WASM files, rebuild needed"
        return 0
    fi
    
    echo "WASM files are up to date, no rebuild needed"
    return 1
}

# Check if rebuild is needed
if needs_rebuild; then
    echo "Building WASM target for debashc..."
    
    # Build the WASM target
    echo "Compiling to WASM..."
    wasm-pack build --target web --out-dir www/pkg
    
    # Create www directory if it doesn't exist
    mkdir -p www
    
    echo "WASM build complete!"
else
    echo "WASM is up to date, skipping build"
fi

echo "To run the demo:"
echo "1. cd www"
echo "2. python3 -m http.server 8000"
echo "3. Open http://localhost:8000 in your browser"
