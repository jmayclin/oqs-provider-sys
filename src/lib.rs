#![allow(non_upper_case_globals)]
use std::ffi::{c_char, c_int, c_void};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

struct OSSL_provider_init_fn;

/// https://github.com/openssl/openssl/blob/b85e6f534906f0bf9114386d227e481d2336a0ff/include/openssl/core.h#L193
/// typedef int (OSSL_provider_init_fn)(const OSSL_CORE_HANDLE *handle,
///     const OSSL_DISPATCH *in,
///     const OSSL_DISPATCH **out,
///     void **provctx);

// liboqs provide entry point
extern "C" {
    fn oqs_provider_init();
}

// TODO: This next
// static int test_group(const OSSL_PARAM params[], void *data) {
//     int ret = 1;
//     int *errcnt = (int *)data;
//     const OSSL_PARAM *p =
//         OSSL_PARAM_locate_const(params, OSSL_CAPABILITY_TLS_GROUP_NAME);
//     if (p == NULL || p->data_type != OSSL_PARAM_UTF8_STRING) {
//         ret = -1;
//         goto err;
//     }

//     char *group_name = OPENSSL_strdup(p->data);

//     ret = test_oqs_groups(group_name);

//     if (ret >= 0) {
//         fprintf(stderr,
//                 cGREEN "  TLS-KEM handshake test succeeded: %s" cNORM "\n",
//                 group_name);
//     } else {
//         fprintf(stderr,
//                 cRED
//                 "  TLS-KEM handshake test failed: %s, return code: %d" cNORM
//                 "\n",
//                 group_name, ret);
//         ERR_print_errors_fp(stderr);
//         (*errcnt)++;
//     }

// err:
//     OPENSSL_free(group_name);
//     return ret;
// }

// static int test_provider_groups(OSSL_PROVIDER *provider, void *vctx) {
//     const char *provname = OSSL_PROVIDER_get0_name(provider);

//     if (!strcmp(provname, PROVIDER_NAME_OQS))
//         return OSSL_PROVIDER_get_capabilities(provider, "TLS-GROUP", test_group,
//                                               vctx);
//     else
//         return 1;
// }

extern "C" {
    fn OSSL_PROVIDER_add_builtin(ctx: *mut openssl_sys::OSSL_LIB_CTX, name: *const c_char, init: unsafe extern "C" fn()) -> c_int;
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use openssl_sys::{OSSL_LIB_CTX_new, OSSL_PROVIDER_load};

    use super::*;

    ///```c
    ///#include <openssl/provider.h>
    ///
    ///// Entrypoint.
    ///extern OSSL_provider_init_fn oqs_provider_init;
    ///
    ///void load_oqs_provider(OSSL_LIB_CTX *libctx) {
    ///  int err;
    ///
    ///  if (OSSL_PROVIDER_add_builtin(libctx, "oqsprovider", oqs_provider_init) == 1) {
    ///    if (OSSL_PROVIDER_load(libctx, "oqsprovider") == 1) {
    ///      fputs("successfully loaded `oqsprovider`.", stderr);
    ///    } else {
    ///      fputs("failed to load `oqsprovider`", stderr);
    ///    }
    ///  } else {
    ///    fputs("failed to add the builtin provider `oqsprovider`", stderr);
    ///  }
    ///}
    ///```
    #[test]
    fn ossl_providers() {
        let name = CString::new("oqsprovider").unwrap();
        unsafe {
            let lib_ctx = OSSL_LIB_CTX_new();
            if OSSL_PROVIDER_add_builtin(lib_ctx, name.as_ptr(), oqs_provider_init) == 1 {
                println!("maybe");
                let provider = OSSL_PROVIDER_load(lib_ctx, name.as_ptr());
                assert!(!provider.is_null());


            } else {
                panic!("loading failed")
            }


        }
    }

    #[test]
    fn it_works() {
        let ptr = unsafe {oqs_sys::kem::OQS_KEM_alg_identifier(3)};
        let result = add(2, 2);
        assert_eq!(result, 4);
        assert_eq!(5, 5);
    }
}
