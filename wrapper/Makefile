.PHONY: help build build-napi build-uniffi test clean

help:
	@echo "PCZT Wrapper Build Targets:"
	@echo "  make build        - Build all variants"
	@echo "  make build-napi   - Build with NAPI bindings (TypeScript/Node.js)"
	@echo "  make build-uniffi - Build with UniFFI bindings (Go/Kotlin/Java)"
	@echo "  make test         - Run tests"
	@echo "  make clean        - Clean build artifacts"

build: build-napi build-uniffi

build-napi:
	@echo "Building with NAPI bindings..."
	cargo build --release --features napi-bindings

build-uniffi:
	@echo "Building with UniFFI bindings..."
	cargo build --release --features uniffi-bindings
	@echo "Generating UniFFI scaffolding..."
	cargo run --features uniffi-bindings --bin uniffi-bindgen generate src/pczt_wrapper.udl --language kotlin --out-dir ../sdks/kotlin/generated || true
	cargo run --features uniffi-bindings --bin uniffi-bindgen generate src/pczt_wrapper.udl --language go --out-dir ../sdks/go/generated || true

test:
	@echo "Running tests..."
	cargo test
	cargo test --features napi-bindings
	cargo test --features uniffi-bindings

clean:
	@echo "Cleaning build artifacts..."
	cargo clean

