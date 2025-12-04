import { useEffect } from 'react';
import { useTransactionFlow } from './hooks/useTransactionFlow';
import { StepIndicator } from './components/StepIndicator';
import { LogPanel } from './components/LogPanel';
import {
  SetupStep,
  InputsStep,
  PaymentsStep,
  ProposeStep,
  VerifyStep,
  SignStep,
  ProveStep,
  FinalizeStep,
} from './components/steps';
import * as t2z from '@d4mr/t2z-wasm';

export function App() {
  const { state, actions } = useTransactionFlow();

  useEffect(() => {
    // Initialize WASM and log version
    try {
      t2z.init();
      actions.addLog('info', 'setup', `t2z-wasm initialized (v${t2z.version()})`);
    } catch (e) {
      // Might already be initialized
      actions.addLog('info', 'setup', `t2z-wasm ready (v${t2z.version()})`);
    }
  }, []);

  const renderStep = () => {
    switch (state.currentStep) {
      case 'setup':
        return (
          <SetupStep
            network={state.network}
            signingKey={state.signingKey}
            onNetworkChange={actions.setNetwork}
            onSigningKeyChange={actions.setSigningKey}
            onNext={() => actions.goToStep('inputs')}
            addLog={actions.addLog}
          />
        );
      
      case 'inputs':
        return state.signingKey ? (
          <InputsStep
            signingKey={state.signingKey}
            inputs={state.inputs}
            totalIn={state.totalIn}
            onAddInput={actions.addInput}
            onRemoveInput={actions.removeInput}
            onNext={() => actions.goToStep('payments')}
            onBack={() => actions.goToStep('setup')}
            addLog={actions.addLog}
          />
        ) : null;
      
      case 'payments':
        return (
          <PaymentsStep
            network={state.network}
            payments={state.payments}
            totalIn={state.totalIn}
            totalOut={state.totalOut}
            onAddPayment={actions.addPayment}
            onRemovePayment={actions.removePayment}
            onNext={() => actions.goToStep('propose')}
            onBack={() => actions.goToStep('inputs')}
            addLog={actions.addLog}
          />
        );
      
      case 'propose':
        return (
          <ProposeStep
            network={state.network}
            inputs={state.inputs}
            payments={state.payments}
            totalIn={state.totalIn}
            totalOut={state.totalOut}
            changeAddress={state.changeAddress}
            onChangeAddressChange={actions.setChangeAddress}
            onFeeChange={actions.setFee}
            onChangeChange={actions.setChange}
            onPcztChange={(hex) => {
              actions.setPcztHex(hex);
              actions.completeStep('propose');
            }}
            onNext={() => actions.goToStep('verify')}
            onBack={() => actions.goToStep('payments')}
            addLog={actions.addLog}
          />
        );
      
      case 'verify':
        return state.pcztHex ? (
          <VerifyStep
            pcztHex={state.pcztHex}
            payments={state.payments}
            changeAddress={state.changeAddress}
            changeAmount={state.change}
            onNext={() => {
              actions.completeStep('verify');
              actions.goToStep('sign');
            }}
            onBack={() => actions.goToStep('propose')}
            addLog={actions.addLog}
          />
        ) : null;
      
      case 'sign':
        return state.pcztHex && state.signingKey ? (
          <SignStep
            pcztHex={state.pcztHex}
            signingKey={state.signingKey}
            inputs={state.inputs}
            onPcztChange={(hex) => {
              actions.setPcztHex(hex);
              actions.completeStep('sign');
            }}
            onNext={() => actions.goToStep('prove')}
            onBack={() => actions.goToStep('verify')}
            addLog={actions.addLog}
          />
        ) : null;
      
      case 'prove':
        return state.pcztHex ? (
          <ProveStep
            pcztHex={state.pcztHex}
            onPcztChange={(hex) => {
              actions.setPcztHex(hex);
              actions.completeStep('prove');
            }}
            onNext={() => actions.goToStep('finalize')}
            onBack={() => actions.goToStep('sign')}
            addLog={actions.addLog}
          />
        ) : null;
      
      case 'finalize':
        return state.pcztHex ? (
          <FinalizeStep
            pcztHex={state.pcztHex}
            onFinalTxChange={(hex) => {
              actions.setFinalTxHex(hex);
              actions.completeStep('finalize');
            }}
            onBack={() => actions.goToStep('prove')}
            addLog={actions.addLog}
          />
        ) : null;
      
      default:
        return null;
    }
  };

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="border-b border-white/10 bg-black/20 backdrop-blur-sm sticky top-0 z-10">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-amber-400 to-amber-600 flex items-center justify-center font-bold text-black">
                t2z
              </div>
              <div>
                <h1 className="text-xl font-bold text-white">
                  Transparent → Shielded
                </h1>
                <p className="text-xs text-gray-400">
                  PCZT Transaction Builder (ZIP 374)
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {/* Icon Links */}
              <a 
                href="https://t2z.d4mr.com"
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-all"
                title="Documentation"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                </svg>
              </a>
              <a 
                href="https://github.com/d4mr/t2z"
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-all"
                title="GitHub"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                </svg>
              </a>
              <a 
                href="https://www.npmjs.com/package/@d4mr/t2z-wasm"
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-all"
                title="NPM Package"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M0 7.334v8h6.666v1.332H12v-1.332h12v-8H0zm6.666 6.664H5.334v-4H3.999v4H1.335V8.667h5.331v5.331zm4 0v1.336H8.001V8.667h5.334v5.332h-2.669v-.001zm12.001 0h-1.33v-4h-1.336v4h-1.335v-4h-1.33v4h-2.671V8.667h8.002v5.331zM10.665 10H12v2.667h-1.335V10z" />
                </svg>
              </a>
              
              <div className="w-px h-6 bg-white/10 mx-1" />
              
              <a 
                href="https://github.com/zcash/zips/pull/1063"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-gray-400 hover:text-amber-400 transition-colors"
              >
                ZIP 374
              </a>
              
              <div className={`ml-2 px-3 py-1 rounded-full text-xs font-medium ${
                state.network === 'mainnet' 
                  ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' 
                  : 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
              }`}>
                {state.network}
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* Sidebar - Step Indicator */}
          <div className="lg:col-span-1">
            <div className="sticky top-24 space-y-6">
              <StepIndicator
                steps={state.steps}
                currentStep={state.currentStep}
                onStepClick={actions.goToStep}
              />
              
              {/* Quick Stats */}
              <div className="p-4 bg-white/5 border border-white/10 rounded-lg space-y-3">
                <h3 className="text-sm font-medium text-gray-400">Transaction</h3>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <div className="text-gray-500">Inputs</div>
                    <div className="text-white font-mono">{state.inputs.length}</div>
                  </div>
                  <div>
                    <div className="text-gray-500">Outputs</div>
                    <div className="text-white font-mono">{state.payments.length}</div>
                  </div>
                  <div>
                    <div className="text-gray-500">Total In</div>
                    <div className="text-white font-mono">
                      {(Number(state.totalIn) / 100_000_000).toFixed(4)}
                    </div>
                  </div>
                  <div>
                    <div className="text-gray-500">Total Out</div>
                    <div className="text-white font-mono">
                      {(Number(state.totalOut) / 100_000_000).toFixed(4)}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Main Content Area */}
          <div className="lg:col-span-3 space-y-6">
            {/* Step Content */}
            <div className="bg-white/5 border border-white/10 rounded-xl p-6">
              {renderStep()}
            </div>

            {/* Log Panel */}
            <div className="space-y-2">
              <h3 className="text-sm font-medium text-gray-400">Activity Log</h3>
              <LogPanel logs={state.logs} />
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-white/10 mt-12">
        <div className="max-w-7xl mx-auto px-6 py-6">
          <div className="flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-gray-500">
            <div className="flex items-center gap-2">
              <a 
                href="https://www.npmjs.com/package/@d4mr/t2z-wasm"
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono hover:text-amber-400 transition-colors"
              >
                @d4mr/t2z-wasm
              </a>
              <span>v{t2z.version()}</span>
            </div>
            <div className="flex flex-wrap justify-center gap-4">
              <a 
                href="https://github.com/d4mr/t2z"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                GitHub
              </a>
              <span className="text-gray-700">•</span>
              <a 
                href="https://t2z.d4mr.com"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                Documentation
              </a>
              <span className="text-gray-700">•</span>
              <a 
                href="https://github.com/zcash/librustzcash"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                librustzcash
              </a>
              <span className="text-gray-700">•</span>
              <a 
                href="https://github.com/zcash/zips/pull/1063"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                ZIP 374
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

