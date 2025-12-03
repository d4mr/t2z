//! Shadow structs that match the PCZT layout exactly for serde round-tripping.
//!
//! These allow us to deserialize, modify, and re-serialize PCZTs without
//! needing direct access to pczt crate internals.
//!
//! IMPORTANT: These structs MUST match the pczt crate's serde layout EXACTLY,
//! including field order, types, and serde_as annotations.

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;

/// Top-level PCZT structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PcztShadow {
    pub global: GlobalShadow,
    pub transparent: TransparentBundleShadow,
    pub sapling: SaplingBundleShadow,
    pub orchard: OrchardBundleShadow,
}

// ============================================================================
// Global
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalShadow {
    pub tx_version: u32,
    pub version_group_id: u32,
    pub consensus_branch_id: u32,
    pub fallback_lock_time: Option<u32>,
    pub expiry_height: u32,
    pub coin_type: u32,
    pub tx_modifiable: u8,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

// ============================================================================
// Common types
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zip32DerivationShadow {
    pub seed_fingerprint: [u8; 32],
    pub derivation_path: Vec<u32>,
}

// ============================================================================
// Transparent
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransparentBundleShadow {
    pub inputs: Vec<TransparentInputShadow>,
    pub outputs: Vec<TransparentOutputShadow>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransparentInputShadow {
    pub prevout_txid: [u8; 32],
    pub prevout_index: u32,
    pub sequence: Option<u32>,
    pub required_time_lock_time: Option<u32>,
    pub required_height_lock_time: Option<u32>,
    pub script_sig: Option<Vec<u8>>,
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub redeem_script: Option<Vec<u8>>,
    // serde_as for [u8; 33] keys - matches pczt crate exactly
    #[serde_as(as = "BTreeMap<[_; 33], _>")]
    pub partial_signatures: BTreeMap<[u8; 33], Vec<u8>>,
    pub sighash_type: u8,
    #[serde_as(as = "BTreeMap<[_; 33], _>")]
    pub bip32_derivation: BTreeMap<[u8; 33], Zip32DerivationShadow>,
    // NO serde_as for these - [u8; 20] and [u8; 32] work natively with serde
    pub ripemd160_preimages: BTreeMap<[u8; 20], Vec<u8>>,
    pub sha256_preimages: BTreeMap<[u8; 32], Vec<u8>>,
    pub hash160_preimages: BTreeMap<[u8; 20], Vec<u8>>,
    pub hash256_preimages: BTreeMap<[u8; 32], Vec<u8>>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransparentOutputShadow {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub redeem_script: Option<Vec<u8>>,
    #[serde_as(as = "BTreeMap<[_; 33], _>")]
    pub bip32_derivation: BTreeMap<[u8; 33], Zip32DerivationShadow>,
    pub user_address: Option<String>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

// ============================================================================
// Sapling
// ============================================================================

const GROTH_PROOF_SIZE: usize = 48 + 96 + 48; // 192

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaplingBundleShadow {
    pub spends: Vec<SaplingSpendShadow>,
    pub outputs: Vec<SaplingOutputShadow>,
    pub value_sum: i128,
    // NOT optional in pczt crate
    pub anchor: [u8; 32],
    pub bsk: Option<[u8; 32]>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaplingSpendShadow {
    pub cv: [u8; 32],
    pub nullifier: [u8; 32],
    pub rk: [u8; 32],
    #[serde_as(as = "Option<[_; GROTH_PROOF_SIZE]>")]
    pub zkproof: Option<[u8; GROTH_PROOF_SIZE]>,
    #[serde_as(as = "Option<[_; 64]>")]
    pub spend_auth_sig: Option<[u8; 64]>,
    #[serde_as(as = "Option<[_; 43]>")]
    pub recipient: Option<[u8; 43]>,
    pub value: Option<u64>,
    // rcm field - note commitment randomness (before ZIP 212)
    pub rcm: Option<[u8; 32]>,
    // rseed field - seed randomness (after ZIP 212)
    pub rseed: Option<[u8; 32]>,
    pub rcv: Option<[u8; 32]>,
    // proof_generation_key is a TUPLE of two 32-byte arrays, not a single 64-byte array
    pub proof_generation_key: Option<([u8; 32], [u8; 32])>,
    // witness is a TUPLE (tree_size, fixed-size path), not a custom struct
    #[serde_as(as = "Option<(_, [[_; 32]; 32])>")]
    pub witness: Option<(u32, [[u8; 32]; 32])>,
    pub alpha: Option<[u8; 32]>,
    // zip32_derivation is Option, not a BTreeMap
    pub zip32_derivation: Option<Zip32DerivationShadow>,
    pub dummy_ask: Option<[u8; 32]>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaplingOutputShadow {
    pub cv: [u8; 32],
    pub cmu: [u8; 32],
    pub ephemeral_key: [u8; 32],
    // enc_ciphertext and out_ciphertext are Vec<u8> in pczt crate, not fixed arrays
    pub enc_ciphertext: Vec<u8>,
    pub out_ciphertext: Vec<u8>,
    #[serde_as(as = "Option<[_; GROTH_PROOF_SIZE]>")]
    pub zkproof: Option<[u8; GROTH_PROOF_SIZE]>,
    #[serde_as(as = "Option<[_; 43]>")]
    pub recipient: Option<[u8; 43]>,
    pub value: Option<u64>,
    pub rseed: Option<SaplingRseedShadow>,
    pub rcv: Option<[u8; 32]>,
    pub ock: Option<[u8; 32]>,
    // zip32_derivation is Option, not a BTreeMap
    pub zip32_derivation: Option<Zip32DerivationShadow>,
    pub user_address: Option<String>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SaplingRseedShadow {
    BeforeZip212([u8; 32]),
    AfterZip212([u8; 32]),
}

// ============================================================================
// Orchard
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchardBundleShadow {
    pub actions: Vec<OrchardActionShadow>,
    pub flags: u8,
    // value_sum is (u64, bool) in pczt crate, not i128
    pub value_sum: (u64, bool),
    // NOT optional in pczt crate
    pub anchor: [u8; 32],
    pub zkproof: Option<Vec<u8>>,
    pub bsk: Option<[u8; 32]>,
}

/// Orchard Action - contains cv_net, spend, output, and rcv
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchardActionShadow {
    pub cv_net: [u8; 32],
    pub spend: OrchardSpendShadow,
    pub output: OrchardOutputShadow,
    pub rcv: Option<[u8; 32]>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchardSpendShadow {
    pub nullifier: [u8; 32],
    pub rk: [u8; 32],
    #[serde_as(as = "Option<[_; 64]>")]
    pub spend_auth_sig: Option<[u8; 64]>,
    #[serde_as(as = "Option<[_; 43]>")]
    pub recipient: Option<[u8; 43]>,
    pub value: Option<u64>,
    pub rho: Option<[u8; 32]>,
    pub rseed: Option<[u8; 32]>,
    #[serde_as(as = "Option<[_; 96]>")]
    pub fvk: Option<[u8; 96]>,
    // witness is a TUPLE (tree_size, fixed-size path), not a custom struct
    #[serde_as(as = "Option<(_, [[_; 32]; 32])>")]
    pub witness: Option<(u32, [[u8; 32]; 32])>,
    pub alpha: Option<[u8; 32]>,
    // zip32_derivation is Option, not a BTreeMap
    pub zip32_derivation: Option<Zip32DerivationShadow>,
    pub dummy_sk: Option<[u8; 32]>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchardOutputShadow {
    pub cmx: [u8; 32],
    pub ephemeral_key: [u8; 32],
    // enc_ciphertext and out_ciphertext are Vec<u8> in pczt crate, not fixed arrays
    pub enc_ciphertext: Vec<u8>,
    pub out_ciphertext: Vec<u8>,
    #[serde_as(as = "Option<[_; 43]>")]
    pub recipient: Option<[u8; 43]>,
    pub value: Option<u64>,
    pub rseed: Option<[u8; 32]>,
    pub ock: Option<[u8; 32]>,
    // zip32_derivation is Option, not a BTreeMap
    pub zip32_derivation: Option<Zip32DerivationShadow>,
    pub user_address: Option<String>,
    pub proprietary: BTreeMap<String, Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_struct_size() {
        // Basic sanity checks
        assert_eq!(GROTH_PROOF_SIZE, 192);
    }
}

