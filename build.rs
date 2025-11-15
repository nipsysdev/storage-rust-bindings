use std::env;
use std::path::PathBuf;
use std::process::Command;

fn check_required_tools() {
    let tools = ["git", "make"];
    for tool in &tools {
        if let Err(_) = Command::new(tool).arg("--version").output() {
            panic!(
                "Required tool '{}' is not installed or not in PATH. Please install it and try again.",
                tool
            );
        }
    }
    println!("All required tools are available");
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LinkingMode {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SourceMode {
    Submodule,
    Cloned,
}

fn determine_linking_mode() -> LinkingMode {
    let static_enabled = cfg!(feature = "static-linking");
    let dynamic_enabled = cfg!(feature = "dynamic-linking");

    match (static_enabled, dynamic_enabled) {
        (true, false) => LinkingMode::Static,
        (false, true) => LinkingMode::Dynamic,
        (false, false) => LinkingMode::Dynamic,
        (true, true) => {
            panic!("Cannot enable both 'static-linking' and 'dynamic-linking' features simultaneously. Please choose one.");
        }
    }
}

fn determine_source_mode() -> SourceMode {
    if env::var("CODEX_USE_CLONED").is_ok() {
        println!("CODEX_USE_CLONED detected, using cloned mode");
        return SourceMode::Cloned;
    }

    let vendor_submodule = PathBuf::from("vendor/nim-codex");
    if vendor_submodule.join(".git").exists() && vendor_submodule.join("codex").exists() {
        println!("Using vendor/nim-codex submodule");
        SourceMode::Submodule
    } else {
        println!("Vendor submodule not found or incomplete, using cloned mode");
        SourceMode::Cloned
    }
}

fn get_nim_codex_dir() -> PathBuf {
    let source_mode = determine_source_mode();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    match source_mode {
        SourceMode::Submodule => PathBuf::from("vendor/nim-codex"),
        SourceMode::Cloned => {
            let cloned_dir = out_dir.join("nim-codex");
            if !cloned_dir.exists() {
                clone_nim_codex(&cloned_dir);
            } else {
                println!("Using previously cloned nim-codex in OUT_DIR");
            }
            cloned_dir
        }
    }
}

fn clone_nim_codex(target_dir: &PathBuf) {
    println!("Cloning nim-codex repository...");

    let status = Command::new("git")
        .args(&[
            "clone",
            "--recurse-submodules",
            "https://github.com/codex-storage/nim-codex",
            &target_dir.to_string_lossy(),
        ])
        .status()
        .expect("Failed to execute git clone. Make sure git is installed and in PATH.");

    if !status.success() {
        panic!(
            "Failed to clone nim-codex repository from https://github.com/codex-storage/nim-codex. \
             Please check your internet connection and repository access."
        );
    }

    println!("Successfully cloned nim-codex");
}

fn build_libcodex_static(nim_codex_dir: &PathBuf) {
    println!("Building libcodex with static linking...");

    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "STATIC=1",
        "libcodex",
    ]);

    if !codex_params.is_empty() {
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
    }

    make_cmd.env("V", "1");
    make_cmd.env("USE_SYSTEM_NIM", "0");

    println!("Running make command to build libcodex (this may take several minutes)...");

    let output = make_cmd
        .output()
        .expect("Failed to execute make command. Make sure make is installed and in PATH.");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        eprintln!("Build failed with stderr:");
        eprintln!("{}", stderr);
        eprintln!("Build stdout:");
        eprintln!("{}", stdout);

        panic!(
            "Failed to build libcodex with static linking. This could be due to:\n\
             1. Missing build dependencies (C compiler, make, git)\n\
             2. Network issues during repository cloning\n\
             3. Insufficient disk space or memory\n\
             4. Build timeout in CI environments\n\
             \n\
             For troubleshooting, try building manually:\n\
             cd {:?}\n\
             make deps\n\
             make STATIC=1 libcodex",
            nim_codex_dir
        );
    }

    println!("Successfully built libcodex (static)");
}

