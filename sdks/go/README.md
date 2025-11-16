# PCZT Go SDK

Go bindings for working with Partially Constructed Zcash Transactions (PCZT).

## Installation

```bash
go get github.com/d4mr/pczt
```

## Building

1. First, build the Rust wrapper with UniFFI bindings:

```bash
cd ../../wrapper
make build-uniffi
```

2. The Go bindings will be generated in `sdks/go/generated/`

3. Build the Go module:

```bash
go build
```

## Usage

```go
package main

import (
    "fmt"
    "github.com/d4mr/pczt"
)

func main() {
    // Example usage
    // TODO: Add example after bindings are generated
}
```

## Status

🚧 Under development - UniFFI bindings need to be generated.

