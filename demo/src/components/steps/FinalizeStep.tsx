import { useState } from 'react';
import { Button } from '../Button';
import { CodeBlock } from '../CodeBlock';
import * as t2z from '@d4mr/t2z-wasm';

interface Props {
  pcztHex: string;
  onFinalTxChange: (hex: string) => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error' | 'code', step: 'finalize', message: string, data?: string) => void;
}

export function FinalizeStep({
  pcztHex,
  onFinalTxChange,
  onBack,
  addLog,
}: Props) {
  const [isFinalizing, setIsFinalizing] = useState(false);
  const [finalized, setFinalized] = useState(false);
  const [finalTxHex, setFinalTxHex] = useState<string | null>(null);

  const handleFinalize = async () => {
    setIsFinalizing(true);
    addLog('info', 'finalize', 'Finalizing transaction and extracting raw bytes...');
    
    try {
      const pczt = t2z.WasmPczt.from_hex(pcztHex);
      const txHex = t2z.finalize_and_extract_hex(pczt);
      
      setFinalTxHex(txHex);
      setFinalized(true);
      onFinalTxChange(txHex);
      
      addLog('success', 'finalize', 'Transaction finalized successfully!');
      addLog('code', 'finalize', 'Raw transaction hex:', txHex);
      
    } catch (err) {
      addLog('error', 'finalize', `Failed to finalize: ${err}`);
    } finally {
      setIsFinalizing(false);
    }
  };

  const handleCopyTx = () => {
    if (finalTxHex) {
      navigator.clipboard.writeText(finalTxHex);
      addLog('info', 'finalize', 'Transaction hex copied to clipboard');
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Finalize & Extract</h2>
        <p className="text-gray-400">
          Execute the Spend Finalizer and Transaction Extractor roles. This produces the 
          final transaction bytes ready to broadcast to the Zcash network.
        </p>
      </div>

      {/* Finalize Section */}
      {!finalized ? (
        <div className="p-4 bg-white/5 border border-white/10 rounded-lg space-y-4">
          <div>
            <div className="text-white font-medium">Ready to Finalize</div>
            <div className="text-gray-400 text-sm mt-1">
              This will perform final validation and extract the raw transaction bytes.
            </div>
          </div>
          
          <Button onClick={handleFinalize} loading={isFinalizing}>
            Finalize Transaction
          </Button>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <div className="text-emerald-400 font-medium">✓ Transaction Ready to Broadcast</div>
              <Button size="sm" variant="secondary" onClick={handleCopyTx}>
                Copy Hex
              </Button>
            </div>
            <div className="text-gray-400 text-sm">
              Transaction size: {finalTxHex ? Math.floor(finalTxHex.length / 2) : 0} bytes
            </div>
          </div>
          
          {/* Raw Transaction */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-300">
              Raw Transaction (Hex)
            </label>
            <pre className="p-4 bg-black/40 border border-white/10 rounded-lg text-xs font-mono text-gray-300 overflow-x-auto whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
              {finalTxHex}
            </pre>
          </div>
        </div>
      )}

      {/* Broadcasting Info */}
      {finalized && (
        <div className="p-4 bg-amber-500/10 border border-amber-500/30 rounded-lg">
          <div className="text-amber-400 font-medium mb-2">Broadcasting</div>
          <p className="text-gray-400 text-sm mb-3">
            The transaction hex can be broadcast to the Zcash network using:
          </p>
          <ul className="text-gray-400 text-sm space-y-2 list-disc list-inside">
            <li>
              <strong>zcash-cli:</strong>{' '}
              <code className="bg-black/40 px-1 py-0.5 rounded text-xs">
                zcash-cli sendrawtransaction "{'<hex>'}"
              </code>
            </li>
            <li>
              <strong>Block Explorer:</strong> Many explorers have a "Broadcast" feature
            </li>
            <li>
              <strong>Lightwalletd:</strong> Via the <code className="bg-black/40 px-1 py-0.5 rounded text-xs">SendTransaction</code> RPC
            </li>
          </ul>
        </div>
      )}

      {/* Code Example */}
      <CodeBlock
        title="finalize_and_extract API"
        code={`import { finalize_and_extract, finalize_and_extract_hex } from '@d4mr/t2z-wasm';

// Extract as bytes
const txBytes: Uint8Array = finalize_and_extract(pczt);

// Or as hex string (convenient for broadcasting)
const txHex: string = finalize_and_extract_hex(pczt);

// The transaction is now ready to broadcast!
// It contains:
// - Transparent inputs with valid signatures
// - Orchard actions with valid proofs
// - Proper binding signatures
// - All required commitments and anchors`}
      />

      {/* Success Message */}
      {finalized && (
        <div className="p-6 bg-gradient-to-r from-emerald-500/20 to-amber-500/20 border border-emerald-500/30 rounded-lg text-center">
          <div className="text-4xl mb-4">🎉</div>
          <div className="text-2xl font-bold text-white mb-2">Transaction Complete!</div>
          <p className="text-gray-300">
            You've successfully created a transparent-to-shielded Zcash transaction 
            using the PCZT (ZIP 374) flow.
          </p>
          <p className="text-gray-400 text-sm mt-4">
            This demo used test/fake UTXOs. For real transactions, use actual UTXOs 
            from your wallet and broadcast the resulting transaction.
          </p>
        </div>
      )}

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        {finalized && (
          <Button variant="secondary" onClick={() => window.location.reload()}>
            Start New Transaction
          </Button>
        )}
      </div>
    </div>
  );
}

