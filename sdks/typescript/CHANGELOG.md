# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-14

### Added
- Initial release of @d4mr/t2z TypeScript SDK
- **T2Z** (Transparent to Zero-knowledge) class for building Zcash transactions
- Full TypeScript support with strict type checking
- Zod validation for all inputs
- Comprehensive error handling with specific error types
- Support for building transactions from transparent inputs to transparent/shielded outputs
- Halo 2 proving (no downloads required!)
- Transparent input signing with ZIP 244 compliance
- Transaction combination for multi-party transactions
- Transaction finalization and extraction
- Serialization support (bytes, hex, base64)
- ZIP 244, 321, 374 compliance
- Production-ready code with no shortcuts
- Comprehensive documentation and examples
- Backward compatibility: PCZT exported as alias for T2Z

### Features
- ✅ Type-safe API with full TypeScript support
- ✅ Zod schema validation
- ✅ Comprehensive error handling
- ✅ ZIP 244 transparent signature hash
- ✅ ZIP 321 payment requests
- ✅ ZIP 374 PCZT format
- ✅ Halo 2 proving (no trusted setup)
- ✅ Orchard shielded outputs
- ✅ Multi-party transaction support
- ✅ Serialization in multiple formats

[0.1.0]: https://github.com/d4mr/pczt/releases/tag/v0.1.0
