/**
 * Basic example of building a Zcash transaction with T2Z
 */

import { T2Z } from '../src';

async function main() {
  console.log('🚀 Building a Zcash transaction with T2Z (Transparent to Zero-knowledge)...\n');

  // Step 1: Propose a transaction
  console.log('Step 1: Proposing transaction...');
  const tx = await T2Z.propose({
    inputs: [
      {
        pubkey: '02' + 'a'.repeat(64), // Example 33-byte pubkey
        prevoutTxid: 'b'.repeat(64), // Example 32-byte txid
        prevoutIndex: 0,
        value: 100000n, // 100,000 zatoshis = 0.001 ZEC
        scriptPubkey: '76a914' + 'c'.repeat(40) + '88ac', // Example P2PKH script
      },
    ],
    request: {
      payments: [
        {
          address: 'u1' + 'd'.repeat(200), // Example unified address
          amount: 90000n, // 90,000 zatoshis
          memo: Buffer.from('Hello Zcash!').toString('hex'),
        },
      ],
      fee: 10000n, // 10,000 zatoshis fee
    },
    network: 'testnet',
    expiryHeight: 2500000,
  });
  console.log('✅ Transaction proposed\n');

  // Step 2: Prove the transaction
  console.log('Step 2: Proving transaction (Halo 2)...');
  console.log('ℹ️  First call builds circuit (~10 seconds), subsequent calls are instant');
  const startTime = Date.now();
  await tx.prove();
  const proveTime = Date.now() - startTime;
  console.log(`✅ Transaction proved in ${proveTime}ms\n');

  // Step 3: Sign transparent inputs
  console.log('Step 3: Signing transparent inputs...');
  const privateKey = 'e'.repeat(64); // Example 32-byte private key
  await tx.signTransparentInput(0, privateKey);
  console.log('✅ Input signed\n');

  // Step 4: Finalize and extract transaction
  console.log('Step 4: Finalizing transaction...');
  const txBytes = await tx.finalize();
  console.log('✅ Transaction finalized\n');

  // Results
  console.log('📊 Results:');
  console.log(`  Transaction size: ${txBytes.length} bytes`);
  console.log(`  Transaction hex: ${Buffer.from(txBytes).toString('hex').slice(0, 100)}...`);
  console.log('\n✨ Ready to broadcast to the Zcash network!');
}

main().catch(error => {
  console.error('❌ Error:', error);
  process.exit(1);
});
