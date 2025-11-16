// Core wrapper for PCZT functionality
// Production-ready implementation following ZIP 244 and ZIP 321 specifications
// Supports NAPI (TypeScript/Node.js/WASM) and uniffi (Go, Kotlin, Java)

use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use zcash_primitives::{
    consensus::{BlockHeight, Parameters},
    transaction::{
        builder::{BuildConfig, Builder},
        fees::zip317::FeeRule,
    },
};
use zcash_protocol::{consensus::NetworkType, value::Zatoshis};

// Re-export pczt types and roles
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

// Feature-gated modules
#[cfg(feature = "napi-bindings")]
pub mod napi_bindings;

#[cfg(feature = "uniffi-bindings")]
pub mod uniffi_bindings;

// UniFFI scaffolding must be at crate root
#[cfg(feature = "uniffi-bindings")]
uniffi::setup_scaffolding!();

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
}

/// Network selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    fn to_network_type(self) -> NetworkType {
        match self {
            Network::Mainnet => NetworkType::Main,
            Network::Testnet => NetworkType::Test,
        }
    }
}

// Create a simple parameters implementation
#[derive(Clone)]
struct SimpleParams(NetworkType);

impl Parameters for SimpleParams {
    fn network_type(&self) -> NetworkType {
        self.0
    }

    fn activation_height(
        &self,
        _nu: zcash_primitives::consensus::NetworkUpgrade,
    ) -> Option<BlockHeight> {
        None
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid memo: {0}")]
    InvalidMemo(String),

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
}

impl From<ParseError> for FfiError {
    fn from(e: ParseError) -> Self {
        FfiError::Parse(e)
    }
}

impl From<IoFinalizerError> for FfiError {
    fn from(e: IoFinalizerError) -> Self {
        FfiError::IoFinalizer(e)
    }
}

impl From<SignerError> for FfiError {
    fn from(e: SignerError) -> Self {
        FfiError::Signer(e)
    }
}

impl From<TxExtractorError> for FfiError {
    fn from(e: TxExtractorError) -> Self {
        FfiError::TxExtractor(e)
    }
}

impl From<CombinerError> for FfiError {
    fn from(e: CombinerError) -> Self {
        FfiError::Combiner(e)
    }
}

impl From<SpendFinalizerError> for FfiError {
    fn from(e: SpendFinalizerError) -> Self {
        FfiError::SpendFinalizer(e)
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
static ORCHARD_PK: once_cell::sync::OnceCell<orchard::circuit::ProvingKey> = once_cell::sync::OnceCell::new();

/// Builds the Orchard circuit proving key
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
///
/// # Implementation Note
/// The proving key represents the circuit structure for Orchard actions.
/// It's built using the Pallas/Vesta curve cycle and requires no external parameters.
#[cfg(not(target_family = "wasm"))]
pub fn load_orchard_proving_key() -> Result<&'static orchard::circuit::ProvingKey, FfiError> {
    ORCHARD_PK.get_or_try_init(|| {
        // Build the Orchard circuit proving key
        // This is a one-time ~10 second operation that happens in-process
        // No downloads, no trusted setup, just pure cryptography!
        Ok(orchard::circuit::ProvingKey::build())
    })
}

#[cfg(target_family = "wasm")]
pub async fn load_orchard_proving_key() -> Result<&'static orchard::circuit::ProvingKey, FfiError> {
    // Check if already loaded
    if let Some(pk) = ORCHARD_PK.get() {
        return Ok(pk);
    }
    
    // Build the Orchard circuit proving key
    // Note: This is expensive (~10 seconds) and should ideally be done at startup
    // or in a background thread in WASM environments
    let pk = orchard::circuit::ProvingKey::build();
    ORCHARD_PK.set(pk).map_err(|_| FfiError::Builder("Proving key already initialized".to_string()))?;
    Ok(ORCHARD_PK.get().unwrap())
}

/// Get the cached proving key if already loaded
pub fn get_cached_proving_key() -> Option<&'static orchard::circuit::ProvingKey> {
    ORCHARD_PK.get()
}

