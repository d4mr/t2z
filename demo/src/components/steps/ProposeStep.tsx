import { useState } from 'react';
import { Button } from '../Button';
import { Input } from '../Input';
import { CodeBlock } from '../CodeBlock';
import type { TransparentInput, Payment, Network } from '../../lib/types';
import * as t2z from '@d4mr/t2z-wasm';

interface Props {
  network: Network;
  inputs: TransparentInput[];
  payments: Payment[];
  totalIn: bigint;
  totalOut: bigint;
  changeAddress: string | null;
  onChangeAddressChange: (address: string | null) => void;
  onFeeChange: (fee: bigint) => void;
  onChangeChange: (change: bigint) => void;
  onPcztChange: (hex: string) => void;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error' | 'code', step: 'propose', message: string, data?: string) => void;
}

export function ProposeStep({
  network,
  inputs,
  payments,
  totalIn,
  totalOut,
  changeAddress,
  onChangeAddressChange,
  onFeeChange,
  onChangeChange,
  onPcztChange,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [isProposing, setIsProposing] = useState(false);
  const [pcztCreated, setPcztCreated] = useState(false);
  const [pcztHex, setPcztHex] = useState<string | null>(null);
  const [customChangeAddress, setCustomChangeAddress] = useState('');

  const isOrchardAddress = (addr: string) => {
    return addr.startsWith('u1') || addr.startsWith('utest1');
  };

  // The Builder calculates the exact ZIP-317 fee automatically
  // For display, we just show that there will be change if input > output
  const estimatedChange = totalIn - totalOut;

  const formatZec = (zatoshis: bigint) => {
    const zec = Number(zatoshis) / 100_000_000;
    return `${zec.toFixed(8)} ZEC`;
  };

  const handleGenerateChangeAddress = () => {
    try {
      const keypair = t2z.generate_test_keypair(network);
      setCustomChangeAddress(keypair.address);
      addLog('info', 'propose', 'Generated test Orchard change address (save spending key to recover funds!)', keypair.spending_key);
    } catch (err) {
      addLog('error', 'propose', `Failed to generate change address: ${err}`);
    }
  };

  const handlePropose = async () => {
    setIsProposing(true);
    try {
      // Build inputs
      const wasmInputs = inputs.map(input => 
        new t2z.WasmTransparentInput(
          input.pubkey,
          input.prevoutTxid,
          input.prevoutIndex,
          input.value,
          input.scriptPubkey,
          input.sequence
        )
      );

      // Build payments
      const wasmPayments = payments.map(payment =>
        new t2z.WasmPayment(
          payment.address,
          payment.amount,
          payment.memo ?? null,
          payment.label ?? null
        )
      );

      // Determine change address - always provide one if there might be change
      // The Builder will calculate the exact fee, so we need a change address
      // even if our estimate shows no change
      const finalChangeAddress = customChangeAddress || changeAddress || null;
      
      // If total input > total output, we'll have change and need an address
      if (totalIn > totalOut && !finalChangeAddress) {
        throw new Error('Change address required when input exceeds payment amount');
      }

      // Nu5 (Orchard) activation heights:
      // - Mainnet: 1,687,104
      // - Testnet: 1,842,420
      // We need to use a height AFTER Nu5 for Orchard to be available
      // In production, use current block height + some buffer for expiry
      const expiryHeight = network === 'mainnet' ? 2_500_000 : 2_500_000;

      addLog('info', 'propose', 'Calling propose_transaction...');
      addLog('code', 'propose', 'Creating PCZT with:', JSON.stringify({
        inputs: inputs.length,
        payments: payments.length,
        fee: 'auto (Builder will calculate ZIP-317 fee)',
        changeAddress: finalChangeAddress ? finalChangeAddress.slice(0, 20) + '...' : null,
        network,
        expiryHeight,
      }, null, 2));

      // The Builder will calculate the exact ZIP-317 fee automatically
      const pczt = t2z.propose_transaction(
        wasmInputs,
        wasmPayments,
        finalChangeAddress,  // change address (separate from TransactionRequest per ZIP 321)
        network,
        expiryHeight
      );

      const hex = pczt.to_hex();
      setPcztHex(hex);
      setPcztCreated(true);
      onPcztChange(hex);
      
      // The fee was calculated by the Builder automatically
      // We don't know the exact fee without parsing the PCZT
      onFeeChange(0n); // TODO: extract actual fee from PCZT if needed
      onChangeChange(0n); // TODO: extract actual change from PCZT if needed
      if (finalChangeAddress) {
        onChangeAddressChange(finalChangeAddress);
      }

      addLog('success', 'propose', `PCZT created successfully (${hex.length} bytes hex)`);
      addLog('code', 'propose', 'PCZT (first 200 chars):', hex.slice(0, 200) + '...');

    } catch (err) {
      addLog('error', 'propose', `Failed to propose transaction: ${err}`);
    } finally {
      setIsProposing(false);
    }
  };

  const needsChangeAddress = totalIn > totalOut && !changeAddress && !customChangeAddress;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Propose Transaction</h2>
        <p className="text-gray-400">
          Create the Partially Constructed Zcash Transaction (PCZT). This implements the 
          Creator, Constructor, and IO Finalizer roles per ZIP 374.
        </p>
      </div>

      {/* Transaction Summary */}
      <div className="p-4 bg-white/5 border border-white/10 rounded-lg space-y-4">
        <h3 className="font-medium text-white">Transaction Summary</h3>
        
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <div className="text-gray-500">Inputs</div>
            <div className="text-white font-mono">{inputs.length} × transparent</div>
          </div>
          <div>
            <div className="text-gray-500">Total In</div>
            <div className="text-white font-mono">{formatZec(totalIn)}</div>
          </div>
          <div>
            <div className="text-gray-500">Outputs</div>
            <div className="text-white font-mono">
              {payments.filter(p => isOrchardAddress(p.address)).length} × Orchard,{' '}
              {payments.filter(p => !isOrchardAddress(p.address)).length} × transparent
            </div>
          </div>
          <div>
            <div className="text-gray-500">Total Out</div>
            <div className="text-white font-mono">{formatZec(totalOut)}</div>
          </div>
          <div>
            <div className="text-gray-500">Fee</div>
            <div className="text-amber-400 font-mono">ZIP-317 auto</div>
          </div>
          <div>
            <div className="text-gray-500">Available for Change</div>
            <div className={`font-mono ${estimatedChange > 0n ? 'text-emerald-400' : 'text-gray-400'}`}>
              {formatZec(estimatedChange)}
            </div>
          </div>
        </div>
      </div>

      {/* Change Address - show if there might be change */}
      {totalIn > totalOut && (
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-300">
            Change Address (required when input {'>'} output)
          </label>
          <div className="flex gap-2">
            <Input
              placeholder="u1... (Orchard) or t1.../tm... (transparent)"
              value={customChangeAddress}
              onChange={(e) => setCustomChangeAddress(e.target.value)}
              className="flex-1"
            />
            <Button variant="secondary" onClick={handleGenerateChangeAddress}>
              Generate
            </Button>
          </div>
          <p className="text-xs text-gray-500">
            Change can go to Orchard (recommended for privacy) or transparent.
            {customChangeAddress && isOrchardAddress(customChangeAddress) && (
              <span className="text-purple-400"> ✓ Orchard address - change will be shielded</span>
            )}
          </p>
        </div>
      )}

      {/* PCZT Result */}
      {pcztCreated && pcztHex && (
        <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg">
          <div className="flex items-center justify-between mb-2">
            <span className="text-emerald-400 font-medium">✓ PCZT Created</span>
            <span className="text-gray-400 text-sm">{pcztHex.length} bytes (hex)</span>
          </div>
          <pre className="text-xs font-mono text-gray-400 overflow-x-auto whitespace-pre-wrap break-all max-h-32 overflow-y-auto">
            {pcztHex}
          </pre>
        </div>
      )}

      {/* Code Example */}
      <CodeBlock
        title="propose_transaction API (ZIP 321 + ZIP 374)"
        code={`import { propose_transaction, WasmTransparentInput, WasmPayment } from '@d4mr/t2z-wasm';

// TransactionRequest is just payments per ZIP 321 (no fee/change)
// https://zips.z.cash/zip-0321

// IMPORTANT: expiry_height must be AFTER Nu5 activation for Orchard to work!
// Nu5 activated at 1,687,104 (mainnet) / 1,842,420 (testnet)
const expiryHeight = 2_500_000;  // Use current height + buffer in production

const pczt = propose_transaction(
  inputs,           // WasmTransparentInput[]
  payments,         // WasmPayment[] (ZIP 321 payments)
  ${totalIn > totalOut ? `'${customChangeAddress || '<change_address>'}'` : 'null'},  // change address (separate parameter)
  '${network}',     // 'mainnet' or 'testnet'
  expiryHeight      // Block height for consensus rules (must be post-Nu5!)
);

// The PCZT is now ready for:
// 1. Verification (verify_before_signing with expected_change)
// 2. Signing (get_sighash + append_signature)
// 3. Proving (prove_transaction)`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <div className="flex gap-2">
          {!pcztCreated ? (
            <Button 
              onClick={handlePropose} 
              loading={isProposing}
              disabled={needsChangeAddress}
            >
              Create PCZT
            </Button>
          ) : (
            <Button onClick={onNext}>
              Continue to Verify →
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

