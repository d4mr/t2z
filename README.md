<p align="center">
  <img src="https://github.com/user-attachments/assets/8182ffdc-5ff6-4157-8c1b-4e112f13e243" alt="t2z - portable libraries for transparent → shielded transactions" />
</p>

<p align="center">
  <strong>Portable libraries for transparent → shielded Zcash transactions</strong>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@d4mr/t2z-wasm"><img src="https://img.shields.io/npm/v/@d4mr/t2z-wasm?style=flat-square&color=f59e0b" alt="npm version"></a>
  <a href="https://t2z.d4mr.com"><img src="https://img.shields.io/badge/docs-t2z.d4mr.com-blue?style=flat-square" alt="Documentation"></a>
  <a href="https://t2z-wasm-demo.d4mr.com"><img src="https://img.shields.io/badge/demo-live-green?style=flat-square" alt="Live Demo"></a>
  <a href="https://github.com/zcash/zips/pull/1063"><img src="https://img.shields.io/badge/ZIP-374-purple?style=flat-square" alt="ZIP 374"></a>
</p>

---

**t2z** enables existing transparent-only Zcash infrastructure to send to shielded Orchard addresses without requiring a full wallet implementation. Built on the [PCZT (Partially Constructed Zcash Transaction)](https://github.com/zcash/zips/pull/1063) format.

## ✨ Features

- **🔐 Privacy upgrade path** — Send from transparent inputs to shielded Orchard outputs
- **📦 PCZT format** — Multi-party transaction construction, hardware wallet support
- **⚡ No downloads** — Orchard uses Halo 2, proving key built on demand (~10s, cached)
- **🦀 Battle-tested** — Built on official Zcash Rust libraries

## 📦 Available SDKs

| Platform | Package | Status |
|----------|---------|--------|
| **TypeScript** | [`@d4mr/t2z-wasm`](https://www.npmjs.com/package/@d4mr/t2z-wasm) | ✅ Available |
| **Go** | `github.com/d4mr/t2z/sdks/go` | ✅ Available (build from source) |
| **Kotlin** | `com.d4mr:t2z` | 🚧 Coming Soon |

## 🚀 Quick Start

```bash
npm install @d4mr/t2z-wasm
```

```typescript
import * as t2z from '@d4mr/t2z-wasm';

// 1. Create the transaction
let pczt = t2z.propose_transaction(
  [new t2z.WasmTransparentInput(
    pubkeyHex,           // 33-byte compressed pubkey
    prevoutTxidLE,       // Previous tx ID (little-endian)
    0,                   // Output index
    1_000_000n,          // 0.01 ZEC in zatoshis
    scriptPubkeyHex,     // P2PKH script
    null                 // Sequence (default)
  )],
  [new t2z.WasmPayment(
    'u1recipient...',    // Unified address with Orchard
    900_000n,            // Amount in zatoshis
    null,                // Memo (hex)
    null                 // Label
  )],
  'u1change...',         // Change address
  'testnet',             // Network
  3720100                // Expiry height
);

// 2. Sign transparent inputs (external signing supported)
const sighash = t2z.get_sighash(pczt, 0);
const signature = await yourSigner.sign(sighash);  // Hardware wallet, HSM, etc.
pczt = t2z.append_signature(pczt, 0, pubkeyHex, signature);

// 3. Generate Orchard proofs
pczt = t2z.prove_transaction(pczt);

// 4. Finalize and broadcast
const txHex = t2z.finalize_and_extract_hex(pczt);
await broadcast(txHex);
```

## 📖 Documentation

- **[Full Documentation](https://t2z.d4mr.com)** — Guides, API reference, examples
- **[Live Demo](https://t2z-wasm-demo.d4mr.com)** — Interactive transaction builder
- **[API Reference](https://t2z.d4mr.com/api-reference)** — Complete function docs

## 🔧 Transaction Flow

```
┌─────────┐    ┌────────┐    ┌──────┐    ┌───────┐    ┌──────────┐
│ Propose │───▶│ Verify │───▶│ Sign │───▶│ Prove │───▶│ Finalize │
└─────────┘    └────────┘    └──────┘    └───────┘    └──────────┘
                (optional)
```

1. **Propose** — Create PCZT from inputs and payments
2. **Verify** — Validate PCZT matches request (if from third party)
3. **Sign** — Add signatures for transparent inputs
4. **Prove** — Generate Orchard ZK proofs
5. **Finalize** — Extract raw transaction for broadcast

## 🏗️ Project Structure

```
t2z/
├── crates/
│   ├── t2z-core/        # Core Rust library
│   ├── t2z-wasm/        # WebAssembly bindings
│   └── t2z-uniffi/      # Go/Kotlin bindings (UniFFI)
├── demo/                # Interactive demo (React + Vite)
└── docs/                # Documentation (Mintlify)
```

## 🛠️ Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (nightly toolchain)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- LLVM (for WASM compilation)

### macOS

```bash
# Install LLVM and wasi-libc
brew install llvm
brew install --HEAD aspect-build/aspect/wasi-libc

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Build the WASM package
cd crates/t2z-wasm
CC="$(brew --prefix llvm)/bin/clang" \
  AR="$(brew --prefix llvm)/bin/llvm-ar" \
  RUSTUP_TOOLCHAIN=nightly \
  wasm-pack build --scope d4mr
```

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install clang lld

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Build
cd crates/t2z-wasm
RUSTUP_TOOLCHAIN=nightly wasm-pack build --scope d4mr
```

The built package will be in `crates/t2z-wasm/pkg/`.

## 📚 Related

- [ZIP 374: PCZT Specification](https://github.com/zcash/zips/pull/1063) (Draft)
- [ZIP 317: Proportional Fee Mechanism](https://zips.z.cash/zip-0317)
- [ZIP 244: Transaction Signature Validation](https://zips.z.cash/zip-0244)
- [ZIP 321: Payment Request URIs](https://zips.z.cash/zip-0321)

## 📄 License

MIT © [d4mr](https://github.com/d4mr)
