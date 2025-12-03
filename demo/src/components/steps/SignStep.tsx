import { useState } from 'react';
import { Button } from '../Button';
import { CodeBlock } from '../CodeBlock';
import type { KeyPair, TransparentInput } from '../../lib/types';
import { hexToBytes, signHash, bytesToHex } from '../../lib/crypto';
import * as t2z from '@d4mr/t2z-wasm';

interface Props {
  pcztHex: string;
  signingKey: KeyPair;
  inputs: TransparentInput[];
  onPcztChange: (hex: string) => void;
  onNext: () => void;
  onBack: () => void;
  addLog: (type: 'info' | 'success' | 'error' | 'code', step: 'sign', message: string, data?: string) => void;
}

export function SignStep({
  pcztHex,
  signingKey,
  inputs,
  onPcztChange,
  onNext,
  onBack,
  addLog,
}: Props) {
  const [isSigning, setIsSigning] = useState(false);
  const [signedInputs, setSignedInputs] = useState<Set<number>>(new Set());
  const [currentPcztHex, setCurrentPcztHex] = useState(pcztHex);
  // Default to convenience method - the get_sighash + append_signature flow
  // requires external signing infrastructure not available in browser
  const [useConvenienceMethod, setUseConvenienceMethod] = useState(true);

  const handleSignInput = async (inputIndex: number) => {
    setIsSigning(true);
    try {
      let pczt = t2z.WasmPczt.from_hex(currentPcztHex);
      
      if (useConvenienceMethod) {
        // Use the convenience sign_transparent_input function
        addLog('info', 'sign', `Signing input ${inputIndex} using sign_transparent_input (convenience method)...`);
        
        pczt = t2z.sign_transparent_input(pczt, inputIndex, signingKey.privateKey);
        
        addLog('success', 'sign', `Input ${inputIndex} signed with convenience method`);
      } else {
        // Use the spec-compliant get_sighash + append_signature flow
        addLog('info', 'sign', `Getting sighash for input ${inputIndex}...`);
        
        // Step 1: Get the sighash
        const sighashHex = t2z.get_sighash(pczt, inputIndex);
        addLog('code', 'sign', `Sighash for input ${inputIndex}:`, sighashHex);
        
        // Step 2: Sign externally (using @noble/secp256k1)
        addLog('info', 'sign', 'Signing sighash with private key (simulating external signer)...');
        
        const sighashBytes = hexToBytes(sighashHex);
        const privateKeyBytes = hexToBytes(signingKey.privateKey);
        
        // Sign and get DER-encoded signature
        const derSignature = await signHash(sighashBytes, privateKeyBytes);
        
        // Append SIGHASH_ALL type byte (0x01)
        const signatureWithType = new Uint8Array(derSignature.length + 1);
        signatureWithType.set(derSignature);
        signatureWithType[derSignature.length] = 0x01; // SIGHASH_ALL
        
        const signatureHex = bytesToHex(signatureWithType);
        addLog('code', 'sign', 'DER signature + sighash type:', signatureHex);
        
        // Step 3: Append signature to PCZT
        addLog('info', 'sign', 'Appending signature to PCZT...');
        pczt = t2z.append_signature(pczt, inputIndex, signingKey.publicKey, signatureHex);
        
        addLog('success', 'sign', `Input ${inputIndex} signed with get_sighash + append_signature flow`);
      }
      
      const newHex = pczt.to_hex();
      setCurrentPcztHex(newHex);
      onPcztChange(newHex);
      setSignedInputs(prev => new Set([...prev, inputIndex]));
      
    } catch (err) {
      addLog('error', 'sign', `Failed to sign input: ${err}`);
    } finally {
      setIsSigning(false);
    }
  };

  const handleSignAll = async () => {
    for (let i = 0; i < inputs.length; i++) {
      if (!signedInputs.has(i)) {
        await handleSignInput(i);
      }
    }
  };

  const allSigned = signedInputs.size === inputs.length;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Sign Transparent Inputs</h2>
        <p className="text-gray-400">
          Sign each transparent input. This demonstrates the two-step signing process:
          get the sighash (ZIP 244), then append the ECDSA signature.
        </p>
      </div>

      {/* Signing Method Toggle */}
      <div className="p-4 bg-white/5 border border-white/10 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-white font-medium">Signing Method</div>
            <div className="text-gray-400 text-sm">
              {useConvenienceMethod 
                ? 'Using sign_transparent_input (combines both steps internally)'
                : 'Using get_sighash + append_signature (spec-compliant, external signer flow)'}
            </div>
          </div>
          <button
            onClick={() => setUseConvenienceMethod(!useConvenienceMethod)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              useConvenienceMethod ? 'bg-amber-500' : 'bg-gray-600'
            }`}
          >
            <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              useConvenienceMethod ? 'translate-x-6' : 'translate-x-1'
            }`} />
          </button>
        </div>
      </div>

      {/* Input List */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="block text-sm font-medium text-gray-300">
            Inputs to Sign ({signedInputs.size}/{inputs.length})
          </label>
          {inputs.length > 1 && !allSigned && (
            <Button size="sm" onClick={handleSignAll} loading={isSigning}>
              Sign All
            </Button>
          )}
        </div>
        
        <div className="space-y-2">
          {inputs.map((input, index) => {
            const isSigned = signedInputs.has(index);
            const formatZec = (z: bigint) => (Number(z) / 100_000_000).toFixed(8);
            
            return (
              <div 
                key={index}
                className={`flex items-center justify-between p-3 rounded-lg border ${
                  isSigned 
                    ? 'bg-emerald-500/10 border-emerald-500/30' 
                    : 'bg-white/5 border-white/10'
                }`}
              >
                <div className="font-mono text-sm">
                  <div className="flex items-center gap-2">
                    {isSigned && <span className="text-emerald-400">✓</span>}
                    <span className="text-white">Input #{index}</span>
                  </div>
                  <div className="text-gray-500 text-xs mt-1">
                    {formatZec(input.value)} ZEC from {input.prevoutTxid.slice(0, 16)}...
                  </div>
                </div>
                <Button 
                  size="sm"
                  variant={isSigned ? 'secondary' : 'primary'}
                  onClick={() => handleSignInput(index)}
                  loading={isSigning}
                  disabled={isSigned}
                >
                  {isSigned ? 'Signed' : 'Sign'}
                </Button>
              </div>
            );
          })}
        </div>
      </div>

      {/* Success State */}
      {allSigned && (
        <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg">
          <div className="text-emerald-400 font-medium">✓ All inputs signed</div>
          <div className="text-gray-400 text-sm mt-1">
            The PCZT now contains valid signatures for all transparent inputs.
          </div>
        </div>
      )}

      {/* Code Example */}
      <CodeBlock
        title={useConvenienceMethod ? "Convenience signing API" : "Spec-compliant signing flow"}
        code={useConvenienceMethod 
          ? `import { sign_transparent_input, WasmPczt } from '@d4mr/t2z-wasm';

// Sign input using convenience method
const signedPczt = sign_transparent_input(
  pczt,
  ${inputs.length > 0 ? '0' : 'inputIndex'},      // input index
  '${signingKey.privateKey.slice(0, 16)}...'       // private key (hex)
);

// Internally this:
// 1. Computes the sighash per ZIP 244
// 2. Signs with ECDSA secp256k1
// 3. Appends signature to partial_signatures map`
          : `import { get_sighash, append_signature, WasmPczt } from '@d4mr/t2z-wasm';
import * as secp256k1 from '@noble/secp256k1';

// Step 1: Get the sighash (32 bytes)
const sighashHex = get_sighash(pczt, inputIndex);

// Step 2: Sign externally (e.g., hardware wallet, HSM)
const signature = await secp256k1.signAsync(
  hexToBytes(sighashHex),
  privateKeyBytes
);
const derSignature = signature.toDERRawBytes();

// Append SIGHASH_ALL type byte (0x01)
const sigWithType = new Uint8Array([...derSignature, 0x01]);

// Step 3: Append signature to PCZT
const signedPczt = append_signature(
  pczt,
  inputIndex,
  pubkeyHex,           // 33-byte compressed pubkey
  bytesToHex(sigWithType)  // DER + sighash type byte
);`}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <Button onClick={onNext} disabled={!allSigned}>
          Continue to Prove →
        </Button>
      </div>
    </div>
  );
}

