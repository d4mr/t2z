import { useState, useCallback, useRef } from 'react';
import type { 
  Network, 
  TransparentInput, 
  Payment, 
  KeyPair, 
  StepId, 
  Step, 
  LogEntry 
} from '../lib/types';

export interface TransactionFlowState {
  // Config
  network: Network;
  
  // Keys
  signingKey: KeyPair | null;
  
  // Transaction data
  inputs: TransparentInput[];
  payments: Payment[];
  changeAddress: string | null;
  
  // Calculated values
  fee: bigint;
  totalIn: bigint;
  totalOut: bigint;
  change: bigint;
  
  // PCZT state (stored as hex for display)
  pcztHex: string | null;
  
  // Final transaction
  finalTxHex: string | null;
  
  // UI state
  steps: Step[];
  currentStep: StepId;
  logs: LogEntry[];
  isLoading: boolean;
  provingKeyReady: boolean;
}

const INITIAL_STEPS: Step[] = [
  { id: 'setup', title: '1. Setup', description: 'Configure network and signing key', completed: false },
  { id: 'inputs', title: '2. Inputs', description: 'Add transparent UTXOs to spend', completed: false },
  { id: 'payments', title: '3. Payments', description: 'Define recipient outputs', completed: false },
  { id: 'propose', title: '4. Propose', description: 'Create the PCZT', completed: false },
  { id: 'verify', title: '5. Verify', description: 'Verify PCZT before signing', completed: false },
  { id: 'sign', title: '6. Sign', description: 'Sign transparent inputs', completed: false },
  { id: 'prove', title: '7. Prove', description: 'Generate Orchard proofs', completed: false },
  { id: 'finalize', title: '8. Finalize', description: 'Extract final transaction', completed: false },
];

export function useTransactionFlow() {
  const [state, setState] = useState<TransactionFlowState>({
    network: 'testnet',
    signingKey: null,
    inputs: [],
    payments: [],
    changeAddress: null,
    fee: 0n,
    totalIn: 0n,
    totalOut: 0n,
    change: 0n,
    pcztHex: null,
    finalTxHex: null,
    steps: INITIAL_STEPS,
    currentStep: 'setup',
    logs: [],
    isLoading: false,
    provingKeyReady: false,
  });
  
  const logIdCounter = useRef(0);

  const addLog = useCallback((
    type: LogEntry['type'],
    step: StepId,
    message: string,
    data?: string
  ) => {
    const entry: LogEntry = {
      id: `log-${++logIdCounter.current}`,
      timestamp: new Date(),
      type,
      step,
      message,
      data,
    };
    setState(s => ({ ...s, logs: [...s.logs, entry] }));
  }, []);

  const setNetwork = useCallback((network: Network) => {
    setState(s => ({ ...s, network }));
    addLog('info', 'setup', `Network set to ${network}`);
  }, [addLog]);

  const setSigningKey = useCallback((key: KeyPair | null) => {
    setState(s => ({ 
      ...s, 
      signingKey: key,
      steps: s.steps.map(step => 
        step.id === 'setup' ? { ...step, completed: key !== null } : step
      ),
    }));
    if (key) {
      addLog('success', 'setup', `Signing key set: ${key.address}`);
    }
  }, [addLog]);

  const addInput = useCallback((input: TransparentInput) => {
    setState(s => {
      const newInputs = [...s.inputs, input];
      const totalIn = newInputs.reduce((sum, i) => sum + i.value, 0n);
      return {
        ...s,
        inputs: newInputs,
        totalIn,
        steps: s.steps.map(step =>
          step.id === 'inputs' ? { ...step, completed: newInputs.length > 0 } : step
        ),
      };
    });
    addLog('info', 'inputs', `Added input: ${input.value.toString()} zatoshis from ${input.prevoutTxid.slice(0, 16)}...`);
  }, [addLog]);

  const removeInput = useCallback((index: number) => {
    setState(s => {
      const newInputs = s.inputs.filter((_, i) => i !== index);
      const totalIn = newInputs.reduce((sum, i) => sum + i.value, 0n);
      return {
        ...s,
        inputs: newInputs,
        totalIn,
        steps: s.steps.map(step =>
          step.id === 'inputs' ? { ...step, completed: newInputs.length > 0 } : step
        ),
      };
    });
  }, []);

  const addPayment = useCallback((payment: Payment) => {
    setState(s => {
      const newPayments = [...s.payments, payment];
      const totalOut = newPayments.reduce((sum, p) => sum + p.amount, 0n);
      return {
        ...s,
        payments: newPayments,
        totalOut,
        steps: s.steps.map(step =>
          step.id === 'payments' ? { ...step, completed: newPayments.length > 0 } : step
        ),
      };
    });
    addLog('info', 'payments', `Added payment: ${payment.amount.toString()} zatoshis to ${payment.address.slice(0, 20)}...`);
  }, [addLog]);

  const removePayment = useCallback((index: number) => {
    setState(s => {
      const newPayments = s.payments.filter((_, i) => i !== index);
      const totalOut = newPayments.reduce((sum, p) => sum + p.amount, 0n);
      return {
        ...s,
        payments: newPayments,
        totalOut,
        steps: s.steps.map(step =>
          step.id === 'payments' ? { ...step, completed: newPayments.length > 0 } : step
        ),
      };
    });
  }, []);

  const setChangeAddress = useCallback((address: string | null) => {
    setState(s => ({ ...s, changeAddress: address }));
    if (address) {
      addLog('info', 'setup', `Change address set: ${address.slice(0, 20)}...`);
    }
  }, [addLog]);

  const setFee = useCallback((fee: bigint) => {
    setState(s => ({ ...s, fee }));
  }, []);

  const setChange = useCallback((change: bigint) => {
    setState(s => ({ ...s, change }));
  }, []);

  const setPcztHex = useCallback((hex: string | null) => {
    setState(s => ({ ...s, pcztHex: hex }));
  }, []);

  const setFinalTxHex = useCallback((hex: string | null) => {
    setState(s => ({ ...s, finalTxHex: hex }));
  }, []);

  const setLoading = useCallback((isLoading: boolean) => {
    setState(s => ({ ...s, isLoading }));
  }, []);

  const setProvingKeyReady = useCallback((ready: boolean) => {
    setState(s => ({ ...s, provingKeyReady: ready }));
  }, []);

  const completeStep = useCallback((stepId: StepId) => {
    setState(s => ({
      ...s,
      steps: s.steps.map(step =>
        step.id === stepId ? { ...step, completed: true } : step
      ),
    }));
  }, []);

  const goToStep = useCallback((stepId: StepId) => {
    setState(s => ({ ...s, currentStep: stepId }));
  }, []);

  const reset = useCallback(() => {
    setState({
      network: 'testnet',
      signingKey: null,
      inputs: [],
      payments: [],
      changeAddress: null,
      fee: 0n,
      totalIn: 0n,
      totalOut: 0n,
      change: 0n,
      pcztHex: null,
      finalTxHex: null,
      steps: INITIAL_STEPS,
      currentStep: 'setup',
      logs: [],
      isLoading: false,
      provingKeyReady: false,
    });
  }, []);

  return {
    state,
    actions: {
      setNetwork,
      setSigningKey,
      addInput,
      removeInput,
      addPayment,
      removePayment,
      setChangeAddress,
      setFee,
      setChange,
      setPcztHex,
      setFinalTxHex,
      setLoading,
      setProvingKeyReady,
      completeStep,
      goToStep,
      addLog,
      reset,
    },
  };
}

