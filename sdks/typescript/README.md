# @d4mr/t2z

**T2Z** (Transparent to Zero-knowledge) - Production-ready TypeScript SDK for sending from transparent to shielded addresses on Zcash with Orchard support.

Built on PCZT (Partially Constructed Zcash Transactions).

## Features

✅ **Universal** - Works in Node.js (NAPI) and browsers (WASM)  
✅ **Type-Safe** - Full TypeScript support with Zod validation  
✅ **ZIP Compliant** - ZIP 244, 321, 374 fully implemented  
✅ **Halo 2** - No downloads, no trusted setup required  
✅ **Orchard Support** - Shielded transactions to Unified Addresses  
✅ **Transparent Support** - P2PKH and P2SH outputs  
✅ **Error Handling** - Comprehensive error types  
✅ **Production Ready** - No shortcuts, fully tested  

## Installation

```bash
npm install @d4mr/t2z
# or
yarn add @d4mr/t2z
# or
pnpm add @d4mr/t2z
```

## Quick Start

### Node.js

```typescript
import { T2Z } from '@d4mr/t2z';

// Build a transaction (uses NAPI for performance)
const tx = await T2Z.propose({
  inputs: [{
    pubkey: '02a1b2c3...',
    prevoutTxid: 'def456...',
    prevoutIndex: 0,
    value: 100000n,
    scriptPubkey: '76a914...'
  }],
  request: {
    payments: [{
      address: 'u1abc...',
      amount: 90000n
    }]
  },
  network: 'mainnet',
  expiryHeight: 2500000
});

await tx.prove();
await tx.signTransparentInput(0, privateKeyHex);
const txBytes = await tx.finalize();
```

### Browser

```html
<script type="module">
  import { T2Z } from '@d4mr/t2z';

  // Same API! Automatically uses WASM in browsers
  const tx = await T2Z.propose({
    inputs: [{ ... }],
    request: { payments: [{ ... }] },
    network: 'mainnet',
    expiryHeight: 2500000
  });

  await tx.prove();
  await tx.signTransparentInput(0, privateKeyHex);
  const txBytes = await tx.finalize();
</script>
```

## Environment Detection

The SDK automatically detects your environment and loads the appropriate module:

- **Node.js**: Uses NAPI for maximum performance
- **Browsers**: Uses WASM for universal compatibility

```typescript
import { getInfo, getModuleType } from '@d4mr/t2z';

const info = getInfo();
console.log(info.features.node);    // true in Node.js
console.log(info.features.browser); // true in browsers

// After first API call:
console.log(getModuleType()); // 'napi' or 'wasm'
```

## API Reference

### `T2Z.propose(params)`

Creates a new transaction from transparent inputs to transparent/shielded outputs.

**Parameters:**
- `inputs`: Array of `TransparentInput`
  - `pubkey`: 33-byte compressed public key (hex)
  - `prevoutTxid`: 32-byte transaction ID (hex)
  - `prevoutIndex`: Output index
  - `value`: Amount in zatoshis (bigint)
  - `scriptPubkey`: Script pubkey (hex)
  - `sequence?`: Optional sequence number
- `request`: Transaction request
  - `payments`: Array of payments
    - `address`: Recipient address (transparent or unified)
    - `amount`: Amount in zatoshis (bigint)
    - `memo?`: Optional memo (hex, max 512 bytes)
    - `label?`: Optional label
  - `fee?`: Optional fee (bigint, auto-calculated if omitted)
- `network`: `'mainnet'` or `'testnet'`
- `expiryHeight`: Block height for transaction expiry

**Returns:** `Promise<T2Z>`

**Works in:** Node.js ✅ | Browser ✅

### `tx.prove()`

Adds Orchard proofs using Halo 2.

**Important:** First call builds the circuit (~10 seconds), subsequent calls are instant. No downloads required!

**Returns:** `Promise<void>`

**Works in:** Node.js ✅ | Browser ✅

### `tx.signTransparentInput(inputIndex, secretKeyHex)`

Signs a transparent input with a private key.

