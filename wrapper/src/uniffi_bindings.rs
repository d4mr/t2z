// UniFFI bindings for Go, Kotlin, Java
//
// This module exposes the PCZT wrapper functionality to other languages via UniFFI.
// Uses procedural macros (not UDL) for cleaner implementation.

use std::sync::Arc;
use crate::{
    Pczt, FfiError,
    propose_transaction, sign_transparent_input,
    combine, finalize_and_extract, parse_pczt, serialize_pczt,
    TransparentInput, Payment, TransactionRequest, Network,
};
use hex;

// ============================================================================
// UniFFI Error Type
// ============================================================================

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniffiError {
    #[error("Error: {msg}")]
    Error { msg: String },
}

impl From<FfiError> for UniffiError {
    fn from(e: FfiError) -> Self {
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
    fn to_internal(&self) -> Result<TransparentInput, UniffiError> {
        let pubkey = hex::decode(&self.pubkey)
            .map_err(|e| UniffiError::Error { msg: format!("Invalid pubkey hex: {}", e) })?;
        
        let prevout_txid = hex::decode(&self.prevout_txid)
            .map_err(|e| UniffiError::Error { msg: format!("Invalid prevout_txid hex: {}", e) })?;
        
        let script_pubkey = hex::decode(&self.script_pubkey)
            .map_err(|e| UniffiError::Error { msg: format!("Invalid script_pubkey hex: {}", e) })?;

        Ok(TransparentInput {
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
    fn to_internal(&self) -> Result<Payment, UniffiError> {
        let memo = if let Some(memo_hex) = &self.memo {
            Some(hex::decode(memo_hex)
                .map_err(|e| UniffiError::Error { msg: format!("Invalid memo hex: {}", e) })?)
        } else {
            None
        };

        Ok(Payment {
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
    fn to_internal(&self) -> Result<TransactionRequest, UniffiError> {
        let payments: Result<Vec<Payment>, UniffiError> = self.payments
            .iter()
            .map(|p| p.to_internal())
            .collect();

        Ok(TransactionRequest {
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
        let pczt = parse_pczt(&bytes)?;
        Ok(Arc::new(UniffiPczt { inner: pczt }))
    }

    /// Serializes the PCZT to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serialize_pczt(&self.inner)
    }

    /// Serializes the PCZT to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(serialize_pczt(&self.inner))
    }
}

// ============================================================================
// UniFFI Exported Functions
// ============================================================================

/// Proposes a transaction from transparent inputs to transparent and/or shielded outputs
#[uniffi::export]
pub fn uniffi_propose_transaction(
    inputs_to_spend: Vec<UniffiTransparentInput>,
    transaction_request: UniffiTransactionRequest,
    network: String,
    expiry_height: u32,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    let inputs: Result<Vec<TransparentInput>, UniffiError> = inputs_to_spend
        .iter()
        .map(|i| i.to_internal())
        .collect();
    let inputs = inputs?;

    let request = transaction_request.to_internal()?;
    
    let network = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "testnet" => Network::Testnet,
        _ => return Err(UniffiError::Error { 
            msg: "Network must be 'mainnet' or 'testnet'".to_string() 
        }),
    };

    let pczt = propose_transaction(&inputs, request, network, expiry_height)?;
    Ok(Arc::new(UniffiPczt { inner: pczt }))
}

/// Proves a transaction with provided proving key
/// 
/// Note: This requires the proving key to be provided separately.
/// Use uniffi_prove_transaction_auto to build the key automatically.
#[uniffi::export]
pub fn uniffi_prove_transaction_with_key(
    _pczt: Arc<UniffiPczt>,
    _proving_key_bytes: Vec<u8>,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    // Note: ProvingKey doesn't support deserialization yet
    // This is a placeholder for when it does
    Err(UniffiError::Error {
        msg: "Proving key deserialization not yet supported by orchard crate. Use uniffi_prove_transaction_auto.".to_string()
    })
}

/// Proves a transaction (builds proving key automatically, ~10 seconds first call)
#[uniffi::export]
pub fn uniffi_prove_transaction_auto(
    pczt: Arc<UniffiPczt>,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    let proved = crate::prove_transaction(pczt.inner.clone())?;
    Ok(Arc::new(UniffiPczt { inner: proved }))
}

/// Signs a transparent input with the provided private key
#[uniffi::export]
pub fn uniffi_sign_transparent_input(
    pczt: Arc<UniffiPczt>,
    input_index: u32,
    secret_key_hex: String,
) -> Result<Arc<UniffiPczt>, UniffiError> {
    let secret_key_bytes = hex::decode(&secret_key_hex)
        .map_err(|e| UniffiError::Error { msg: format!("Invalid secret key hex: {}", e) })?;

    if secret_key_bytes.len() != 32 {
        return Err(UniffiError::Error { msg: "Secret key must be 32 bytes".to_string() });
    }

    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&secret_key_bytes);

    let signed = sign_transparent_input(pczt.inner.clone(), input_index as usize, &secret_key)?;
    Ok(Arc::new(UniffiPczt { inner: signed }))
}

/// Combines multiple PCZTs into one
#[uniffi::export]
pub fn uniffi_combine(pczt_list: Vec<Arc<UniffiPczt>>) -> Result<Arc<UniffiPczt>, UniffiError> {
    let pczts: Vec<Pczt> = pczt_list.iter().map(|p| p.inner.clone()).collect();
    let combined = combine(pczts)?;
    Ok(Arc::new(UniffiPczt { inner: combined }))
}

/// Finalizes the PCZT and extracts the transaction bytes
#[uniffi::export]
pub fn uniffi_finalize_and_extract(pczt: Arc<UniffiPczt>) -> Result<Vec<u8>, UniffiError> {
    let tx_bytes = finalize_and_extract(pczt.inner.clone())?;
    Ok(tx_bytes)
}

/// Parses a PCZT from hex string
#[uniffi::export]
pub fn uniffi_parse_pczt_hex(hex_string: String) -> Result<Arc<UniffiPczt>, UniffiError> {
    let bytes = hex::decode(&hex_string)
        .map_err(|e| UniffiError::Error { msg: format!("Invalid hex: {}", e) })?;
    
    let pczt = parse_pczt(&bytes)?;
    Ok(Arc::new(UniffiPczt { inner: pczt }))
}

/// Gets the version of the library
#[uniffi::export]
pub fn uniffi_version() -> String {
    format!("pczt-wrapper v{}", env!("CARGO_PKG_VERSION"))
}
