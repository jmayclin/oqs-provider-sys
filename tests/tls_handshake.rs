use std::ffi::{c_char, c_int};

use common::Server;
use openssl_sys::OSSL_PROVIDER_load;

mod common;

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

#[test]
fn handshakes() {
    let provider = unsafe { load_provider() };
    assert!(provider.is_ok());

    // PQ handshake
    {
        let mut server = Server::builder();
        server.ctx().set_groups_list("p256_kyber512").unwrap();
        let server = server.build();

        let mut client = server.client();
        client.ctx().set_groups_list("p256_kyber512").unwrap();
        let s = client.connect().unwrap();
        let state = s.ssl().state_string();
        assert_eq!(state, "SSLOK");
    }

    // failed handshake sanity check. No common groups, should fail to negotiate
    {
        let mut server = Server::builder();
        server.ctx().set_groups_list("P-256").unwrap();
        let server = server.build();

        let mut client = server.client();
        client.ctx().set_groups_list("p256_kyber512").unwrap();
        let s = client.connect();
        assert!(s.is_err());
    }
}
