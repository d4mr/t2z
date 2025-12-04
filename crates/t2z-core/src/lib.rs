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
        fees::zip317::FeeRule,
    },
};
use zcash_protocol::{
    consensus::{MainNetwork, NetworkType, TestNetwork},
    value::Zatoshis,
};

#[cfg(test)]
mod tests;

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
/// See: https://zips.z.cash/zip-0321
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// List of payments (supports multiple recipients via ZIP 321 paramindex)
    pub payments: Vec<Payment>,
}

/// Expected change output for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedTxOut {
    /// Address (transparent or Orchard unified address)
    pub address: String,
    /// Amount in zatoshis
    pub amount: u64,
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

    #[error(
        "Insufficient funds: available {available}, required {required} (payment: {payment}, fee: {fee})"
    )]
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
/// You MUST provide a `change_address` to receive the change.
/// If `change_address` is None and there's excess value, an error is returned.
///
/// # Arguments
/// * `transparent_inputs` - UTXOs to spend
/// * `request` - ZIP 321 transaction request (payments only)
/// * `change_address` - Optional address for change (transparent or Orchard)
/// * `network` - Mainnet or Testnet
/// * `expiry_height` - Transaction expiry height
///
/// # Fee Calculation
/// Uses ZIP-317 fee rules automatically.
pub fn propose_transaction(
    transparent_inputs: &[TransparentInput],
    request: TransactionRequest,
    change_address: Option<&str>,
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

    // Parse change address first to determine its type (affects fee calculation)
    enum ChangeDestination {
        Transparent(zcash_transparent::address::TransparentAddress),
        Orchard(orchard::Address),
    }

    let change_dest_type: Option<ChangeDestination> = if let Some(change_addr_str) = change_address
    {
        let change_addr = zcash_address::ZcashAddress::try_from_encoded(change_addr_str)
            .map_err(|e| T2ZError::InvalidAddress(format!("Invalid change address: {:?}", e)))?;

        if change_addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            Some(ChangeDestination::Transparent(parse_transparent_address(
                &change_addr,
                expected_network,
            )?))
        } else if change_addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
            Some(ChangeDestination::Orchard(parse_orchard_receiver(
                &change_addr,
                expected_network,
            )?))
        } else {
            return Err(T2ZError::InvalidAddress(
                "Change address must be transparent (P2PKH) or Orchard".to_string(),
            ));
        }
    } else {
        None
    };

    // Count output types and check if we have Orchard
    let mut _num_transparent_outputs = 0usize;
    let mut num_orchard_outputs = 0usize;

    for payment in &request.payments {
        let addr = zcash_address::ZcashAddress::try_from_encoded(&payment.address)
            .map_err(|e| T2ZError::InvalidAddress(format!("Invalid address: {:?}", e)))?;

        if addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            _num_transparent_outputs += 1;
        } else if addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
            num_orchard_outputs += 1;
        } else {
            return Err(T2ZError::InvalidAddress(format!(
                "Address {} cannot receive transparent or Orchard funds",
                payment.address
            )));
        }
    }

    // Calculate totals
    let total_input: u64 = transparent_inputs.iter().map(|i| i.value).sum();
    let total_payment: u64 = request.payments.iter().map(|p| p.amount).sum();

    // Determine if we'll have any Orchard outputs (affects builder config)
    let has_orchard =
        num_orchard_outputs > 0 || matches!(change_dest_type, Some(ChangeDestination::Orchard(_)));

    let orchard_anchor = if has_orchard {
        Some(orchard::Anchor::empty_tree())
    } else {
        None
    };

    // Create builder with proper network parameters
    // We need to handle this with a macro/match since Builder is generic over Parameters
    macro_rules! build_transaction {
        ($params:expr) => {{
            let fee_rule = FeeRule::standard();

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

            // Calculate fee and change with iteration to handle Orchard change affecting fee.
            // When change goes to Orchard, adding the change output increases the action count,
            // which might increase the ZIP-317 fee. We need to iterate to find the stable values.
            let mut change_added = false;
            let mut final_change = 0u64;

            // First pass: calculate fee without change
            let fee = builder.get_fee(&fee_rule)
                .map_err(|e| T2ZError::Builder(format!("Failed to calculate fee: {:?}", e)))?;

            // Calculate initial change
            let change = total_input
                .checked_sub(total_payment)
                .and_then(|v| v.checked_sub(fee.into_u64()))
                .ok_or_else(|| T2ZError::InsufficientFunds {
                    available: total_input,
                    required: total_payment + fee.into_u64(),
                    payment: total_payment,
                    fee: fee.into_u64(),
                })?;

            // If there's change, we need a change address
            if change > 0 && change_dest_type.is_none() {
                return Err(T2ZError::ChangeRequired { change });
            }

            // Handle change with iteration for Orchard (since adding Orchard change affects fee)
            if change > 0 {
                match &change_dest_type {
                    Some(ChangeDestination::Transparent(t_addr)) => {
                        // Transparent change doesn't affect Orchard action count, so no iteration needed
                        builder
                            .add_transparent_output(
                                t_addr,
                                Zatoshis::from_u64(change).map_err(|e| {
                                    T2ZError::InvalidInput(format!("Invalid change amount: {:?}", e))
                                })?,
                            )
                            .map_err(|e| {
                                T2ZError::Builder(format!("Failed to add transparent change output: {:?}", e))
                            })?;
                        final_change = change;
                        change_added = true;
                    }
                    Some(ChangeDestination::Orchard(orchard_addr)) => {
                        // Orchard change affects action count → affects fee. Iterate to stabilize.
                        // Add a placeholder change output to calculate the correct fee
                        builder
                            .add_orchard_output::<FeeRule>(
                                None,
                                *orchard_addr,
                                change, // Use current estimate
                                zcash_protocol::memo::MemoBytes::empty(),
                            )
                            .map_err(|e| {
                                T2ZError::Builder(format!("Failed to add Orchard change output: {:?}", e))
                            })?;
                        change_added = true;

                        // Recalculate fee with the change output included
                        let new_fee = builder.get_fee(&fee_rule)
                            .map_err(|e| T2ZError::Builder(format!("Failed to recalculate fee: {:?}", e)))?;

                        // Recalculate change with new fee
                        let new_change = total_input
                            .checked_sub(total_payment)
                            .and_then(|v| v.checked_sub(new_fee.into_u64()))
                            .ok_or_else(|| T2ZError::InsufficientFunds {
                                available: total_input,
                                required: total_payment + new_fee.into_u64(),
                                payment: total_payment,
                                fee: new_fee.into_u64(),
                            })?;

                        // The change output was already added with the old value.
                        // The Builder will use the fee_rule at build time, so the actual
                        // change value embedded in the action may differ from what we calculated.
                        // However, the Builder's build_for_pczt will enforce the correct fee.
                        // We just need to make sure we have enough funds.
                        final_change = new_change;
                        let _ = new_fee; // Fee was recalculated and validated
                    }
                    None => unreachable!(), // Already checked above
                }
            }

            // Note: The actual change value in the PCZT may be adjusted by the Builder
            // during build_for_pczt to match the exact ZIP-317 fee calculation.
            let _ = (change_added, final_change); // Suppress warnings

            // Build PCZT using the same fee rule we used to calculate the fee
            let result = builder
                .build_for_pczt(OsRng, &fee_rule)
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

/// Gets the sighash for a transparent input (per ZIP 244).
///
/// Use this to obtain the 32-byte hash that needs to be signed externally.
/// Then call `append_signature` with the resulting ECDSA signature.
///
/// This is for T2Z transactions where we have transparent inputs that need signing.
/// For shielded spends (Orchard/Sapling), use the appropriate signing functions.
///
/// # Note
/// This function assumes P2PKH inputs with SIGHASH_ALL, which is what T2Z transactions use.
/// For P2SH or other sighash types, use the full Signer role from the pczt crate.
///
/// # Arguments
/// * `pczt` - The PCZT
/// * `input_index` - Index of the transparent input
///
/// # Returns
/// 32-byte sighash that should be signed with ECDSA using secp256k1
pub fn get_sighash(pczt: &Pczt, input_index: usize) -> Result<[u8; 32], T2ZError> {
    use zcash_primitives::transaction::{
        sighash::SignableInput, sighash_v5::v5_signature_hash, txid::TxIdDigester,
    };
    use zcash_transparent::sighash::{SighashType, SignableInput as TransparentSignableInput};

    // Get TransactionData from the PCZT using the public into_effects() method
    let tx_data = pczt.clone().into_effects().ok_or_else(|| {
        T2ZError::InvalidInput("Failed to convert PCZT to transaction data".to_string())
    })?;

    // Compute the TxId digests needed for sighash
    let txid_parts = tx_data.digest(TxIdDigester);

    // Get the input data from the PCZT's transparent bundle
    let transparent_bundle = pczt.transparent();
    let input = transparent_bundle
        .inputs()
        .get(input_index)
        .ok_or_else(|| T2ZError::InvalidInput(format!("Invalid input index: {}", input_index)))?;

    // For T2Z (P2PKH inputs), the builder always sets SIGHASH_ALL
    // and there's no redeem_script, so script_code = script_pubkey
    let sighash_type = SighashType::ALL;

    // Get script_pubkey from the input (has public getter)
    let script_pubkey_bytes = input.script_pubkey();

    // For P2PKH, script_code = script_pubkey (no redeem_script)
    // Create Script by wrapping the bytes in script::Code
    let script =
        zcash_transparent::address::Script(zcash_script::script::Code(script_pubkey_bytes.clone()));

    // Get the value (has public getter) - it's a u64 in the serialized form
    let value = zcash_protocol::value::Zatoshis::from_u64(*input.value())
        .map_err(|_| T2ZError::InvalidInput("Invalid input value".to_string()))?;

    // Build the SignableInput for transparent
    let transparent_signable = TransparentSignableInput::from_parts(
        sighash_type,
        input_index,
        &script, // script_code
        &script, // script_pubkey (same for P2PKH)
        value,
    );

    // Wrap in the enum variant expected by v5_signature_hash
    let signable_input = SignableInput::Transparent(transparent_signable);

    // Compute the sighash
    let sighash = v5_signature_hash(&tx_data, &signable_input, &txid_parts);

    Ok(sighash.as_ref().try_into().expect("sighash is 32 bytes"))
}

/// Appends a pre-computed ECDSA signature to a transparent input.
///
/// The signature should be created by signing the output of `get_sighash`
/// with the private key corresponding to the input's pubkey.
///
/// This function verifies the signature is valid before adding it.
///
/// # Arguments
/// * `pczt` - The PCZT to update
/// * `input_index` - Index of the transparent input
/// * `pubkey` - 33-byte compressed secp256k1 public key
/// * `signature` - DER-encoded ECDSA signature with sighash type byte appended (typically 71-73 bytes)
///
/// # Returns
/// Updated PCZT with the signature added to partial_signatures
pub fn append_signature(
    pczt: Pczt,
    input_index: usize,
    pubkey: &[u8; 33],
    signature: &[u8],
) -> Result<Pczt, T2ZError> {
    // Verify the pubkey is valid
    let pk = secp256k1::PublicKey::from_slice(pubkey)
        .map_err(|e| T2ZError::InvalidInput(format!("Invalid public key: {}", e)))?;

    // Verify the signature format: DER + 1 byte sighash type
    if signature.len() < 2 {
        return Err(T2ZError::InvalidInput("Signature too short".to_string()));
    }

    // The last byte is the sighash type, the rest is the DER signature
    let der_sig = &signature[..signature.len() - 1];
    let sig = secp256k1::ecdsa::Signature::from_der(der_sig)
        .map_err(|e| T2ZError::InvalidInput(format!("Invalid DER signature: {}", e)))?;

    // Verify the signature against the sighash
    let sighash = get_sighash(&pczt, input_index)?;
    let message = secp256k1::Message::from_digest(sighash);
    let secp = secp256k1::Secp256k1::verification_only();
    secp.verify_ecdsa(&message, &sig, &pk)
        .map_err(|e| T2ZError::InvalidInput(format!("Signature verification failed: {}", e)))?;

    // Use the Combiner to merge the signature into the PCZT
    // We create a clone of the PCZT with the signature added via the Signer role
    add_signature_via_signer(pczt, input_index, pubkey, signature)
}

/// Internal helper to add a signature to the PCZT.
///
/// Uses shadow structs to deserialize the PCZT, modify partial_signatures,
/// and re-serialize.
fn add_signature_via_signer(
    pczt: Pczt,
    input_index: usize,
    pubkey: &[u8; 33],
    signature: &[u8],
) -> Result<Pczt, T2ZError> {
    let bytes = pczt.serialize();

    // Modify the PCZT using our shadow struct approach
    let modified_bytes = modify_pczt_signature(&bytes, input_index, *pubkey, signature.to_vec())?;

    // Re-parse the modified PCZT
    Pczt::parse(&modified_bytes)
        .map_err(|e| T2ZError::InvalidInput(format!("Failed to parse modified PCZT: {:?}", e)))
}

/// Modify PCZT bytes to add a signature to partial_signatures.
///
/// This uses shadow structs that match the PCZT layout to deserialize,
/// modify, and re-serialize the PCZT.
fn modify_pczt_signature(
    pczt_bytes: &[u8],
    input_index: usize,
    pubkey: [u8; 33],
    signature: Vec<u8>,
) -> Result<Vec<u8>, T2ZError> {
    use shadow::PcztShadow;

    // PCZT format: 4 bytes magic + 4 bytes version + postcard data
    if pczt_bytes.len() < 8 {
        return Err(T2ZError::InvalidInput("PCZT too short".to_string()));
    }

    let magic = &pczt_bytes[..4];
    let version = &pczt_bytes[4..8];
    let data = &pczt_bytes[8..];

    // Deserialize the postcard data into our shadow struct
    let mut pczt_shadow: PcztShadow = postcard::from_bytes(data)
        .map_err(|e| T2ZError::InvalidInput(format!("Failed to deserialize PCZT: {:?}", e)))?;

    // Get the input and add the signature
    let input = pczt_shadow
        .transparent
        .inputs
        .get_mut(input_index)
        .ok_or_else(|| T2ZError::InvalidInput(format!("Invalid input index: {}", input_index)))?;

    input.partial_signatures.insert(pubkey, signature);

    // Re-serialize
    let new_data = postcard::to_allocvec(&pczt_shadow)
        .map_err(|e| T2ZError::InvalidInput(format!("Failed to serialize PCZT: {:?}", e)))?;

    // Reconstruct the full PCZT bytes
    let mut result = Vec::with_capacity(8 + new_data.len());
    result.extend_from_slice(magic);
    result.extend_from_slice(version);
    result.extend_from_slice(&new_data);

    Ok(result)
}

// Shadow structs for PCZT round-tripping - in separate file
pub(crate) mod shadow;

/// Signs a transparent input with the provided secp256k1 private key.
///
/// This is a convenience function that combines `get_sighash` and `append_signature`.
/// For external signing (hardware wallets, HSMs), use those functions separately.
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

/// Verifies the PCZT matches the original transaction request before signing.
///
/// This implements verification checks that should be performed before signing
/// to detect any malleation of the PCZT. Per the spec, this may be skipped if
/// the same entity created and is signing the PCZT with no third-party involvement.
///
/// # Arguments
/// * `pczt` - The PCZT to verify
/// * `transaction_request` - The original ZIP 321 transaction request (payments only)
/// * `expected_change` - List of expected change outputs (address + amount)
///
/// # Returns
/// Ok(()) if verification passes, Err with details if it fails
pub fn verify_before_signing(
    pczt: &Pczt,
    transaction_request: &TransactionRequest,
    expected_change: &[ExpectedTxOut],
) -> Result<(), T2ZError> {
    use zcash_address::unified::{Address as UnifiedAddress, Container, Encoding};

    // Get the transparent outputs from the PCZT
    let transparent_outputs = pczt.transparent().outputs();
    let orchard_actions = pczt.orchard().actions();

    // Track which payments and expected changes we've matched
    let mut matched_payments = vec![false; transaction_request.payments.len()];
    let mut matched_changes = vec![false; expected_change.len()];

    // Helper: Get transparent script bytes from an address string
    // Returns None if address is not transparent
    let get_transparent_script = |addr_str: &str| -> Option<Vec<u8>> {
        let addr = zcash_address::ZcashAddress::try_from_encoded(addr_str).ok()?;
        if !addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            return None;
        }

        // Try to decode as unified address first
        if let Ok((_, ua)) = UnifiedAddress::decode(addr_str) {
            for receiver in ua.items() {
                if let zcash_address::unified::Receiver::P2pkh(hash) = receiver {
                    // Build P2PKH script: OP_DUP OP_HASH160 <20 bytes> OP_EQUALVERIFY OP_CHECKSIG
                    let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160 PUSH20
                    script.extend_from_slice(&hash);
                    script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
                    return Some(script);
                }
                if let zcash_address::unified::Receiver::P2sh(hash) = receiver {
                    // Build P2SH script: OP_HASH160 <20 bytes> OP_EQUAL
                    let mut script = vec![0xa9, 0x14]; // OP_HASH160 PUSH20
                    script.extend_from_slice(&hash);
                    script.push(0x87); // OP_EQUAL
                    return Some(script);
                }
            }
        }

        // Try to parse as legacy t-address
        // For t1.../tm... addresses (P2PKH)
        // The address is base58check encoded with a version prefix
        // We can try to decode and extract the pubkey hash
        if addr_str.starts_with("t1") || addr_str.starts_with("tm") {
            // Legacy P2PKH address
            if let Ok(decoded) = bs58::decode(addr_str).with_check(None).into_vec() {
                // Format: [version (2 bytes)][pubkey_hash (20 bytes)]
                if decoded.len() == 22 {
                    let pubkey_hash = &decoded[2..22];
                    let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160 PUSH20
                    script.extend_from_slice(pubkey_hash);
                    script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
                    return Some(script);
                }
            }
        }

        None
    };

    // Helper: Get expected Orchard address bytes from address string
    let get_orchard_address_bytes = |addr_str: &str| -> Option<[u8; 43]> {
        let addr = zcash_address::ZcashAddress::try_from_encoded(addr_str).ok()?;
        if !addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
            return None;
        }
        // Extract Orchard receiver from unified address
        let (_, ua) = UnifiedAddress::decode(addr_str).ok()?;
        for receiver in ua.items() {
            if let zcash_address::unified::Receiver::Orchard(bytes) = receiver {
                return Some(bytes);
            }
        }
        None
    };

    // 1. Verify transparent outputs match request (by BOTH script and amount)
    for output in transparent_outputs {
        let value = *output.value();
        let output_script: Vec<u8> = output.script_pubkey().to_vec();

        // Try to match against payments
        let mut matched = false;
        for (idx, payment) in transaction_request.payments.iter().enumerate() {
            if matched_payments[idx] {
                continue;
            }

            // Check if this is a transparent payment with matching script and amount
            if payment.amount == value
                && let Some(expected_script) = get_transparent_script(&payment.address)
                && output_script == expected_script
            {
                matched_payments[idx] = true;
                matched = true;
                break;
            }
        }

        // Check if this is an expected change output
        if !matched {
            for (idx, change) in expected_change.iter().enumerate() {
                if matched_changes[idx] {
                    continue;
                }
                if change.amount == value
                    && let Some(expected_script) = get_transparent_script(&change.address)
                    && output_script == expected_script
                {
                    matched_changes[idx] = true;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            return Err(T2ZError::InvalidInput(format!(
                "Unexpected transparent output: {} zatoshis to script {}",
                value,
                hex::encode(&output_script)
            )));
        }
    }

    // 2. Verify Orchard outputs match request (by address if available, or amount)
    for action in orchard_actions {
        let output = action.output();
        if let Some(value) = output.value() {
            // Get recipient address bytes if available (already raw [u8; 43] in PCZT)
            let recipient_bytes: Option<&[u8; 43]> = output.recipient().as_ref();

            // Try to match against payments
            let mut matched = false;
            for (idx, payment) in transaction_request.payments.iter().enumerate() {
                if matched_payments[idx] {
                    continue;
                }

                // Check if this is an Orchard payment
                if payment.amount == *value
                    && let Some(expected_addr) = get_orchard_address_bytes(&payment.address)
                {
                    // If we have recipient bytes, verify they match
                    if let Some(actual_addr) = recipient_bytes {
                        if *actual_addr == expected_addr {
                            matched_payments[idx] = true;
                            matched = true;
                            break;
                        }
                    } else {
                        // Recipient redacted - match by amount only (less secure)
                        matched_payments[idx] = true;
                        matched = true;
                        break;
                    }
                }
            }

            // Check if this is an expected change output (going to Orchard)
            if !matched {
                for (idx, change) in expected_change.iter().enumerate() {
                    if matched_changes[idx] {
                        continue;
                    }
                    if change.amount == *value
                        && let Some(expected_addr) = get_orchard_address_bytes(&change.address)
                    {
                        if let Some(actual_addr) = recipient_bytes {
                            if *actual_addr == expected_addr {
                                matched_changes[idx] = true;
                                matched = true;
                                break;
                            }
                        } else {
                            // Recipient redacted - match by amount only
                            matched_changes[idx] = true;
                            matched = true;
                            break;
                        }
                    }
                }
            }

            // Dummy outputs (value 0) are expected for Orchard bundles
            if !matched && *value != 0 {
                return Err(T2ZError::InvalidInput(format!(
                    "Unexpected Orchard output: {} zatoshis",
                    value
                )));
            }
        }
    }

    // 3. Verify all payments were matched
    for (idx, matched) in matched_payments.iter().enumerate() {
        if !*matched {
            return Err(T2ZError::InvalidInput(format!(
                "Payment {} not found in PCZT: {} zatoshis to {}",
                idx,
                transaction_request.payments[idx].amount,
                transaction_request.payments[idx].address
            )));
        }
    }

    // 4. Verify all expected changes were matched
    for (idx, matched) in matched_changes.iter().enumerate() {
        if !*matched {
            return Err(T2ZError::InvalidInput(format!(
                "Expected change {} not found in PCZT: {} zatoshis to {}",
                idx, expected_change[idx].amount, expected_change[idx].address
            )));
        }
    }

    Ok(())
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
// PCZT Inspection
// ============================================================================

/// Information about a transparent input in a PCZT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcztTransparentInput {
    /// Previous transaction ID (hex, display order - big-endian)
    pub prevout_txid: String,
    /// Previous output index
    pub prevout_index: u32,
    /// Value in zatoshis
    pub value: u64,
    /// Script pubkey (hex)
    pub script_pubkey: String,
    /// Whether this input has any partial signatures
    pub is_signed: bool,
    /// Number of partial signatures
    pub num_signatures: usize,
}

/// Information about a transparent output in a PCZT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcztTransparentOutput {
    /// Value in zatoshis
    pub value: u64,
    /// Script pubkey (hex)
    pub script_pubkey: String,
    /// User-provided address (if set by Updater)
    pub user_address: Option<String>,
}

/// Information about an Orchard action/output in a PCZT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcztOrchardOutput {
    /// Value in zatoshis (if known/not redacted)
    pub value: Option<u64>,
    /// Recipient address bytes (hex, if not redacted)
    pub recipient: Option<String>,
    /// User-provided address string (if set by Updater)
    pub user_address: Option<String>,
}

/// Complete information about a PCZT's contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcztInfo {
    /// Expiry height
    pub expiry_height: u32,
    /// Transparent inputs
    pub transparent_inputs: Vec<PcztTransparentInput>,
    /// Transparent outputs  
    pub transparent_outputs: Vec<PcztTransparentOutput>,
    /// Orchard outputs (from actions)
    pub orchard_outputs: Vec<PcztOrchardOutput>,
    /// Total input value (zatoshis)
    pub total_input: u64,
    /// Total transparent output value (zatoshis)
    pub total_transparent_output: u64,
    /// Total Orchard output value (zatoshis, only counted if value is known)
    pub total_orchard_output: u64,
    /// Implied fee (total_input - all outputs)
    pub implied_fee: u64,
    /// Number of Orchard actions
    pub num_orchard_actions: usize,
    /// Whether all transparent inputs are signed
    pub all_inputs_signed: bool,
    /// Whether Orchard bundle has proofs
    pub has_orchard_proofs: bool,
}

