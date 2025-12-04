#!/bin/bash
# Build the t2z Go SDK
# This script builds the native library and generates Go bindings

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CRATES_DIR="$ROOT_DIR/crates"
GO_SDK_DIR="$ROOT_DIR/sdks/go"

echo "==> Building t2z Go SDK"
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

# Check for uniffi-bindgen-go
if ! command -v uniffi-bindgen-go &> /dev/null; then
    echo "==> Installing uniffi-bindgen-go..."
    cargo install uniffi-bindgen-go \
        --git https://github.com/NordSecurity/uniffi-bindgen-go \
        --tag v0.4.0+v0.28.3
fi

# Generate Go bindings
echo "==> Generating Go bindings..."
rm -rf "$GO_SDK_DIR/t2z_uniffi"
uniffi-bindgen-go --library "$LIB_FILE" --out-dir "$GO_SDK_DIR"

# Add CGO flags to generated file
echo "==> Patching CGO flags..."
GO_FILE="$GO_SDK_DIR/t2z_uniffi/t2z_uniffi.go"
if [[ -f "$GO_FILE" ]]; then
    # Replace the simple #include with cgo directives
    sed -i.bak 's|// #include <t2z_uniffi.h>|/*\n#cgo LDFLAGS: -lt2z_uniffi\n#include <t2z_uniffi.h>\n*/|' "$GO_FILE"
    rm -f "$GO_FILE.bak"
fi

# Format Go code
echo "==> Formatting Go code..."
cd "$GO_SDK_DIR"
go fmt ./... 2>/dev/null || true

echo ""
echo "==> Build complete!"
echo ""
echo "To use the SDK, set your library path:"
echo ""
echo "    export $LIB_PATH_VAR=\"$CRATES_DIR/target/release:\$$LIB_PATH_VAR\""
echo ""
echo "Then in your Go code:"
echo ""
echo '    import t2z "github.com/d4mr/t2z/sdks/go/t2z_uniffi"'
echo ""

