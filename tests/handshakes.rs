use std::{ffi::CStr, mem::ManuallyDrop};

use bench::TlsConnPair;
use common::load_provider;
use openssl::ssl::{SslContext, SslFiletype, SslMethod};
use s2n_tls::security::Policy;

mod common;

struct HostNameHandler {
    expected_server_name: &'static str,
}
impl s2n_tls::callbacks::VerifyHostNameCallback for HostNameHandler {
    fn verify_host_name(&self, hostname: &str) -> bool {
        self.expected_server_name == hostname
    }
}

trait S2NExtension {
    fn kem_group(&self) -> Option<&str>;
}

unsafe fn gimme_pointer(
    connection: &s2n_tls::connection::Connection,
) -> *mut s2n_tls_sys::s2n_connection {
    *(connection as *const s2n_tls::connection::Connection as *mut s2n_tls::connection::Connection
        as *mut *mut s2n_tls_sys::s2n_connection)
}

// TODO: This is being added in the TLS 1.2 PQ deprecation PR, remove this after release
impl S2NExtension for s2n_tls::connection::Connection {
    fn kem_group(&self) -> Option<&str> {
        let name_bytes = {
            let name =
                unsafe { s2n_tls_sys::s2n_connection_get_kem_group_name(gimme_pointer(self)) };
            if name.is_null() {
                return None;
            }
            name
        };

        let name_str = unsafe { CStr::from_ptr(name_bytes).to_str() };

        match name_str {
            Ok("NONE") => None,
            Ok(name) => Some(name),
            Err(_) => {
                // Unreachable: This would indicate a non-utf-8 string literal in
                // the s2n-tls C codebase.
                None
            }
        }
    }
}

const CERT_PEM: &[u8] = include_bytes!("../cert.pem");
const KEY_PEM: &[u8] = include_bytes!("../key.pem");

fn s2n_client_config() -> Result<s2n_tls::config::Config, s2n_tls::error::Error> {
    let mut config = s2n_tls::config::Config::builder();
    let pq_policy = Policy::from_version("default_pq")?;
    config.set_security_policy(&pq_policy)?;
    config.with_system_certs(false)?;
    config.trust_pem(CERT_PEM)?;
    config.set_verify_host_callback(HostNameHandler {
        expected_server_name: "foobar.com",
    })?;

    config.build()
}

fn s2n_server_config() -> Result<s2n_tls::config::Config, s2n_tls::error::Error> {
    let mut config = s2n_tls::config::Config::builder();
    let pq_policy = Policy::from_version("default_pq")?;
    config.set_security_policy(&pq_policy)?;
    config.with_system_certs(false)?;
    config.load_pem(CERT_PEM, KEY_PEM)?;
    config.build()
}

/// return a server that supports all of the defined PQ groups
fn openssl_server_config() -> Result<openssl::ssl::SslContext, Box<dyn std::error::Error>> {
    let mut ctx = SslContext::builder(SslMethod::tls_server()).unwrap();
    ctx.set_certificate_chain_file("cert.pem").unwrap();
    ctx.set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    // SSL_CTX_set1_groups_list() sets the supported groups for ctx to string list.
    // The string is a colon separated list of group names, for example
    // "P-521:P-384:P-256:X25519:ffdhe2048"
    // https://docs.openssl.org/master/man3/SSL_CTX_set1_curves/#description
    ctx.set_groups_list(&TEST_GROUPS.join(":")).unwrap();
    //ctx.set_groups_list("X25519MLKEM768").unwrap();
    ctx.set_ciphersuites("TLS_AES_256_GCM_SHA384").unwrap();
    Ok(ctx.build())
}

fn openssl_client_config(
    group: &str,
) -> Result<openssl::ssl::SslContext, Box<dyn std::error::Error>> {
    let mut ctx = SslContext::builder(SslMethod::tls_client()).unwrap();
    //ctx.set_ca_file("cert.pem")?;
    ctx.set_certificate_chain_file("cert.pem").unwrap();
    ctx.set_groups_list(group)?;
    ctx.set_ciphersuites("TLS_AES_256_GCM_SHA384").unwrap();

    Ok(ctx.build())
}

// TODO: need to make some changes to the bench harness to make this file less ugly.
// move session ticket storage into the overall harness, and let that be "smuggled"
// in during construction

// https://github.com/open-quantum-safe/oqs-provider/blob/4638c0510271cbec3cd474bf471d386b3363590d/ALGORITHMS.md?plain=1#L37
const TEST_GROUPS: &[&str] = &[
    "X25519MLKEM768",
    "SecP256r1MLKEM768",
    "p384_mlkem1024", // TODO: oqs-provider to get new code point
    "mlkem512",
    "mlkem768",
    "mlkem1024",
];

#[test]
fn s2n_self_talk() -> Result<(), Box<dyn std::error::Error>> {
    let client = s2n_client_config()?;
    let server = s2n_server_config()?;

    let mut pair = TlsConnPair::<bench::S2NConnection, bench::S2NConnection>::from_configs(
        &client.into(),
        &server.into(),
    );

    assert!(pair.handshake().is_ok());
    assert_eq!(pair.client.connection().kem_group(), Some("X25519MLKEM768"));
    Ok(())
}

#[test]
fn s2n_client_ossl_server() {
    assert!(unsafe { load_provider().is_ok() });

    let server: bench::openssl::OpenSslConfig = openssl_server_config().unwrap().into();

    let client: bench::s2n_tls::S2NConfig = s2n_client_config().unwrap().into();

    let mut pair =
        bench::harness::TlsConnPair::<bench::S2NConnection, bench::OpenSslConnection>::from_configs(
            &client, &server,
        );

    assert!(pair.handshake().is_ok());
    assert_eq!(pair.client.connection().kem_group(), Some("X25519MLKEM768"))
}

#[test]
fn ossl_client_s2n_server() -> Result<(), Box<dyn std::error::Error>> {
    assert!(unsafe { load_provider().is_ok() });

    let client = openssl_client_config("X25519MLKEM768")?;
    let server = s2n_server_config()?;

    let pair = TlsConnPair::<bench::OpenSslConnection, bench::S2NConnection>::from_configs(
        &client.into(),
        &server.into(),
    );
    let mut pair = ManuallyDrop::new(pair);

    assert!(pair.handshake().is_ok());
    assert_eq!(pair.server.connection().kem_group(), Some("X25519MLKEM768"));
    Ok(())
}

#[test]
fn ossl_self_talk() -> Result<(), Box<dyn std::error::Error>> {
    assert!(unsafe { load_provider().is_ok() });
    for g in TEST_GROUPS {
        let server = openssl_server_config()?;
        let client = openssl_client_config(g)?;
        let mut pair =
            TlsConnPair::<bench::OpenSslConnection, bench::OpenSslConnection>::from_configs(
                &client.into(),
                &server.into(),
            );
        // TODO: This panics because we do shutdown().unwrap in the drop implementation.
        // should probably just log a warning instead.
        assert!(pair.handshake().is_ok());
    }

    Ok(())
}
