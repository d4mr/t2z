/**
 * Cryptographic utilities using @noble and @scure libraries
 * 
 * These are well-audited, widely-used cryptographic libraries.
 */
import { secp256k1 } from '@noble/curves/secp256k1';
import { sha256 } from '@noble/hashes/sha256';
import { ripemd160 } from '@noble/hashes/ripemd160';
import { base58check } from '@scure/base';
import { bytesToHex as nobleToHex, hexToBytes as nobleFromHex } from '@noble/hashes/utils';

/**
 * Generate a random 32-byte private key
 */
export function generatePrivateKey(): Uint8Array {
  return secp256k1.utils.randomPrivateKey();
}

/**
 * Get the compressed public key (33 bytes) from a private key
 */
export function getPublicKey(privateKey: Uint8Array): Uint8Array {
  return secp256k1.getPublicKey(privateKey, true); // compressed
}

/**
 * Sign a 32-byte message hash and return DER-encoded signature
 */
export async function signHash(
  messageHash: Uint8Array,
  privateKey: Uint8Array
): Promise<Uint8Array> {
  const signature = secp256k1.sign(messageHash, privateKey);
  return signature.toDERRawBytes();
}

/**
 * Convert bytes to hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
  return nobleToHex(bytes);
}

/**
 * Convert hex string to bytes
 */
export function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  return nobleFromHex(cleanHex);
}

/**
 * Hash160: SHA256 then RIPEMD160 (used for P2PKH addresses)
 */
export function hash160(data: Uint8Array): Uint8Array {
  return ripemd160(sha256(data));
}

/**
 * Generate P2PKH scriptPubkey from public key
 * OP_DUP OP_HASH160 <20-byte-pubkey-hash> OP_EQUALVERIFY OP_CHECKSIG
 */
export function pubkeyToScriptPubkey(pubkey: Uint8Array): Uint8Array {
  const pubkeyHash = hash160(pubkey);
  const script = new Uint8Array(25);
  script[0] = 0x76;  // OP_DUP
  script[1] = 0xa9;  // OP_HASH160
  script[2] = 0x14;  // Push 20 bytes
  script.set(pubkeyHash, 3);
  script[23] = 0x88; // OP_EQUALVERIFY
  script[24] = 0xac; // OP_CHECKSIG
  return script;
}

/**
 * Create a base58check encoder/decoder with double SHA256 checksum
 * This is the standard used by Bitcoin and Zcash
 */
const base58checkSha256 = base58check(sha256);

/**
 * Convert public key to Zcash transparent P2PKH address
 * 
 * Zcash uses 2-byte version prefixes:
 * - Mainnet P2PKH: 0x1cb8 (addresses start with "t1")
 * - Testnet P2PKH: 0x1d25 (addresses start with "tm")
 */
export function pubkeyToAddress(
  pubkey: Uint8Array,
  network: 'mainnet' | 'testnet'
): string {
  const pubkeyHash = hash160(pubkey);
  
  // Zcash uses 2-byte version prefixes
  const version = network === 'mainnet' 
    ? new Uint8Array([0x1c, 0xb8]) // t1
    : new Uint8Array([0x1d, 0x25]); // tm
  
  // Combine version + payload
  const data = new Uint8Array(version.length + pubkeyHash.length);
  data.set(version);
  data.set(pubkeyHash, version.length);
  
  return base58checkSha256.encode(data);
}

/**
 * Generate a complete transparent keypair
 */
export function generateTransparentKeypair(
  network: 'mainnet' | 'testnet'
): {
  privateKey: string;
  publicKey: string;
  address: string;
  scriptPubkey: string;
} {
  const privateKeyBytes = generatePrivateKey();
  const publicKeyBytes = getPublicKey(privateKeyBytes);
  const scriptPubkeyBytes = pubkeyToScriptPubkey(publicKeyBytes);
  const address = pubkeyToAddress(publicKeyBytes, network);
  
  return {
    privateKey: bytesToHex(privateKeyBytes),
    publicKey: bytesToHex(publicKeyBytes),
    address,
    scriptPubkey: bytesToHex(scriptPubkeyBytes),
  };
}
