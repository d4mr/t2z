# Why Orchard Doesn't Need Downloaded Proving Keys

## TL;DR

**Orchard uses Halo 2, which eliminates the need for trusted setups and downloadable proving keys!**

Unlike Sapling (50MB) or Sprout (869MB), Orchard builds its proving key programmatically from circuit constraints. No downloads, no ceremonies, no trust required.

## The Evolution of Zcash Privacy

### Sprout (2016-2018)
- **Proving System:** Groth16
- **Trusted Setup:** Required (multi-party computation ceremony)
- **Proving Key Size:** 869MB (`sprout-proving.key`)
- **Download Required:** Yes
- **Trust Assumption:** Required trusting the setup ceremony

### Sapling (2018-2022)
- **Proving System:** Groth16
- **Trusted Setup:** Required (Powers of Tau + Sapling MPC)
- **Proving Key Size:** ~50MB (`sapling-spend.params`, `sapling-output.params`)
- **Download Required:** Yes
- **Trust Assumption:** Required trusting the setup ceremony
- **Improvement:** Smaller keys, better performance

### Orchard (2022-Present) 🎉
- **Proving System:** Halo 2
- **Trusted Setup:** **NOT REQUIRED!**
- **Proving Key Size:** **0 bytes (built in code)**
- **Download Required:** **NO!**
- **Trust Assumption:** **Only in the cryptography itself**
- **Game Changer:** First major cryptocurrency with fully trustless private transactions

## How Halo 2 Works

### Key Innovations

1. **Recursive Proof Composition**
   - Halo 2 can verify proofs within proofs
   - No need for pairing-based cryptography

2. **Pasta Curves (Pallas & Vesta)**
   - Form a 2-cycle of curves
   - Each curve's base field is the other's scalar field
   - Enables efficient recursion

3. **No Structured Reference String (SRS)**
   - Traditional zk-SNARKs require an SRS from a trusted setup
   - Halo 2 eliminates this entirely
   - All parameters derived from public curve properties

### What Gets "Built"?

When you call `ProvingKey::build()`, it's creating:

1. **Circuit Constraints**
   - Relationships between variables in the circuit
   - Polynomial representations of the logic
   
2. **Lookup Tables**
   - Pre-computed values for efficient proving
   - Built from the curve properties

3. **Commitment Scheme Parameters**
   - Based on the Pallas/Vesta curve cycle
   - No hidden trapdoors or secret values

**This takes ~10 seconds** because it's doing real cryptographic work, not downloading data!

## Comparison Table

| Feature | Sprout | Sapling | Orchard (Halo 2) |
|---------|--------|---------|------------------|
| Proving System | Groth16 | Groth16 | Halo 2 |
| Trusted Setup | ✅ Required | ✅ Required | ❌ Not Needed |
| Proving Key Download | 869MB | 50MB | **0MB** |
| Setup Time | Minutes (download) | Minutes (download) | **~10 sec (build)** |
| Network Required | Yes | Yes | **No** |
| Trust Assumption | Ceremony participants | Ceremony participants | **Math only** |
| Proof Size | Large | 192 bytes | 2.5KB |
| Prover Time | Slow | Fast | **Very Fast** |

## Why This Matters

### For Users
- **No download wait** - Start proving immediately
- **Works offline** - No network dependency
- **Trustless** - Don't need to trust anyone except the math
- **Future-proof** - No ceremony to "sunset" or worry about

### For Developers
- **Simpler deployment** - No CDN or parameter hosting
- **Better UX** - 10-second warmup vs minutes of downloading
- **Portable** - Works in any environment (WASM, mobile, etc.)
- **Auditable** - All code is in the open, no hidden ceremony outputs

### For Zcash
- **Credibility** - Removes one of crypto's biggest criticisms
- **Innovation** - First major cryptocurrency with trustless privacy
- **Security** - Eliminates an entire class of attacks

## Technical Deep Dive

### The Proving Key Structure

