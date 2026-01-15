use std::env;
use std::path::PathBuf;

/// Generates Rust FFI bindings from the prebuilt header file
pub fn generate_bindings(lib_dir: &PathBuf) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // The header file is at the root of the extracted directory
    let libstorage_header_path = lib_dir.join("libstorage.h");

    if !libstorage_header_path.exists() {
        panic!(
            "libstorage.h not found in prebuilt package at '{}'. \
             This should not happen - please report this issue.",
            libstorage_header_path.display()
        );
    }

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

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo:rerun-if-changed={}",
        libstorage_header_path.display()
    );
}
