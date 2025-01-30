//! This build script has two primary responsibilities, building oqs-provider and
//! generating the rust bindings for oqs-provider

use std::{env, fs, path::PathBuf, str::FromStr};

/// build the liboqs-provider, and return the installation path.
///
/// This function will
/// 1. locate openssl and liboqs dependencies of the oqs-provider
/// 2. configure the oqs-provider cmake project with those dependencies
/// 3. build the oqs-provider cmake project
/// 4. link the rust application with the resulting artifact.
///
/// TODO: why is it DEP_OQS_ROOT and not DEP_OQS_SYS_ROOT
/// TODO: who sets DEP_OQS_ROOT?
fn build() -> PathBuf {
    //> To be able to build `oqsprovider`, OpenSSL 3.0 and liboqs need to be installed.
    //> It's not important where they are installed, just that they are. If installed
    //> in non-standard locations, these must be provided when running `cmake` via
    //> the variables "OPENSSL_ROOT_DIR" and "liboqs_DIR". See [CONFIGURE.md](CONFIGURE.md)
    //> for details.

    // Configure a cmake project using the CMakeLists.txt located at
    // <CRATE_ROOT>/oqs-provider/CMakeLists.txt, and set it to the "Release" build
    // profile.
    let mut config = cmake::Config::new("oqs-provider");
    config.profile("Release");

    // Locate the liboqs installation, which is installed by oqs-sys
    // https://github.com/open-quantum-safe/liboqs-rust/tree/main/oqs-sys
    let oqs_root = env::var("DEP_OQS_ROOT").expect("vendored liboqs must export root");
    env::set_var("liboqs_DIR", oqs_root);

    // locate the openssl installation, which is installed by openssl-sys
    // https://github.com/sfackler/rust-openssl/tree/master/openssl-sys
    let openssl_root = env::var("DEP_OPENSSL_ROOT").expect("vendored liboqs must export root");
    config.define("OPENSSL_ROOT_DIR", openssl_root);

    //> ### OQS_PROVIDER_BUILD_STATIC
    //> By setting `-DOQS_PROVIDER_BUILD_STATIC=ON` at compile-time, oqs-provider can be
    //> compiled as a static library (`oqs-provider.a`).
    //> When built as a static library, the name of the provider entrypoint is `oqs_provider_init`.
    //> The provider can be added using the [`OSSL_PROVIDER_add_builtin`](https://www.openssl.org/docs/man3.1/man3/OSSL_PROVIDER_add_builtin.html)
    //> function:
    //
    // Static builds are "neater". There is one less thing that can go wrong at
    // runtime.
    config.define("OQS_PROVIDER_BUILD_STATIC", "ON");

    // ancient errors with weird lib dl stuff and bad glibc's
    config.define("BUILD_TESTING", "OFF");

    // example path: /home/ubuntu/workspace/liboqs-provider-sys/target/debug/build/liboqs-provider-sys-d409fba8457bd0ca/out
    let outdir = config.build();
    println!("cargo:warning={:?}", outdir);

    // TODO: remove the build directory. Although it shouldn't change the size of
    // the final artifact? I think?

    let libdir = outdir.join("lib");
    let libdir64 = outdir.join("lib64");

    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-search=native={}", libdir64.display());
    println!("cargo:rustc-link-lib=static=oqsprovider");

    //println!("cargo:rustc-link-arg=-fopenmp");
    // needed for platforms with ancient glibcs
    println!("cargo:rustc-link-lib=dylib=dl");

    outdir
}

fn generate_bindings(oqsprovider_install: PathBuf) {
    // Locate the liboqs installation, which is installed by oqs-sys
    // https://github.com/open-quantum-safe/liboqs-rust/tree/main/oqs-sys
    let oqs_root = env::var("DEP_OQS_ROOT").expect("vendored liboqs must export root");
    let oqs_headers = PathBuf::from_str(&oqs_root).unwrap().join("include");

    // locate the openssl installation, which is installed by openssl-sys
    // https://github.com/sfackler/rust-openssl/tree/master/openssl-sys
    let openssl_headers =
        env::var("DEP_OPENSSL_INCLUDE").expect("vendored liboqs must export root");

    let header_folder = oqsprovider_install.join("include/oqs-provider");
    // oqs-provider exposes a single header
    let provider_header = header_folder.join("oqs_prov.h");
    eprintln!("provider header path: {}", provider_header.display());

    let bindings = bindgen::Builder::default()
        // tell bindgen (clang) where to find the openssl & oqs headers
        .clang_arg(format!("-I{}", openssl_headers))
        .clang_arg(format!("-I{}", oqs_headers.display()))
        //.raw_line(r#"#![allow(unused_imports, non_camel_case_types)]"#)
        // generate bindings for the oqs-provider header
        .header(provider_header.to_str().unwrap())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("oqs_prov.*")
        .generate_comments(false)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:warning={:?}", out_path);

    let wrapped_out_path = out_path.join("bindings.rs");

    // Write the generated bindings with a module wrapper
    // TODO: There has to be a better way to do this, but I can't find one.
    //     option 1: use #[allow(...)] on top of the include! macro. This doesn't
    //               work because the generated bindings.rs is still compiled and
    //               generates warnings.
    //     option 2: use raw_line and #![allow(...)], this doesn't work because
    //               of errors about "an inner attribute is not permitted in this context"
    //               I think it's an issue with the concat macro?
    //
    // let wrapped_bindings = format!(
    //     r#"
    //     #[allow(unused_imports, non_camel_case_types, dead_code)]
    //     pub mod ffi {{
    //         {}
    //     }}"#,
    //     bindings
    // );

    // fs::write(&wrapped_out_path, wrapped_bindings).expect("Couldn't write bindings!");
}

/// invoke the cmake build for liboqs
fn main() {
    // This is _incredibly_ handy for debugging
    for (key, value) in env::vars() {
        eprintln!("{key} = {value}");
    }

    let oqsprovider_install = build();
    generate_bindings(oqsprovider_install);
}
