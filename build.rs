use std::env;
use std::path::PathBuf;
use std::process::Command;

#[path = "src/patch_system.rs"]
mod patch_system;

#[path = "build_android.rs"]
mod build_android;

use build_android::*;

/// Gets the current target architecture string for comparison
fn get_current_architecture() -> String {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        format!("android-{}", target)
    } else {
        format!("desktop-{}", target)
    }
}

/// Reads the last built architecture from a shared file in the project root
fn get_last_built_architecture() -> Option<String> {
    let arch_file = PathBuf::from(".last_built_architecture");

    if arch_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&arch_file) {
            Some(content.trim().to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Saves the current architecture to a shared file in the project root for future comparison
fn save_current_architecture() {
    let arch_file = PathBuf::from(".last_built_architecture");
    let current_arch = get_current_architecture();

    // Ensure we overwrite the file completely, not append
    if let Err(e) = std::fs::write(&arch_file, &current_arch) {
        println!("cargo:warning=Failed to save architecture: {}", e);
    } else {
        println!("cargo:warning=Saved architecture: {}", current_arch);
    }
}

/// Cleans build artifacts to prevent cross-architecture contamination
fn clean_build_artifacts() {
    let current_arch = get_current_architecture();
    let last_arch = get_last_built_architecture();
    let nim_codex_dir = get_nim_codex_dir();
    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    // Check if we have an incompatible library even if architecture appears unchanged
    let mut force_cleanup = false;
    let libcodex_so_path = nim_codex_dir.join("build").join("libcodex.so");

    if libcodex_so_path.exists() {
        if let Ok(output) = Command::new("file").arg(&libcodex_so_path).output() {
            let file_info = String::from_utf8_lossy(&output.stdout);
            let is_android_lib = (file_info.contains("ARM aarch64")
                || file_info.contains("x86-64"))
                && file_info.contains("Android");
            let is_desktop_lib = file_info.contains("x86-64") || file_info.contains("x86_64");
            let is_desktop_build = current_arch.starts_with("desktop-");
            let is_android_build = current_arch.starts_with("android-");

            // Force cleanup if we have a desktop library but building for Android
            if is_desktop_lib && is_android_build {
                println!(
                    "cargo:warning=Detected desktop library on Android build, forcing cleanup"
                );
                force_cleanup = true;
            }
            // Force cleanup if we have an Android library but building for desktop
            else if is_android_lib && is_desktop_build {
                println!(
                    "cargo:warning=Detected Android library on desktop build, forcing cleanup"
                );
                force_cleanup = true;
            }
        }
    }

    // Only clean if architecture changed or we have incompatible libraries
    if let Some(ref last_arch_value) = last_arch {
        if last_arch_value == &current_arch && !force_cleanup {
            println!(
                "cargo:warning=Architecture unchanged ({}), skipping cleanup",
                current_arch
            );
            return;
        }
    }

    if force_cleanup {
        println!(
            "cargo:warning=Forcing cleanup due to incompatible library (arch: {})",
            current_arch
        );
    } else {
        println!(
            "cargo:warning=Architecture changed from {:?} to {}, cleaning artifacts...",
            last_arch, current_arch
        );
    }

    println!(
        "cargo:warning=Cleaning build artifacts for target: {}",
        target
    );

    // Clean Nim cache to prevent architecture conflicts
    if let Some(home_dir) = std::env::var_os("HOME") {
        let nim_cache_path = PathBuf::from(home_dir).join(".cache/nim/libcodex_d");
        if nim_cache_path.exists() {
            println!("cargo:warning=Removing Nim cache: {:?}", nim_cache_path);
            let _ = std::fs::remove_dir_all(&nim_cache_path);
        }
    }

    // Clean problematic pre-built libraries and build directories that cause architecture conflicts
    let artifacts_to_clean = [
        // NAT traversal libraries
        "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build",
        "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/libnatpmp.a",
        "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/build",
        // Circom compatibility FFI
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target",
        // Main build directory
        "build",
        // Nim cache directories
        "nimcache/release",
        "nimcache/debug",
    ];

    for artifact in &artifacts_to_clean {
        let path = nim_codex_dir.join(artifact);
        if path.exists() {
            println!("cargo:warning=Removing build artifact: {}", artifact);
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    // Clean any extracted .o files that might be left behind
    let object_file_dirs = [
        "vendor/nim-nat-traversal/vendor/libnatpmp-upstream",
        "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc",
    ];

    for dir in &object_file_dirs {
        let dir_path = nim_codex_dir.join(dir);
        if dir_path.exists() && dir_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "o") {
                        println!("cargo:warning=Removing object file: {:?}", path.file_name());
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }

    // Clean the main libcodex library files to prevent cross-architecture linking issues
    let lib_files_to_clean = ["libcodex.so", "libcodex.a"];
    for lib_file in &lib_files_to_clean {
        let lib_path = nim_codex_dir.join("build").join(lib_file);
        if lib_path.exists() {
            println!("cargo:warning=Removing library file: {}", lib_file);
            let _ = std::fs::remove_file(&lib_path);
        }
    }

    // Revert Android patches when switching away from Android to ensure clean state
    if !is_android {
        println!("cargo:warning=Reverting Android patches for desktop build...");
        if let Ok(output) = Command::new("./revert_patches.sh").output() {
            if output.status.success() {
                println!("cargo:warning=✅ Successfully reverted Android patches");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "cargo:warning=⚠️ Failed to revert Android patches: {}",
                    stderr
                );
            }
        } else {
            println!("cargo:warning=⚠️ Could not execute revert_patches.sh");
        }
    }

    println!("cargo:warning=Build artifacts cleanup completed");
}

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
            // Clone to OUT_DIR/vendor/nim-codex to maintain path consistency with patches
            let vendor_dir = out_dir.join("vendor");
            let cloned_dir = vendor_dir.join("nim-codex");

            if !cloned_dir.exists() {
                // Create vendor directory if it doesn't exist
                if !vendor_dir.exists() {
                    std::fs::create_dir_all(&vendor_dir)
                        .expect("Failed to create vendor directory in OUT_DIR");
                }
                clone_nim_codex(&cloned_dir);
            } else {
                println!("Using previously cloned nim-codex in OUT_DIR/vendor");
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
            "--branch",
            "master",
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

    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");
    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    if is_android {
        build_libcodex_static_android(nim_codex_dir, &codex_params);
        return;
    }

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        "-j12",
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "STATIC=1",
        "libcodex",
    ]);

    make_cmd.env("USE_LIBBACKTRACE", "1");
    // For desktop static builds, ensure we don't use Android CPU
    make_cmd.env("CODEX_ANDROID_CPU", "");
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

    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    if is_android {
        build_libcodex_dynamic_android(nim_codex_dir, &target);
        return;
    }

    let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&["-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

    if !codex_params.is_empty() {
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
    }

    make_cmd.env("V", "1");
    make_cmd.env("USE_SYSTEM_NIM", "0");
    make_cmd.env("USE_LIBBACKTRACE", "1");
    make_cmd.env("CODEX_LIB_PARAMS", "-d:release");

    let status = make_cmd
        .status()
        .expect("Failed to execute make command. Make sure make is installed and in PATH.");

    if !status.success() {
        panic!("Failed to build libcodex with dynamic linking.");
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
    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    // Only add libbacktrace search paths for non-Android builds
    if !is_android {
        println!(
            "cargo:rustc-link-search=native={}",
            nim_codex_dir
                .join("vendor/nim-libbacktrace/vendor/libbacktrace-upstream/.libs")
                .display()
        );
    }

    let circom_dir = if is_android {
        get_android_circom_dir(nim_codex_dir, &target)
    } else {
        nim_codex_dir.join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release")
    };

    // Check if the Android-specific directory exists, fallback to regular directory
    let circom_dir = if is_android && !circom_dir.exists() {
        println!(
            "cargo:warning=Android-specific circom directory not found, falling back to default"
        );
        nim_codex_dir.join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release")
    } else {
        circom_dir
    };

    println!("cargo:rustc-link-search=native={}", circom_dir.display());

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

    // Only add libbacktrace install search paths for non-Android builds
    if !is_android {
        println!(
            "cargo:rustc-link-search=native={}",
            nim_codex_dir
                .join("vendor/nim-libbacktrace/install/usr/lib")
                .display()
        );
    }

    let leopard_dir_release = nim_codex_dir.join("nimcache/release/libcodex/vendor_leopard");
    let leopard_dir_debug = nim_codex_dir.join("nimcache/debug/libcodex/vendor_leopard");

    let leopard_dir = if leopard_dir_release.exists() {
        leopard_dir_release
    } else {
        println!("Warning: Leopard library not found in release directory, using debug directory");
        leopard_dir_debug
    };

    println!("cargo:rustc-link-search=native={}", leopard_dir.display());

    println!("cargo:rustc-link-arg=-Wl,--whole-archive");

    // Only link libbacktrace on non-Android builds (it's disabled for Android)
    if !is_android {
        println!("cargo:rustc-link-lib=static=backtrace");
        println!("cargo:rustc-link-lib=static=backtracenim");
    }
    println!("cargo:rustc-link-lib=static=circom_compat_ffi");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("cargo:rustc-link-lib=static=libleopard");

    println!("cargo:rustc-link-lib=static=codex");

    println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");

    println!("cargo:rustc-link-lib=stdc++");

    if is_android {
        println!("cargo:rustc-link-lib=static=omp");
    } else {
        println!("cargo:rustc-link-lib=dylib=gomp");
    }

    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");

    // Only use --defsym for non-Android targets
    // Android builds get these symbols from cmdline_symbols.c
    if !is_android {
        println!("cargo:rustc-link-arg=-Wl,--defsym=cmdCount=0");
        println!("cargo:rustc-link-arg=-Wl,--defsym=cmdLine=0");
    }
}

fn link_dynamic_library(lib_dir: &PathBuf) {
    println!("cargo:rustc-link-lib=dylib=codex");

    let lib_dir_abs = std::fs::canonicalize(lib_dir).unwrap_or_else(|_| lib_dir.to_path_buf());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir_abs.display());

    // Also add the absolute path to the library search path
    println!("cargo:rustc-link-search=native={}", lib_dir_abs.display());
}

fn generate_bindings(nim_codex_dir: &PathBuf) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let libcodex_header_path_release = nim_codex_dir.join("nimcache/release/libcodex/libcodex.h");
    let libcodex_header_path_debug = nim_codex_dir.join("nimcache/debug/libcodex/libcodex.h");
    let libcodex_header_path_library = nim_codex_dir.join("library/libcodex.h");

    // Try release directory first, then debug directory, then library directory
    let libcodex_header_path = if libcodex_header_path_release.exists() {
        libcodex_header_path_release
    } else if libcodex_header_path_debug.exists() {
        println!("Warning: Header file not found in release directory, using debug directory");
        libcodex_header_path_debug
    } else {
        println!("Warning: Header file not found in release or debug directories, using library directory");
        libcodex_header_path_library
    };

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
        .clang_arg("-D__STDC_VERSION__=201112L") // Define C11 standard for bool support
        .clang_arg("-D__bool_true_false_are_defined=1") // Ensure bool is defined
        .clang_arg("-includestdbool.h"); // Include stdbool.h for bool type

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
    setup_cargo_rerun_triggers();

    let linking_mode = determine_linking_mode();
    let nim_codex_dir = get_nim_codex_dir();
    let target = env::var("TARGET").unwrap_or_default();

    if target.contains("android") {
        setup_android_cross_compilation(target.clone());

        match apply_android_patches_during_build() {
            Ok(patches) => {
                println!(
                    "cargo:warning=✅ Successfully applied {} Android patches with validation",
                    patches.len()
                );
            }
            Err(e) => {
                println!("cargo:warning=❌ Android patch system failed: {}", e);
                if e.to_string().contains("validation failed") {
                    panic!("Critical Android patch validation failed: {}. Build cannot continue with incorrect configuration.", e);
                }
            }
        };
    }

    // Clean build artifacts to prevent cross-architecture contamination
    clean_build_artifacts();

    let lib_dir = nim_codex_dir.join("build");
    let _include_dir = nim_codex_dir.join("nimcache/release/libcodex");

    println!("cargo:rerun-if-changed=build.rs");

    // Set up appropriate rerun triggers based on source mode
    match determine_source_mode() {
        SourceMode::Submodule => {
            println!("cargo:rerun-if-changed=vendor/nim-codex");
            println!("cargo:rerun-if-changed=vendor/libcodex.h");
        }
        SourceMode::Cloned => {
            // In cloned mode, watch the OUT_DIR directory
            let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
            println!(
                "cargo:rerun-if-changed={}",
                out_dir.join("vendor/nim-codex").display()
            );
            // Also watch the original vendor directory for libcodex.h
            println!("cargo:rerun-if-changed=vendor/libcodex.h");
        }
    }

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

    generate_bindings(&nim_codex_dir);

    // Save the current architecture after successful build
    save_current_architecture();
}
