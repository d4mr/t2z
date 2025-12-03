//! T2Z WASM - WebAssembly bindings for T2Z
//!
//! This crate provides WebAssembly bindings for the T2Z library,
//! enabling Zcash transparent-to-shielded transactions in browsers and Node.js.
//!
//! Built with wasm-pack for easy consumption in JavaScript/TypeScript.

use wasm_bindgen::prelude::*;

mod utils;

// Re-export core types for documentation
pub use t2z_core::{Network, Payment, T2ZError, TransactionRequest, TransparentInput};

// ============================================================================
// Initialization
// ============================================================================

/// Initialize the WASM module. Call this once at startup.
/// Sets up panic hooks for better error messages in the console.
#[wasm_bindgen(start)]
pub fn init() {
    utils::set_panic_hook();
}

/// Manually initialize panic hooks (alternative to auto-init)
#[wasm_bindgen]
pub fn init_panic_hook() {
    utils::set_panic_hook();
}

// ============================================================================
// Proving Key Management
// ============================================================================

/// Pre-build the Orchard proving key.
///
/// This is an expensive operation (~10 seconds) that builds the Halo 2 circuit.
/// Call this at application startup or in a web worker to avoid blocking the UI.
///
/// The proving key is cached globally, so subsequent calls are instant.
///
/// # Important
/// Unlike Sapling which requires downloading ~50MB proving keys,
/// Orchard uses Halo 2 and builds the circuit programmatically - no downloads needed!
#[wasm_bindgen]
pub fn prebuild_proving_key() {
    t2z_core::load_orchard_proving_key();
}

/// Check if the proving key has been built and cached.
#[wasm_bindgen]
pub fn is_proving_key_ready() -> bool {
    t2z_core::is_proving_key_loaded()
}

// ============================================================================
// WASM-friendly Input Types
// ============================================================================

/// Transparent input for transaction construction (WASM-friendly)
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmTransparentInput {
    /// Public key (33 bytes as hex string)
    pubkey: String,
    /// Previous transaction ID (32 bytes as hex string)
    prevout_txid: String,
    /// Previous output index
    prevout_index: u32,
    /// Value in zatoshis
    value: u64,
    /// Script pubkey (hex encoded)
    script_pubkey: String,
    /// Optional sequence number
    sequence: Option<u32>,
}

