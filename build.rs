use std::env;
use std::path::PathBuf;
use std::process::Command;

#[path = "src_build/patch_system.rs"]
mod patch_system;

#[path = "src_build/build_android.rs"]
mod build_android;

#[path = "src_build/parallelism.rs"]
mod parallelism;

use build_android::*;
use parallelism::get_parallel_jobs;

/// Gets the current target architecture string for comparison
fn get_current_architecture() -> String {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        format!("android-{}", target)
    } else {
        format!("desktop-{}", target)
    }
}

/// List of static library artifacts to check for architecture compatibility
static STATIC_ARTIFACTS: &[&str] = &[
    "build/libcodex.a",
    "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/libnatpmp.a",
    "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build/libminiupnpc.a",
    "nimcache/release/libcodex/vendor_leopard/liblibleopard.a",
    "vendor/nim-leveldbstatic/build/libleveldb.a",
    "vendor/nim-leveldbstatic/libleveldb.a",
];

/// List of dynamic library artifacts to check for architecture compatibility
static DYNAMIC_ARTIFACTS: &[&str] = &[
    "build/libcodex.so",
    "vendor/nim-leveldbstatic/libleveldb.so",
];

/// Checks if a file is compatible with the current target architecture
fn is_artifact_compatible(artifact_path: &PathBuf, current_arch: &str) -> bool {
    if !artifact_path.exists() {
        println!(
            "cargo:warning=Artifact {} does not exist, assuming compatible",
            artifact_path.display()
        );
        return true; // Non-existent artifacts are trivially compatible
    }

    println!(
        "cargo:warning=Checking compatibility for {} with {}",
        artifact_path.display(),
        current_arch
    );

    // For static libraries (.a), we need to extract and check object files
    if artifact_path.extension().map_or(false, |ext| ext == "a") {
        let compatible = check_static_library_compatibility(artifact_path, current_arch);
        println!(
            "cargo:warning=Static library {} compatibility: {}",
            artifact_path.display(),
            compatible
        );
        return compatible;
    }
    // For shared libraries (.so), we can check directly
    else if artifact_path.extension().map_or(false, |ext| ext == "so") {
        let compatible = check_shared_library_compatibility(artifact_path, current_arch);
        println!(
            "cargo:warning=Shared library {} compatibility: {}",
            artifact_path.display(),
            compatible
        );
        return compatible;
    }

    println!(
        "cargo:warning=Unknown file type for {}, assuming compatible",
        artifact_path.display()
    );
    true // Unknown file types are assumed compatible
}

/// Checks if a static library (.a) is compatible with the current architecture
fn check_static_library_compatibility(lib_path: &PathBuf, current_arch: &str) -> bool {
    let temp_dir = lib_path.parent().unwrap_or(lib_path).join("temp_check");

    // Create temporary directory for extraction
    if let Err(_) = std::fs::create_dir_all(&temp_dir) {
        println!(
            "cargo:warning=Failed to create temp dir for {}, assuming compatible",
            lib_path.display()
        );
        return true; // If we can't create temp dir, assume compatible to avoid false positives
    }

    // Extract the archive using absolute path
    let extraction_result = Command::new("ar")
        .arg("x")
        .arg(&lib_path.canonicalize().unwrap_or_else(|_| lib_path.clone()))
        .current_dir(&temp_dir)
        .output();

    let mut compatible = true;

    if let Ok(output) = extraction_result {
        if output.status.success() {
            // Check the first .o file we can find
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "o") {
                        if let Ok(file_output) = Command::new("file").arg(&path).output() {
                            let file_info = String::from_utf8_lossy(&file_output.stdout);
                            println!(
                                "cargo:warning=Object file info: {} -> {}",
                                path.display(),
                                file_info.trim()
                            );

                            let object_compatible =
                                is_object_file_compatible(&file_info, current_arch);
                            println!(
                                "cargo:warning=Object file compatibility: {} for {}",
                                object_compatible, current_arch
                            );

                            if !object_compatible {
                                compatible = false;
                                break;
                            }
                        }
                        break; // Only need to check one object file
                    }
                }
            }
        } else {
            println!(
                "cargo:warning=Failed to extract archive {}: {:?}",
                lib_path.display(),
                output
            );
            println!(
                "cargo:warning=stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        println!(
            "cargo:warning=Failed to run ar command on {}",
            lib_path.display()
        );
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    println!(
        "cargo:warning=Final static library compatibility for {}: {}",
        lib_path.display(),
        compatible
    );
    compatible
}

/// Checks if a shared library (.so) is compatible with the current architecture
fn check_shared_library_compatibility(lib_path: &PathBuf, current_arch: &str) -> bool {
    if let Ok(output) = Command::new("file").arg(lib_path).output() {
        let file_info = String::from_utf8_lossy(&output.stdout);
        return is_object_file_compatible(&file_info, current_arch);
    }
    true // If we can't check, assume compatible
}

fn is_object_file_compatible(file_info: &str, current_arch: &str) -> bool {
    if current_arch.contains("aarch64") {
        return file_info.contains("aarch64");
    } else if current_arch.contains("x86_64") {
        return file_info.contains("x86-64");
    }
    true
}

