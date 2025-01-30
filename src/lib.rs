use std::ffi::c_char;

// These `use` statements are necessary to tell that rust compiler that we need
// these libraries to be linked, even through no symbols from them are explicitly
// referenced.
// TODO: Do these need to be pub use?
use openssl_sys as _;
use oqs_sys as _;

// TODO: add a nice doc comment that shows the linking process.

// TODO: do I actually need to invoke bindgen? I think the answer is no. The only
// two symbols that I actually use are the provider_init function (why isn't that
// captured by bindgen?) and the provider name, which also isn't covered by bindgen.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
pub use ffi::*;

// Unfortunately we have to do an itty-bitty lie here. oqs_prov.h does not export
// the module name, nor does it export the init function.

// DANGER: we lie about the function signature for the purpose of convenience.
// https://github.com/openssl/openssl/blob/b85e6f534906f0bf9114386d227e481d2336a0ff/include/openssl/core.h#L193
// ```
// typedef int (OSSL_provider_init_fn)(const OSSL_CORE_HANDLE *handle,
//     const OSSL_DISPATCH *in,
//     const OSSL_DISPATCH **out,
//     void **provctx);
// ```
// Because we end up handwritten the bindings for add_builtin, as long as
// both *share* the lie it isn't a problem.
#[link(name = "oqsprovider", kind = "static")]
extern "C" {
    /// Entry point for the liboqs provider
    pub fn oqs_provider_init();
}

/// Name of the OQS provider
pub const OQS_PROV_NAME: *const c_char = c"oqsprovider".as_ptr();

// extern "C" {
//     fn OSSL_PROVIDER_add_builtin(
//         ctx: *mut openssl_sys::OSSL_LIB_CTX,
//         name: *const c_char,
//         init: unsafe extern "C" fn(),
//     ) -> c_int;
// }
