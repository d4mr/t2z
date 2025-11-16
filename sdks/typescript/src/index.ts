/**
 * @d4mr/t2z - Production-ready TypeScript SDK for Zcash Transparent to Zero-knowledge
 * 
 * Send from transparent addresses to shielded (Orchard) addresses with ease.
 * Built on PCZT (Partially Constructed Zcash Transactions).
 * 
 * Features:
 * - ✅ Type-safe API with Zod validation
 * - ✅ Comprehensive error handling
 * - ✅ ZIP 244, 321, 374 compliant
 * - ✅ Halo 2 proving (no downloads!)
 * - ✅ Transparent to Orchard support
 * - ✅ Universal: Works in Node.js (NAPI) and browsers (WASM)
 * 
 * @example
 * ```typescript
 * import { T2Z } from '@d4mr/t2z';
 * 
 * // Build a transaction (works in Node and browsers!)
 * const tx = await T2Z.propose({
 *   inputs: [{
 *     pubkey: '02...',
 *     prevoutTxid: 'abc...',
 *     prevoutIndex: 0,
 *     value: 100000n,
 *     scriptPubkey: '76a914...'
 *   }],
 *   request: {
 *     payments: [{
 *       address: 'u1...',
 *       amount: 90000n
 *     }]
 *   },
 *   network: 'mainnet',
 *   expiryHeight: 2500000
 * });
 * 
 * // Prove with Halo 2 (no downloads!)
 * await tx.prove();
 * 
 * // Sign
 * await tx.signTransparentInput(0, privateKeyHex);
 * 
 * // Finalize
 * const txBytes = await tx.finalize();
 * ```
 * 
 * @packageDocumentation
 */

export { T2Z } from './t2z';

// Also export as PCZT for backward compatibility
export { T2Z as PCZT } from './t2z';

export type {
  Network,
  TransparentInput,
  Payment,
  TransactionRequest,
  T2ZBytes,
  PcztBytes, // Backward compatibility
} from './types';

export {
  T2ZError,
  ValidationError,
  ProposalError,
  ProvingError,
  SigningError,
  CombineError,
  FinalizationError,
  ParseError,
  // Backward compatibility
  T2ZError as PcztError,
} from './types';

// Export loader utilities
export { getModuleType, isModuleLoaded } from './napi';

/**
 * Library version
 */
export const VERSION = '0.1.0';

/**
 * Get library information
 */
export function getInfo() {
  return {
    version: VERSION,
    features: {
      halo2: true,
      orchard: true,
      transparent: true,
      sapling: false,
      node: typeof process !== 'undefined',
      browser: typeof window !== 'undefined',
    },
    zips: {
      zip244: true,
      zip321: true,
      zip374: true,
    },
  };
}
