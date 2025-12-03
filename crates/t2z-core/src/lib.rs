//! T2Z Core - Transparent to Zero-knowledge Zcash Transactions
//!
//! Core library for building Zcash transactions that send from transparent
//! inputs to shielded (Orchard) outputs. Implements ZIP 244, ZIP 321, and ZIP 374.
//!
//! This crate provides the core functionality used by platform-specific bindings:
//! - `t2z-wasm` for browser/Node.js via WebAssembly
//! - `t2z-uniffi` for Go, Kotlin, and Java via UniFFI

use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use zcash_primitives::{
    consensus::BlockHeight,
    transaction::{
        builder::{BuildConfig, Builder},
        fees::zip317::{
            FeeRule, GRACE_ACTIONS, MARGINAL_FEE, P2PKH_STANDARD_INPUT_SIZE,
            P2PKH_STANDARD_OUTPUT_SIZE,
        },
    },
};
use zcash_protocol::{
    consensus::{MainNetwork, NetworkType, TestNetwork},
    value::Zatoshis,
};

// Re-export pczt types and roles for consumers
pub use pczt::roles::{
    combiner::{Combiner, Error as CombinerError},
    creator::Creator,
    io_finalizer::{Error as IoFinalizerError, IoFinalizer},
    prover::Prover,
    signer::{Error as SignerError, Signer},
    spend_finalizer::{Error as SpendFinalizerError, SpendFinalizer},
    tx_extractor::{Error as TxExtractorError, TransactionExtractor},
};
pub use pczt::{ParseError, Pczt};

// Re-export orchard proving key for WASM crate
pub use orchard::circuit::ProvingKey as OrchardProvingKey;

// ============================================================================
// Core Types (ZIP 244 and ZIP 321 compliant)
// ============================================================================

/// Transparent input with all data required for ZIP 244 signature validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentInput {
    /// Compressed public key (33 bytes)
    pub pubkey: Vec<u8>,
    /// Previous transaction ID (32 bytes)
    pub prevout_txid: Vec<u8>,
    /// Previous output index
    pub prevout_index: u32,
    /// Output value in zatoshis (required for sighash per ZIP 244)
    pub value: u64,
    /// scriptPubKey of the output being spent (required for sighash per ZIP 244)
    pub script_pubkey: Vec<u8>,
    /// nSequence value (optional, defaults to 0xFFFFFFFF)
    pub sequence: Option<u32>,
}

/// Single payment following ZIP 321 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Address string (unified address with Orchard, or transparent P2PKH/P2SH)
    pub address: String,
    /// Amount in zatoshis
    pub amount: u64,
    /// Memo bytes (already decoded, max 512 bytes)
    #[serde(with = "serde_bytes")]
    pub memo: Option<Vec<u8>>,
    /// Optional label for payment
    pub label: Option<String>,
}

/// Transaction request following ZIP 321 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// List of payments (supports multiple recipients via ZIP 321 paramindex)
    pub payments: Vec<Payment>,
    /// Fee in zatoshis (if None, will be calculated using FeeRule::standard())
    pub fee: Option<u64>,
    /// Change address for any leftover funds (transparent address)
    /// If not provided and there's change, the transaction will fail
    pub change_address: Option<String>,
}

/// Network selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn to_network_type(self) -> NetworkType {
        match self {
            Network::Mainnet => NetworkType::Main,
            Network::Testnet => NetworkType::Test,
        }
    }
}

// Note: We use MainNetwork and TestNetwork from zcash_protocol::consensus
// which properly implement the Parameters trait with correct activation heights

// ============================================================================
// ZIP-317 Fee Calculation
// ============================================================================

