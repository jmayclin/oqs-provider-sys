use std::ffi::{c_char, c_int, c_void, CStr, CString};

// These `use` statements are necessary to tell that rust compiler that we need
// these libraries to be linked, even through no symbols from them are explicitly
// referenced.
use openssl_sys as _;
use oqs_sys as _;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Unfortunately we have to do an itty-bitty lie here. oqs_prov.h does not export
// the module name, nor does it export the init function.

// DANGER: we lie about the function signature.
// https://github.com/openssl/openssl/blob/b85e6f534906f0bf9114386d227e481d2336a0ff/include/openssl/core.h#L193
// ```
// typedef int (OSSL_provider_init_fn)(const OSSL_CORE_HANDLE *handle,
//     const OSSL_DISPATCH *in,
//     const OSSL_DISPATCH **out,
//     void **provctx);
// ```
// But this is incredibly messy, so we just treat it as a dumb function pointer.
#[link(name = "oqsprovider", kind = "static")]
extern "C" {
    /// Entry point for the liboqs provider
    pub fn oqs_provider_init();
}

/// Name of the OQS provider
pub const OQS_PROV_NAME: *const c_char = c"oqsprovider".as_ptr();

extern "C" {
    fn OSSL_PROVIDER_add_builtin(
        ctx: *mut openssl_sys::OSSL_LIB_CTX,
        name: *const c_char,
        init: unsafe extern "C" fn(),
    ) -> c_int;
}
