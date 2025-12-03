//! T2Z UniFFI - UniFFI bindings for T2Z
//!
//! This crate provides UniFFI bindings for the T2Z library,
//! enabling Zcash transparent-to-shielded transactions in Go, Kotlin, and Java.

use std::sync::Arc;
use t2z_core::{Pczt, T2ZError};

// UniFFI scaffolding
uniffi::setup_scaffolding!();

// ============================================================================
// UniFFI Error Type
// ============================================================================

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniffiError {
    #[error("Error: {msg}")]
    Error { msg: String },
}

impl From<T2ZError> for UniffiError {
    fn from(e: T2ZError) -> Self {
        UniffiError::Error { msg: e.to_string() }
    }
}

// ============================================================================
// UniFFI Record Types
// ============================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct UniffiTransparentInput {
    /// Public key (33 bytes as hex string)
    pub pubkey: String,
    /// Previous transaction ID (32 bytes as hex string)
    pub prevout_txid: String,
    /// Previous output index
    pub prevout_index: u32,
    /// Value in zatoshis
    pub value: u64,
    /// Script pubkey (hex encoded)
    pub script_pubkey: String,
    /// Optional sequence number
    pub sequence: Option<u32>,
}

impl UniffiTransparentInput {
    fn to_core(&self) -> Result<t2z_core::TransparentInput, UniffiError> {
        let pubkey = hex::decode(&self.pubkey)
            .map_err(|e| UniffiError::Error {
                msg: format!("Invalid pubkey hex: {}", e),
            })?;

        let prevout_txid = hex::decode(&self.prevout_txid)
            .map_err(|e| UniffiError::Error {
                msg: format!("Invalid prevout_txid hex: {}", e),
            })?;

        let script_pubkey = hex::decode(&self.script_pubkey)
            .map_err(|e| UniffiError::Error {
                msg: format!("Invalid script_pubkey hex: {}", e),
            })?;

        Ok(t2z_core::TransparentInput {
            pubkey,
            prevout_txid,
            prevout_index: self.prevout_index,
            value: self.value,
            script_pubkey,
            sequence: self.sequence,
        })
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct UniffiPayment {
    /// Address (transparent P2PKH/P2SH or unified with Orchard)
    pub address: String,
    /// Value in zatoshis
    pub amount: u64,
    /// Optional memo (hex encoded, max 512 bytes)
    pub memo: Option<String>,
    /// Optional label
    pub label: Option<String>,
}

impl UniffiPayment {
    fn to_core(&self) -> Result<t2z_core::Payment, UniffiError> {
        let memo = if let Some(memo_hex) = &self.memo {
            Some(hex::decode(memo_hex).map_err(|e| UniffiError::Error {
                msg: format!("Invalid memo hex: {}", e),
            })?)
        } else {
            None
        };

        Ok(t2z_core::Payment {
            address: self.address.clone(),
            amount: self.amount,
            memo,
            label: self.label.clone(),
        })
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct UniffiTransactionRequest {
    /// List of payments
    pub payments: Vec<UniffiPayment>,
    /// Optional fee in zatoshis
    pub fee: Option<u64>,
}

impl UniffiTransactionRequest {
    fn to_core(&self) -> Result<t2z_core::TransactionRequest, UniffiError> {
        let payments: Result<Vec<t2z_core::Payment>, UniffiError> =
            self.payments.iter().map(|p| p.to_core()).collect();

        Ok(t2z_core::TransactionRequest {
            payments: payments?,
            fee: self.fee,
        })
    }
}

// ============================================================================
// UniFFI PCZT Object
// ============================================================================

#[derive(uniffi::Object)]
pub struct UniffiPczt {
    inner: Pczt,
}

#[uniffi::export]
impl UniffiPczt {
    /// Creates a UniffiPczt from raw bytes
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Arc<Self>, UniffiError> {
        let pczt = t2z_core::parse_pczt(&bytes)?;
        Ok(Arc::new(UniffiPczt { inner: pczt }))
    }

    /// Creates a UniffiPczt from hex string
    #[uniffi::constructor]
    pub fn from_hex(hex_string: String) -> Result<Arc<Self>, UniffiError> {
        let bytes = hex::decode(&hex_string).map_err(|e| UniffiError::Error {
            msg: format!("Invalid hex: {}", e),
        })?;
        Self::from_bytes(bytes)
    }

    /// Serializes the PCZT to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        t2z_core::serialize_pczt(&self.inner)
    }

    /// Serializes the PCZT to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(t2z_core::serialize_pczt(&self.inner))
    }
}

// ============================================================================
// UniFFI Exported Functions
// ============================================================================

/// Proposes a transaction from transparent inputs to transparent and/or shielded outputs
#[uniffi::export]
pub fn propose_transaction(
    inputs_to_spend: Vec<UniffiTransparentInput>,
    transaction_request: UniffiTransactionRequest,
    network: String,
    expiry_height: u32,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    let inputs: Result<Vec<t2z_core::TransparentInput>, UniffiError> =
        inputs_to_spend.iter().map(|i| i.to_core()).collect();
    let inputs = inputs?;

    let request = transaction_request.to_core()?;

    let network = match network.as_str() {
        "mainnet" => t2z_core::Network::Mainnet,
        "testnet" => t2z_core::Network::Testnet,
        _ => {
            return Err(UniffiError::Error {
                msg: "Network must be 'mainnet' or 'testnet'".to_string(),
            })
        }
    };

    let pczt = t2z_core::propose_transaction(&inputs, request, network, expiry_height)?;
    Ok(Arc::new(UniffiPczt { inner: pczt }))
}

/// Proves a transaction (builds proving key automatically, ~10 seconds first call)
///
/// This uses Halo 2, which requires NO external downloads or trusted setup.
/// The proving key is built programmatically and cached for subsequent calls.
#[uniffi::export]
pub fn prove_transaction(pczt: Arc<UniffiPczt>) -> Result<Arc<UniffiPczt>, UniffiError> {
    let proved = t2z_core::prove_transaction(pczt.inner.clone())?;
    Ok(Arc::new(UniffiPczt { inner: proved }))
}

/// Signs a transparent input with the provided private key
#[uniffi::export]
pub fn sign_transparent_input(
    pczt: Arc<UniffiPczt>,
    input_index: u32,
    secret_key_hex: String,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    let secret_key_bytes = hex::decode(&secret_key_hex).map_err(|e| UniffiError::Error {
        msg: format!("Invalid secret key hex: {}", e),
    })?;

    if secret_key_bytes.len() != 32 {
        return Err(UniffiError::Error {
            msg: "Secret key must be 32 bytes".to_string(),
        });
    }

    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&secret_key_bytes);

    let signed =
        t2z_core::sign_transparent_input(pczt.inner.clone(), input_index as usize, &secret_key)?;
    Ok(Arc::new(UniffiPczt { inner: signed }))
}

/// Combines multiple PCZTs into one
#[uniffi::export]
pub fn combine_pczts(pczt_list: Vec<Arc<UniffiPczt>>) -> Result<Arc<UniffiPczt>, UniffiError> {
    let pczts: Vec<Pczt> = pczt_list.iter().map(|p| p.inner.clone()).collect();
    let combined = t2z_core::combine(pczts)?;
    Ok(Arc::new(UniffiPczt { inner: combined }))
}

/// Finalizes the PCZT and extracts the transaction bytes
#[uniffi::export]
pub fn finalize_and_extract(pczt: Arc<UniffiPczt>) -> Result<Vec<u8>, UniffiError> {
    let tx_bytes = t2z_core::finalize_and_extract(pczt.inner.clone())?;
    Ok(tx_bytes)
}

/// Finalizes the PCZT and extracts the transaction as hex string
#[uniffi::export]
pub fn finalize_and_extract_hex(pczt: Arc<UniffiPczt>) -> Result<String, UniffiError> {
    let tx_bytes = finalize_and_extract(pczt)?;
    Ok(hex::encode(tx_bytes))
}

/// Check if the proving key has been built and cached
#[uniffi::export]
pub fn is_proving_key_ready() -> bool {
    t2z_core::is_proving_key_loaded()
}

/// Pre-build the Orchard proving key
///
/// Call this at application startup to avoid blocking during transaction proving.
#[uniffi::export]
pub fn prebuild_proving_key() {
    t2z_core::load_orchard_proving_key();
}

/// Gets the version of the library
#[uniffi::export]
pub fn version() -> String {
    format!("t2z-uniffi v{}", env!("CARGO_PKG_VERSION"))
}

