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

    //> ### OPENSSL_ROOT_DIR
    //> Defines a non-standard location for an OpenSSL(v3) installation via `cmake` define.
    //> By default this value is unset, requiring presence of an OpenSSL3 installation
    //> in a standard OS deployment location.

    //> ### OQS_PROVIDER_BUILD_STATIC
    //> By setting `-DOQS_PROVIDER_BUILD_STATIC=ON` at compile-time, oqs-provider can be
    //> compiled as a static library (`oqs-provider.a`).
    //> When built as a static library, the name of the provider entrypoint is `oqs_provider_init`.
    //> The provider can be added using the [`OSSL_PROVIDER_add_builtin`](https://www.openssl.org/docs/man3.1/man3/OSSL_PROVIDER_add_builtin.html)
    //> function:

    //> ### liboqs_DIR
    //> This environment variable must be set to the location of the `liboqs` installation to be
    //> utilized in the build.
    //> By default, this is un-set, requiring installation of `liboqs` in a standard
    //> location for the OS.
    //> This uses the [`find_package`](https://cmake.org/cmake/help/latest/command/find_package.html)
    //> command in `cmake`, which checks for local builds of a package at `<PackageName>_DIR`

    for (key, value) in env::vars() {
        eprintln!("{key} = {value}");
    }


    // they are silly gooses and their roots are at different levels

    // why is this DEP_OQS_ROOT and not DEP_OQS_SYS_ROOT
    // who sets this?
    // {DEP_OQS_ROOT}/build/lib
    // {DEP_OQS_ROOT}/build/include
    let oqs_root = env::var("DEP_OQS_ROOT").expect("vendored liboqs must export root");
    //let oqs_install = format!("{oqs_root}/build/");


    let openssl_root = env::var("DEP_OPENSSL_ROOT").expect("vendored liboqs must export root");

    // this is relative from the root of the crate

    env::set_var("liboqs_DIR", oqs_root);
    let mut config = cmake::Config::new("oqs-provider");
    config.profile("Release");
    config.define("OPENSSL_ROOT_DIR", openssl_root);
    let outdir = config.build();

    // lib is put into $outdir/build/lib
    //let mut libdir = outdir.join("build").join("lib");

    // Statically linking makes it easier to use the sys crate
    println!("cargo:rustc-link-lib=static=oqs-provider");

    //build_from_source();
}
