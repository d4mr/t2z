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
            <div className="flex items-center gap-4">
              <a 
                href="https://zips.z.cash/zip-0374"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-gray-400 hover:text-amber-400 transition-colors"
              >
                ZIP 374 →
              </a>
              <div className={`px-3 py-1 rounded-full text-xs font-medium ${
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
          <div className="flex items-center justify-between text-sm text-gray-500">
            <div>
              <span className="font-mono">@d4mr/t2z-wasm</span> v{t2z.version()}
            </div>
            <div className="flex gap-4">
              <a 
                href="https://github.com/zcash/librustzcash"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                librustzcash
              </a>
              <a 
                href="https://github.com/zcash/pczt"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-amber-400 transition-colors"
              >
                pczt crate
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

