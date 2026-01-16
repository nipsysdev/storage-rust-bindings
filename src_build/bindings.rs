use std::env;
use std::path::PathBuf;

/// Generates Rust FFI bindings from the prebuilt header file
pub fn generate_bindings(lib_dir: &PathBuf) {
    println!("  [BINDINGS] Starting generate_bindings");
    println!("  [BINDINGS] Library directory: {}", lib_dir.display());

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("  [BINDINGS] Output directory: {}", out_path.display());

    // The header file is at the root of the extracted directory
    let libstorage_header_path = lib_dir.join("libstorage.h");
    println!(
        "  [BINDINGS] Header file path: {}",
        libstorage_header_path.display()
    );

    if !libstorage_header_path.exists() {
        println!("  [BINDINGS] ✗ Header file does not exist!");
        panic!(
            "libstorage.h not found in prebuilt package at '{}'. \
             This should not happen - please report this issue.",
            libstorage_header_path.display()
        );
    }
    println!("  [BINDINGS] ✓ Header file exists");

    println!("  [BINDINGS] Configuring bindgen builder...");
    let builder = bindgen::Builder::default()
        .header(libstorage_header_path.to_str().expect("Invalid path"))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate_block(true)
        .layout_tests(false)
        .allowlist_function("storage_.*")
        .allowlist_type("storage_.*")
        .allowlist_var("storage_.*")
        .allowlist_var("RET_.*")
        .raw_line("#[allow(non_camel_case_types)]")
        .raw_line("#[allow(non_snake_case)]")
        .clang_arg("-D__STDC_VERSION__=201112L")
        .clang_arg("-D__bool_true_false_are_defined=1")
        .clang_arg("-includestdbool.h");
    println!("  [BINDINGS] ✓ Bindgen builder configured");

    println!("  [BINDINGS] Generating bindings...");
    let bindings = builder.generate().expect("Unable to generate bindings");
    println!("  [BINDINGS] ✓ Bindings generated successfully");

    let bindings_file = out_path.join("bindings.rs");
    println!(
        "  [BINDINGS] Writing bindings to: {}",
        bindings_file.display()
    );
    bindings
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");
    println!("  [BINDINGS] ✓ Bindings written successfully");

    println!(
        "cargo:rerun-if-changed={}",
        libstorage_header_path.display()
    );
    println!("  [BINDINGS] ✓ generate_bindings completed successfully");
}
