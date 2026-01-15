use std::path::PathBuf;

/// Links against the prebuilt static library
pub fn link_prebuilt_library(lib_dir: &PathBuf) {
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Link each library separately
    // The logos-storage-nim-bin build process now provides individual static libraries
    // instead of a nested archive, which resolves linking issues
    println!("cargo:rustc-link-lib=static=storage");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("cargo:rustc-link-lib=static=circom_compat_ffi");
    println!("cargo:rustc-link-lib=static=backtrace");
    println!("cargo:rustc-link-lib=static=libleopard");

    // System libraries required by the prebuilt library
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=dylib=gomp");

    // Linker flags
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");
}