/// Prove transaction using the cached proving key (convenience function)
///
/// This automatically builds the Orchard circuit proving key if not already loaded.
///
/// # Performance Note
/// - First call: ~10 seconds (builds Halo 2 circuit, no download required)
/// - Subsequent calls: Fast (uses cached circuit)
///
/// Unlike Sapling which requires downloading ~50MB proving keys, Orchard uses
/// Halo 2 and builds the circuit programmatically with no trusted setup!
#[cfg(not(target_family = "wasm"))]
pub fn prove_transaction(pczt: Pczt) -> Result<Pczt, FfiError> {
    let proving_key = load_orchard_proving_key()?;
    prove_transaction_with_key(pczt, proving_key)
}

#[cfg(target_family = "wasm")]
pub async fn prove_transaction(pczt: Pczt) -> Result<Pczt, FfiError> {
    let proving_key = load_orchard_proving_key().await?;
    prove_transaction_with_key(pczt, proving_key)
}

// ============================================================================
// Address Parsing Helpers
// ============================================================================

/// Parses a transparent address from a ZcashAddress
fn parse_transparent_address(
    addr: &zcash_address::ZcashAddress,
    expected_network: NetworkType,
) -> Result<zcash_transparent::address::TransparentAddress, FfiError> {
    use zcash_address::{ConversionError, TryFromAddress};
    
    struct TransparentReceiver(zcash_transparent::address::TransparentAddress);
    
    impl TryFromAddress for TransparentReceiver {
        type Error = String;
        
        fn try_from_transparent_p2pkh(
            _net: NetworkType,
            data: [u8; 20],
        ) -> Result<Self, ConversionError<Self::Error>> {
            Ok(TransparentReceiver(zcash_transparent::address::TransparentAddress::PublicKeyHash(data)))
        }
        
        fn try_from_transparent_p2sh(
            _net: NetworkType,
            data: [u8; 20],
        ) -> Result<Self, ConversionError<Self::Error>> {
            Ok(TransparentReceiver(zcash_transparent::address::TransparentAddress::ScriptHash(data)))
        }
    }
    
    addr.clone()
        .convert_if_network::<TransparentReceiver>(expected_network)
        .map(|r| r.0)
        .map_err(|e| FfiError::InvalidAddress(format!("Not a transparent address: {:?}", e)))
}

