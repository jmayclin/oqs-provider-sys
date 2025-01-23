# tin-ladybug-oqs-provider-sys

> Note: The "tin ladybug" prefix is a random prefix used to indicate that this crate is not officially associated with the [Open Quantum Safe](https://github.com/open-quantum-safe) project.

This crate provides low level "sys" bindings for the [oqs-provider](https://github.com/open-quantum-safe/oqs-provider). This makes it _relatively_ easy to consume the oqs-provider in a rust application, which allows [rust-openssl](https://github.com/sfackler/rust-openssl) to use PQ algorithms.

# Usage

Currently this crate only consists of low level symbols, which requires consuming applications to use `unsafe` to load the OQS provider.

```rust
extern "C" {
    /// This symbol isn't currently exposed in openssl-sys, so define it here
    /// 
    /// DANGER: the init function pointer is actually a significantly more complicated
    /// type, but we are just using this as "dumb function pointer" right now.
    fn OSSL_PROVIDER_add_builtin(
        ctx: *mut openssl_sys::OSSL_LIB_CTX,
        name: *const c_char,
        init: unsafe extern "C" fn(),
    ) -> c_int;
}

/// This is almost certainly not thread safe. Should be wrapped in a OnceLock
/// or some other handy synchronization primitive when it is being used in rust.
unsafe fn load_provider() -> Result<(), Box<dyn std::error::Error>> {
    //let lib_ctx = openssl_sys::OSSL_LIB_CTX_new();
    // must be null to load the algorithms into the global context.
    let lib_ctx = std::ptr::null_mut();
    let res = OSSL_PROVIDER_add_builtin(
        lib_ctx,
        oqs_provider_sys::OQS_PROV_NAME,
        oqs_provider_sys::oqs_provider_init,
    );
    if res != 1 {
        return Err("unable to add builtin module".into());
    };
    let provider = OSSL_PROVIDER_load(lib_ctx, oqs_provider_sys::OQS_PROV_NAME);
    if provider.is_null() {
        return Err("failed to load provider".into());
    }

    // Load the default provider
    // This is necessary because it appear that loading the oqs provider will
    // cause OpenSSL to fallback to _only_ using explicitly loaded providers,
    // so we have to explicitly load the default provider.
    let default_provider = OSSL_PROVIDER_load(std::ptr::null_mut(), c"default".as_ptr());
    if default_provider.is_null() {
        return Err("Failed to load the default provider".into());
    }

    Ok(())
}
```

# Dangers

## CMake Configs are Very Sticky
I forgot to run cargo clean after updating a submodule, and the new submodule didn't produce a proper cmake config, so my provider-sys crate was just picking up the old cmake config.

## git submodules are sticky
If you git update the main body, it won't bring the crate along with it.

# TODO External PRs

## liboqs-rust
Needs to install the target rather than just building it for things to be findable from other crates.

## liboqs-provider
Should expose the init functions and the module name in the header