#[wasm_bindgen]
impl WasmTransparentInput {
    #[wasm_bindgen(constructor)]
    pub fn new(
        pubkey: String,
        prevout_txid: String,
        prevout_index: u32,
        value: u64,
        script_pubkey: String,
        sequence: Option<u32>,
    ) -> Self {
        Self {
            pubkey,
            prevout_txid,
            prevout_index,
            value,
            script_pubkey,
            sequence,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> String {
        self.pubkey.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn prevout_txid(&self) -> String {
        self.prevout_txid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn prevout_index(&self) -> u32 {
        self.prevout_index
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> u64 {
        self.value
    }

    #[wasm_bindgen(getter)]
    pub fn script_pubkey(&self) -> String {
        self.script_pubkey.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn sequence(&self) -> Option<u32> {
        self.sequence
    }
}

impl WasmTransparentInput {
    fn to_core(&self) -> Result<t2z_core::TransparentInput, JsError> {
        let pubkey = hex::decode(&self.pubkey)
            .map_err(|e| JsError::new(&format!("Invalid pubkey hex: {}", e)))?;

        let prevout_txid = hex::decode(&self.prevout_txid)
            .map_err(|e| JsError::new(&format!("Invalid prevout_txid hex: {}", e)))?;

        let script_pubkey = hex::decode(&self.script_pubkey)
            .map_err(|e| JsError::new(&format!("Invalid script_pubkey hex: {}", e)))?;

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

/// Payment for transaction construction (WASM-friendly)
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPayment {
    /// Address (transparent P2PKH/P2SH or unified with Orchard)
    address: String,
    /// Value in zatoshis
    amount: u64,
    /// Optional memo (hex encoded, max 512 bytes)
    memo: Option<String>,
    /// Optional label
    label: Option<String>,
}

#[wasm_bindgen]
impl WasmPayment {
    #[wasm_bindgen(constructor)]
    pub fn new(
        address: String,
        amount: u64,
        memo: Option<String>,
        label: Option<String>,
    ) -> Self {
        Self {
            address,
            amount,
            memo,
            label,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    #[wasm_bindgen(getter)]
    pub fn memo(&self) -> Option<String> {
        self.memo.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn label(&self) -> Option<String> {
        self.label.clone()
    }
}

impl WasmPayment {
    fn to_core(&self) -> Result<t2z_core::Payment, JsError> {
        let memo = if let Some(memo_hex) = &self.memo {
            Some(
                hex::decode(memo_hex)
                    .map_err(|e| JsError::new(&format!("Invalid memo hex: {}", e)))?,
            )
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

// ============================================================================
// Expected TxOut (for verify_before_signing)
// ============================================================================

/// Expected transaction output for verification
/// Per spec: verify_before_signing takes expected_change: [TxOut]
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmExpectedTxOut {
    /// Address (transparent or Orchard unified address)
    address: String,
    /// Value in zatoshis
    amount: u64,
}

#[wasm_bindgen]
impl WasmExpectedTxOut {
    #[wasm_bindgen(constructor)]
    pub fn new(address: String, amount: u64) -> Self {
        Self { address, amount }
    }

    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl WasmExpectedTxOut {
    fn to_core(&self) -> t2z_core::ExpectedTxOut {
        t2z_core::ExpectedTxOut {
            address: self.address.clone(),
            amount: self.amount,
        }
    }
}

// ============================================================================
// PCZT Wrapper
// ============================================================================

/// A Partially Constructed Zcash Transaction (PCZT)
///
/// This wraps the internal PCZT representation and provides methods
/// for proving, signing, combining, and finalizing transactions.
#[wasm_bindgen]
pub struct WasmPczt {
    inner: t2z_core::Pczt,
}

#[wasm_bindgen]
impl WasmPczt {
    /// Parse a PCZT from bytes
    #[wasm_bindgen(constructor)]
    pub fn from_bytes(bytes: &[u8]) -> Result<WasmPczt, JsError> {
        let pczt = t2z_core::parse_pczt(bytes)
            .map_err(|e| JsError::new(&format!("Failed to parse PCZT: {}", e)))?;
        Ok(WasmPczt { inner: pczt })
    }

    /// Parse a PCZT from a hex string
    #[wasm_bindgen]
    pub fn from_hex(hex_string: &str) -> Result<WasmPczt, JsError> {
        let bytes = hex::decode(hex_string)
            .map_err(|e| JsError::new(&format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Serialize the PCZT to bytes
    #[wasm_bindgen]
    pub fn to_bytes(&self) -> Vec<u8> {
        t2z_core::serialize_pczt(&self.inner)
    }

    /// Serialize the PCZT to a hex string
    #[wasm_bindgen]
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Clone this PCZT
    #[wasm_bindgen]
    pub fn clone_pczt(&self) -> WasmPczt {
        WasmPczt {
            inner: self.inner.clone(),
        }
    }
}

// ============================================================================
// Core API Functions
// ============================================================================

/// Propose a transaction from transparent inputs to transparent and/or shielded outputs.
///
/// This implements the Creator, Constructor, and IO Finalizer roles per ZIP 374.
///
/// # Arguments
/// * `inputs` - Array of transparent inputs to spend
/// * `payments` - Array of payments (outputs)
/// * `fee` - Optional fee in zatoshis (calculated automatically if not provided)
/// * `change_address` - Optional transparent address for change (required if there's leftover)
/// * `network` - "mainnet" or "testnet"
/// * `expiry_height` - Block height at which transaction expires
///
/// # Returns
/// A PCZT ready for proving and signing
#[wasm_bindgen]
pub fn propose_transaction(
    inputs: Vec<WasmTransparentInput>,
    payments: Vec<WasmPayment>,
    change_address: Option<String>,
    network: &str,
    expiry_height: u32,
) -> Result<WasmPczt, JsError> {
    let core_inputs: Result<Vec<t2z_core::TransparentInput>, JsError> =
        inputs.iter().map(|i| i.to_core()).collect();
    let core_inputs = core_inputs?;

    let core_payments: Result<Vec<t2z_core::Payment>, JsError> =
        payments.iter().map(|p| p.to_core()).collect();
    let core_payments = core_payments?;

    let network = match network {
        "mainnet" => t2z_core::Network::Mainnet,
        "testnet" => t2z_core::Network::Testnet,
        _ => return Err(JsError::new("Network must be 'mainnet' or 'testnet'")),
    };

    let request = t2z_core::TransactionRequest {
        payments: core_payments,
    };

    let pczt = t2z_core::propose_transaction(
        &core_inputs,
        request,
        change_address.as_deref(),
        network,
        expiry_height,
    )
    .map_err(|e| JsError::new(&format!("Failed to propose transaction: {}", e)))?;

    Ok(WasmPczt { inner: pczt })
}

/// Prove the transaction (adds Orchard proofs).
///
/// This builds the Halo 2 circuit proving key on first call (~10 seconds),
/// then caches it for subsequent calls. No downloads required!
///
/// # Arguments
/// * `pczt` - The PCZT to prove
///
/// # Returns
/// The PCZT with proofs added
#[wasm_bindgen]
pub fn prove_transaction(pczt: &WasmPczt) -> Result<WasmPczt, JsError> {
    let proved = t2z_core::prove_transaction(pczt.inner.clone())
        .map_err(|e| JsError::new(&format!("Failed to prove transaction: {}", e)))?;
    Ok(WasmPczt { inner: proved })
}

/// Sign a transparent input with the provided private key.
///
/// This is a convenience function that combines `get_sighash` and signing internally.
/// For external signing (HSM/hardware wallets), use `get_sighash` and `append_signature`.
///
/// # Arguments
/// * `pczt` - The PCZT to sign
/// * `input_index` - Index of the transparent input to sign
/// * `secret_key_hex` - 32-byte private key as hex string
///
/// # Returns
/// The PCZT with the signature added
#[wasm_bindgen]
pub fn sign_transparent_input(
    pczt: &WasmPczt,
    input_index: u32,
    secret_key_hex: &str,
) -> Result<WasmPczt, JsError> {
    let secret_key_bytes = hex::decode(secret_key_hex)
        .map_err(|e| JsError::new(&format!("Invalid secret key hex: {}", e)))?;

    if secret_key_bytes.len() != 32 {
        return Err(JsError::new("Secret key must be 32 bytes"));
    }

    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&secret_key_bytes);

    let signed =
        t2z_core::sign_transparent_input(pczt.inner.clone(), input_index as usize, &secret_key)
            .map_err(|e| JsError::new(&format!("Failed to sign input: {}", e)))?;

    Ok(WasmPczt { inner: signed })
}

/// Get the sighash for a transparent input (ZIP 244).
///
/// Use this for external signing (HSM/hardware wallets):
/// 1. Call `get_sighash` to get the 32-byte hash
/// 2. Sign the hash externally with ECDSA secp256k1
/// 3. Call `append_signature` with the result
///
/// # Arguments
/// * `pczt` - The PCZT
/// * `input_index` - Index of the transparent input
///
/// # Returns
/// 32-byte sighash as hex string
#[wasm_bindgen]
pub fn get_sighash(pczt: &WasmPczt, input_index: u32) -> Result<String, JsError> {
    let sighash = t2z_core::get_sighash(&pczt.inner, input_index as usize)
        .map_err(|e| JsError::new(&format!("Failed to get sighash: {}", e)))?;
    Ok(hex::encode(sighash))
}

/// Append a pre-computed signature to a transparent input.
///
/// The signature should be created by signing the output of `get_sighash`
/// with ECDSA secp256k1, then appending the sighash type byte (0x01 for SIGHASH_ALL).
///
/// # Arguments
/// * `pczt` - The PCZT to update
/// * `input_index` - Index of the transparent input
/// * `pubkey_hex` - 33-byte compressed public key as hex
/// * `signature_hex` - DER-encoded signature + sighash type byte as hex
///
/// # Returns
/// Updated PCZT with the signature added
#[wasm_bindgen]
pub fn append_signature(
    pczt: &WasmPczt,
    input_index: u32,
    pubkey_hex: &str,
    signature_hex: &str,
) -> Result<WasmPczt, JsError> {
    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| JsError::new(&format!("Invalid pubkey hex: {}", e)))?;

    if pubkey_bytes.len() != 33 {
        return Err(JsError::new("Public key must be 33 bytes (compressed)"));
    }

    let mut pubkey = [0u8; 33];
    pubkey.copy_from_slice(&pubkey_bytes);

    let signature = hex::decode(signature_hex)
        .map_err(|e| JsError::new(&format!("Invalid signature hex: {}", e)))?;

    let updated = t2z_core::append_signature(
        pczt.inner.clone(),
        input_index as usize,
        &pubkey,
        &signature,
    )
    .map_err(|e| JsError::new(&format!("Failed to append signature: {}", e)))?;

    Ok(WasmPczt { inner: updated })
}

/// Verify the PCZT matches the original transaction request before signing.
///
/// This is an important security check for multi-party transaction construction.
/// It verifies:
/// - All requested payments are present
/// - No unexpected outputs were added
/// - Change output matches expectations (if any)
///
/// # Arguments
/// * `pczt` - The PCZT to verify
/// * `payments` - The original payments array used to create the PCZT
/// * `change_address` - Expected change address (optional)
/// * `change_amount` - Expected change amount in zatoshis (optional)
///
/// # Returns
/// Ok if verification passes, error with details otherwise
#[wasm_bindgen]
pub fn verify_before_signing(
    pczt: &WasmPczt,
    payments: Vec<WasmPayment>,
    expected_change: Vec<WasmExpectedTxOut>,
) -> Result<(), JsError> {
    let core_payments: Result<Vec<t2z_core::Payment>, JsError> =
        payments.iter().map(|p| p.to_core()).collect();
    let core_payments = core_payments?;

    let core_expected_change: Vec<t2z_core::ExpectedTxOut> =
        expected_change.iter().map(|c| c.to_core()).collect();

    let request = t2z_core::TransactionRequest {
        payments: core_payments,
    };

    t2z_core::verify_before_signing(&pczt.inner, &request, &core_expected_change)
        .map_err(|e| JsError::new(&format!("Verification failed: {}", e)))
}

/// Combine multiple PCZTs into one.
///
/// Useful for multi-party transaction construction.
///
/// # Arguments
/// * `pczts` - Array of PCZTs to combine
///
/// # Returns
/// Combined PCZT
#[wasm_bindgen]
pub fn combine(pczts: Vec<WasmPczt>) -> Result<WasmPczt, JsError> {
    let core_pczts: Vec<t2z_core::Pczt> = pczts.into_iter().map(|p| p.inner).collect();

    let combined = t2z_core::combine(core_pczts)
        .map_err(|e| JsError::new(&format!("Failed to combine PCZTs: {}", e)))?;

    Ok(WasmPczt { inner: combined })
}

/// Finalize the PCZT and extract the raw transaction bytes.
///
/// This implements the Spend Finalizer and Transaction Extractor roles.
/// The returned bytes are ready to be broadcast to the Zcash network.
///
/// # Arguments
/// * `pczt` - The fully signed and proved PCZT
///
/// # Returns
/// Raw transaction bytes ready for broadcast
#[wasm_bindgen]
pub fn finalize_and_extract(pczt: &WasmPczt) -> Result<Vec<u8>, JsError> {
    t2z_core::finalize_and_extract(pczt.inner.clone())
        .map_err(|e| JsError::new(&format!("Failed to finalize transaction: {}", e)))
}

/// Finalize and extract as hex string (convenience method)
#[wasm_bindgen]
pub fn finalize_and_extract_hex(pczt: &WasmPczt) -> Result<String, JsError> {
    let bytes = finalize_and_extract(pczt)?;
    Ok(hex::encode(bytes))
}

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================================
// Test Address Generation
// ============================================================================

/// Generate a random test Orchard address.
///
/// This generates a new random spending key and derives a unified address
/// containing an Orchard receiver. Useful for testing.
///
/// # Arguments
/// * `network` - "mainnet" or "testnet"
///
/// # Returns
/// A valid unified address string with an Orchard receiver
///
/// # Warning
/// The spending key is discarded - you cannot spend funds sent to this address!
/// Only use for testing receive functionality.
#[wasm_bindgen]
pub fn generate_test_address(network: &str) -> Result<String, JsError> {
    use orchard::keys::{FullViewingKey, Scope, SpendingKey};
    use rand_core::RngCore;
    use zcash_address::unified::{self, Encoding};
    use zcash_protocol::consensus::NetworkType;

    let network_type = match network {
        "mainnet" => NetworkType::Main,
        "testnet" => NetworkType::Test,
        _ => return Err(JsError::new("Network must be 'mainnet' or 'testnet'")),
    };

    // Generate random bytes for spending key
    let mut rng = rand_core::OsRng;

    // Create spending key from random bytes (loop until valid)
    let sk: SpendingKey = loop {
        let mut attempt = [0u8; 32];
        rng.fill_bytes(&mut attempt);
        let ct_option = SpendingKey::from_bytes(attempt);
        if ct_option.is_some().into() {
            break ct_option.unwrap();
        }
    };

    // Derive full viewing key and address
    let fvk = FullViewingKey::from(&sk as &SpendingKey);
    let address = fvk.address_at(0u32, Scope::External);

    // Get the raw address bytes
    let orchard_bytes = address.to_raw_address_bytes();

    // Create unified address with just the Orchard receiver
    let ua = unified::Address::try_from_items(vec![unified::Receiver::Orchard(orchard_bytes)])
        .map_err(|e| JsError::new(&format!("Failed to create unified address: {:?}", e)))?;

    // Encode for the network
    let encoded = ua.encode(&network_type);

    Ok(encoded)
}

/// Generate a test keypair (address + spending key).
///
/// Returns an object with:
/// - `address`: Unified address with Orchard receiver
/// - `spending_key`: Hex-encoded spending key (keep secret!)
///
/// # Warning
/// This is for testing only. Store the spending key securely if you want
/// to be able to spend funds sent to the address.
#[wasm_bindgen]
pub fn generate_test_keypair(network: &str) -> Result<JsValue, JsError> {
    use orchard::keys::{FullViewingKey, Scope, SpendingKey};
    use rand_core::RngCore;
    use zcash_address::unified::{self, Encoding};
    use zcash_protocol::consensus::NetworkType;

    let network_type = match network {
        "mainnet" => NetworkType::Main,
        "testnet" => NetworkType::Test,
        _ => return Err(JsError::new("Network must be 'mainnet' or 'testnet'")),
    };

    // Generate random bytes and create spending key (loop until valid)
    let mut rng = rand_core::OsRng;
    let (sk, sk_bytes): (SpendingKey, [u8; 32]) = loop {
        let mut attempt = [0u8; 32];
        rng.fill_bytes(&mut attempt);
        let ct_option = SpendingKey::from_bytes(attempt);
        if ct_option.is_some().into() {
            break (ct_option.unwrap(), attempt);
        }
    };

    // Derive full viewing key and address
    let fvk = FullViewingKey::from(&sk as &SpendingKey);
    let address = fvk.address_at(0u32, Scope::External);

    // Get the raw address bytes
    let orchard_bytes = address.to_raw_address_bytes();

    // Create unified address with just the Orchard receiver
    let ua = unified::Address::try_from_items(vec![unified::Receiver::Orchard(orchard_bytes)])
        .map_err(|e| JsError::new(&format!("Failed to create unified address: {:?}", e)))?;

    // Encode for the network
    let encoded = ua.encode(&network_type);

    // Return as JS object
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"address".into(), &encoded.into())
        .map_err(|_| JsError::new("Failed to set address"))?;
    js_sys::Reflect::set(&obj, &"spending_key".into(), &hex::encode(sk_bytes).into())
        .map_err(|_| JsError::new("Failed to set spending_key"))?;

    Ok(obj.into())
}
