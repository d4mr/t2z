fn main() {
    // Build NAPI bindings when the feature is enabled
    #[cfg(feature = "napi-bindings")]
    {
        extern crate napi_build;
        napi_build::setup();
    }

    // UniFFI bindings are now handled by procedural macros
    // No build script needed for uniffi with macros
}

