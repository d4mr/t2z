/**
 * T2Z (Transparent to Zero-knowledge) class
 * 
 * This is the main entry point for sending from transparent to shielded addresses.
 * Provides a type-safe, ergonomic API for building, proving, signing,
 * and finalizing Zcash transactions.
 * 
 * Built on PCZT (Partially Constructed Zcash Transactions).
 */

import {
  type Network,
  type TransparentInput,
  type TransactionRequest,
  type T2ZBytes,
  ValidationError,
  ProposalError,
  ProvingError,
  SigningError,
  CombineError,
  FinalizationError,
  ParseError,
  transparentInputSchema,
  transactionRequestSchema,
  networkSchema,
} from './types';
import { getNativeModule } from './napi';
import { z } from 'zod';

/**
 * T2Z class for sending from transparent to shielded (zero-knowledge) addresses
 * 
 * @example
 * ```typescript
 * // Propose a transaction
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
 *     }],
 *     fee: 10000n
 *   },
 *   network: 'mainnet',
 *   expiryHeight: 2500000
 * });
 * 
 * // Prove the transaction (builds Halo 2 circuit on first call)
 * await tx.prove();
 * 
 * // Sign transparent inputs
 * await tx.signTransparentInput(0, 'private_key_hex');
 * 
 * // Finalize and extract transaction bytes
 * const txBytes = await tx.finalize();
 * 
 * // Broadcast txBytes to the network
 * ```
 */
export class T2Z {
  private constructor(private readonly bytes: T2ZBytes) {}

  /**
   * Proposes a new transaction from transparent inputs to transparent/shielded outputs
   * 
   * This implements the Creator, Constructor, and IO Finalizer roles per ZIP 374.
   * 
   * @param params - Transaction parameters
   * @param params.inputs - Transparent UTXOs to spend
   * @param params.request - Payment request (outputs and optional fee)
   * @param params.network - Zcash network (mainnet or testnet)
   * @param params.expiryHeight - Block height at which transaction expires
   * 
   * @throws {ValidationError} If input validation fails
   * @throws {ProposalError} If transaction proposal fails
   * 
   * @example
   * ```typescript
   * const tx = await T2Z.propose({
   *   inputs: [{
   *     pubkey: '02a1b2c3...',
   *     prevoutTxid: 'def456...',
   *     prevoutIndex: 0,
   *     value: 100000n,
   *     scriptPubkey: '76a914...'
   *   }],
   *   request: {
   *     payments: [{
   *       address: 'u1abc...',
   *       amount: 90000n,
   *       memo: '48656c6c6f' // "Hello" in hex
   *     }]
   *   },
   *   network: 'mainnet',
   *   expiryHeight: 2500000
   * });
   * ```
   */
  static async propose(params: {
    inputs: TransparentInput[];
    request: TransactionRequest;
    network: Network;
    expiryHeight: number;
  }): Promise<T2Z> {
    // Validate inputs
    try {
      z.array(transparentInputSchema).min(1).parse(params.inputs);
      transactionRequestSchema.parse(params.request);
      networkSchema.parse(params.network);
      z.number().int().positive().parse(params.expiryHeight);
    } catch (error) {
      if (error instanceof z.ZodError) {
        throw new ValidationError(
          `Input validation failed: ${error.issues.map(i => i.message).join(', ')}`,
          error.issues
        );
      }
      throw error;
    }

    const native = await getNativeModule();

    try {
      // Convert BigInts to numbers for NAPI (i64)
      const napiInputs = params.inputs.map(input => ({
        pubkey: input.pubkey,
        prevoutTxid: input.prevoutTxid,
        prevoutIndex: input.prevoutIndex,
        value: Number(input.value),
        scriptPubkey: input.scriptPubkey,
        sequence: input.sequence ?? null,
      }));

      const napiRequest = {
        payments: params.request.payments.map(payment => ({
          address: payment.address,
          value: Number(payment.amount),
          memo: payment.memo ?? null,
          label: payment.label ?? null,
        })),
        fee: params.request.fee ? Number(params.request.fee) : null,
      };

      const txBuffer = await native.napiProposeTransaction(
        napiInputs,
        napiRequest,
        params.network,
        params.expiryHeight
      );

      return new T2Z(new Uint8Array(txBuffer));
    } catch (error: any) {
      throw new ProposalError(`Failed to propose transaction: ${error.message}`);
    }
  }

