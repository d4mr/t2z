// NAPI bindings for TypeScript/Node.js/WASM
//
// This module exposes the PCZT wrapper functionality to Node.js and browsers
// via napi-rs.

#[cfg(feature = "napi-bindings")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi-bindings")]
use napi_derive::napi;

#[cfg(feature = "napi-bindings")]
use crate::*;

// ============================================================================
// Type Conversions for NAPI
// ============================================================================

/// NAPI-compatible transparent input
#[cfg(feature = "napi-bindings")]
#[napi(object)]
#[derive(Debug, Clone)]
pub struct NapiTransparentInput {
    /// Public key (33 bytes as hex string)
    pub pubkey: String,
    /// Previous transaction ID (32 bytes as hex string)
    pub prevout_txid: String,
    /// Previous output index
    pub prevout_index: u32,
    /// Value in zatoshis
    pub value: i64,
    /// Script pubkey (hex encoded)
    pub script_pubkey: String,
    /// Optional sequence number
    pub sequence: Option<u32>,
}

#[cfg(feature = "napi-bindings")]
impl NapiTransparentInput {
    fn to_internal(&self) -> Result<TransparentInput> {
        let pubkey = hex::decode(&self.pubkey)
            .map_err(|e| Error::from_reason(format!("Invalid pubkey hex: {}", e)))?;
        
        let prevout_txid = hex::decode(&self.prevout_txid)
            .map_err(|e| Error::from_reason(format!("Invalid prevout_txid hex: {}", e)))?;
        
        let script_pubkey = hex::decode(&self.script_pubkey)
            .map_err(|e| Error::from_reason(format!("Invalid script_pubkey hex: {}", e)))?;

        Ok(TransparentInput {
            pubkey,
            prevout_txid,
            prevout_index: self.prevout_index,
            value: self.value as u64,
            script_pubkey,
            sequence: self.sequence,
        })
    }
}

/// NAPI-compatible payment (supports both transparent and shielded)
#[cfg(feature = "napi-bindings")]
#[napi(object)]
#[derive(Debug, Clone)]
pub struct NapiPayment {
    /// Address (transparent P2PKH/P2SH or unified with Orchard)
    pub address: String,
    /// Value in zatoshis
    pub value: i64,
    /// Optional memo (hex encoded, max 512 bytes)
    pub memo: Option<String>,
    /// Optional label
    pub label: Option<String>,
}

#[cfg(feature = "napi-bindings")]
impl NapiPayment {
    fn to_internal(&self) -> Result<Payment> {
        let memo = if let Some(memo_hex) = &self.memo {
            Some(hex::decode(memo_hex)
                .map_err(|e| Error::from_reason(format!("Invalid memo hex: {}", e)))?)
        } else {
            None
        };

        Ok(Payment {
            address: self.address.clone(),
            amount: self.value as u64,
            memo,
            label: self.label.clone(),
        })
    }
}

/// NAPI-compatible transaction request
#[cfg(feature = "napi-bindings")]
#[napi(object)]
#[derive(Debug, Clone)]
pub struct NapiTransactionRequest {
    /// List of payments
    pub payments: Vec<NapiPayment>,
    /// Optional fee in zatoshis
    pub fee: Option<i64>,
}

#[cfg(feature = "napi-bindings")]
impl NapiTransactionRequest {
    fn to_internal(&self) -> Result<TransactionRequest> {
        let payments: Result<Vec<Payment>> = self.payments
            .iter()
            .map(|p| p.to_internal())
            .collect();

        Ok(TransactionRequest {
            payments: payments?,
            fee: self.fee.map(|f| f as u64),
        })
    }
}

// ============================================================================
// NAPI Functions
// ============================================================================

