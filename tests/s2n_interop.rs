use std::{
    ffi::CStr,
    sync::{Arc, Mutex},
};

use bench::TlsConnection;
use common::load_provider;
use openssl::ssl::{SslContext, SslFiletype, SslMethod, SslSession};
use s2n_tls::security::{Policy, DEFAULT_TLS13};

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

// TODO: This is being added in the TLS 1.2 PQ deprecation PR
impl S2NExtension for s2n_tls::connection::Connection {
    fn kem_group(&self) -> Option<&str> {
        let name_bytes = {
            let name = unsafe { s2n_tls_sys::s2n_connection_get_kem_group_name(gimme_pointer(self)) };
            if name.is_null() {
                return None;
            }
            name
        };

        let name_str = unsafe {
            // SAFETY: The data is null terminated because it is declared as a C
            //         string literal.
            // SAFETY: kem_name has a static lifetime because it lives on a const
            //         struct s2n_kem with file scope.
            CStr::from_ptr(name_bytes).to_str()
        };

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
    let mut client_config = s2n_tls::config::Config::builder();
    let pq_policy = Policy::from_version("default_pq")?;
    client_config.set_security_policy(&pq_policy)?;
    client_config.with_system_certs(false)?;
    client_config.trust_pem(CERT_PEM)?;
    client_config.set_verify_host_callback(HostNameHandler {
        expected_server_name: "foobar.com",
    })?;

    client_config.build()
}

fn openssl_server_config() -> Result<openssl::ssl::SslContext, Box<dyn std::error::Error>> {
    let mut ctx = SslContext::builder(SslMethod::tls_server()).unwrap();
    ctx.set_certificate_chain_file("cert.pem").unwrap();
    ctx.set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    ctx.set_groups_list("X25519MLKEM768").unwrap();
    Ok(ctx.build())
}

// TODO: need to make some changes to the bench harness to make this file less ugly.
// move session ticket storage into the overall harness, and let that be "smuggled"
// in during construction

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
