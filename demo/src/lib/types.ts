/**
 * T2Z Demo Types
 * These mirror the WASM API for TypeScript type safety
 */

export type Network = 'mainnet' | 'testnet';

export interface TransparentInput {
  pubkey: string;           // 33-byte compressed pubkey (hex)
  prevoutTxid: string;      // 32-byte txid (hex, little-endian)
  prevoutIndex: number;     // Output index
  value: bigint;            // Value in zatoshis
  scriptPubkey: string;     // P2PKH scriptPubkey (hex)
  sequence?: number;        // Sequence number (default: 0xffffffff)
}

export interface Payment {
  address: string;          // Unified address with Orchard, or transparent
  amount: bigint;           // Amount in zatoshis
  memo?: string;            // Memo for shielded outputs (512 bytes max)
  label?: string;           // Display label
}

export interface KeyPair {
  privateKey: string;       // 32-byte private key (hex)
  publicKey: string;        // 33-byte compressed public key (hex)
  address: string;          // P2PKH transparent address
  scriptPubkey: string;     // P2PKH scriptPubkey (hex)
}

export interface OrchardKeypair {
  address: string;          // Unified address with Orchard receiver
  spendingKey: string;      // Hex-encoded spending key
}

export type StepId = 
  | 'setup'
  | 'inputs'
  | 'payments'
  | 'propose'
  | 'verify'
  | 'sign'
  | 'prove'
  | 'finalize';

export interface Step {
  id: StepId;
  title: string;
  description: string;
  completed: boolean;
}

export interface LogEntry {
  id: string;
  timestamp: Date;
  type: 'info' | 'success' | 'error' | 'code';
  step: StepId;
  message: string;
  data?: string;
}