/// Proposes a transaction from transparent inputs to transparent and/or shielded outputs
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_propose_transaction(
    inputs_to_spend: Vec<NapiTransparentInput>,
    transaction_request: NapiTransactionRequest,
    network: String,
    expiry_height: u32,
) -> Result<Buffer> {
    let inputs: Result<Vec<TransparentInput>> = inputs_to_spend
        .iter()
        .map(|i| i.to_internal())
        .collect();
    let inputs = inputs?;

    let request = transaction_request.to_internal()?;
    
    let network = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "testnet" => Network::Testnet,
        _ => return Err(Error::from_reason("Network must be 'mainnet' or 'testnet'")),
    };

    let pczt = propose_transaction(&inputs, request, network, expiry_height)
        .map_err(|e| Error::from_reason(format!("Failed to propose transaction: {}", e)))?;

    let bytes = serialize_pczt(&pczt);
    Ok(Buffer::from(bytes))
}

/// Proves a transaction (builds proving key on first call, then caches)
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_prove_transaction(pczt_bytes: Buffer) -> Result<Buffer> {
    let pczt = parse_pczt(&pczt_bytes)
        .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))?;

    let pczt = prove_transaction(pczt)
        .map_err(|e| Error::from_reason(format!("Failed to prove transaction: {}", e)))?;

    let bytes = serialize_pczt(&pczt);
    Ok(Buffer::from(bytes))
}

/// Signs a transparent input with the provided private key
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_sign_transparent_input(
    pczt_bytes: Buffer,
    input_index: u32,
    secret_key_hex: String,
) -> Result<Buffer> {
    let pczt = parse_pczt(&pczt_bytes)
        .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))?;

    let secret_key_bytes = hex::decode(&secret_key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid secret key hex: {}", e)))?;

    if secret_key_bytes.len() != 32 {
        return Err(Error::from_reason("Secret key must be 32 bytes"));
    }

    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&secret_key_bytes);

    let pczt = sign_transparent_input(pczt, input_index as usize, &secret_key)
        .map_err(|e| Error::from_reason(format!("Failed to sign input: {}", e)))?;

    let bytes = serialize_pczt(&pczt);
    Ok(Buffer::from(bytes))
}

/// Combines multiple PCZTs into one
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_combine(pczt_bytes_list: Vec<Buffer>) -> Result<Buffer> {
    let pczts: Result<Vec<Pczt>> = pczt_bytes_list
        .iter()
        .map(|bytes| {
            parse_pczt(bytes)
                .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))
        })
        .collect();

    let pczts = pczts?;

    let combined = combine(pczts)
        .map_err(|e| Error::from_reason(format!("Failed to combine PCZTs: {}", e)))?;

    let bytes = serialize_pczt(&combined);
    Ok(Buffer::from(bytes))
}

/// Finalizes the PCZT and extracts the transaction bytes
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_finalize_and_extract(pczt_bytes: Buffer) -> Result<Buffer> {
    let pczt = parse_pczt(&pczt_bytes)
        .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))?;

    let tx_bytes = finalize_and_extract(pczt)
        .map_err(|e| Error::from_reason(format!("Failed to finalize transaction: {}", e)))?;

    Ok(Buffer::from(tx_bytes))
}

/// Parses a PCZT from bytes
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_parse_pczt(pczt_bytes: Buffer) -> Result<Buffer> {
    let pczt = parse_pczt(&pczt_bytes)
        .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))?;

    let bytes = serialize_pczt(&pczt);
    Ok(Buffer::from(bytes))
}

/// Serializes a PCZT to bytes
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_serialize_pczt(pczt_bytes: Buffer) -> Result<Buffer> {
    // Already in serialized form, but validate by parsing
    let pczt = parse_pczt(&pczt_bytes)
        .map_err(|e| Error::from_reason(format!("Failed to parse PCZT: {}", e)))?;

    let bytes = serialize_pczt(&pczt);
    Ok(Buffer::from(bytes))
}

/// Gets the version of the library
#[cfg(feature = "napi-bindings")]
#[napi]
pub fn napi_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
