use std::env;
use std::path::PathBuf;

mod src_build;

fn main() {
    println!("=== Starting build.rs ===");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");
    println!("Build configuration:");
    println!("  OUT_DIR: {}", out_dir.display());
    println!("  TARGET: {}", target);
    println!("  HOST: {}", env::var("HOST").unwrap_or_default());
    println!("  PROFILE: {}", env::var("PROFILE").unwrap_or_default());
    println!("  OPT_LEVEL: {}", env::var("OPT_LEVEL").unwrap_or_default());

    // Step 1: Compile cmdline symbols to provide missing Nim symbols
    println!("\n=== Step 1: Compiling cmdline symbols ===");
    src_build::cmdline::compile_cmdline_symbols();
    println!("✓ Cmdline symbols compiled successfully");

    // Step 2: Ensure prebuilt binary is available
    println!("\n=== Step 2: Ensuring prebuilt binary ===");
    let lib_dir = src_build::prebuilt::ensure_prebuilt_binary(&out_dir, &target)
        .expect("Failed to download/extract prebuilt binary");
    println!("✓ Prebuilt binary available at: {}", lib_dir.display());

    // Step 3: Generate bindings
    println!("\n=== Step 3: Generating bindings ===");
    src_build::bindings::generate_bindings(&lib_dir);
    println!("✓ Bindings generated successfully");

    // Step 4: Link against prebuilt library
    println!("\n=== Step 4: Linking against prebuilt library ===");
    src_build::linker::link_prebuilt_library(&lib_dir);
    println!("✓ Linking configuration complete");

    println!("\n=== build.rs completed successfully ===");
}
