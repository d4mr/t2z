import { useState } from 'react';
import { Button } from '../Button';
import { Input } from '../Input';
import { CodeBlock } from '../CodeBlock';
import type { Network } from '../../lib/types';
import { generateTransparentKeypair, hexToBytes, bytesToHex, getPublicKey, pubkeyToScriptPubkey, pubkeyToAddress } from '../../lib/crypto';
import type { KeyPair } from '../../lib/types';

interface Props {
  network: Network;
  signingKey: KeyPair | null;
  onNetworkChange: (network: Network) => void;
  onSigningKeyChange: (key: KeyPair | null) => void;
  onNext: () => void;
  addLog: (type: 'info' | 'success' | 'error', step: 'setup', message: string, data?: string) => void;
}

export function SetupStep({
  network,
  signingKey,
  onNetworkChange,
  onSigningKeyChange,
  onNext,
  addLog,
}: Props) {
  const [privateKeyInput, setPrivateKeyInput] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);

  const handleGenerateKey = () => {
    setIsGenerating(true);
    try {
      const keypair = generateTransparentKeypair(network);
      onSigningKeyChange(keypair);
      addLog('success', 'setup', 'Generated new transparent keypair', keypair.address);
    } catch (err) {
      addLog('error', 'setup', `Failed to generate keypair: ${err}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleImportKey = () => {
    try {
      const cleanHex = privateKeyInput.trim().replace(/^0x/, '');
      if (cleanHex.length !== 64) {
        throw new Error('Private key must be 32 bytes (64 hex characters)');
      }

      const privateKeyBytes = hexToBytes(cleanHex);
      const publicKeyBytes = getPublicKey(privateKeyBytes);
      const scriptPubkeyBytes = pubkeyToScriptPubkey(publicKeyBytes);
      const address = pubkeyToAddress(publicKeyBytes, network);

      const keypair: KeyPair = {
        privateKey: cleanHex,
        publicKey: bytesToHex(publicKeyBytes),
        address,
        scriptPubkey: bytesToHex(scriptPubkeyBytes),
      };

      onSigningKeyChange(keypair);
      addLog('success', 'setup', 'Imported private key', address);
      setPrivateKeyInput('');
    } catch (err) {
      addLog('error', 'setup', `Failed to import key: ${err}`);
    }
  };

  const canProceed = signingKey !== null;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">Setup</h2>
        <p className="text-gray-400">
          Configure the network and your signing key. The signing key controls the transparent
          UTXOs you'll spend in this transaction.
        </p>
      </div>

      {/* Network Selection */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-300">Network</label>
        <div className="flex gap-2">
          <button
            onClick={() => onNetworkChange('mainnet')}
            className={`px-4 py-2 rounded-lg border transition-all ${network === 'mainnet'
              ? 'bg-amber-500/20 border-amber-500 text-amber-300'
              : 'bg-white/5 border-white/20 text-gray-400 hover:bg-white/10'
              }`}
          >
            Mainnet
          </button>
          <button
            onClick={() => onNetworkChange('testnet')}
            className={`px-4 py-2 rounded-lg border transition-all ${network === 'testnet'
              ? 'bg-amber-500/20 border-amber-500 text-amber-300'
              : 'bg-white/5 border-white/20 text-gray-400 hover:bg-white/10'
              }`}
          >
            Testnet
          </button>
        </div>
        <p className="text-xs text-gray-500">
          {network === 'mainnet'
            ? 'Using mainnet - real ZEC. Addresses start with t1/t3.'
            : 'Using testnet - test ZEC only. Addresses start with tm/t2.'}
        </p>
      </div>

      {/* Signing Key */}
      <div className="space-y-4">
        <label className="block text-sm font-medium text-gray-300">Signing Key</label>

        {signingKey ? (
          <div className="space-y-3 p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg">
            <div className="flex items-center justify-between">
              <span className="text-sm text-emerald-400">✓ Key loaded</span>
              <Button
                variant="danger"
                size="sm"
                onClick={() => onSigningKeyChange(null)}
              >
                Clear
              </Button>
            </div>
            <div className="space-y-2 text-sm font-mono">
              <div>
                <span className="text-gray-500">Address: </span>
                <span className="text-white break-all">{signingKey.address}</span>
              </div>
              <div>
                <span className="text-gray-500">Public Key: </span>
                <span className="text-gray-300 break-all">{signingKey.publicKey}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-gray-500">Private Key: </span>
                {showPrivateKey ? (
                  <span className="text-red-400 break-all">{signingKey.privateKey}</span>
                ) : (
                  <span className="text-gray-500">••••••••</span>
                )}
                <button
                  onClick={() => setShowPrivateKey(!showPrivateKey)}
                  className="text-xs text-amber-500 hover:text-amber-400"
                >
                  {showPrivateKey ? 'Hide' : 'Show'}
                </button>
              </div>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex gap-2">
              <Button onClick={handleGenerateKey} loading={isGenerating}>
                Generate Random Key
              </Button>
            </div>

            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/10"></div>
              </div>
              <div className="relative flex justify-center text-xs">
                <span className="px-2 bg-[#1a1a2e] text-gray-500">or import existing</span>
              </div>
            </div>

            <div className="flex gap-2">
              <Input
                placeholder="32-byte private key (hex)"
                value={privateKeyInput}
                onChange={(e) => setPrivateKeyInput(e.target.value)}
                className="flex-1"
              />
              <Button
                variant="secondary"
                onClick={handleImportKey}
                disabled={!privateKeyInput.trim()}
              >
                Import
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Code Example */}
      <CodeBlock
        title="Usage Example"
        code={`import { propose_transaction, WasmTransparentInput } from '@d4mr/t2z-wasm';

// The signing key's public key and scriptPubkey are needed for inputs
const input = new WasmTransparentInput(
  '${signingKey?.publicKey || '<pubkey_hex>'}',  // 33-byte compressed pubkey
  '<prevout_txid>',                                // 32-byte txid (hex)
  0,                                               // prevout index
  100000n,                                         // value in zatoshis
  '${signingKey?.scriptPubkey || '<script_pubkey>'}' // P2PKH scriptPubkey
);`}
      />

      <div className="flex justify-end">
        <Button onClick={onNext} disabled={!canProceed}>
          Continue to Inputs →
        </Button>
      </div>
    </div>
  );
}

