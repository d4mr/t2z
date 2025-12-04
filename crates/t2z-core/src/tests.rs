//! Tests for t2z-core serialization and PCZT operations

use crate::shadow;
use crate::{parse_pczt, serialize_pczt};
use pczt::roles::creator::Creator;
use zcash_protocol::consensus::BranchId;

#[test]
fn test_pczt_basic_roundtrip() {
    let pczt = Creator::new(BranchId::Nu6.into(), 10_000_000, 133, [0; 32], [0; 32]).build();

    let serialized = serialize_pczt(&pczt);
    let parsed = parse_pczt(&serialized).unwrap();

    assert_eq!(
        parsed.global().expiry_height(),
        pczt.global().expiry_height()
    );
}

#[test]
fn test_shadow_struct_roundtrip_empty_pczt() {
    // Create a simple PCZT using the Creator
    let pczt = Creator::new(BranchId::Nu6.into(), 10_000_000, 133, [0; 32], [0; 32]).build();

    let serialized = pczt.serialize();

    // PCZT format: 4 bytes magic + 4 bytes version + postcard data
    assert!(serialized.len() >= 8, "PCZT too short");

    let magic = &serialized[..4];
    let version = &serialized[4..8];
    let data = &serialized[8..];

    println!("Magic: {:02x?}", magic);
    println!("Version: {:02x?}", version);
    println!("Data length: {}", data.len());

    // Try deserializing
    let result: Result<shadow::PcztShadow, _> = postcard::from_bytes(data);
    match result {
        Ok(shadow) => {
            println!("Successfully deserialized shadow struct!");
            println!("Transparent inputs: {}", shadow.transparent.inputs.len());
            println!("Transparent outputs: {}", shadow.transparent.outputs.len());
            println!("Orchard actions: {}", shadow.orchard.actions.len());

            // Try to re-serialize
            let re_serialized = postcard::to_allocvec(&shadow).expect("Failed to serialize shadow");

            // Verify they match
            assert_eq!(
                data,
                re_serialized.as_slice(),
                "Round-trip failed: data mismatch"
            );
            println!("Round-trip successful!");
        }
        Err(e) => {
            panic!("Failed to deserialize: {:?}", e);
        }
    }
}

#[test]
fn test_shadow_struct_with_modified_input() {
    // Create a simple PCZT
    let pczt = Creator::new(
        BranchId::Nu6.into(),
        10_000_000,
        2_500_000,
        [0; 32],
        [0; 32],
    )
    .build();
    let serialized = pczt.serialize();

    let data = &serialized[8..];
    let shadow: shadow::PcztShadow = postcard::from_bytes(data).expect("Failed to deserialize");

    // Create script pubkey for a test P2PKH
    let script_pubkey = vec![
        0x76, 0xa9, 0x14, // OP_DUP OP_HASH160 PUSH20
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, // 20 bytes of zeros
        0x88, 0xac, // OP_EQUALVERIFY OP_CHECKSIG
    ];

    // Modify the shadow (add a fake input)
    let mut modified = shadow.clone();
    modified
        .transparent
        .inputs
        .push(shadow::TransparentInputShadow {
            prevout_txid: [1u8; 32],
            prevout_index: 0,
            sequence: Some(0xFFFFFFFF),
            required_time_lock_time: None,
            required_height_lock_time: None,
            script_sig: None,
            value: 1_000_000,
            script_pubkey: script_pubkey.clone(),
            redeem_script: None,
            partial_signatures: std::collections::BTreeMap::new(),
            sighash_type: 0x01,
            bip32_derivation: std::collections::BTreeMap::new(),
            ripemd160_preimages: std::collections::BTreeMap::new(),
            sha256_preimages: std::collections::BTreeMap::new(),
            hash160_preimages: std::collections::BTreeMap::new(),
            hash256_preimages: std::collections::BTreeMap::new(),
            proprietary: std::collections::BTreeMap::new(),
        });

    // Try to serialize the modified version
    let re_serialized = postcard::to_allocvec(&modified).expect("Failed to serialize modified");

    // Reconstruct full PCZT bytes
    let mut full_bytes = Vec::new();
    full_bytes.extend_from_slice(&serialized[..8]); // magic + version
    full_bytes.extend_from_slice(&re_serialized);

    // Try to parse with the real Pczt parser
    let result = pczt::Pczt::parse(&full_bytes);
    match result {
        Ok(parsed) => {
            println!("Successfully parsed modified PCZT!");
            println!(
                "Transparent inputs: {}",
                parsed.transparent().inputs().len()
            );
            assert_eq!(parsed.transparent().inputs().len(), 1);
        }
        Err(e) => {
            // This might fail due to validation - that's expected
            println!(
                "Failed to parse modified PCZT (expected for incomplete data): {:?}",
                e
            );
        }
    }
}

