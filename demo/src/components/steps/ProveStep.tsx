import { useState, useEffect } from 'react';
import { Button } from '../Button';
import { CodeBlock } from '../CodeBlock';
import * as t2z from '@d4mr/t2z-wasm';

interface Props {
  pcztHex: string;
  onPcztChange: (hex: string) => void;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error' | 'code', step: 'prove', message: string, data?: string) => void;
}

export function ProveStep({
  pcztHex,
  onPcztChange,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [isProving, setIsProving] = useState(false);
  const [proved, setProved] = useState(false);
  const [provingKeyReady, setProvingKeyReady] = useState(false);
  const [isBuildingKey, setIsBuildingKey] = useState(false);

  useEffect(() => {
    // Check if proving key is already built
    setProvingKeyReady(t2z.is_proving_key_ready());
  }, []);

  const handlePrebuildKey = async () => {
    setIsBuildingKey(true);
    addLog('info', 'prove', 'Building Orchard proving key (Halo 2 circuit)... This takes ~10 seconds');
    
    try {
      // Run in next tick to allow UI to update
      await new Promise(resolve => setTimeout(resolve, 100));
      t2z.prebuild_proving_key();
      setProvingKeyReady(true);
      addLog('success', 'prove', 'Proving key built and cached. Subsequent proofs will be fast.');
    } catch (err) {
      addLog('error', 'prove', `Failed to build proving key: ${err}`);
    } finally {
      setIsBuildingKey(false);
    }
  };

  const handleProve = async () => {
    setIsProving(true);
    addLog('info', 'prove', 'Generating Orchard proofs...');
    
    try {
      const startTime = Date.now();
      
      const pczt = t2z.WasmPczt.from_hex(pcztHex);
      const provedPczt = t2z.prove_transaction(pczt);
      
      const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
      const newHex = provedPczt.to_hex();
      
      onPcztChange(newHex);
      setProved(true);
      
      addLog('success', 'prove', `Orchard proofs generated in ${elapsed}s`);
      addLog('code', 'prove', 'Proved PCZT size:', `${newHex.length} bytes (hex)`);
      
    } catch (err) {
      addLog('error', 'prove', `Failed to generate proofs: ${err}`);
    } finally {
      setIsProving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Generate Proofs</h2>
        <p className="text-gray-400">
          Generate zero-knowledge proofs for any Orchard outputs. This uses Halo 2 - 
          no trusted setup required, and the circuit is built programmatically (no downloads).
        </p>
      </div>

      {/* Proving Key Status */}
      <div className={`p-4 rounded-lg border ${
        provingKeyReady 
          ? 'bg-emerald-500/10 border-emerald-500/30' 
          : 'bg-amber-500/10 border-amber-500/30'
      }`}>
        <div className="flex items-center justify-between">
          <div>
            <div className={`font-medium ${provingKeyReady ? 'text-emerald-400' : 'text-amber-400'}`}>
              {provingKeyReady ? '✓ Proving Key Ready' : '⏳ Proving Key Not Built'}
            </div>
            <div className="text-gray-400 text-sm mt-1">
              {provingKeyReady 
                ? 'The Halo 2 circuit is cached. Proving will be fast.'
                : 'Building the proving key takes ~10 seconds on first use.'}
            </div>
          </div>
          {!provingKeyReady && (
            <Button 
              variant="secondary" 
              onClick={handlePrebuildKey}
              loading={isBuildingKey}
            >
              Pre-build Now
            </Button>
          )}
        </div>
      </div>

      {/* Prove Section */}
      <div className={`p-4 rounded-lg border ${
        proved 
          ? 'bg-emerald-500/10 border-emerald-500/30' 
          : 'bg-white/5 border-white/10'
      }`}>
        {proved ? (
          <div>
            <div className="text-emerald-400 font-medium">✓ Proofs Generated</div>
            <div className="text-gray-400 text-sm mt-1">
              The PCZT now contains valid Orchard proofs for all shielded outputs.
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div>
              <div className="text-white font-medium">Ready to Prove</div>
              <div className="text-gray-400 text-sm mt-1">
                This will generate zero-knowledge proofs for all Orchard actions in the transaction.
                The proofs ensure the transaction is valid without revealing amounts or addresses.
              </div>
            </div>
            
            <Button onClick={handleProve} loading={isProving}>
              {isProving ? 'Generating Proofs...' : 'Generate Proofs'}
            </Button>
          </div>
        )}
      </div>

      {/* Info Box */}
      <div className="p-4 bg-purple-500/10 border border-purple-500/30 rounded-lg">
        <div className="text-purple-400 font-medium mb-2">About Orchard Proofs</div>
        <ul className="text-gray-400 text-sm space-y-2">
          <li>
            <strong>Halo 2:</strong> Orchard uses Halo 2, a recursive zero-knowledge proof system 
            that requires no trusted setup (unlike Sprout and Sapling).
          </li>
          <li>
            <strong>No Downloads:</strong> The proving key is generated programmatically from the 
            circuit definition. No 50MB+ downloads like Sapling.
          </li>
          <li>
            <strong>Privacy:</strong> The proofs hide the amounts and addresses of shielded outputs 
            while proving the transaction is valid.
          </li>
        </ul>
      </div>

      {/* Code Example */}
      <CodeBlock
        title="prove_transaction API"
        code={`import { prove_transaction, prebuild_proving_key, is_proving_key_ready } from '@d4mr/t2z-wasm';

// Optionally pre-build the proving key at app startup
if (!is_proving_key_ready()) {
  prebuild_proving_key(); // ~10 seconds, cached globally
}

// Generate proofs (fast after key is built)
const provedPczt = prove_transaction(pczt);

// The PCZT now contains:
// - Orchard Action proofs (one per action)
// - Binding signature proof
// - All cryptographic commitments`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <Button onClick={onNext} disabled={!proved}>
          Continue to Finalize →
        </Button>
      </div>
    </div>
  );
}

