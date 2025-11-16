/**
 * Error handling example
 * 
 * Shows how to properly handle different error types.
 */

import {
  T2Z,
  ValidationError,
  ProposalError,
  ProvingError,
  SigningError,
  FinalizationError,
  ParseError,
} from '../src';

async function exampleWithErrors() {
  // Example 1: Validation error
  try {
    await T2Z.propose({
      inputs: [
        {
          pubkey: 'invalid', // Too short!
          prevoutTxid: 'b'.repeat(64),
          prevoutIndex: 0,
          value: 100000n,
          scriptPubkey: '76a914' + 'c'.repeat(40) + '88ac',
        },
      ],
      request: {
        payments: [
          {
            address: 'u1' + 'd'.repeat(200),
            amount: 90000n,
          },
        ],
      },
      network: 'testnet',
      expiryHeight: 2500000,
    });
  } catch (error) {
    if (error instanceof ValidationError) {
      console.log('✅ Caught ValidationError:');
      console.log('  Message:', error.message);
      console.log('  Issues:', error.issues);
    }
  }

  // Example 2: Signing error
  const tx = await T2Z.propose({
    inputs: [
      {
        pubkey: '02' + 'a'.repeat(64),
        prevoutTxid: 'b'.repeat(64),
        prevoutIndex: 0,
        value: 100000n,
        scriptPubkey: '76a914' + 'c'.repeat(40) + '88ac',
      },
    ],
    request: {
      payments: [
        {
          address: 'u1' + 'd'.repeat(200),
          amount: 90000n,
        },
      ],
    },
    network: 'testnet',
    expiryHeight: 2500000,
  });

  try {
    await tx.signTransparentInput(0, 'invalid-key'); // Invalid key length
  } catch (error) {
    if (error instanceof ValidationError) {
      console.log('\n✅ Caught ValidationError for invalid key:');
      console.log('  Message:', error.message);
    }
  }

  // Example 3: Parse error
  try {
    await T2Z.fromHex('not-valid-hex-for-transaction');
  } catch (error) {
    if (error instanceof ValidationError) {
      console.log('\n✅ Caught ValidationError for invalid hex:');
      console.log('  Message:', error.message);
    } else if (error instanceof ParseError) {
      console.log('\n✅ Caught ParseError:');
      console.log('  Message:', error.message);
    }
  }

  // Example 4: Proper error handling pattern
  try {
    const result = await T2Z.propose({
      inputs: [
        {
          pubkey: '02' + 'a'.repeat(64),
          prevoutTxid: 'b'.repeat(64),
          prevoutIndex: 0,
          value: 100000n,
          scriptPubkey: '76a914' + 'c'.repeat(40) + '88ac',
        },
      ],
      request: {
        payments: [
          {
            address: 'u1' + 'd'.repeat(200),
            amount: 90000n,
          },
        ],
      },
      network: 'testnet',
      expiryHeight: 2500000,
    });

    await result.prove();
    await result.signTransparentInput(0, 'e'.repeat(64));
    const txBytes = await result.finalize();

    console.log('\n✅ Transaction built successfully!');
    console.log(`  Size: ${txBytes.length} bytes`);
  } catch (error) {
    // Handle specific errors
    if (error instanceof ValidationError) {
      console.error('Validation failed:', error.message, error.issues);
    } else if (error instanceof ProposalError) {
      console.error('Proposal failed:', error.message);
    } else if (error instanceof ProvingError) {
      console.error('Proving failed:', error.message);
    } else if (error instanceof SigningError) {
      console.error('Signing failed:', error.message);
    } else if (error instanceof FinalizationError) {
      console.error('Finalization failed:', error.message);
    } else {
      console.error('Unexpected error:', error);
    }

    // Re-throw if needed
    // throw error;
  }
}

exampleWithErrors().catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});
