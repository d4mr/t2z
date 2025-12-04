<div align="center">
  <img src="https://raw.githubusercontent.com/d4mr/t2z/master/demo/public/t2z_icon.png" alt="t2z" width="120" />
  
  <h1>t2z-wasm</h1>
  
  <p><strong>Transparent → Shielded Zcash Transactions in JavaScript/TypeScript</strong></p>
  
  <p>
    <a href="https://www.npmjs.com/package/@d4mr/t2z-wasm"><img src="https://img.shields.io/npm/v/@d4mr/t2z-wasm.svg?style=flat-square&color=f59e0b" alt="npm version" /></a>
    <a href="https://github.com/d4mr/t2z/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=flat-square" alt="License" /></a>
  </p>
  
  <p>
    <a href="https://t2z.d4mr.com">Documentation</a> •
    <a href="https://github.com/d4mr/t2z">GitHub</a> •
    <a href="https://github.com/zcash/zips/pull/1063">ZIP 374</a>
  </p>
</div>

---

Build Zcash transactions that send from **transparent inputs** to **shielded Orchard outputs** using the [PCZT](https://github.com/zcash/zips/pull/1063) (Partially Constructed Zcash Transaction) format.

## Features

- 🔐 **Transparent → Shielded**: Send from t-addresses to Orchard shielded addresses
- 📦 **PCZT Format**: Full ZIP 374 support for multi-party transaction construction
- 🌐 **Browser & Node.js**: Works in modern browsers and Node.js via WebAssembly
- 🔑 **External Signing**: Supports hardware wallets and HSMs via `get_sighash` + `append_signature`
- ⚡ **Halo 2 Proofs**: No trusted setup, no 50MB downloads - proving key built on demand
- 🦀 **Rust Core**: Battle-tested Zcash libraries under the hood

## Installation

```bash
npm install @d4mr/t2z-wasm
# or
pnpm add @d4mr/t2z-wasm
# or
yarn add @d4mr/t2z-wasm
```

## Quick Start

```typescript
import * as t2z from '@d4mr/t2z-wasm';

// 1. Create the transaction
const pczt = t2z.propose_transaction(
  [new t2z.WasmTransparentInput(
    pubkeyHex,      // 33-byte compressed pubkey
    prevoutTxid,    // Previous tx ID (little-endian hex)
    0,              // Output index
    1000000n,       // Value in zatoshis (0.01 ZEC)
    scriptPubkey,   // P2PKH script
    null            // Sequence (default: 0xffffffff)
  )],
  [new t2z.WasmPayment(
    'u1...',        // Unified address with Orchard receiver
    900000n,        // Amount in zatoshis
    null,           // Optional memo (hex)
    null            // Optional label
  )],
  'u1...',          // Change address (Orchard recommended)
  'testnet',        // 'mainnet' or 'testnet'
  3720100           // Expiry height (must be post-Nu5)
);

// 2. Sign transparent inputs
const sighash = t2z.get_sighash(pczt, 0);
// Sign externally (e.g., hardware wallet)
const signature = await yourSigner.sign(sighash);
pczt = t2z.append_signature(pczt, 0, pubkeyHex, signatureHex);

// 3. Generate Orchard proofs (~10s first time, cached after)
pczt = t2z.prove_transaction(pczt);

// 4. Finalize and broadcast
const txHex = t2z.finalize_and_extract_hex(pczt);
// Submit txHex to the Zcash network
```

## API Reference

### Transaction Construction

| Function | Description |
|----------|-------------|
| `propose_transaction(inputs, payments, change_address, network, expiry_height)` | Create a PCZT from transparent inputs and payment outputs |
| `inspect_pczt(pczt_hex)` | Get detailed info about a PCZT (inputs, outputs, fee, signing status) |

### Signing (ZIP 244)

| Function | Description |
|----------|-------------|
| `get_sighash(pczt, input_index)` | Get the 32-byte sighash for external signing |
| `append_signature(pczt, input_index, pubkey, signature)` | Add a DER signature to the PCZT |
| `sign_transparent_input(pczt, input_index, private_key)` | Convenience: sign internally with a private key |

### Proving (Halo 2)

| Function | Description |
|----------|-------------|
| `prove_transaction(pczt)` | Generate Orchard zero-knowledge proofs |
| `prebuild_proving_key()` | Pre-build the proving key (~10s, cached globally) |
| `is_proving_key_ready()` | Check if proving key is cached |

### Finalization

| Function | Description |
|----------|-------------|
| `verify_before_signing(pczt, payments, expected_change)` | Verify PCZT matches original request |
| `finalize_and_extract(pczt)` | Extract raw transaction bytes |
| `finalize_and_extract_hex(pczt)` | Extract transaction as hex string |

### Utilities

| Function | Description |
|----------|-------------|
| `parse_pczt(bytes)` | Parse PCZT from bytes |
| `serialize_pczt(pczt)` | Serialize PCZT to bytes |
| `generate_test_address(network)` | Generate a random Orchard test address |
| `generate_test_keypair(network)` | Generate address + spending key + viewing key |
| `version()` | Get library version |

## Browser Setup

For multithreaded WASM (faster proving), set these headers:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

### Vite Configuration

```typescript
// vite.config.ts
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

## Important Notes

### Expiry Height

The `expiry_height` must be:
1. **After Nu5 activation** (mainnet: 1,687,104 / testnet: 1,842,420)
2. **At least current_height + 40** to avoid "tx-expiring-soon" errors

```typescript
// In production, fetch current height from lightwalletd
const currentHeight = await fetchCurrentBlockHeight();
const expiryHeight = currentHeight + 100; // ~2.5 hour buffer
```

### Transaction IDs

Block explorers show txids in **big-endian** (display order), but Zcash internally uses **little-endian**. When using a txid from an explorer:

```typescript
// Reverse bytes for internal use
const internalTxid = explorerTxid.match(/../g).reverse().join('');
```

## Demo

Try the interactive demo at [t2z.d4mr.com/demo](https://t2z.d4mr.com/demo) to see the full transaction flow.

## Related

- [ZIP 374](https://github.com/zcash/zips/pull/1063) - PCZT specification
- [ZIP 244](https://zips.z.cash/zip-0244) - Transaction signature validation
- [ZIP 317](https://zips.z.cash/zip-0317) - Fee calculation
- [ZIP 321](https://zips.z.cash/zip-0321) - Payment request format

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
