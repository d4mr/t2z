import { useState } from 'react';
import { Button } from '../Button';
import { Input } from '../Input';
import { CodeBlock } from '../CodeBlock';
import type { Payment, Network } from '../../lib/types';
import * as t2z from '@d4mr/t2z-wasm';
import { bytesToHex } from '../../lib/crypto';

/**
 * Convert a text memo to hex encoding (for WASM binding)
 * The memo is encoded as UTF-8 bytes then hex-encoded
 */
function textToHex(text: string): string {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(text);
  return bytesToHex(bytes);
}

interface Props {
  network: Network;
  payments: Payment[];
  totalIn: bigint;
  totalOut: bigint;
  onAddPayment: (payment: Payment) => void;
  onRemovePayment: (index: number) => void;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error', step: 'payments', message: string, data?: string) => void;
}

export function PaymentsStep({
  network,
  payments,
  totalIn,
  totalOut,
  onAddPayment,
  onRemovePayment,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [address, setAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [memo, setMemo] = useState('');
  const [label, setLabel] = useState('');

  const handleGenerateTestAddress = () => {
    try {
      // Use generate_test_keypair to get address + keys for viewing
      const result = t2z.generate_test_keypair(network) as {
        address: string;
        spending_key: string;
        full_viewing_key: string;  // uviewtest1... or uview1...
        full_viewing_key_hex: string;
      };
      setAddress(result.address);
      addLog('info', 'payments', 'Generated test Orchard address with viewing key');
      addLog('info', 'payments', `Address: ${result.address}`);
      addLog('info', 'payments', `Unified Viewing Key: ${result.full_viewing_key}`);
      addLog('info', 'payments', `⚠️ Spending Key (hex, save to spend!): ${result.spending_key}`);
    } catch (err) {
      addLog('error', 'payments', `Failed to generate address: ${err}`);
    }
  };

  const handleAddPayment = () => {
    try {
      const amountNum = BigInt(amount);
      if (amountNum <= 0n) {
        throw new Error('Amount must be positive');
      }

      // Convert memo text to hex for WASM binding
      // The WASM binding expects hex-encoded memo bytes
      const memoText = memo.trim();
      const memoHex = memoText ? textToHex(memoText) : undefined;
      
      if (memoHex && memoHex.length > 1024) { // 512 bytes = 1024 hex chars
        throw new Error('Memo too long (max 512 bytes)');
      }

      const payment: Payment = {
        address: address.trim(),
        amount: amountNum,
        memo: memoHex,  // Now hex-encoded
        label: label.trim() || undefined,
      };

      onAddPayment(payment);
      addLog('info', 'payments', `Added payment: ${Number(amountNum) / 100_000_000} ZEC to ${address.slice(0, 20)}...`);
      
      // Reset form
      setAddress('');
      setAmount('');
      setMemo('');
      setLabel('');
    } catch (err) {
      addLog('error', 'payments', `Invalid payment: ${err}`);
    }
  };

  const formatZec = (zatoshis: bigint) => {
    const zec = Number(zatoshis) / 100_000_000;
    return `${zec.toFixed(8)} ZEC`;
  };

  const isOrchardAddress = (addr: string) => {
    return addr.startsWith('u1') || addr.startsWith('utest1');
  };

  // ZIP-317 fees are typically 10,000-20,000 zatoshis
  // The exact fee is calculated by the Builder in the propose step
  const MIN_FEE_ESTIMATE = 20_000n;
  const remaining = totalIn - totalOut - MIN_FEE_ESTIMATE;

  // Allow proceeding if we have payments and enough input to cover outputs + min fee
  const canProceed = payments.length > 0 && totalIn > 0n;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Payment Outputs</h2>
        <p className="text-gray-400">
          Define where to send funds. Use a Unified Address (starting with u1/utest1) 
          for shielded Orchard outputs, or a transparent address for t-address outputs.
        </p>
      </div>

      {/* Balance Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-4 bg-white/5 border border-white/10 rounded-lg">
        <div>
          <div className="text-xs text-gray-500 uppercase">Total Input</div>
          <div className="text-lg font-mono text-white">{formatZec(totalIn)}</div>
        </div>
        <div>
          <div className="text-xs text-gray-500 uppercase">Total Output</div>
          <div className="text-lg font-mono text-amber-400">{formatZec(totalOut)}</div>
        </div>
        <div>
          <div className="text-xs text-gray-500 uppercase">Fee</div>
          <div className="text-lg font-mono text-gray-400">ZIP-317 auto</div>
        </div>
        <div>
          <div className="text-xs text-gray-500 uppercase">Remaining</div>
          <div className={`text-lg font-mono ${remaining >= 0n ? 'text-emerald-400' : 'text-red-400'}`}>
            {formatZec(remaining)}
          </div>
        </div>
      </div>

      {/* Current Payments */}
      {payments.length > 0 && (
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-300">
            Payments ({payments.length})
          </label>
          <div className="space-y-2">
            {payments.map((payment, index) => (
              <div 
                key={index}
                className="flex items-center justify-between p-3 bg-white/5 border border-white/10 rounded-lg"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className={`text-xs px-2 py-0.5 rounded ${
                      isOrchardAddress(payment.address)
                        ? 'bg-purple-500/20 text-purple-400'
                        : 'bg-blue-500/20 text-blue-400'
                    }`}>
                      {isOrchardAddress(payment.address) ? 'Orchard' : 'Transparent'}
                    </span>
                    {payment.label && (
                      <span className="text-gray-400 text-sm">{payment.label}</span>
                    )}
                  </div>
                  <div className="font-mono text-sm text-white mt-1">{formatZec(payment.amount)}</div>
                  <div className="text-gray-500 text-xs font-mono truncate">
                    {payment.address}
                  </div>
                  {payment.memo && (
                    <div className="text-gray-400 text-xs mt-1">Memo: {payment.memo}</div>
                  )}
                </div>
                <Button variant="danger" size="sm" onClick={() => onRemovePayment(index)}>
                  Remove
                </Button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Add Payment Form */}
      <div className="space-y-4 p-4 bg-white/5 border border-white/10 rounded-lg">
        <h3 className="font-medium text-white">Add Payment</h3>
        
        <div className="space-y-4">
          <div>
            <div className="flex items-end gap-2">
              <div className="flex-1">
                <Input
                  label="Recipient Address"
                  placeholder="u1... (Orchard) or t1.../tm... (transparent)"
                  value={address}
                  onChange={(e) => setAddress(e.target.value)}
                />
              </div>
              <Button variant="secondary" onClick={handleGenerateTestAddress}>
                Generate Test
              </Button>
            </div>
            {address && (
              <p className="text-xs mt-1 text-gray-500">
                {isOrchardAddress(address) 
                  ? '✓ Unified address with Orchard receiver - will create shielded output'
                  : 'Transparent address - will create transparent output'}
              </p>
            )}
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              label="Amount (zatoshis)"
              type="number"
              min="1"
              placeholder="e.g. 500000 (0.005 ZEC)"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              hint="Leave room for fees (~10,000-20,000 zatoshis)"
            />
            <Input
              label="Label (optional)"
              placeholder="e.g. 'Payment for coffee'"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
          </div>
          
          {address && isOrchardAddress(address) && (
            <Input
              label="Memo (optional, shielded only)"
              placeholder="Private message to recipient (max 512 bytes)"
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              hint="Enter plain text - it will be encrypted and only visible to the recipient"
            />
          )}
        </div>

        <Button onClick={handleAddPayment} disabled={!address || !amount}>
          Add Payment
        </Button>
      </div>

      {/* Warning for remaining funds */}
      {remaining > 0n && payments.length > 0 && (
        <div className="p-4 bg-amber-500/10 border border-amber-500/30 rounded-lg">
          <p className="text-amber-400 text-sm">
            <strong>Note:</strong> You have {formatZec(remaining)} remaining after payments and fees.
            You'll need to specify a change address in the next step, or add more payments.
          </p>
        </div>
      )}

      {/* Code Example */}
      <CodeBlock
        title="Creating payments"
        code={`import { WasmPayment } from '@d4mr/t2z-wasm';

// Payment to a shielded Orchard address
const payment = new WasmPayment(
  '${address || 'u1...'}',  // Unified address with Orchard receiver
  ${amount || '50000'}n,    // Amount in zatoshis
  ${memo ? `'${memo}'` : 'null'},      // Optional encrypted memo
  ${label ? `'${label}'` : 'null'}     // Optional label
);

// The address determines the output type:
// - u1.../utest1... → Orchard shielded output
// - t1.../tm...     → Transparent output`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <Button onClick={onNext} disabled={!canProceed}>
          Continue to Propose →
        </Button>
      </div>
    </div>
  );
}

