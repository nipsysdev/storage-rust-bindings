use std::env;
use std::path::PathBuf;

mod src_build;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");

    // Step 1: Compile cmdline symbols to provide missing Nim symbols
    src_build::cmdline::compile_cmdline_symbols();

    // Step 2: Ensure prebuilt binary is available
    let lib_dir = src_build::prebuilt::ensure_prebuilt_binary(&out_dir, &target)
        .expect("Failed to download/extract prebuilt binary");

    // Step 3: Generate bindings
    src_build::bindings::generate_bindings(&lib_dir);

    // Step 4: Link against prebuilt library
    src_build::linker::link_prebuilt_library(&lib_dir);
}
