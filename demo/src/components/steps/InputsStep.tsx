import { useState } from 'react';
import { Button } from '../Button';
import { Input } from '../Input';
import { CodeBlock } from '../CodeBlock';
import type { TransparentInput, KeyPair } from '../../lib/types';
import { hexToBytes, bytesToHex } from '../../lib/crypto';

/**
 * Reverse the byte order of a hex string.
 * Bitcoin/Zcash txids are displayed in big-endian (display order) but stored 
 * in little-endian (internal order) in raw transactions.
 */
function reverseTxidBytes(txidHex: string): string {
  const bytes = hexToBytes(txidHex);
  bytes.reverse();
  return bytesToHex(bytes);
}

interface Props {
  signingKey: KeyPair;
  inputs: TransparentInput[];
  totalIn: bigint;
  onAddInput: (input: TransparentInput) => void;
  onRemoveInput: (index: number) => void;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error', step: 'inputs', message: string, data?: string) => void;
}

export function InputsStep({
  signingKey,
  inputs,
  totalIn,
  onAddInput,
  onRemoveInput,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [txid, setTxid] = useState('');
  const [vout, setVout] = useState('0');
  const [value, setValue] = useState('');

  const handleAddInput = () => {
    try {
      // Clean and validate txid
      let cleanTxid = txid.trim().replace(/^0x/, '').toLowerCase();
      if (cleanTxid.length !== 64) {
        throw new Error('Transaction ID must be 32 bytes (64 hex characters)');
      }
      if (!/^[0-9a-f]+$/.test(cleanTxid)) {
        throw new Error('Transaction ID must be valid hexadecimal');
      }
      
      const valueNum = BigInt(value);
      if (valueNum <= 0n) {
        throw new Error('Value must be positive');
      }

      // IMPORTANT: Reverse the txid from display order (big-endian) to internal order (little-endian)
      // Block explorers show txids in display order, but raw transactions use internal byte order
      const internalTxid = reverseTxidBytes(cleanTxid);
      
      addLog('info', 'inputs', `Converting txid from display order to internal order`);
      addLog('info', 'inputs', `Display (explorer): ${cleanTxid}`);
      addLog('info', 'inputs', `Internal (raw tx):  ${internalTxid}`);

      const input: TransparentInput = {
        pubkey: signingKey.publicKey,
        prevoutTxid: internalTxid,
        prevoutIndex: parseInt(vout, 10),
        value: valueNum,
        scriptPubkey: signingKey.scriptPubkey,
        sequence: 0xffffffff,
      };

      onAddInput(input);
      
      // Reset form
      setTxid('');
      setVout('0');
      setValue('');
    } catch (err) {
      addLog('error', 'inputs', `Invalid input: ${err}`);
    }
  };

  const handleAddTestInput = () => {
    // Generate a fake txid for testing
    const fakeTxid = Array.from({ length: 32 }, () => 
      Math.floor(Math.random() * 256).toString(16).padStart(2, '0')
    ).join('');
    
    const input: TransparentInput = {
      pubkey: signingKey.publicKey,
      prevoutTxid: fakeTxid,
      prevoutIndex: 0,
      value: 1_000_000n, // 0.01 ZEC - enough for payments + fees
      scriptPubkey: signingKey.scriptPubkey,
      sequence: 0xffffffff,
    };

    onAddInput(input);
    addLog('info', 'inputs', 'Added test input (fake txid - for demo only)');
  };

  const formatZec = (zatoshis: bigint) => {
    const zec = Number(zatoshis) / 100_000_000;
    return `${zec.toFixed(8)} ZEC`;
  };

  const canProceed = inputs.length > 0;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Transparent Inputs</h2>
        <p className="text-gray-400">
          Add the transparent UTXOs you want to spend. These must be controlled by your 
          signing key ({signingKey.address.slice(0, 12)}...).
        </p>
      </div>

      {/* Current Inputs */}
      {inputs.length > 0 && (
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-300">
            Added Inputs ({inputs.length})
          </label>
          <div className="space-y-2">
            {inputs.map((input, index) => (
              <div 
                key={index}
                className="flex items-center justify-between p-3 bg-white/5 border border-white/10 rounded-lg"
              >
                <div className="font-mono text-sm">
                  <div className="text-white">{formatZec(input.value)}</div>
                  <div className="text-gray-500 text-xs">
                    {input.prevoutTxid.slice(0, 16)}...:{input.prevoutIndex}
                  </div>
                </div>
                <Button variant="danger" size="sm" onClick={() => onRemoveInput(index)}>
                  Remove
                </Button>
              </div>
            ))}
          </div>
          <div className="text-right text-sm">
            <span className="text-gray-400">Total: </span>
            <span className="text-amber-400 font-mono">{formatZec(totalIn)}</span>
          </div>
        </div>
      )}

      {/* Add Input Form */}
      <div className="space-y-4 p-4 bg-white/5 border border-white/10 rounded-lg">
        <h3 className="font-medium text-white">Add UTXO</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="md:col-span-2">
            <Input
              label="Transaction ID (hex)"
              placeholder="64 character hex string (from block explorer)"
              value={txid}
              onChange={(e) => setTxid(e.target.value)}
              hint="Enter the txid as shown on the block explorer (display order). It will be automatically converted to internal byte order."
            />
          </div>
          <Input
            label="Output Index (vout)"
            type="number"
            min="0"
            value={vout}
            onChange={(e) => setVout(e.target.value)}
          />
          <Input
            label="Value (zatoshis)"
            type="number"
            min="1"
            placeholder="e.g. 1000000 (0.01 ZEC)"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            hint="1 ZEC = 100,000,000 zatoshis. Minimum ~20,000 for fees."
          />
        </div>

        <div className="flex gap-2">
          <Button onClick={handleAddInput} disabled={!txid || !value}>
            Add Input
          </Button>
          <Button variant="secondary" onClick={handleAddTestInput}>
            Add Test Input (Demo)
          </Button>
        </div>
      </div>

      {/* Code Example */}
      <CodeBlock
        title="How inputs are constructed"
        code={`// Each input references a UTXO from a previous transaction
const input = new WasmTransparentInput(
  pubkey,        // Your compressed public key (33 bytes hex)
  prevoutTxid,   // Transaction ID in INTERNAL byte order (reversed from explorer)
  prevoutIndex,  // Index of the output in that transaction
  value,         // Amount in zatoshis (bigint)
  scriptPubkey   // P2PKH script: OP_DUP OP_HASH160 <pubkeyhash> OP_EQUALVERIFY OP_CHECKSIG
);

// IMPORTANT: TxID Byte Order
// Block explorers show txids in "display order" (big-endian)
// Raw transactions use "internal order" (little-endian, reversed bytes)
// Example:
//   Explorer shows: 3813f53766c6cc...
//   Internal order: ...cc66c66637f51338

// For real usage, get UTXO data from:
// - Your wallet's UTXO list
// - A block explorer API (e.g. zcashblockexplorer.com)
// - Your own Zcash node's RPC`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <Button onClick={onNext} disabled={!canProceed}>
          Continue to Payments →
        </Button>
      </div>
    </div>
  );
}