  /**
   * Proves the transaction using Halo 2
   * 
   * This builds the Orchard circuit proving key on first call (~10 seconds),
   * then caches it for subsequent calls. No downloads required!
   * 
   * @throws {ProvingError} If proving fails
   * 
   * @example
   * ```typescript
   * await tx.prove(); // First call: ~10 seconds, subsequent: instant
   * ```
   */
  async prove(): Promise<void> {
    const native = await getNativeModule();

    try {
      const provedBuffer = await native.napiProveTransaction(Buffer.from(this.bytes));
      (this as any).bytes = new Uint8Array(provedBuffer);
    } catch (error: any) {
      throw new ProvingError(`Failed to prove transaction: ${error.message}`);
    }
  }

  /**
   * Signs a transparent input with the provided private key
   * 
   * Implements ZIP 244 signature hash computation for transparent inputs.
   * 
   * @param inputIndex - Index of the transparent input to sign
   * @param secretKeyHex - Private key as hex string (32 bytes)
   * 
   * @throws {ValidationError} If secret key is invalid
   * @throws {SigningError} If signing fails
   * 
   * @example
   * ```typescript
   * await tx.signTransparentInput(0, 'a1b2c3d4...');
   * ```
   */
  async signTransparentInput(inputIndex: number, secretKeyHex: string): Promise<void> {
    // Validate secret key
    try {
      z.string().regex(/^[0-9a-fA-F]{64}$/).parse(secretKeyHex);
      z.number().int().nonnegative().parse(inputIndex);
    } catch (error) {
      if (error instanceof z.ZodError) {
        throw new ValidationError(
          `Invalid parameters: ${error.issues.map(i => i.message).join(', ')}`,
          error.issues
        );
      }
      throw error;
    }

    const native = await getNativeModule();

    try {
      const signedBuffer = await native.napiSignTransparentInput(
        Buffer.from(this.bytes),
        inputIndex,
        secretKeyHex
      );
      (this as any).bytes = new Uint8Array(signedBuffer);
    } catch (error: any) {
      throw new SigningError(`Failed to sign input ${inputIndex}: ${error.message}`);
    }
  }

  /**
   * Combines multiple T2Z transactions into one
   * 
   * Useful for multi-party transaction construction where different parties
   * contribute inputs, outputs, or signatures.
   * 
   * @param txs - Array of T2Z transactions to combine
   * @returns New T2Z with combined data
   * 
   * @throws {ValidationError} If no transactions provided
   * @throws {CombineError} If combination fails
   * 
   * @example
   * ```typescript
   * const tx1 = await T2Z.propose({ ... });
   * const tx2 = await T2Z.propose({ ... });
   * const combined = await T2Z.combine([tx1, tx2]);
   * ```
   */
  static async combine(txs: T2Z[]): Promise<T2Z> {
    if (txs.length === 0) {
      throw new ValidationError('At least one transaction is required for combination');
    }

    if (txs.length === 1) {
      return new T2Z(txs[0]!.bytes);
    }

    const native = await getNativeModule();

    try {
      const buffers = txs.map(tx => Buffer.from(tx.bytes));
      const combinedBuffer = await native.napiCombine(buffers);
      return new T2Z(new Uint8Array(combinedBuffer));
    } catch (error: any) {
      throw new CombineError(`Failed to combine transactions: ${error.message}`);
    }
  }