/// Calculates the ZIP-317 fee for a transaction.
///
/// # Arguments
/// * `num_transparent_inputs` - Number of P2PKH transparent inputs
/// * `num_transparent_outputs` - Number of P2PKH transparent outputs
/// * `num_orchard_actions` - Number of Orchard actions (minimum 2 if any Orchard outputs)
///
/// # Returns
/// The fee in zatoshis
pub fn calculate_zip317_fee(
    num_transparent_inputs: usize,
    num_transparent_outputs: usize,
    num_orchard_actions: usize,
) -> u64 {
    // ZIP-317 fee formula:
    // fee = marginal_fee * max(grace_actions, ceil((t_in_size + t_out_size) / 1024), orchard_actions, sapling_actions)
    
    let t_in_size = num_transparent_inputs * P2PKH_STANDARD_INPUT_SIZE;
    let t_out_size = num_transparent_outputs * P2PKH_STANDARD_OUTPUT_SIZE;
    let total_transparent_size = t_in_size + t_out_size;
    
    // ceil(total_transparent_size / 1024)
    let transparent_logical_actions = (total_transparent_size + 1023) / 1024;
    
    let logical_actions = std::cmp::max(
        GRACE_ACTIONS,
        std::cmp::max(transparent_logical_actions, num_orchard_actions),
    );
    
    // MARGINAL_FEE is Zatoshis(5000)
    MARGINAL_FEE.into_u64() * logical_actions as u64
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum T2ZError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid memo: {0}")]
    InvalidMemo(String),
    
    #[error("Insufficient funds: available {available}, required {required} (payment: {payment}, fee: {fee})")]
    InsufficientFunds {
        available: u64,
        required: u64,
        payment: u64,
        fee: u64,
    },
    
    #[error("Change required: {change} zatoshis left over but no change_address provided")]
    ChangeRequired { change: u64 },

    #[error("Parse error: {0:?}")]
    Parse(ParseError),

    #[error("IO Finalizer error: {0:?}")]
    IoFinalizer(IoFinalizerError),

    #[error("Signer error: {0:?}")]
    Signer(SignerError),

    #[error("Transaction Extractor error: {0:?}")]
    TxExtractor(TxExtractorError),

    #[error("Combiner error: {0:?}")]
    Combiner(CombinerError),

    #[error("Spend Finalizer error: {0:?}")]
    SpendFinalizer(SpendFinalizerError),

    #[error("Builder error: {0}")]
    Builder(String),

    #[error("Proving error: {0}")]
    Proving(String),
}

impl From<ParseError> for T2ZError {
    fn from(e: ParseError) -> Self {
        T2ZError::Parse(e)
    }
}

impl From<IoFinalizerError> for T2ZError {
    fn from(e: IoFinalizerError) -> Self {
        T2ZError::IoFinalizer(e)
    }
}

impl From<SignerError> for T2ZError {
    fn from(e: SignerError) -> Self {
        T2ZError::Signer(e)
    }
}

impl From<TxExtractorError> for T2ZError {
    fn from(e: TxExtractorError) -> Self {
        T2ZError::TxExtractor(e)
    }
}

impl From<CombinerError> for T2ZError {
    fn from(e: CombinerError) -> Self {
        T2ZError::Combiner(e)
    }
}

impl From<SpendFinalizerError> for T2ZError {
    fn from(e: SpendFinalizerError) -> Self {
        T2ZError::SpendFinalizer(e)
    }
}

// ============================================================================
// Orchard Proving Key Management (Halo 2 - No Trusted Setup!)
// ============================================================================

/// Orchard proving key cache
///
/// Unlike Sapling/Sprout which require downloading large proving keys from a trusted setup,
/// Orchard uses Halo 2 which requires NO external parameters or trusted setup.
/// The proving key is built programmatically from circuit constraints.
static ORCHARD_PK: once_cell::sync::OnceCell<OrchardProvingKey> = once_cell::sync::OnceCell::new();

/// Builds the Orchard circuit proving key (synchronous, for native targets)
///
/// # Important: No Download Required!
/// Orchard uses Halo 2, which eliminates the need for trusted setups and downloadable
/// proving keys. Unlike Sapling (which requires ~50MB params files) or Sprout (869MB),
/// Orchard builds its proving key programmatically from circuit constraints.
///
/// # Returns
/// Reference to the cached proving key
///
/// # Performance
/// - First call: ~10 seconds to build circuit (one-time cost)
/// - Subsequent calls: Instant (cached in memory)
pub fn load_orchard_proving_key() -> &'static OrchardProvingKey {
    ORCHARD_PK.get_or_init(OrchardProvingKey::build)
}

/// Get the cached proving key if already loaded
pub fn get_cached_proving_key() -> Option<&'static OrchardProvingKey> {
    ORCHARD_PK.get()
}

/// Check if the proving key is already loaded
pub fn is_proving_key_loaded() -> bool {
    ORCHARD_PK.get().is_some()
}

