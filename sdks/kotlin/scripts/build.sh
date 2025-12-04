#!/bin/bash
# Build the t2z Kotlin SDK
# This script builds the native library and generates Kotlin bindings

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CRATES_DIR="$ROOT_DIR/crates"
KOTLIN_SDK_DIR="$ROOT_DIR/sdks/kotlin"

echo "==> Building t2z Kotlin SDK"
echo "    Root: $ROOT_DIR"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Build the native library
echo "==> Building native library (t2z-uniffi)..."
cd "$CRATES_DIR"
cargo build --release -p t2z-uniffi

# Determine library extension
if [[ "$OSTYPE" == "darwin"* ]]; then
    LIB_EXT="dylib"
    LIB_PATH_VAR="DYLD_LIBRARY_PATH"
elif [[ "$OSTYPE" == "linux"* ]]; then
    LIB_EXT="so"
    LIB_PATH_VAR="LD_LIBRARY_PATH"
else
    echo "Warning: Unknown OS type: $OSTYPE"
    LIB_EXT="so"
    LIB_PATH_VAR="LD_LIBRARY_PATH"
fi

LIB_FILE="$CRATES_DIR/target/release/libt2z_uniffi.$LIB_EXT"

if [[ ! -f "$LIB_FILE" ]]; then
    echo "Error: Library not found at $LIB_FILE"
    exit 1
fi

echo "    Library built: $LIB_FILE"
echo ""

# Generate Kotlin bindings
echo "==> Generating Kotlin bindings..."
cargo run -p t2z-uniffi --bin uniffi-bindgen -- \
    generate --library "$LIB_FILE" \
    --language kotlin \
    --out-dir "$KOTLIN_SDK_DIR/src/main/kotlin"

echo ""
echo "==> Build complete!"
echo ""
echo "To run tests:"
echo ""
echo "    cd $KOTLIN_SDK_DIR"
echo "    ./gradlew test"
echo ""
echo "The native library is at: $LIB_FILE"
echo "Set $LIB_PATH_VAR or java.library.path to include that directory."
echo ""