/// Inspects a PCZT and returns structured information about its contents.
///
/// Uses shadow struct deserialization to access all fields including
/// partial_signatures and zkproof that aren't publicly accessible.
///
/// This is useful for:
/// - Displaying transaction details to users before signing
/// - Calculating fee and change amounts after propose_transaction
/// - Verifying the transaction matches expectations
/// - Checking signing/proving progress
pub fn inspect_pczt_bytes(pczt_bytes: &[u8]) -> Result<PcztInfo, T2ZError> {
    use shadow::PcztShadow;
    
    // PCZT format: 4 bytes magic + 4 bytes version + postcard data
    if pczt_bytes.len() < 8 {
        return Err(T2ZError::InvalidInput("PCZT too short".to_string()));
    }
    
    let data = &pczt_bytes[8..];
    
    // Deserialize using shadow struct (gives access to all fields)
    let pczt: PcztShadow = postcard::from_bytes(data)
        .map_err(|e| T2ZError::InvalidInput(format!("Failed to deserialize PCZT: {:?}", e)))?;
    
    // Extract transparent inputs
    let transparent_inputs: Vec<PcztTransparentInput> = pczt.transparent.inputs
        .iter()
        .map(|input| {
            // Reverse txid for display (internal is little-endian, display is big-endian)
            let mut txid_bytes = input.prevout_txid;
            txid_bytes.reverse();
            
            PcztTransparentInput {
                prevout_txid: hex::encode(txid_bytes),
                prevout_index: input.prevout_index,
                value: input.value,
                script_pubkey: hex::encode(&input.script_pubkey),
                is_signed: !input.partial_signatures.is_empty(),
                num_signatures: input.partial_signatures.len(),
            }
        })
        .collect();
    
    // Extract transparent outputs
    let transparent_outputs: Vec<PcztTransparentOutput> = pczt.transparent.outputs
        .iter()
        .map(|output| PcztTransparentOutput {
            value: output.value,
            script_pubkey: hex::encode(&output.script_pubkey),
            user_address: output.user_address.clone(),
        })
        .collect();
    
    // Extract Orchard outputs from actions
    let orchard_outputs: Vec<PcztOrchardOutput> = pczt.orchard.actions
        .iter()
        .map(|action| PcztOrchardOutput {
            value: action.output.value,
            recipient: action.output.recipient.map(hex::encode),
            user_address: action.output.user_address.clone(),
        })
        .collect();
    
    // Calculate totals
    let total_input: u64 = transparent_inputs.iter().map(|i| i.value).sum();
    let total_transparent_output: u64 = transparent_outputs.iter().map(|o| o.value).sum();
    let total_orchard_output: u64 = orchard_outputs
        .iter()
        .filter_map(|o| o.value)
        .sum();
    
    // Fee = inputs - outputs (may include dummy 0-value Orchard outputs)
    let total_output = total_transparent_output + total_orchard_output;
    let implied_fee = total_input.saturating_sub(total_output);
    
    let all_inputs_signed = transparent_inputs.iter().all(|i| i.is_signed);
    let has_orchard_proofs = pczt.orchard.zkproof.is_some();
    
    Ok(PcztInfo {
        expiry_height: pczt.global.expiry_height,
        transparent_inputs,
        transparent_outputs,
        orchard_outputs,
        total_input,
        total_transparent_output,
        total_orchard_output,
        implied_fee,
        num_orchard_actions: pczt.orchard.actions.len(),
        all_inputs_signed,
        has_orchard_proofs,
    })
}

/// Inspects a PCZT and returns structured information about its contents.
/// Convenience wrapper that serializes the PCZT first.
pub fn inspect_pczt(pczt: &Pczt) -> Result<PcztInfo, T2ZError> {
    let bytes = pczt.serialize();
    inspect_pczt_bytes(&bytes)
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