// ============================================================================
// Address Parsing Helpers
// ============================================================================

/// Parses a transparent address from a ZcashAddress
fn parse_transparent_address(
    addr: &zcash_address::ZcashAddress,
    expected_network: NetworkType,
) -> Result<zcash_transparent::address::TransparentAddress, T2ZError> {
    use zcash_address::{ConversionError, TryFromAddress};

    struct TransparentReceiver(zcash_transparent::address::TransparentAddress);

    impl TryFromAddress for TransparentReceiver {
        type Error = String;

        fn try_from_transparent_p2pkh(
            _net: NetworkType,
            data: [u8; 20],
        ) -> Result<Self, ConversionError<Self::Error>> {
            Ok(TransparentReceiver(
                zcash_transparent::address::TransparentAddress::PublicKeyHash(data),
            ))
        }

        fn try_from_transparent_p2sh(
            _net: NetworkType,
            data: [u8; 20],
        ) -> Result<Self, ConversionError<Self::Error>> {
            Ok(TransparentReceiver(
                zcash_transparent::address::TransparentAddress::ScriptHash(data),
            ))
        }
    }

    addr.clone()
        .convert_if_network::<TransparentReceiver>(expected_network)
        .map(|r| r.0)
        .map_err(|e| T2ZError::InvalidAddress(format!("Not a transparent address: {:?}", e)))
}

/// Parses an Orchard receiver from a ZcashAddress
fn parse_orchard_receiver(
    addr: &zcash_address::ZcashAddress,
    expected_network: NetworkType,
) -> Result<orchard::Address, T2ZError> {
    use zcash_address::{
        ConversionError, TryFromAddress,
        unified::{Container, Receiver},
    };

    struct OrchardReceiver(orchard::Address);

    impl TryFromAddress for OrchardReceiver {
        type Error = String;

        fn try_from_unified(
            _net: NetworkType,
            unified_addr: zcash_address::unified::Address,
        ) -> Result<Self, ConversionError<Self::Error>> {
            // Iterate through receivers to find Orchard
            for receiver in unified_addr.items_as_parsed() {
                if let Receiver::Orchard(data) = receiver {
                    // Parse the Orchard address from the 43-byte data
                    let orchard_addr = orchard::Address::from_raw_address_bytes(data);
                    if orchard_addr.is_some().into() {
                        return Ok(OrchardReceiver(orchard_addr.unwrap()));
                    } else {
                        return Err(ConversionError::User(
                            "Invalid Orchard receiver data".to_string(),
                        ));
                    }
                }
            }
            Err(ConversionError::User(
                "Unified address has no Orchard receiver".to_string(),
            ))
        }
    }

    addr.clone()
        .convert_if_network::<OrchardReceiver>(expected_network)
        .map(|r| r.0)
        .map_err(|e| T2ZError::InvalidAddress(format!("Not an Orchard address: {:?}", e)))
}

// ============================================================================
// Core API Implementation
// ============================================================================