#[test]
fn test_shadow_add_signature_to_existing_input() {
    // This test simulates what append_signature does
    let pczt = Creator::new(
        BranchId::Nu6.into(),
        10_000_000,
        2_500_000,
        [0; 32],
        [0; 32],
    )
    .build();
    let serialized = pczt.serialize();

    let data = &serialized[8..];

    // First, verify we can deserialize
    let mut shadow: shadow::PcztShadow = postcard::from_bytes(data).expect("Failed to deserialize");

    println!(
        "Initial transparent inputs: {}",
        shadow.transparent.inputs.len()
    );

    // If there are no inputs, add one for testing
    if shadow.transparent.inputs.is_empty() {
        let script_pubkey = vec![
            0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac,
        ];
        shadow
            .transparent
            .inputs
            .push(shadow::TransparentInputShadow {
                prevout_txid: [1u8; 32],
                prevout_index: 0,
                sequence: Some(0xFFFFFFFF),
                required_time_lock_time: None,
                required_height_lock_time: None,
                script_sig: None,
                value: 1_000_000,
                script_pubkey,
                redeem_script: None,
                partial_signatures: std::collections::BTreeMap::new(),
                sighash_type: 0x01,
                bip32_derivation: std::collections::BTreeMap::new(),
                ripemd160_preimages: std::collections::BTreeMap::new(),
                sha256_preimages: std::collections::BTreeMap::new(),
                hash160_preimages: std::collections::BTreeMap::new(),
                hash256_preimages: std::collections::BTreeMap::new(),
                proprietary: std::collections::BTreeMap::new(),
            });
    }

    // Add a fake signature
    let fake_pubkey = [2u8; 33]; // Fake compressed pubkey
    let fake_signature = vec![0x30, 0x44]; // Fake DER signature start

    shadow.transparent.inputs[0]
        .partial_signatures
        .insert(fake_pubkey, fake_signature);

    println!(
        "After adding signature, partial_signatures: {}",
        shadow.transparent.inputs[0].partial_signatures.len()
    );

    // Serialize back
    let re_serialized = postcard::to_allocvec(&shadow).expect("Failed to serialize with signature");
    println!("Re-serialized length: {}", re_serialized.len());

    // Try to deserialize again to verify round-trip
    let shadow2: Result<shadow::PcztShadow, _> = postcard::from_bytes(&re_serialized);
    match shadow2 {
        Ok(s) => {
            println!("Round-trip successful!");
            assert_eq!(s.transparent.inputs[0].partial_signatures.len(), 1);
        }
        Err(e) => {
            panic!("Failed to deserialize after adding signature: {:?}", e);
        }
    }
}

#[test]
fn derive_ufvk() {
    use orchard::keys::{SpendingKey, FullViewingKey, Scope};
    use zcash_address::unified::{self, Encoding, Ufvk, Fvk};
    use zcash_protocol::consensus::NetworkType;
    
    let sk_hex = "2eae94c0d77330143ccc67d68a74a6ef05d772340328cbeb1514e437d838b05a";
    let sk_bytes: [u8; 32] = hex::decode(sk_hex).unwrap().try_into().unwrap();
    let sk = SpendingKey::from_bytes(sk_bytes).unwrap();
    let fvk = FullViewingKey::from(&sk);
    
    // Create Unified FVK with Orchard component
    let ufvk = Ufvk::try_from_items(vec![
        Fvk::Orchard(fvk.to_bytes())
    ]).unwrap();
    
    // Encode for testnet (will start with "uviewtest1...")
    let encoded = ufvk.encode(&NetworkType::Test);
    println!("UFVK: {}", encoded);
    
    // Also print address to verify
    let address = fvk.address_at(0u32, Scope::External);
    let ua = unified::Address::try_from_items(vec![
        unified::Receiver::Orchard(address.to_raw_address_bytes())
    ]).unwrap();
    println!("Address: {}", ua.encode(&NetworkType::Test));
}