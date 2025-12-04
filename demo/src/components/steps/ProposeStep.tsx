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
      const wasmPayments = payments.map((payment, idx) => {
        // Debug logging to verify amounts
        console.log(`Payment ${idx}: address=${payment.address.slice(0, 20)}..., amount=${payment.amount} (type: ${typeof payment.amount})`);
        addLog('info', 'propose', `Payment ${idx}: ${Number(payment.amount) / 100_000_000} ZEC (${payment.amount} zatoshis)`);
        
        // Ensure amount is converted properly for WASM
        // wasm-bindgen expects u64 which can accept BigInt
        const amount = payment.amount;
        
        return new t2z.WasmPayment(
          payment.address,
          amount,
          payment.memo ?? null,
          payment.label ?? null
        );
      });

      // Determine change address - always provide one if there might be change
      // The Builder will calculate the exact fee, so we need a change address
      // even if our estimate shows no change
      const finalChangeAddress = customChangeAddress || changeAddress || null;
      
      // If total input > total output, we'll have change and need an address
      if (totalIn > totalOut && !finalChangeAddress) {
        throw new Error('Change address required when input exceeds payment amount');
      }

      // Expiry height must be:
      // 1. After Nu5 activation (mainnet: 1,687,104, testnet: 1,842,420)
      // 2. At least current block height + ~40 blocks to avoid "expiring soon" error
      // 
      // Current approximate heights (Dec 2025):
      // - Mainnet: ~3,720,000
      // - Testnet: ~3,720,000 (similar to mainnet)
      //
      // In production, fetch current height from lightwalletd and add a buffer (e.g., +100 blocks)
      const currentApproxHeight = 3_720_000; // Same for both networks
      const expiryHeight = currentApproxHeight + 100; // ~2.5 hours buffer

      // Debug: Log total payment amounts
      const totalPaymentAmount = payments.reduce((sum, p) => sum + p.amount, 0n);
      console.log('Total payment amount:', totalPaymentAmount, 'type:', typeof totalPaymentAmount);
      
      addLog('info', 'propose', 'Calling propose_transaction...');
      addLog('info', 'propose', `Total payment amount: ${Number(totalPaymentAmount) / 100_000_000} ZEC (${totalPaymentAmount} zatoshis)`);
      addLog('code', 'propose', 'Creating PCZT with:', JSON.stringify({
        inputs: inputs.length,
        payments: payments.length,
        totalPaymentZatoshis: totalPaymentAmount.toString(),
        totalInputZatoshis: totalIn.toString(),
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
      
      // Use inspect_pczt to get the actual fee and transaction details
      const info = t2z.inspect_pczt(hex);
      console.log('PCZT inspection:', info);
      
      // Calculate actual fee and change from the PCZT
      // Change = total_input - fee - payment_amount
      const actualFee = BigInt(info.implied_fee);
      const totalInputValue = BigInt(info.total_input);
      const actualChange = totalInputValue - actualFee - totalOut;
      
      onFeeChange(actualFee);
      onChangeChange(actualChange > 0n ? actualChange : 0n);
      if (finalChangeAddress) {
        onChangeAddressChange(finalChangeAddress);
      }

      addLog('success', 'propose', `PCZT created successfully (${hex.length} bytes hex)`);
      addLog('code', 'propose', 'Transaction Details:', JSON.stringify({
        expiryHeight: info.expiry_height,
        totalInput: info.total_input,
        totalTransparentOutput: info.total_transparent_output,
        totalOrchardOutput: info.total_orchard_output,
        impliedFee: info.implied_fee,
        numOrchardActions: info.num_orchard_actions,
        inputs: info.transparent_inputs.map((i: any) => ({
          txid: i.prevout_txid.slice(0, 16) + '...',
          value: i.value,
          signed: i.is_signed,
        })),
        orchardOutputs: info.orchard_outputs.map((o: any) => ({
          value: o.value,
          hasRecipient: !!o.recipient,
        })),
      }, null, 2));

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

// IMPORTANT: expiry_height must be:
// 1. After Nu5 activation (mainnet: 1,687,104, testnet: 1,842,420)
// 2. At least current_height + 40 to avoid "tx-expiring-soon" error
// In production, fetch current height from lightwalletd
const currentHeight = await fetchCurrentBlockHeight(); // ~3,930,000 on mainnet (Dec 2025)
const expiryHeight = currentHeight + 100; // ~2.5 hours buffer

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

