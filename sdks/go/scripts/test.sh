#!/bin/bash
# Run tests for the t2z Go SDK

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CRATES_DIR="$ROOT_DIR/crates"
GO_SDK_DIR="$ROOT_DIR/sdks/go"

# Determine library path variable
if [[ "$OSTYPE" == "darwin"* ]]; then
    LIB_PATH_VAR="DYLD_LIBRARY_PATH"
elif [[ "$OSTYPE" == "linux"* ]]; then
    LIB_PATH_VAR="LD_LIBRARY_PATH"
else
    LIB_PATH_VAR="LD_LIBRARY_PATH"
fi

# Check if library exists
LIB_DIR="$CRATES_DIR/target/release"
if [[ ! -d "$LIB_DIR" ]]; then
    echo "Error: Native library not built. Run ./scripts/build.sh first."
    exit 1
fi

echo "==> Running t2z Go SDK tests"
echo ""

cd "$GO_SDK_DIR"

# Set environment variables for CGO
export CGO_LDFLAGS="-L$LIB_DIR"
export CGO_CFLAGS="-I$GO_SDK_DIR/t2z_uniffi"
export $LIB_PATH_VAR="$LIB_DIR:${!LIB_PATH_VAR}"

# Run tests
go test -v ./...

echo ""
echo "==> All tests passed!"

