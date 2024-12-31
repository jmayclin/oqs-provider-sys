//! This build script has two primary responsibilities, building oqs-provider and
//! generating the rust bindings for oqs-provider
//! 
//! ### building oqs provider
//! 1. locate openssl and liboqs dependencies of the oqs-provider
//! 2. configure the oqs-provider cmake project with those dependencies
//! 3. build the oqs-provider cmake project
//! 4. link the rust application with the resulting artifact.
//! 
//! ### generating the rust bindings
//! The build script will then run bindgen to expose the symbols of the oqs-provider
//! library as a rust crate.

use std::{env, path::{Path, PathBuf}};

fn build_from_source() -> PathBuf {
    let mut config = cmake::Config::new("liboqs");
    config.profile("Release");
    config.define("OQS_BUILD_ONLY_LIB", "Yes");

    // if cfg!(feature = "openssl") {
    //     config.define("OQS_USE_OPENSSL", "Yes");
    //     if cfg!(windows) {
    //         // Windows doesn't prefix with lib
    //         println!("cargo:rustc-link-lib=libcrypto");
    //     } else {
    //     }

    //     println!("cargo:rerun-if-env-changed=OPENSSL_ROOT_DIR");
    //     if let Ok(dir) = std::env::var("OPENSSL_ROOT_DIR") {
    //         let dir = Path::new(&dir).join("lib");
    //         println!("cargo:rustc-link-search={}", dir.display());
    //     } else if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
    //         println!("cargo:warning=You may need to specify OPENSSL_ROOT_DIR or disable the default `openssl` feature.");
    //     }
    // } else {
    //     config.define("OQS_USE_OPENSSL", "No");
    // }

    let outdir = config.build_target("oqs-provider").build();

    // lib is put into $outdir/build/lib
    //let mut libdir = outdir.join("build").join("lib");

    // Statically linking makes it easier to use the sys crate
    println!("cargo:rustc-link-lib=static=oqs-provider");

    outdir
}

/// invoke the cmake build for liboqs
fn main() {
    //> To be able to build `oqsprovider`, OpenSSL 3.0 and liboqs need to be installed.
    //> It's not important where they are installed, just that they are. If installed
    //> in non-standard locations, these must be provided when running `cmake` via
    //> the variables "OPENSSL_ROOT_DIR" and "liboqs_DIR". See [CONFIGURE.md](CONFIGURE.md)
    //> for details.

    for (key, value) in env::vars() {
        eprintln!("{key} = {value}");
    }


    // they are silly gooses and their roots are at different levels

    // Configure a cmake project using the CMakeLists.txt located at 
    // <CRATE_ROOT>/oqs-provider/CMakeLists.txt, and set it to the "Release" build
    // profile.
    let mut config = cmake::Config::new("oqs-provider");
    config.profile("Release");

    // TODO: why is this DEP_OQS_ROOT and not DEP_OQS_SYS_ROOT
    // TODO: who sets this?
    // who sets this?
    // {DEP_OQS_ROOT}/build/lib
    // {DEP_OQS_ROOT}/build/include


    //> ### liboqs_DIR
    //> This environment variable must be set to the location of the `liboqs` installation to be
    //> utilized in the build.
    //> By default, this is un-set, requiring installation of `liboqs` in a standard
    //> location for the OS.
    //> This uses the [`find_package`](https://cmake.org/cmake/help/latest/command/find_package.html)
    //> command in `cmake`, which checks for local builds of a package at `<PackageName>_DIR`

    // Locate the liboqs installation, which is installed by oqs-sys
    // https://github.com/open-quantum-safe/liboqs-rust/tree/main/oqs-sys
    let oqs_root = env::var("DEP_OQS_ROOT").expect("vendored liboqs must export root");
    env::set_var("liboqs_DIR", oqs_root);
        
    //> ### OPENSSL_ROOT_DIR
    //> Defines a non-standard location for an OpenSSL(v3) installation via `cmake` define.
    //> By default this value is unset, requiring presence of an OpenSSL3 installation
    //> in a standard OS deployment location.

    // locate the openssl linstallation, which is installed by openssl-sys
    // https://github.com/sfackler/rust-openssl/tree/master/openssl-sys
    let openssl_root = env::var("DEP_OPENSSL_ROOT").expect("vendored liboqs must export root");
    config.define("OPENSSL_ROOT_DIR", openssl_root);
    
    //> ### OQS_PROVIDER_BUILD_STATIC
    //> By setting `-DOQS_PROVIDER_BUILD_STATIC=ON` at compile-time, oqs-provider can be
    //> compiled as a static library (`oqs-provider.a`).
    //> When built as a static library, the name of the provider entrypoint is `oqs_provider_init`.
    //> The provider can be added using the [`OSSL_PROVIDER_add_builtin`](https://www.openssl.org/docs/man3.1/man3/OSSL_PROVIDER_add_builtin.html)
    //> function:
    // Static builds are "neater". There is one less thing that can go wrong at
    // runtime.
    config.define("OQS_PROVIDER_BUILD_STATIC", "ON");

    config.very_verbose(true);
    let outdir = config.build();
    println!("cargo:warning={:?}", outdir);

    //let outdir = config.build_target("oqs").build();
    // I think it's showing up in both out/build/lib and build/lib
    // TODO: use out/build/lib or build/lib?
    let libdir = outdir.join("lib");
    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=static=oqsprovider");

    // // lib is put into $outdir/build/lib
    // let mut libdir = outdir.join("build").join("lib");
    // if cfg!(windows) {
    //     libdir.push("Release");
    //     // Static linking doesn't work on Windows
    //     println!("cargo:rustc-link-lib=oqs");
    // } else {
    //     // Statically linking makes it easier to use the sys crate
    //     println!("cargo:rustc-link-lib=static=oqs");
    // }
    // println!("cargo:rustc-link-search=native={}", libdir.display());

    // lib is put into $outdir/build/lib
    //let mut libdir = outdir.join("build").join("lib");

    // Statically linking makes it easier to use the sys crate

    //build_from_source();
}
