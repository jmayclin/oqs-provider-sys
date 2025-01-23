//! ~~stolen~~ respectfully appropriated from rust-openssl repo
use tin_ladybug_oqs_provider_sys as oqs_provider_sys;

use std::ffi::{c_char, c_int};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use openssl::ssl::{Ssl, SslContext, SslContextBuilder, SslFiletype, SslMethod, SslRef, SslStream};
use openssl_sys::OSSL_PROVIDER_load;

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
pub unsafe fn load_provider() -> Result<(), Box<dyn std::error::Error>> {
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

pub struct Server {
    handle: Option<JoinHandle<()>>,
    addr: SocketAddr,
}

impl Server {
    pub fn builder() -> Builder {
        let mut ctx = SslContext::builder(SslMethod::tls()).unwrap();
        ctx.set_certificate_chain_file("cert.pem").unwrap();
        ctx.set_private_key_file("key.pem", SslFiletype::PEM)
            .unwrap();

        Builder {
            ctx,
            ssl_cb: Box::new(|_| {}),
            io_cb: Box::new(|_| {}),
            should_error: false,
        }
    }

    pub fn client(&self) -> ClientBuilder {
        ClientBuilder {
            ctx: SslContext::builder(SslMethod::tls()).unwrap(),
            addr: self.addr,
        }
    }

    pub fn connect_tcp(&self) -> TcpStream {
        TcpStream::connect(self.addr).unwrap()
    }
}

pub struct Builder {
    ctx: SslContextBuilder,
    ssl_cb: Box<dyn FnMut(&mut SslRef) + Send>,
    io_cb: Box<dyn FnMut(SslStream<TcpStream>) + Send>,
    should_error: bool,
}

impl Builder {
    pub fn ctx(&mut self) -> &mut SslContextBuilder {
        &mut self.ctx
    }

    pub fn ssl_cb<F>(&mut self, cb: F)
    where
        F: 'static + FnMut(&mut SslRef) + Send,
    {
        self.ssl_cb = Box::new(cb);
    }

    pub fn io_cb<F>(&mut self, cb: F)
    where
        F: 'static + FnMut(SslStream<TcpStream>) + Send,
    {
        self.io_cb = Box::new(cb);
    }

    pub fn should_error(&mut self) {
        self.should_error = true;
    }

    pub fn build(self) -> Server {
        let ctx = self.ctx.build();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let mut ssl_cb = self.ssl_cb;
        let mut io_cb = self.io_cb;
        let should_error = self.should_error;

        let handle = thread::spawn(move || {
            let socket = socket.accept().unwrap().0;
            let mut ssl = Ssl::new(&ctx).unwrap();
            ssl_cb(&mut ssl);
            let r = ssl.accept(socket);
            if should_error {
                r.unwrap_err();
            } else {
                let mut socket = r.unwrap();
                socket.write_all(&[0]).unwrap();
                io_cb(socket);
            }
        });

        Server {
            handle: Some(handle),
            addr,
        }
    }
}

pub struct ClientBuilder {
    ctx: SslContextBuilder,
    addr: SocketAddr,
}

impl ClientBuilder {
    pub fn ctx(&mut self) -> &mut SslContextBuilder {
        &mut self.ctx
    }

    pub fn build(self) -> Client {
        Client {
            ctx: self.ctx.build(),
            addr: self.addr,
        }
    }

    pub fn connect(self) -> Result<SslStream<TcpStream>, Box<dyn std::error::Error>> {
        self.build().builder().connect()
    }

    pub fn connect_err(self) {
        self.build().builder().connect_err();
    }
}

pub struct Client {
    ctx: SslContext,
    addr: SocketAddr,
}

impl Client {
    pub fn builder(&self) -> ClientSslBuilder {
        ClientSslBuilder {
            ssl: Ssl::new(&self.ctx).unwrap(),
            addr: self.addr,
        }
    }
}

pub struct ClientSslBuilder {
    ssl: Ssl,
    addr: SocketAddr,
}

impl ClientSslBuilder {
    pub fn ssl(&mut self) -> &mut SslRef {
        &mut self.ssl
    }

    pub fn connect(self) -> Result<SslStream<TcpStream>, Box<dyn std::error::Error>> {
        let socket = TcpStream::connect(self.addr)?;
        let mut s = self.ssl.connect(socket)?;
        s.read_exact(&mut [0]).unwrap();
        Ok(s)
    }

    pub fn connect_err(self) {
        let socket = TcpStream::connect(self.addr).unwrap();
        self.ssl.connect(socket).unwrap_err();
    }
}