/// Proposes a transaction from transparent inputs to transparent and/or shielded outputs.
///
/// Implements Creator, Constructor, and IO Finalizer roles per ZIP 374.
/// Uses zcash_primitives::Builder per ZIP 244 requirements.
///
/// # Arguments
/// * `transparent_inputs` - Transparent UTXOs to spend (must include pubkey, value, scriptPubKey per ZIP 244)
/// * `request` - Payment request following ZIP 321 specification
/// * `network` - Network selection (Mainnet or Testnet)
/// * `expiry_height` - Block height at which transaction expires
///
/// # Returns
/// A PCZT with IO finalized, ready for proving and signing
///
/// # Change Handling
/// If the sum of inputs exceeds the sum of outputs plus fee, change is required.
/// You MUST provide a `change_address` in the `TransactionRequest` to receive the change.
/// If `change_address` is not provided and there's excess value, an error is returned.
///
/// # Fee Calculation
/// Uses ZIP-317 fee rules. If `fee` is not specified in the request, it will be
/// calculated automatically based on the transaction structure.
pub fn propose_transaction(
    transparent_inputs: &[TransparentInput],
    request: TransactionRequest,
    network: Network,
    expiry_height: u32,
) -> Result<Pczt, T2ZError> {
    if transparent_inputs.is_empty() {
        return Err(T2ZError::InvalidInput(
            "No transparent inputs provided".to_string(),
        ));
    }

    if request.payments.is_empty() {
        return Err(T2ZError::InvalidInput("No payments specified".to_string()));
    }

    // Validate all inputs have correct sizes
    for (idx, input) in transparent_inputs.iter().enumerate() {
        if input.pubkey.len() != 33 {
            return Err(T2ZError::InvalidInput(format!(
                "Input {} pubkey must be 33 bytes (got {})",
                idx,
                input.pubkey.len()
            )));
        }
        if input.prevout_txid.len() != 32 {
            return Err(T2ZError::InvalidInput(format!(
                "Input {} prevout_txid must be 32 bytes (got {})",
                idx,
                input.prevout_txid.len()
            )));
        }
    }

    // Validate memo sizes (ZIP 321: max 512 bytes)
    for (idx, payment) in request.payments.iter().enumerate() {
        if let Some(memo) = &payment.memo
            && memo.len() > 512
        {
            return Err(T2ZError::InvalidMemo(format!(
                "Payment {} memo exceeds 512 bytes ({} bytes)",
                idx,
                memo.len()
            )));
        }
    }

    let expected_network = network.to_network_type();

    // Count output types and check if we have Orchard
    let mut num_transparent_outputs = 0usize;
    let mut num_orchard_outputs = 0usize;
    
    for payment in &request.payments {
        let addr = zcash_address::ZcashAddress::try_from_encoded(&payment.address)
            .map_err(|e| T2ZError::InvalidAddress(format!("Invalid address: {:?}", e)))?;
        
        if addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            num_transparent_outputs += 1;
        } else if addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
            num_orchard_outputs += 1;
        } else {
            return Err(T2ZError::InvalidAddress(format!(
                "Address {} cannot receive transparent or Orchard funds",
                payment.address
            )));
        }
    }
    
    let has_orchard = num_orchard_outputs > 0;
    
    // For Orchard, we need at least 2 actions (dummy spend + outputs)
    // Each output requires an action, and we need at least one dummy spend
    let num_orchard_actions = if has_orchard {
        std::cmp::max(2, num_orchard_outputs)
    } else {
        0
    };

    // Calculate totals
    let total_input: u64 = transparent_inputs.iter().map(|i| i.value).sum();
    let total_payment: u64 = request.payments.iter().map(|p| p.amount).sum();
    
    // Check if we need a change output (we'll determine this after fee calculation)
    let has_change_address = request.change_address.is_some();
    
    // Calculate fee assuming we might have a change output
    // (This is conservative and ensures we have enough for fees)
    let potential_t_outputs = num_transparent_outputs + if has_change_address { 1 } else { 0 };
    let fee = request.fee.unwrap_or_else(|| {
        calculate_zip317_fee(
            transparent_inputs.len(),
            potential_t_outputs,
            num_orchard_actions,
        )
    });
    
    // Check we have enough funds
    let required = total_payment + fee;
    if total_input < required {
        return Err(T2ZError::InsufficientFunds {
            available: total_input,
            required,
            payment: total_payment,
            fee,
        });
    }
    
    // Calculate change
    let change = total_input - total_payment - fee;
    
    // If there's change, we MUST have a change address
    if change > 0 && !has_change_address {
        return Err(T2ZError::ChangeRequired { change });
    }
    
    // Parse and validate change address if we have change
    let change_t_addr = if change > 0 {
        let change_addr_str = request.change_address.as_ref().unwrap();
        let change_addr = zcash_address::ZcashAddress::try_from_encoded(change_addr_str)
            .map_err(|e| T2ZError::InvalidAddress(format!("Invalid change address: {:?}", e)))?;
        
        if !change_addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            return Err(T2ZError::InvalidAddress(
                "Change address must be a transparent address (P2PKH)".to_string(),
            ));
        }
        
        Some(parse_transparent_address(&change_addr, expected_network)?)
    } else {
        None
    };

    let orchard_anchor = if has_orchard {
        Some(orchard::Anchor::empty_tree())
    } else {
        None
    };

    // Create builder with proper network parameters
    // We need to handle this with a macro/match since Builder is generic over Parameters
    macro_rules! build_transaction {
        ($params:expr) => {{
            let mut builder = Builder::new(
                $params,
                BlockHeight::from_u32(expiry_height),
                BuildConfig::Standard {
                    sapling_anchor: None,
                    orchard_anchor,
                },
            );

            // Add transparent inputs
            for input in transparent_inputs {
                let pubkey_bytes: [u8; 33] = input.pubkey.as_slice().try_into().map_err(|_| {
                    T2ZError::InvalidInput("Public key must be 33 bytes".to_string())
                })?;

                let pubkey = secp256k1::PublicKey::from_slice(&pubkey_bytes)
                    .map_err(|e| T2ZError::InvalidInput(format!("Invalid public key: {}", e)))?;

                let txid_bytes: [u8; 32] =
                    input.prevout_txid.as_slice().try_into().map_err(|_| {
                        T2ZError::InvalidInput("Transaction ID must be 32 bytes".to_string())
                    })?;

                let outpoint =
                    zcash_transparent::bundle::OutPoint::new(txid_bytes, input.prevout_index);

                let script = zcash_script::script::Code(input.script_pubkey.clone());
                let txout = zcash_transparent::bundle::TxOut::new(
                    Zatoshis::from_u64(input.value)
                        .map_err(|e| T2ZError::InvalidInput(format!("Invalid value: {:?}", e)))?,
                    zcash_transparent::address::Script(script),
                );

                builder
                    .add_transparent_input(pubkey, outpoint, txout)
                    .map_err(|e| {
                        T2ZError::Builder(format!("Failed to add transparent input: {:?}", e))
                    })?;
            }

            // Add payment outputs
            for payment in &request.payments {
                let addr = zcash_address::ZcashAddress::try_from_encoded(&payment.address)
                    .map_err(|e| T2ZError::InvalidAddress(format!("Invalid address: {:?}", e)))?;

                if addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
                    let t_addr = parse_transparent_address(&addr, expected_network)?;
                    builder
                        .add_transparent_output(
                            &t_addr,
                            Zatoshis::from_u64(payment.amount).map_err(|e| {
                                T2ZError::InvalidInput(format!("Invalid amount: {:?}", e))
                            })?,
                        )
                        .map_err(|e| {
                            T2ZError::Builder(format!("Failed to add transparent output: {:?}", e))
                        })?;
                } else if addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
                    let orchard_receiver = parse_orchard_receiver(&addr, expected_network)?;

                    let memo_bytes = if let Some(memo) = &payment.memo {
                        let mut padded = [0u8; 512];
                        padded[..memo.len()].copy_from_slice(memo);
                        zcash_protocol::memo::MemoBytes::from_bytes(&padded)
                            .map_err(|e| T2ZError::InvalidMemo(format!("Invalid memo: {:?}", e)))?
                    } else {
                        zcash_protocol::memo::MemoBytes::empty()
                    };

                    builder
                        .add_orchard_output::<FeeRule>(
                            None,
                            orchard_receiver,
                            payment.amount,
                            memo_bytes,
                        )
                        .map_err(|e| {
                            T2ZError::Builder(format!("Failed to add Orchard output: {:?}", e))
                        })?;
                }
            }
            
            // Add change output if needed
            if let Some(t_addr) = &change_t_addr {
                builder
                    .add_transparent_output(
                        t_addr,
                        Zatoshis::from_u64(change).map_err(|e| {
                            T2ZError::InvalidInput(format!("Invalid change amount: {:?}", e))
                        })?,
                    )
                    .map_err(|e| {
                        T2ZError::Builder(format!("Failed to add change output: {:?}", e))
                    })?;
            }

            // Build PCZT
            let result = builder
                .build_for_pczt(OsRng, &FeeRule::standard())
                .map_err(|e| T2ZError::Builder(format!("Failed to build PCZT: {:?}", e)))?;

            let pczt = Creator::build_from_parts(result.pczt_parts)
                .ok_or_else(|| T2ZError::Builder("Failed to create PCZT from parts".to_string()))?;

            IoFinalizer::new(pczt).finalize_io()
        }};
    }

    let pczt = match network {
        Network::Mainnet => build_transaction!(MainNetwork),
        Network::Testnet => build_transaction!(TestNetwork),
    }?;

    Ok(pczt)
}

