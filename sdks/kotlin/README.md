# t2z-kotlin

<p align="center">
  <strong>Kotlin SDK for transparent → shielded Zcash transactions</strong>
</p>

<p align="center">
  <a href="https://t2z.d4mr.com"><img src="https://img.shields.io/badge/docs-t2z.d4mr.com-blue?style=flat-square" alt="Documentation"></a>
  <a href="https://github.com/zcash/zips/pull/1063"><img src="https://img.shields.io/badge/ZIP-374-purple?style=flat-square" alt="ZIP 374"></a>
</p>

---

Build Zcash transactions that send from **transparent inputs** to **shielded Orchard outputs** using the [PCZT](https://github.com/zcash/zips/pull/1063) format.

## Prerequisites

- **Java 11+** (JDK)
- **Rust** (nightly toolchain)

## Installation

Since the Kotlin SDK requires the native `libt2z_uniffi` library, you need to build from source:

```bash
# Clone the repository
git clone https://github.com/d4mr/t2z
cd t2z/sdks/kotlin

# Build the native library and generate Kotlin bindings
./scripts/build.sh

# Run tests to verify
./scripts/test.sh
```

### Gradle Dependency

Add JNA dependency to your project:

```kotlin
dependencies {
    implementation("net.java.dev.jna:jna:5.14.0")
}
```

Copy the generated source files from `src/main/kotlin/uniffi/t2z_uniffi/` to your project.

## Quick Start

```kotlin
import uniffi.t2z_uniffi.*

fun main() {
    // 1. Create transparent input
    val input = UniffiTransparentInput(
        pubkey = "03abc123...",      // 33-byte compressed pubkey (hex)
        prevoutTxid = "ce15f716...", // 32-byte txid (little-endian hex)
        prevoutIndex = 0u,
        value = 1_000_000uL,         // 0.01 ZEC in zatoshis
        scriptPubkey = "76a914...88ac", // P2PKH script (hex)
        sequence = null
    )

    // 2. Create payment
    val payment = UniffiPayment(
        address = "u1recipient...", // Unified address with Orchard
        amount = 900_000uL,
        memo = null,
        label = null
    )

    val request = UniffiTransactionRequest(
        payments = listOf(payment)
    )

    // 3. Propose transaction
    var pczt = proposeTransaction(
        inputsToSpend = listOf(input),
        transactionRequest = request,
        changeAddress = "u1change...",
        network = "testnet",
        expiryHeight = 3720100u
    )

    // 4. Sign transparent inputs (external signing)
    val sighash = getSighash(pczt, 0u)
    
    // Sign the sighash with your key (ECDSA secp256k1)
    val signature = sign(sighash, privateKey) // Your signing logic
    
    pczt = appendSignature(pczt, 0u, pubkeyHex, signature)

    // 5. Generate Orchard proofs (~10 seconds first time)
    pczt = proveTransaction(pczt)

    // 6. Finalize and get raw transaction
    val txHex = finalizeAndExtractHex(pczt)

    println("Transaction ready: $txHex")
}
```

## API Reference

### Transaction Construction

| Function | Description |
|----------|-------------|
| `proposeTransaction` | Create a PCZT from inputs and payments |
| `verifyBeforeSigning` | Verify PCZT matches original request |

### Signing

| Function | Description |
|----------|-------------|
| `getSighash` | Get sighash for external signing |
| `appendSignature` | Add a pre-computed signature |
| `signTransparentInput` | Sign with in-memory private key |

### Proving & Finalization

| Function | Description |
|----------|-------------|
| `proveTransaction` | Generate Orchard ZK proofs |
| `finalizeAndExtract` | Extract final transaction bytes |
| `finalizeAndExtractHex` | Extract as hex string |
| `combinePczts` | Combine multiple PCZTs |

### Utilities

| Function | Description |
|----------|-------------|
| `prebuildProvingKey` | Pre-build proving key at startup |
| `isProvingKeyReady` | Check if proving key is cached |
| `version` | Get library version |

## Types

### UniffiTransparentInput

```kotlin
data class UniffiTransparentInput(
    val pubkey: String,       // 33-byte compressed pubkey (hex)
    val prevoutTxid: String,  // 32-byte txid (little-endian hex)
    val prevoutIndex: UInt,   // Output index
    val value: ULong,         // Value in zatoshis
    val scriptPubkey: String, // P2PKH scriptPubkey (hex)
    val sequence: UInt?       // Optional sequence number
)
```

### UniffiPayment

```kotlin
data class UniffiPayment(
    val address: String,  // Unified or transparent address
    val amount: ULong,    // Amount in zatoshis
    val memo: String?,    // Optional memo (hex-encoded)
    val label: String?    // Optional label
)
```

### UniffiExpectedTxOut

```kotlin
data class UniffiExpectedTxOut(
    val address: String, // Expected address
    val amount: ULong    // Expected amount (0 = wildcard)
)
```

## Error Handling

All functions throw `UniffiException` on error:

```kotlin
try {
    val pczt = proposeTransaction(inputs, request, changeAddr, network, expiry)
} catch (e: UniffiException) {
    println("Error: ${e.message}")
}
```

## Performance Tips

### Pre-build Proving Key

The first proof generation takes ~10 seconds. Pre-build at startup:

```kotlin
fun initT2z() {
    if (!isProvingKeyReady()) {
        println("Building Orchard proving key...")
        prebuildProvingKey()
        println("Proving key ready")
    }
}
```

## Android Integration

For Android, ensure the native library is placed in the correct `jniLibs` directory:

```
app/src/main/jniLibs/
├── arm64-v8a/
│   └── libt2z_uniffi.so
├── armeabi-v7a/
│   └── libt2z_uniffi.so
└── x86_64/
    └── libt2z_uniffi.so
```

Add to `build.gradle.kts`:

```kotlin
android {
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/d4mr/t2z
cd t2z/crates

# Build the UniFFI library
cargo build --release -p t2z-uniffi

# Generate Kotlin bindings
cargo run -p t2z-uniffi --bin uniffi-bindgen -- \
    generate --library target/release/libt2z_uniffi.dylib \
    --language kotlin \
    --out-dir ../sdks/kotlin/src/main/kotlin
```

## Related

- [t2z Documentation](https://t2z.d4mr.com)
- [ZIP 374: PCZT Specification](https://github.com/zcash/zips/pull/1063)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)

## License

MIT
