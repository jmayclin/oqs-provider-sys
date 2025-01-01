//! This build script has two primary responsibilities, building oqs-provider and
//! generating the rust bindings for oqs-provider

use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

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

    //config.very_verbose(true);
    // /home/ubuntu/workspace/liboqs-provider-sys/target/debug/build/liboqs-provider-sys-d409fba8457bd0ca/out
    let outdir = config.build();
    println!("cargo:warning={:?}", outdir);

    //let outdir = config.build_target("oqs").build();
    // I think it's showing up in both out/build/lib and build/lib
    // TODO: use out/build/lib or out/lib?
    // I think "out/lib" is the more correct one, and where it is "installed" to
    let libdir = outdir.join("lib");
    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=static=oqsprovider");

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
        // generate bindings for the oqs-provider header
        .header(provider_header.to_str().unwrap())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("oqs_prov.*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:warning={:?}", out_path);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// invoke the cmake build for liboqs
fn main() {
    // This is _incredibly_ handy for debugging
    // for (key, value) in env::vars() {
    //     eprintln!("{key} = {value}");
    // }

    let oqsprovider_install = build();
    generate_bindings(oqsprovider_install);
}