/// Parses an Orchard receiver from a ZcashAddress
fn parse_orchard_receiver(
    addr: &zcash_address::ZcashAddress,
    expected_network: NetworkType,
) -> Result<orchard::Address, FfiError> {
    use zcash_address::{unified::{Container, Receiver}, ConversionError, TryFromAddress};
    
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
                        return Err(ConversionError::User("Invalid Orchard receiver data".to_string()));
                    }
                }
            }
            Err(ConversionError::User("Unified address has no Orchard receiver".to_string()))
        }
    }
    
    addr.clone()
        .convert_if_network::<OrchardReceiver>(expected_network)
        .map(|r| r.0)
        .map_err(|e| FfiError::InvalidAddress(format!("Not an Orchard address: {:?}", e)))
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
pub fn propose_transaction(
    transparent_inputs: &[TransparentInput],
    request: TransactionRequest,
    network: Network,
    expiry_height: u32,
) -> Result<Pczt, FfiError> {
    if transparent_inputs.is_empty() {
        return Err(FfiError::InvalidInput(
            "No transparent inputs provided".to_string(),
        ));
    }

    if request.payments.is_empty() {
        return Err(FfiError::InvalidInput("No payments specified".to_string()));
    }

    // Validate all inputs have correct sizes
    for (idx, input) in transparent_inputs.iter().enumerate() {
        if input.pubkey.len() != 33 {
            return Err(FfiError::InvalidInput(format!(
                "Input {} pubkey must be 33 bytes (got {})",
                idx,
                input.pubkey.len()
            )));
        }
        if input.prevout_txid.len() != 32 {
            return Err(FfiError::InvalidInput(format!(
                "Input {} prevout_txid must be 32 bytes (got {})",
                idx,
                input.prevout_txid.len()
            )));
        }
    }

    // Validate memo sizes (ZIP 321: max 512 bytes)
    for (idx, payment) in request.payments.iter().enumerate() {
        if let Some(memo) = &payment.memo
            && memo.len() > 512 {
                return Err(FfiError::InvalidMemo(format!(
                    "Payment {} memo exceeds 512 bytes ({} bytes)",
                    idx,
                    memo.len()
                )));
            }
    }

    // Calculate totals
    let total_input: u64 = transparent_inputs.iter().map(|i| i.value).sum();
    let total_output: u64 = request.payments.iter().map(|p| p.amount).sum();
    let fee = request.fee.unwrap_or(10_000); // Default 10k zatoshi fee if not specified

    if total_input < total_output + fee {
        return Err(FfiError::InvalidInput(format!(
            "Insufficient funds: {} < {} + {}",
            total_input, total_output, fee
        )));
    }

    // Check if we have any Orchard outputs
    let has_orchard = request.payments.iter().any(|p| {
        zcash_address::ZcashAddress::try_from_encoded(&p.address)
            .ok()
            .map(|addr| addr.can_receive_as(zcash_protocol::PoolType::ORCHARD))
            .unwrap_or(false)
    });

    let orchard_anchor = if has_orchard {
        Some(orchard::Anchor::empty_tree())
    } else {
        None
    };

    // Create builder
    let params = SimpleParams(network.to_network_type());
    let mut builder = Builder::new(
        params,
        BlockHeight::from_u32(expiry_height),
        BuildConfig::Standard {
            sapling_anchor: None,
            orchard_anchor,
        },
    );

    // Add transparent inputs
    for input in transparent_inputs {
        let pubkey_bytes: [u8; 33] = input
            .pubkey
            .as_slice()
            .try_into()
            .map_err(|_| FfiError::InvalidInput("Public key must be 33 bytes".to_string()))?;

        let pubkey = secp256k1::PublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid public key: {}", e)))?;

        let txid_bytes: [u8; 32] =
            input.prevout_txid.as_slice().try_into().map_err(|_| {
                FfiError::InvalidInput("Transaction ID must be 32 bytes".to_string())
            })?;

        let outpoint = zcash_transparent::bundle::OutPoint::new(txid_bytes, input.prevout_index);

        let script = zcash_script::script::Code(input.script_pubkey.clone());
        let txout = zcash_transparent::bundle::TxOut::new(
            Zatoshis::from_u64(input.value)
                .map_err(|e| FfiError::InvalidInput(format!("Invalid value: {:?}", e)))?,
            zcash_transparent::address::Script(script),
        );

        builder
            .add_transparent_input(pubkey, outpoint, txout)
            .map_err(|e| FfiError::Builder(format!("Failed to add transparent input: {:?}", e)))?;
    }

    // Add outputs - parse addresses and add appropriate outputs
    for payment in &request.payments {
        let addr = zcash_address::ZcashAddress::try_from_encoded(&payment.address)
            .map_err(|e| FfiError::InvalidAddress(format!("Invalid address: {:?}", e)))?;

        // Validate network matches
        let expected_network = network.to_network_type();
        
        // Check if this address can receive on the specified pool types
        if addr.can_receive_as(zcash_protocol::PoolType::TRANSPARENT) {
            // Handle transparent output
            let t_addr = parse_transparent_address(&addr, expected_network)?;
            builder
                .add_transparent_output(
                    &t_addr,
                    Zatoshis::from_u64(payment.amount)
                        .map_err(|e| FfiError::InvalidInput(format!("Invalid amount: {:?}", e)))?,
                )
                .map_err(|e| FfiError::Builder(format!("Failed to add transparent output: {:?}", e)))?;
        } else if addr.can_receive_as(zcash_protocol::PoolType::ORCHARD) {
            // Handle Orchard output
            let orchard_receiver = parse_orchard_receiver(&addr, expected_network)?;
            
            let memo_bytes = if let Some(memo) = &payment.memo {
                let mut padded = [0u8; 512];
                padded[..memo.len()].copy_from_slice(memo);
                zcash_protocol::memo::MemoBytes::from_bytes(&padded)
                    .map_err(|e| FfiError::InvalidMemo(format!("Invalid memo: {:?}", e)))?
            } else {
                zcash_protocol::memo::MemoBytes::empty()
            };

            builder
                .add_orchard_output::<FeeRule>(
                    None, // ovk - None means the output is not recoverable by sender
                    orchard_receiver,
                    payment.amount,
                    memo_bytes,
                )
                .map_err(|e| FfiError::Builder(format!("Failed to add Orchard output: {:?}", e)))?;
        } else {
            return Err(FfiError::InvalidAddress(format!(
                "Address {} cannot receive transparent or Orchard funds",
                payment.address
            )));
        }
    }

    // Build PCZT
    let result = builder
        .build_for_pczt(OsRng, &FeeRule::standard())
        .map_err(|e| FfiError::Builder(format!("Failed to build PCZT: {:?}", e)))?;

    let pczt = Creator::build_from_parts(result.pczt_parts)
        .ok_or_else(|| FfiError::Builder("Failed to create PCZT from parts".to_string()))?;

    // Finalize IO
    let pczt = IoFinalizer::new(pczt).finalize_io()?;

    Ok(pczt)
}