/// Adds Orchard proofs to the PCZT using the Prover role.
///
/// This uses the cached proving key if available, otherwise builds it first.
///
/// # Performance
/// - First call: ~10 seconds (builds Halo 2 circuit, no download required)
/// - Subsequent calls: Fast (uses cached circuit)
pub fn prove_transaction(pczt: Pczt) -> Result<Pczt, T2ZError> {
    let proving_key = load_orchard_proving_key();
    prove_transaction_with_key(pczt, proving_key)
}

/// Adds Orchard proofs to the PCZT using the Prover role with a provided key.
///
/// Use this if you want to manage the proving key lifecycle yourself.
pub fn prove_transaction_with_key(
    pczt: Pczt,
    proving_key: &OrchardProvingKey,
) -> Result<Pczt, T2ZError> {
    let mut prover = Prover::new(pczt);

    if prover.requires_orchard_proof() {
        prover = prover
            .create_orchard_proof(proving_key)
            .map_err(|e| T2ZError::Proving(format!("Proving failed: {:?}", e)))?;
    }

    Ok(prover.finish())
}

/// Signs a transparent input with the provided secp256k1 private key.
///
/// # Arguments
/// * `pczt` - The PCZT to sign
/// * `input_index` - Index of the transparent input to sign
/// * `secret_key_bytes` - 32-byte secp256k1 private key
///
/// # Returns
/// Updated PCZT with the signature added
pub fn sign_transparent_input(
    pczt: Pczt,
    input_index: usize,
    secret_key_bytes: &[u8; 32],
) -> Result<Pczt, T2ZError> {
    let secret_key = secp256k1::SecretKey::from_slice(secret_key_bytes)
        .map_err(|e| T2ZError::InvalidInput(format!("Invalid secret key: {}", e)))?;

    let mut signer = Signer::new(pczt)?;
    signer.sign_transparent(input_index, &secret_key)?;

    Ok(signer.finish())
}