  /**
   * Finalizes the transaction and extracts the raw transaction bytes
   * 
   * This implements the Spend Finalizer and Transaction Extractor roles.
   * The returned bytes are ready to be broadcast to the Zcash network.
   * 
   * @returns Transaction bytes ready for broadcast
   * 
   * @throws {FinalizationError} If finalization fails
   * 
   * @example
   * ```typescript
   * const txBytes = await tx.finalize();
   * // Broadcast txBytes using your preferred method
   * await broadcastTransaction(txBytes);
   * ```
   */
  async finalize(): Promise<Uint8Array> {
    const native = await getNativeModule();

    try {
      const txBuffer = await native.napiFinalizeAndExtract(Buffer.from(this.bytes));
      return new Uint8Array(txBuffer);
    } catch (error: any) {
      throw new FinalizationError(`Failed to finalize transaction: ${error.message}`);
    }
  }

  /**
   * Parses a T2Z transaction from serialized bytes
   * 
   * @param bytes - Serialized transaction bytes
   * @returns Parsed T2Z instance
   * 
   * @throws {ParseError} If parsing fails
   * 
   * @example
   * ```typescript
   * const tx = await T2Z.parse(txBytes);
   * ```
   */
  static async parse(bytes: Uint8Array): Promise<T2Z> {
    const native = await getNativeModule();

    try {
      const parsedBuffer = await native.napiParsePczt(Buffer.from(bytes));
      return new T2Z(new Uint8Array(parsedBuffer));
    } catch (error: any) {
      throw new ParseError(`Failed to parse transaction: ${error.message}`);
    }
  }

  /**
   * Parses a T2Z transaction from a hex string
   * 
   * @param hex - Hex-encoded transaction string
   * @returns Parsed T2Z instance
   * 
   * @throws {ValidationError} If hex string is invalid
   * @throws {ParseError} If parsing fails
   * 
   * @example
   * ```typescript
   * const tx = await T2Z.fromHex('abcdef...');
   * ```
   */
  static async fromHex(hex: string): Promise<T2Z> {
    try {
      z.string().regex(/^[0-9a-fA-F]+$/).parse(hex);
    } catch (error) {
      if (error instanceof z.ZodError) {
        throw new ValidationError('Invalid hex string', error.issues);
      }
      throw error;
    }

    const bytes = Buffer.from(hex, 'hex');
    return T2Z.parse(new Uint8Array(bytes));
  }

  /**
   * Parses a T2Z transaction from a base64 string
   * 
   * @param base64 - Base64-encoded transaction string
   * @returns Parsed T2Z instance
   * 
   * @throws {ParseError} If parsing fails
   * 
   * @example
   * ```typescript
   * const tx = await T2Z.fromBase64('YWJjZGVm...');
   * ```
   */
  static async fromBase64(base64: string): Promise<T2Z> {
    const bytes = Buffer.from(base64, 'base64');
    return T2Z.parse(new Uint8Array(bytes));
  }

  /**
   * Serializes the transaction to bytes
   * 
   * @returns Serialized transaction bytes
   * 
   * @example
   * ```typescript
   * const bytes = tx.toBytes();
   * ```
   */
  toBytes(): Uint8Array {
    return new Uint8Array(this.bytes);
  }

  /**
   * Serializes the transaction to a hex string
   * 
   * @returns Hex-encoded transaction string
   * 
   * @example
   * ```typescript
   * const hex = tx.toHex();
   * ```
   */
  toHex(): string {
    return Buffer.from(this.bytes).toString('hex');
  }

  /**
   * Serializes the transaction to a base64 string
   * 
   * @returns Base64-encoded transaction string
   * 
   * @example
   * ```typescript
   * const base64 = tx.toBase64();
   * ```
   */
  toBase64(): string {
    return Buffer.from(this.bytes).toString('base64');
  }

  /**
   * Creates a clone of this T2Z transaction
   * 
   * @returns New T2Z instance with the same data
   */
  clone(): T2Z {
    return new T2Z(new Uint8Array(this.bytes));
  }
}

/**
 * @deprecated Use T2Z instead
 */
export const PCZT = T2Z;

