# Dangers

## CMake Configs are Very Sticky
I forgot to run cargo clean after updating a submodule, and the new submodule didn't produce a proper cmake config, so my provider-sys crate was just picking up the old cmake config.

## git submodules are sticky
If you git update the main body, it won't bring the crate along with it.

## liboqs-rust
Needs to install the target rather than just building it for things to be findable from other crates.
