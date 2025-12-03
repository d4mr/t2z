import { useState } from 'react';
import { Button } from '../Button';
import { CodeBlock } from '../CodeBlock';
import type { Payment } from '../../lib/types';
import * as t2z from '@d4mr/t2z-wasm';

interface Props {
  pcztHex: string;
  payments: Payment[];
  changeAddress: string | null;
  changeAmount: bigint;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error' | 'code', step: 'verify', message: string, data?: string) => void;
}

export function VerifyStep({
  pcztHex,
  payments,
  changeAddress,
  changeAmount,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [isVerifying, setIsVerifying] = useState(false);
  const [verified, setVerified] = useState(false);
  const [skipVerification, setSkipVerification] = useState(false);

  const handleVerify = async () => {
    setIsVerifying(true);
    try {
      const pczt = t2z.WasmPczt.from_hex(pcztHex);
      
      // Build payments for verification (ZIP 321 format)
      const wasmPayments = payments.map(payment =>
        new t2z.WasmPayment(
          payment.address,
          payment.amount,
          payment.memo ?? null,
          payment.label ?? null
        )
      );

      // Build expected change outputs (per spec: expected_change: [TxOut])
      const expectedChange: t2z.WasmExpectedTxOut[] = [];
      if (changeAddress && changeAmount > 0n) {
        expectedChange.push(new t2z.WasmExpectedTxOut(changeAddress, changeAmount));
      }

      addLog('info', 'verify', 'Calling verify_before_signing...');
      addLog('code', 'verify', 'Verification parameters:', JSON.stringify({
        paymentsCount: payments.length,
        expectedChange: expectedChange.length > 0 ? [{
          address: changeAddress?.slice(0, 20) + '...',
          amount: changeAmount.toString(),
        }] : [],
      }, null, 2));

      // Verify the PCZT matches our original request
      // Per spec: verify_before_signing(pczt, transaction_request, expected_change)
      t2z.verify_before_signing(
        pczt,
        wasmPayments,
        expectedChange
      );

      setVerified(true);
      addLog('success', 'verify', 'PCZT verification passed - all outputs match the original request');

    } catch (err) {
      addLog('error', 'verify', `Verification failed: ${err}`);
    } finally {
      setIsVerifying(false);
    }
  };

  const handleSkip = () => {
    setSkipVerification(true);
    addLog('info', 'verify', 'Skipped verification (same entity created and will sign the PCZT)');
  };

  const canProceed = verified || skipVerification;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Verify Before Signing</h2>
        <p className="text-gray-400">
          Verify the PCZT matches the original transaction request. This is a critical security 
          check to detect any malleation if the PCZT was handled by a third party.
        </p>
      </div>

      {/* Verification Status */}
      <div className={`p-4 rounded-lg border ${
        verified 
          ? 'bg-emerald-500/10 border-emerald-500/30' 
          : skipVerification
            ? 'bg-amber-500/10 border-amber-500/30'
            : 'bg-white/5 border-white/10'
      }`}>
        {verified ? (
          <div className="flex items-center gap-2">
            <span className="text-2xl">✓</span>
            <div>
              <div className="text-emerald-400 font-medium">Verification Passed</div>
              <div className="text-gray-400 text-sm">
                All outputs match the original request. Safe to sign.
              </div>
            </div>
          </div>
        ) : skipVerification ? (
          <div className="flex items-center gap-2">
            <span className="text-2xl">⚡</span>
            <div>
              <div className="text-amber-400 font-medium">Verification Skipped</div>
              <div className="text-gray-400 text-sm">
                You chose to skip verification. Only do this if you created the PCZT yourself.
              </div>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="text-gray-300">
              <strong>What this checks:</strong>
              <ul className="list-disc list-inside mt-2 text-sm text-gray-400 space-y-1">
                <li>All requested payments are present in the PCZT</li>
                <li>Payment amounts match exactly</li>
                <li>No unexpected outputs were added (malleation check)</li>
                <li>Change output matches expected address and amount</li>
              </ul>
            </div>
            
            <div className="flex gap-2">
              <Button onClick={handleVerify} loading={isVerifying}>
                Verify PCZT
              </Button>
              <Button variant="secondary" onClick={handleSkip}>
                Skip (Same Entity)
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* When to skip */}
      <div className="p-4 bg-blue-500/10 border border-blue-500/30 rounded-lg">
        <div className="text-blue-400 font-medium mb-2">When can verification be skipped?</div>
        <p className="text-gray-400 text-sm">
          Per ZIP 374: "If the entity that invoked propose_transaction is the same as the entity 
          that is adding the signatures, and no third party may have malleated the PCZT before 
          signing, this step may be skipped."
        </p>
        <p className="text-gray-400 text-sm mt-2">
          In this demo, you created the PCZT yourself, so skipping is safe. In production with 
          multi-party construction (e.g., hardware wallets, MPC), always verify.
        </p>
      </div>

      {/* Code Example */}
      <CodeBlock
        title="verify_before_signing API (per ZIP 374 spec)"
        code={`import { verify_before_signing, WasmPczt, WasmPayment, WasmExpectedTxOut } from '@d4mr/t2z-wasm';

// Re-create the original payments (ZIP 321 format)
const originalPayments = [
${payments.map(p => `  new WasmPayment('${p.address.slice(0, 30)}...', ${p.amount}n, ${p.memo ? `'${p.memo}'` : 'null'}, null)`).join(',\n')}
];

// Expected change outputs (per spec: expected_change: [TxOut])
const expectedChange = [
${changeAddress && changeAmount > 0n ? `  new WasmExpectedTxOut('${changeAddress.slice(0, 30)}...', ${changeAmount}n)` : '  // No change expected'}
];

// Verify PCZT matches original request
// Per spec: verify_before_signing(pczt, transaction_request, expected_change)
verify_before_signing(
  pczt,                    // The PCZT to verify
  originalPayments,        // ZIP 321 payments
  expectedChange           // Expected change outputs [TxOut]
);

// Throws an error if verification fails`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <Button onClick={onNext} disabled={!canProceed}>
          Continue to Sign →
        </Button>
      </div>
    </div>
  );
}