fn build_libcodex_dynamic(nim_codex_dir: &PathBuf) {
    println!("Building libcodex with dynamic linking...");

    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&["-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

    if !codex_params.is_empty() {
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
    }

    let status = make_cmd
        .status()
        .expect("Failed to execute make command. Make sure make is installed and in PATH.");

    if !status.success() {
        panic!(
            "Failed to build libcodex with dynamic linking. Please ensure:\n\
             1. Nim compiler is installed and in PATH\n\
             2. All build dependencies are available\n\
             3. The nim-codex repository is complete and not corrupted"
        );
    }

    println!("Successfully built libcodex (dynamic)");
}

fn ensure_libcodex(nim_codex_dir: &PathBuf, lib_dir: &PathBuf, linking_mode: LinkingMode) {
    let lib_exists = match linking_mode {
        LinkingMode::Static => lib_dir.join("libcodex.a").exists(),
        LinkingMode::Dynamic => lib_dir.join("libcodex.so").exists(),
    };

    if lib_exists {
        println!("libcodex already built, skipping build step");
        return;
    }

    match linking_mode {
        LinkingMode::Static => build_libcodex_static(nim_codex_dir),
        LinkingMode::Dynamic => build_libcodex_dynamic(nim_codex_dir),
    }
}

fn link_static_library(nim_codex_dir: &PathBuf, _lib_dir: &PathBuf) {
    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/vendor/libbacktrace-upstream/.libs")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-nat-traversal/vendor/libnatpmp-upstream")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/install/usr/lib")
            .display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("nimcache/release/libcodex/vendor_leopard")
            .display()
    );

    println!("cargo:rustc-link-arg=-Wl,--whole-archive");

    println!("cargo:rustc-link-lib=static=backtrace");
    println!("cargo:rustc-link-lib=static=circom_compat_ffi");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("cargo:rustc-link-lib=static=backtracenim");
    println!("cargo:rustc-link-lib=static=libleopard");

    println!("cargo:rustc-link-lib=static=codex");

    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rustc-link-lib=dylib=gomp");

    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");

    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdCount=0");
    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdLine=0");
}

fn link_dynamic_library(lib_dir: &PathBuf) {
    println!("cargo:rustc-link-lib=dylib=codex");

    let lib_dir_abs = std::fs::canonicalize(lib_dir).unwrap_or_else(|_| lib_dir.to_path_buf());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir_abs.display());
}

fn copy_libcodex_header() -> PathBuf {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vendor_header = PathBuf::from("vendor/libcodex.h");
    let target_header = out_path.join("libcodex.h");

    if !vendor_header.exists() {
        panic!(
            "vendor/libcodex.h not found. Please ensure the vendor directory is properly set up."
        );
    }

    std::fs::copy(&vendor_header, &target_header).expect("Unable to copy libcodex.h to OUT_DIR");

    println!(
        "Copied libcodex.h from vendor to {}",
        target_header.display()
    );
    target_header
}

fn generate_bindings(libcodex_header_path: &PathBuf, nim_codex_dir: &PathBuf) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut builder = bindgen::Builder::default()
        .header(libcodex_header_path.to_str().expect("Invalid path"))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate_block(true)
        .layout_tests(false)
        .allowlist_function("codex_.*")
        .allowlist_type("codex_.*")
        .allowlist_var("codex_.*")
        .allowlist_var("RET_.*")
        .raw_line("#[allow(non_camel_case_types)]")
        .raw_line("pub type CodexCallback = tyProc__crazOL9c5Gf8j9cqs2fd61EA;");

    let nim_lib_path = nim_codex_dir.join("vendor/nimbus-build-system/vendor/Nim/lib");
    if nim_lib_path.exists() {
        builder = builder.clang_arg(format!("-I{}", nim_lib_path.display()));
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed={}", libcodex_header_path.display());
    println!("cargo:rerun-if-changed=vendor/libcodex.h");
}

fn main() {
    check_required_tools();

    let linking_mode = determine_linking_mode();
    let nim_codex_dir = get_nim_codex_dir();

    let lib_dir = nim_codex_dir.join("build");
    let _include_dir = nim_codex_dir.join("nimcache/release/libcodex");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/nim-codex");
    println!("cargo:rerun-if-changed=vendor/libcodex.h");

    match linking_mode {
        LinkingMode::Static => {
            ensure_libcodex(&nim_codex_dir, &lib_dir, LinkingMode::Static);
            link_static_library(&nim_codex_dir, &lib_dir);
        }
        LinkingMode::Dynamic => {
            ensure_libcodex(&nim_codex_dir, &lib_dir, LinkingMode::Dynamic);
            link_dynamic_library(&lib_dir);
        }
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    let libcodex_header_path = copy_libcodex_header();
    generate_bindings(&libcodex_header_path, &nim_codex_dir);
}