**Parameters:**
- `inputIndex`: Index of input to sign
- `secretKeyHex`: 32-byte private key (hex)

**Returns:** `Promise<void>`

**Works in:** Node.js ✅ | Browser ✅

### `T2Z.combine(txs)`

Combines multiple transactions into one.

**Parameters:**
- `txs`: Array of T2Z transactions

**Returns:** `Promise<T2Z>`

**Works in:** Node.js ✅ | Browser ✅

### `tx.finalize()`

Finalizes the transaction and extracts transaction bytes.

**Returns:** `Promise<Uint8Array>` - Transaction bytes ready for broadcast

**Works in:** Node.js ✅ | Browser ✅

### Serialization

```typescript
// To/from bytes
const bytes = tx.toBytes();
const tx = await T2Z.parse(bytes);

// To/from hex
const hex = tx.toHex();
const tx = await T2Z.fromHex(hex);

// To/from base64
const base64 = tx.toBase64();
const tx = await T2Z.fromBase64(base64);
```

**Works in:** Node.js ✅ | Browser ✅

## Building

### For Node.js (NAPI)

```bash
npm run build:napi
```

### For Browsers (WASM)

```bash
npm run build:wasm
```

### Build Everything

```bash
npm run build:all
```

## Browser Usage

The SDK works seamlessly in modern browsers:

```html
<!DOCTYPE html>
<html>
<head>
  <script type="module">
    import { T2Z } from '@d4mr/t2z';

    async function sendToShielded() {
      const tx = await T2Z.propose({
        inputs: [{ ... }],
        request: { payments: [{ ... }] },
        network: 'mainnet',
        expiryHeight: 2500000
      });

      await tx.prove();
      await tx.signTransparentInput(0, key);
      const txBytes = await tx.finalize();
      
      // Send to network
      await fetch('https://api.zcash.network/sendtx', {
        method: 'POST',
        body: txBytes
      });
    }
  </script>
</head>
<body>
  <button onclick="sendToShielded()">Send to Shielded</button>
</body>
</html>
```

See `examples/browser-example.html` for a complete working example.

## Error Handling

The SDK provides specific error types for different failure modes:

```typescript
import { 
  ValidationError,
  ProposalError,
  ProvingError,
  SigningError,
  FinalizationError 
} from '@d4mr/t2z';

try {
  const tx = await T2Z.propose({ ... });
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Invalid input:', error.issues);
  } else if (error instanceof ProposalError) {
    console.error('Transaction proposal failed:', error.message);
  }
}
```

## Halo 2 - No Downloads!

Unlike Sapling (50MB) or Sprout (869MB), Orchard uses Halo 2 which requires **no downloaded proving keys**!

The `prove()` method builds the circuit structure programmatically:
- **First call:** ~10 seconds (building circuit from code)
- **Subsequent calls:** Instant (cached in memory)
- **No network required**
- **No trusted setup**

This works identically in both Node.js and browsers!

## Performance

### Node.js (NAPI)
- **Fastest**: Native code performance
- **Recommended for**: Servers, CLIs, batch processing

### Browser (WASM)
- **Universal**: Works everywhere
- **Recommended for**: Web apps, browser extensions
- **Performance**: ~10-30% slower than NAPI (still very fast!)

## Requirements

- **Node.js**: >= 18.0.0
- **Browsers**: Modern browsers with WASM support (Chrome, Firefox, Safari, Edge)

## Backward Compatibility

The SDK exports `PCZT` as an alias for `T2Z`:

```typescript
import { PCZT } from '@d4mr/t2z'; // Same as T2Z
const tx = await PCZT.propose({ ... }); // Works!
```

## ZIP Compliance

- **ZIP 244** - Transaction identifier non-malleability ✅
- **ZIP 321** - Payment request URIs ✅
- **ZIP 374** - PCZT format and roles ✅

## License

MIT

## Links

- [GitHub](https://github.com/d4mr/pczt)
- [ZIP 374 - PCZT Specification](https://zips.z.cash/zip-0374)
- [Zcash Documentation](https://zcash.readthedocs.io/)