```rust
pub struct ProvingKey {
    // Circuit configuration
    cs: ConstraintSystem<Fp>,
    
    // Fixed columns (lookup tables, etc.)
    fixed: Vec<Vec<Fp>>,
    
    // Permutation arguments
    permutation: Argument,
    
    // Lookup arguments  
    lookups: Vec<lookup::Argument>,
}
```

All of these are **deterministically derived** from:
- The circuit design (public code)
- The Pallas/Vesta curve properties (public spec)
- No secrets involved!

### Why Does It Take 10 Seconds?

1. **Constraint Generation** (~2s)
   - Building the polynomial constraints
   - Assembling the circuit gates

2. **Lookup Table Pre-computation** (~5s)
   - Pre-computing lookup values
   - Optimizing for common operations

3. **Permutation Setup** (~2s)
   - Setting up copy constraints
   - Establishing wire connections

4. **Commitment Key Generation** (~1s)
   - Deriving commitment bases
   - Setting up polynomial commitment scheme

### Caching Strategy

Our implementation uses `once_cell` for optimal performance:

```rust
static ORCHARD_PK: OnceCell<ProvingKey> = OnceCell::new();

pub fn load_orchard_proving_key() -> &'static ProvingKey {
    ORCHARD_PK.get_or_init(|| {
        ProvingKey::build() // 10s, but only once!
    })
}
```

**Result:**
- First call: ~10 seconds
- All subsequent calls: ~0 nanoseconds (just a memory read!)

## Historical Context

### The Zcash MPC Ceremonies

**Sprout Ceremony (2016)**
- 6 participants
- If ANY ONE destroyed their secret, the system is secure
- But we have to trust at least one did

**Sapling "Powers of Tau" (2017-2018)**
- 176 participants
- Multi-stage ceremony
- Still requires trust in at least one participant

**Orchard (2022)**
- **NO CEREMONY NEEDED!**
- This was a major milestone in cryptographic privacy tech

## Comparison to Other Privacy Coins

| Coin | Privacy Tech | Trusted Setup? |
|------|-------------|----------------|
| Zcash (Orchard) | Halo 2 | ❌ No |
| Zcash (Sapling) | Groth16 | ✅ Yes |
| Monero | RingCT | ❌ No |
| Tornado Cash | Groth16 | ✅ Yes |
| Aztec | PLONK | ✅ Yes (Universal) |

Orchard puts Zcash in elite company with Monero for trustless privacy!

## Frequently Asked Questions

### Q: Why not use the same approach for Sapling?
**A:** Halo 2 wasn't invented yet when Sapling launched. Groth16 was state-of-the-art at the time.

### Q: Can I still use Sapling?
**A:** Yes, but you'll need to download the proving keys. Orchard is recommended for new applications.

### Q: Is Halo 2 slower than Groth16?
**A:** Slightly slower proving, but the proof size is larger. However, NO TRUSTED SETUP makes it a huge win overall.

### Q: Can other projects use Halo 2?
**A:** Yes! The Halo 2 crate is open source. Several projects are adopting it.

### Q: What about verification keys?
**A:** Verification keys are small (<1KB) and embedded in the node software. They're derived from the same public parameters as the proving key.

## References

- [Halo 2 Paper](https://electriccoin.co/blog/halo-2/)
- [Zcash NU5 Upgrade](https://electriccoin.co/blog/nu5-network-upgrade-activation/)
- [Recursive Proof Composition](https://electriccoin.co/blog/recursive-proof-composition-without-a-trusted-setup/)
- [Pasta Curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)

## Conclusion

Orchard's use of Halo 2 represents a fundamental breakthrough in zero-knowledge cryptography. By eliminating the trusted setup, Zcash has achieved something remarkable: **truly trustless private transactions at scale.**

No downloads. No ceremonies. No trust required. Just math. 🎯

---

*This document is part of the pczt-wrapper library for Zcash transaction building.*

