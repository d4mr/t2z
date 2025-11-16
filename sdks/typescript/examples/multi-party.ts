/**
 * Multi-party transaction example
 * 
 * Shows how multiple parties can collaborate to build a single transaction.
 */

import { T2Z } from '../src';

async function main() {
  console.log('🤝 Multi-party transaction example\n');

  // Party 1: Create their part
  console.log('Party 1: Creating their contribution...');
  const tx1 = await T2Z.propose({
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
  console.log('✅ Party 1 contribution ready\n');

  // Party 2: Create their part
  console.log('Party 2: Creating their contribution...');
  const tx2 = await T2Z.propose({
    inputs: [
      {
        pubkey: '03' + 'e'.repeat(64),
        prevoutTxid: 'f'.repeat(64),
        prevoutIndex: 1,
        value: 50000n,
        scriptPubkey: '76a914' + 'a'.repeat(40) + '88ac',
      },
    ],
    request: {
      payments: [
        {
          address: 't1' + 'b'.repeat(30),
          amount: 45000n,
        },
      ],
    },
    network: 'testnet',
    expiryHeight: 2500000,
  });
  console.log('✅ Party 2 contribution ready\n');

  // Combine contributions
  console.log('Combining contributions...');
  const combined = await T2Z.combine([tx1, tx2]);
  console.log('✅ Transactions combined\n');

  // Prove
  console.log('Proving combined transaction...');
  await combined.prove();
  console.log('✅ Combined transaction proved\n');

  // Each party signs their own inputs
  console.log('Party 1: Signing their input...');
  await combined.signTransparentInput(0, 'e'.repeat(64));
  console.log('✅ Party 1 signed\n');

  console.log('Party 2: Signing their input...');
  await combined.signTransparentInput(1, 'f'.repeat(64));
  console.log('✅ Party 2 signed\n');

  // Finalize
  console.log('Finalizing...');
  const txBytes = await combined.finalize();
  console.log('✅ Transaction finalized\n');

  console.log('📊 Results:');
  console.log(`  Transaction size: ${txBytes.length} bytes`);
  console.log('\n✨ Multi-party transaction ready to broadcast!');
}

main().catch(error => {
  console.error('❌ Error:', error);
  process.exit(1);
});