/// Checks if all artifacts exist and are compatible with the current architecture
fn are_all_artifacts_compatible(nim_codex_dir: &PathBuf, current_arch: &str) -> bool {
    println!(
        "cargo:warning=Checking artifact compatibility for {}",
        current_arch
    );

    // Check static artifacts
    for artifact_path in STATIC_ARTIFACTS {
        let full_path = nim_codex_dir.join(artifact_path);
        println!(
            "cargo:warning=Checking static artifact: {}",
            full_path.display()
        );
        if !is_artifact_compatible(&full_path, current_arch) {
            println!(
                "cargo:warning=Static artifact {} is incompatible with {}",
                artifact_path, current_arch
            );
            return false;
        }
        println!(
            "cargo:warning=Static artifact {} is compatible",
            artifact_path
        );
    }

    // Check dynamic artifacts
    for artifact_path in DYNAMIC_ARTIFACTS {
        let full_path = nim_codex_dir.join(artifact_path);
        println!(
            "cargo:warning=Checking dynamic artifact: {}",
            full_path.display()
        );
        if !is_artifact_compatible(&full_path, current_arch) {
            println!(
                "cargo:warning=Dynamic artifact {} is incompatible with {}",
                artifact_path, current_arch
            );
            return false;
        }
        println!(
            "cargo:warning=Dynamic artifact {} is compatible",
            artifact_path
        );
    }

    true
}

/// Cleans all build artifacts and directories
fn clean_all_artifacts(nim_codex_dir: &PathBuf, is_android: bool) {
    println!("cargo:warning=Cleaning build artifacts...");

    // Execute the cleaning script
    if let Ok(output) = Command::new("./clean_build_artifacts.sh")
        .arg(nim_codex_dir.as_os_str())
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning={}", stdout);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!(
                "cargo:warning=⚠️ Cleaning script failed: {}\nstdout: {}\nstderr: {}",
                output.status, stdout, stderr
            );
        }
    } else {
        println!("cargo:warning=⚠️ Could not execute clean_build_artifacts.sh");
    }

    // Revert Android patches when switching away from Android
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

/// Simplified artifact cleaning function
/// Only cleans if artifacts are incompatible with current architecture
fn clean_build_artifacts() {
    let current_arch = get_current_architecture();
    let nim_codex_dir = get_nim_codex_dir();
    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    // Check if all artifacts are compatible with current architecture
    if are_all_artifacts_compatible(&nim_codex_dir, &current_arch) {
        println!(
            "cargo:warning=All artifacts are compatible with {}, no cleanup needed",
            current_arch
        );
        return;
    }

    // If we get here, we need to clean
    println!(
        "cargo:warning=Incompatible artifacts detected for {}, cleaning...",
        current_arch
    );

    clean_all_artifacts(&nim_codex_dir, is_android);
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
        &format!("-j{}", get_parallel_jobs()),
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

        panic!("Failed to build libcodex with static linking.");
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

/// Compiles the cmdline_symbols.c file for all builds (desktop and Android)
fn compile_cmdline_symbols() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cmdline_symbols_c = PathBuf::from("src_build/cmdline_symbols.c");
    let cmdline_symbols_o = out_dir.join("cmdline_symbols.o");

    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    // Use appropriate compiler for the target
    let cc = if is_android {
        env::var(format!("CC_{}", target)).unwrap_or_else(|_| {
            // Fallback to Android NDK clang if target-specific CC is not set
            env::var("CODEX_ANDROID_CC").unwrap_or_else(|_| "clang".to_string())
        })
    } else {
        "cc".to_string()
    };

    // Compile the C file
    let mut compile_cmd = Command::new(&cc);
    compile_cmd.args(&[
        "-c",
        &cmdline_symbols_c.to_string_lossy(),
        "-o",
        &cmdline_symbols_o.to_string_lossy(),
    ]);

    // Add Android-specific flags if needed
    if is_android {
        if let Ok(cflags) = env::var("CFLAGS") {
            compile_cmd.args(cflags.split_whitespace());
        }
    }

    let output = compile_cmd
        .output()
        .expect("Failed to compile cmdline_symbols.c");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Failed to compile cmdline_symbols.c with {}:\nstdout: {}\nstderr: {}",
            cc, stdout, stderr
        );
    }

    // Create static library
    let ar = if is_android {
        env::var(format!("AR_{}", target))
            .unwrap_or_else(|_| env::var("CODEX_ANDROID_AR").unwrap_or_else(|_| "ar".to_string()))
    } else {
        "ar".to_string()
    };

    let output = Command::new(&ar)
        .args(&[
            "rcs",
            &out_dir.join("libcmdline_symbols.a").to_string_lossy(),
            &cmdline_symbols_o.to_string_lossy(),
        ])
        .output()
        .expect("Failed to create libcmdline_symbols.a");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Failed to create libcmdline_symbols.a with {}:\nstdout: {}\nstderr: {}",
            ar, stdout, stderr
        );
    }

    // Tell cargo to link the static library
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rerun-if-changed=src_build/cmdline_symbols.c");
}

fn link_static_library(nim_codex_dir: &PathBuf, _lib_dir: &PathBuf) {
    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");

    // Compile and link cmdline_symbols.c for all builds (desktop and Android)
    compile_cmdline_symbols();

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

    // Link cmdline_symbols.o for all builds (desktop and Android)
    println!("cargo:rustc-link-lib=static=cmdline_symbols");
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

        match apply_android_patches_during_build(&nim_codex_dir) {
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
}
