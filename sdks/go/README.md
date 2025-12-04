# t2z-go

<p align="center">
  <strong>Go SDK for transparent → shielded Zcash transactions</strong>
</p>

<p align="center">
  <a href="https://t2z.d4mr.com"><img src="https://img.shields.io/badge/docs-t2z.d4mr.com-blue?style=flat-square" alt="Documentation"></a>
  <a href="https://github.com/zcash/zips/pull/1063"><img src="https://img.shields.io/badge/ZIP-374-purple?style=flat-square" alt="ZIP 374"></a>
</p>

---

Build Zcash transactions that send from **transparent inputs** to **shielded Orchard outputs** using the [PCZT](https://github.com/zcash/zips/pull/1063) format.

## Installation

Since the Go SDK requires the native `libt2z_uniffi` library, you need to build from source:

```bash
# Clone the repository
git clone https://github.com/d4mr/t2z
cd t2z/sdks/go

# Build the native library and generate Go bindings
./scripts/build.sh

# Run tests to verify
./scripts/test.sh
```

The build script will:
1. Build the Rust `libt2z_uniffi` library
2. Install `uniffi-bindgen-go` if needed
3. Generate Go bindings
4. Print instructions for setting library path

### Manual Setup

If you prefer manual setup:

```bash
cd t2z/crates
cargo build --release -p t2z-uniffi

# Set library path
# macOS
export DYLD_LIBRARY_PATH=$(pwd)/target/release:$DYLD_LIBRARY_PATH
# Linux
export LD_LIBRARY_PATH=$(pwd)/target/release:$LD_LIBRARY_PATH
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    
    t2z "github.com/d4mr/t2z/sdks/go/t2z_uniffi"
)

func main() {
    // 1. Create transparent input
    input := t2z.UniffiTransparentInput{
        Pubkey:       "03abc123...",  // 33-byte compressed pubkey (hex)
        PrevoutTxid:  "ce15f716...",  // 32-byte txid (little-endian hex)
        PrevoutIndex: 0,
        Value:        1_000_000,      // 0.01 ZEC in zatoshis
        ScriptPubkey: "76a914...88ac", // P2PKH script (hex)
        Sequence:     nil,             // Default: 0xffffffff
    }

    // 2. Create payment
    payment := t2z.UniffiPayment{
        Address: "u1recipient...", // Unified address with Orchard
        Amount:  900_000,          // 0.009 ZEC
        Memo:    nil,              // Optional memo (hex)
        Label:   nil,              // Optional label
    }

    request := t2z.UniffiTransactionRequest{
        Payments: []t2z.UniffiPayment{payment},
    }

    // 3. Propose transaction
    changeAddr := "u1change..."
    pczt, err := t2z.ProposeTransaction(
        []t2z.UniffiTransparentInput{input},
        request,
        &changeAddr,
        "testnet",
        3720100,
    )
    if err != nil {
        log.Fatal(err)
    }

    // 4. Sign transparent inputs (external signing)
    sighash, err := t2z.GetSighash(pczt, 0)
    if err != nil {
        log.Fatal(err)
    }
    
    // Sign the sighash with your key (ECDSA secp256k1)
    signature := sign(sighash, privateKey) // Your signing logic
    
    pczt, err = t2z.AppendSignature(pczt, 0, pubkeyHex, signature)
    if err != nil {
        log.Fatal(err)
    }

    // 5. Generate Orchard proofs (~10 seconds first time)
    pczt, err = t2z.ProveTransaction(pczt)
    if err != nil {
        log.Fatal(err)
    }

    // 6. Finalize and get raw transaction
    txHex, err := t2z.FinalizeAndExtractHex(pczt)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Transaction ready:", txHex)
}
```

## API Reference

### Transaction Construction

| Function | Description |
|----------|-------------|
| `ProposeTransaction` | Create a PCZT from inputs and payments |
| `VerifyBeforeSigning` | Verify PCZT matches original request |

### Signing

| Function | Description |
|----------|-------------|
| `GetSighash` | Get sighash for external signing |
| `AppendSignature` | Add a pre-computed signature |
| `SignTransparentInput` | Sign with in-memory private key |

### Proving & Finalization

| Function | Description |
|----------|-------------|
| `ProveTransaction` | Generate Orchard ZK proofs |
| `FinalizeAndExtract` | Extract final transaction bytes |
| `FinalizeAndExtractHex` | Extract as hex string |
| `CombinePczts` | Combine multiple PCZTs |

### Utilities

| Function | Description |
|----------|-------------|
| `PrebuildProvingKey` | Pre-build proving key at startup |
| `IsProvingKeyReady` | Check if proving key is cached |
| `Version` | Get library version |

### PCZT Object Methods

```go
// Create from bytes or hex
pczt, err := t2z.UniffiPcztFromBytes(bytes)
pczt, err := t2z.UniffiPcztFromHex(hexString)

// Serialize
bytes := pczt.ToBytes()
hexStr := pczt.ToHex()
```

## Types

### UniffiTransparentInput

```go
type UniffiTransparentInput struct {
    Pubkey       string   // 33-byte compressed pubkey (hex)
    PrevoutTxid  string   // 32-byte txid (little-endian hex)
    PrevoutIndex uint32   // Output index
    Value        uint64   // Value in zatoshis
    ScriptPubkey string   // P2PKH scriptPubkey (hex)
    Sequence     *uint32  // Optional sequence number
}
```

### UniffiPayment

```go
type UniffiPayment struct {
    Address string   // Unified or transparent address
    Amount  uint64   // Amount in zatoshis
    Memo    *string  // Optional memo (hex-encoded)
    Label   *string  // Optional label
}
```

### UniffiExpectedTxOut

```go
type UniffiExpectedTxOut struct {
    Address string  // Expected address
    Amount  uint64  // Expected amount
}
```

## Error Handling

All functions return Go-style errors:

```go
pczt, err := t2z.ProposeTransaction(inputs, request, changeAddr, network, expiry)
if err != nil {
    // Handle error
    log.Printf("Failed to propose: %v", err)
    return
}
```

## Performance Tips

### Pre-build Proving Key

The first proof generation takes ~10 seconds to build the proving key. Pre-build at startup:

```go
func init() {
    if !t2z.IsProvingKeyReady() {
        log.Println("Building Orchard proving key...")
        t2z.PrebuildProvingKey()
        log.Println("Proving key ready")
    }
}
```

## Related

- [t2z Documentation](https://t2z.d4mr.com)
- [ZIP 374: PCZT Specification](https://github.com/zcash/zips/pull/1063) (Draft)
- [ZIP 317: Fee Calculation](https://zips.z.cash/zip-0317)
- [ZIP 244: Signature Validation](https://zips.z.cash/zip-0244)

## License

MIT