/// Combines multiple PCZTs into one (Combiner role).
pub fn combine(pczts: Vec<Pczt>) -> Result<Pczt, T2ZError> {
    if pczts.is_empty() {
        return Err(T2ZError::InvalidInput("No PCZTs to combine".to_string()));
    }

    if pczts.len() == 1 {
        return Ok(pczts.into_iter().next().unwrap());
    }

    Ok(Combiner::new(pczts).combine()?)
}

/// Finalizes spends and extracts transaction bytes (Spend Finalizer + Transaction Extractor roles).
pub fn finalize_and_extract(pczt: Pczt) -> Result<Vec<u8>, T2ZError> {
    let pczt = SpendFinalizer::new(pczt).finalize_spends()?;
    let extractor = TransactionExtractor::new(pczt);
    let transaction = extractor.extract()?;

    let mut tx_bytes = Vec::new();
    transaction
        .write(&mut tx_bytes)
        .map_err(|e| T2ZError::Builder(format!("Transaction serialization failed: {:?}", e)))?;

    Ok(tx_bytes)
}

/// Parses a PCZT from bytes.
pub fn parse_pczt(pczt_bytes: &[u8]) -> Result<Pczt, T2ZError> {
    Ok(Pczt::parse(pczt_bytes)?)
}

/// Serializes a PCZT to bytes.
pub fn serialize_pczt(pczt: &Pczt) -> Vec<u8> {
    pczt.serialize()
}

// ============================================================================
// Serde support for byte arrays
// ============================================================================

mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match bytes {
            Some(b) => serializer.serialize_some(b),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Vec<u8>>::deserialize(deserializer)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_protocol::consensus::BranchId;

    #[test]
    fn test_pczt_roundtrip() {
        let pczt = Creator::new(BranchId::Nu6.into(), 10_000_000, 133, [0; 32], [0; 32]).build();

        let serialized = serialize_pczt(&pczt);
        let parsed = parse_pczt(&serialized).unwrap();

        assert_eq!(
            parsed.global().expiry_height(),
            pczt.global().expiry_height()
        );
    }

    #[test]
    fn test_combine() {
        let pczt = Creator::new(BranchId::Nu6.into(), 10_000_000, 133, [0; 32], [0; 32]).build();

        let combined = combine(vec![pczt.clone()]).unwrap();
        assert_eq!(
            combined.global().expiry_height(),
            pczt.global().expiry_height()
        );
    }
}