/// Adds Orchard proofs to the PCZT using the Prover role.
///
/// Requires the Orchard proving key to be provided separately.
/// The proving key can be generated with `orchard::circuit::ProvingKey::build()`.
pub fn prove_transaction_with_key(
    pczt: Pczt,
    proving_key: &orchard::circuit::ProvingKey,
) -> Result<Pczt, FfiError> {
    let mut prover = Prover::new(pczt);

    if prover.requires_orchard_proof() {
        prover = prover
            .create_orchard_proof(proving_key)
            .map_err(|e| FfiError::Builder(format!("Proving failed: {:?}", e)))?;
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
) -> Result<Pczt, FfiError> {
    let secret_key = secp256k1::SecretKey::from_slice(secret_key_bytes)
        .map_err(|e| FfiError::InvalidInput(format!("Invalid secret key: {}", e)))?;

    let mut signer = Signer::new(pczt)?;
    signer.sign_transparent(input_index, &secret_key)?;

    Ok(signer.finish())
}

/// Combines multiple PCZTs into one (Combiner role).
pub fn combine(pczts: Vec<Pczt>) -> Result<Pczt, FfiError> {
    if pczts.is_empty() {
        return Err(FfiError::InvalidInput("No PCZTs to combine".to_string()));
    }

    if pczts.len() == 1 {
        return Ok(pczts.into_iter().next().unwrap());
    }

    Ok(Combiner::new(pczts).combine()?)
}

/// Finalizes spends and extracts transaction bytes (Spend Finalizer + Transaction Extractor roles).
pub fn finalize_and_extract(pczt: Pczt) -> Result<Vec<u8>, FfiError> {
    let pczt = SpendFinalizer::new(pczt).finalize_spends()?;
    let extractor = TransactionExtractor::new(pczt);
    let transaction = extractor.extract()?;

    let mut tx_bytes = Vec::new();
    transaction
        .write(&mut tx_bytes)
        .map_err(|e| FfiError::Builder(format!("Transaction serialization failed: {:?}", e)))?;

    Ok(tx_bytes)
}

/// Parses a PCZT from bytes.
pub fn parse_pczt(pczt_bytes: &[u8]) -> Result<Pczt, FfiError> {
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
