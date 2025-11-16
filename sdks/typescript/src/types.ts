/**
 * Type definitions for PCZT TypeScript SDK
 * 
 * This module provides type-safe interfaces for all PCZT operations.
 */

import { z } from 'zod';

// ============================================================================
// Core Types
// ============================================================================

/**
 * Network types for Zcash
 */
export type Network = 'mainnet' | 'testnet';

/**
 * Transparent input for PCZT construction
 * 
 * Includes all data required for ZIP 244 signature validation:
 * - pubkey: Required for signature verification
 * - prevout: Identifies the UTXO being spent
 * - value: Required for sighash (prevents fee attacks)
 * - scriptPubKey: Required for sighash
 */
export interface TransparentInput {
  /** Compressed public key (33 bytes, hex encoded) */
  pubkey: string;
  /** Previous transaction ID (32 bytes, hex encoded) */
  prevoutTxid: string;
  /** Previous output index */
  prevoutIndex: number;
  /** Value in zatoshis */
  value: bigint;
  /** Script pubkey (hex encoded) */
  scriptPubkey: string;
  /** Optional sequence number (defaults to 0xFFFFFFFF) */
  sequence?: number;
}

/**
 * Payment request following ZIP 321
 * 
 * Supports both transparent and shielded (Orchard) outputs:
 * - Transparent: P2PKH and P2SH addresses
 * - Orchard: Unified addresses with Orchard receivers
 */
export interface Payment {
  /** 
   * Recipient address
   * - Transparent: t1... (P2PKH) or t3... (P2SH)
   * - Unified: u1... (with Orchard receiver)
   */
  address: string;
  /** Amount in zatoshis */
  amount: bigint;
  /** Optional memo (max 512 bytes, hex encoded) */
  memo?: string;
  /** Optional label for the payment */
  label?: string;
}

/**
 * Transaction request following ZIP 321
 */
export interface TransactionRequest {
  /** List of payments (outputs) */
  payments: Payment[];
  /** Optional fee in zatoshis (auto-calculated if not provided) */
  fee?: bigint;
}

/**
 * T2Z transaction bytes (internally a PCZT - Partially Constructed Zcash Transaction)
 * 
 * This is an opaque type that wraps the serialized transaction bytes.
 * Use the T2Z class methods to work with it.
 */
export type T2ZBytes = Uint8Array;

/**
 * @deprecated Use T2ZBytes instead
 */
export type PcztBytes = T2ZBytes;

// ============================================================================
// Zod Schemas for Runtime Validation
// ============================================================================

const hexStringSchema = z.string().regex(/^[0-9a-fA-F]+$/, 'Must be a valid hex string');

const pubkeySchema = hexStringSchema.length(66, 'Public key must be 33 bytes (66 hex chars)');

const txidSchema = hexStringSchema.length(64, 'Transaction ID must be 32 bytes (64 hex chars)');

const bigintSchema = z.union([
  z.bigint(),
  z.number().transform(n => BigInt(n)),
  z.string().transform(s => BigInt(s))
]);

export const transparentInputSchema = z.object({
  pubkey: pubkeySchema,
  prevoutTxid: txidSchema,
  prevoutIndex: z.number().int().nonnegative(),
  value: bigintSchema,
  scriptPubkey: hexStringSchema,
  sequence: z.number().int().min(0).max(0xFFFFFFFF).optional()
});

export const paymentSchema = z.object({
  address: z.string().min(1),
  amount: bigintSchema,
  memo: hexStringSchema.optional(),
  label: z.string().optional()
}).refine(
  (data) => {
    if (data.memo) {
      const memoBytes = Buffer.from(data.memo, 'hex');
      return memoBytes.length <= 512;
    }
    return true;
  },
  { message: 'Memo must be at most 512 bytes' }
);

export const transactionRequestSchema = z.object({
  payments: z.array(paymentSchema).min(1, 'At least one payment is required'),
  fee: bigintSchema.optional()
});

export const networkSchema = z.enum(['mainnet', 'testnet']);

// ============================================================================
// Error Types
// ============================================================================

/**
 * Base error class for all T2Z errors
 */
export class T2ZError extends Error {
  constructor(message: string, public readonly code?: string) {
    super(message);
    this.name = 'T2ZError';
    Object.setPrototypeOf(this, T2ZError.prototype);
  }
}

/**
 * @deprecated Use T2ZError instead
 */
export class PcztError extends T2ZError {
  constructor(message: string, code?: string) {
    super(message, code);
    this.name = 'PcztError';
    Object.setPrototypeOf(this, PcztError.prototype);
  }
}

/**
 * Validation error for input data
 */
export class ValidationError extends T2ZError {
  constructor(message: string, public readonly issues?: z.ZodIssue[]) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'ValidationError';
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}

/**
 * Error during transaction proposal
 */
export class ProposalError extends T2ZError {
  constructor(message: string) {
    super(message, 'PROPOSAL_ERROR');
    this.name = 'ProposalError';
    Object.setPrototypeOf(this, ProposalError.prototype);
  }
}

/**
 * Error during proving
 */
export class ProvingError extends T2ZError {
  constructor(message: string) {
    super(message, 'PROVING_ERROR');
    this.name = 'ProvingError';
    Object.setPrototypeOf(this, ProvingError.prototype);
  }
}

/**
 * Error during signing
 */
export class SigningError extends T2ZError {
  constructor(message: string) {
    super(message, 'SIGNING_ERROR');
    this.name = 'SigningError';
    Object.setPrototypeOf(this, SigningError.prototype);
  }
}

/**
 * Error during combination
 */
export class CombineError extends T2ZError {
  constructor(message: string) {
    super(message, 'COMBINE_ERROR');
    this.name = 'CombineError';
    Object.setPrototypeOf(this, CombineError.prototype);
  }
}

/**
 * Error during finalization
 */
export class FinalizationError extends T2ZError {
  constructor(message: string) {
    super(message, 'FINALIZATION_ERROR');
    this.name = 'FinalizationError';
    Object.setPrototypeOf(this, FinalizationError.prototype);
  }
}

/**
 * Error during parsing
 */
export class ParseError extends T2ZError {
  constructor(message: string) {
    super(message, 'PARSE_ERROR');
    this.name = 'ParseError';
    Object.setPrototypeOf(this, ParseError.prototype);
  }
}

