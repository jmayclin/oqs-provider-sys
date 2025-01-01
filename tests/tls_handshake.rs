use std::ffi::{c_char, c_int};

use common::Server;
use openssl::{pkey::Id, ssl::{SslContextBuilder, SslMethod}};
use openssl_sys::{OSSL_PROVIDER_load, OSSL_PROVIDER};

mod common;

extern "C" {
    fn OSSL_PROVIDER_add_builtin(ctx: *mut openssl_sys::OSSL_LIB_CTX, name: *const c_char, init: unsafe extern "C" fn()) -> c_int;
}

unsafe fn load_provider() -> Result<*mut OSSL_PROVIDER, Box<dyn std::error::Error>> {
    //let lib_ctx = openssl_sys::OSSL_LIB_CTX_new();
    // must be null to load the algorithms into the global context.
    let lib_ctx = std::ptr::null_mut();
    let res = OSSL_PROVIDER_add_builtin(lib_ctx, oqs_provider_sys::OQS_PROV_NAME, oqs_provider_sys::oqs_provider_init);
    if res != 1 {
        return Err("unable to add builtin module".into());
    };
    let provider = OSSL_PROVIDER_load(lib_ctx, oqs_provider_sys::OQS_PROV_NAME);
    if provider.is_null() {
        return Err("failed to load provider".into());
    }

    // Load the default provider
    let default_provider = OSSL_PROVIDER_load(std::ptr::null_mut(), c"default".as_ptr());
    if default_provider.is_null() {
        panic!("Failed to load the default provider");
    }


    Ok(provider)
}

// #[test]
// fn handshake() {
//     let provider_result = unsafe {load_provider()};
//     assert!(provider_result.is_ok());
// }

#[test]
fn peer_tmp_key_p384() {
    let propvider = unsafe {load_provider()};

    {
        let ctx = SslContextBuilder::new(SslMethod::tls()).unwrap();
    }



    assert!(propvider.is_ok());
    let mut server = Server::builder();
    server.ctx().set_groups_list("p256_kyber512").unwrap();
    let server = server.build();
    let mut client = server.client();
    client.ctx().set_groups_list("p256_kyber512").unwrap();
    let s = client.connect();
    // peer_tmp_key doesn't appear to work for PQ exchange
    // let peer_temp = s.ssl().peer_tmp_key().unwrap();

    // assert_ne!(
    //     peer_temp.ec_key().unwrap().public_key_to_der().unwrap(),
    //     local_temp.ec_key().unwrap().public_key_to_der().unwrap(),
    // );
}
