use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::patch_system::{get_android_arch_from_target, PatchEngine};

#[path = "src/patch_system.rs"]
mod patch_system;

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

fn setup_android_cross_compilation(target: String) {
    println!(
        "cargo:warning=Setting up Android cross-compilation for target: {}",
        target
    );

    let android_sdk = env::var("ANDROID_SDK_ROOT").expect("ANDROID_SDK_ROOT hasn't been set");

    let android_ndk = env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME hasn't been set");

    if !std::path::Path::new(&android_sdk).exists() {
        panic!("Android SDK not found at {}.", android_sdk);
    }
    if !std::path::Path::new(&android_ndk).exists() {
        panic!("Android NDK not found at {}.", android_ndk);
    }

    // Clean architecture-specific build artifacts to prevent cross-architecture contamination
    println!("cargo:warning=Cleaning architecture-specific build artifacts for Android...");
    clean_android_build_artifacts();

    let target_clone = target.clone();

    unsafe {
        env::set_var(&format!("CARGO_TARGET_{}", target), "1");
        env::set_var(&format!("CARGO_LINKER_{}", target), "clang");

        env::set_var("CARGO_TARGET_{}_LINKER", target);
    }

    let (arch, _) = get_android_arch_from_target(&target_clone);

    let toolchain_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);
    let cc = format!("{}/{}21-clang", toolchain_path, target_clone);
    let cxx = format!("{}/{}21-clang++", toolchain_path, target_clone);
    let ar = format!("{}/llvm-ar", toolchain_path);
    let ranlib = format!("{}/llvm-ranlib", toolchain_path);

    println!("cargo:warning=Android CC path: {}", cc);
    println!(
        "cargo:warning=Android CC exists: {}",
        std::path::Path::new(&cc).exists()
    );

    unsafe {
        env::set_var(format!("CC_{}", target_clone), &cc);
        env::set_var(format!("CXX_{}", target_clone), &cxx);
        env::set_var(format!("AR_{}", target_clone), &ar);
        env::set_var(format!("RANLIB_{}", target_clone), &ranlib);

        env::set_var("CC_aarch64_linux_android", &cc);
        env::set_var("CXX_aarch64_linux_android", &cxx);
        env::set_var("AR_aarch64_linux_android", &ar);
        env::set_var("RANLIB_aarch64_linux_android", &ranlib);
    }

    let sysroot = format!(
        "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
        android_ndk
    );

    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}/21",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}/31",
        sysroot, target_clone
    );

    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}/21",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}/31",
        sysroot, target_clone
    );

    let arch_flag = match target_clone.as_str() {
        "aarch64-linux-android" => "-march=armv8-a",
        _ => panic!("Unsupported Android target: {}", target_clone),
    };

    let arch_define = match target_clone.as_str() {
        "aarch64-linux-android" => "-d:arm64",
        _ => panic!("Unsupported Android target: {}", target_clone),
    };
    let android_defines = format!("{} -d:android -d:debug -d:disable_libbacktrace -d:noIntrinsicsBitOpts -d:NO_X86_INTRINSICS -d:__NO_INLINE_ASM__ -d:noX86 -d:noSSE -d:noAVX -d:noAVX2 -d:noAVX512 -d:noX86Intrinsics -d:noSimd -d:noInlineAsm", arch_define);

    unsafe {
        env::set_var("NO_X86_INTRINSICS", "1");
        env::set_var("BR_NO_X86_INTRINSICS", "1");
        env::set_var("BR_NO_X86", "1");
        env::set_var("BR_NO_ASM", "1");
    }

    unsafe {
        match target_clone.as_str() {
            "aarch64-linux-android" => {
                env::set_var("ANDROID_ARM64_BUILD", "1");
                env::set_var("TARGET_ARCH", "arm64");
            }
            _ => panic!("Unsupported Android target: {}", target_clone),
        }
    }

    unsafe {
        env::set_var("CODEX_ANDROID_STATIC", "1");
        env::set_var("CODEX_ANDROID_CPU", arch);
        env::set_var("CODEX_ANDROID_CC", &cc);
        env::set_var("CODEX_ANDROID_AR", &ar);
        env::set_var("CODEX_ANDROID_RANLIB", &ranlib);
        env::set_var("CODEX_ANDROID_DEFINES", &android_defines);
        env::set_var("CODEX_ANDROID_ARCH_FLAG", arch_flag);

        env::set_var("CODEX_LIB_PARAMS", &android_defines);

        env::set_var("NIM_TARGET", "android");
        env::set_var("NIM_ARCH", arch);

        env::set_var("ANDROID", "1");
        env::set_var("CODEX_SKIP_GIT_RESET", "1");
        env::set_var("CODEX_SKIP_SUBMODULE_RESET", "1");
        env::set_var("CODEX_SKIP_SUBMODULE_UPDATE", "1");
    }

    // Set Rust/Cargo cross-compilation environment variables for circom-compat-ffi
    unsafe {
        env::set_var("CARGO_BUILD_TARGET", &target_clone);
        env::set_var("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", &cc);

        // Ensure the cc crate can find the Android compiler
        env::set_var("CC_aarch64_linux_android", &cc);
        env::set_var("CXX_aarch64_linux_android", &cxx);
        env::set_var("AR_aarch64_linux_android", &ar);
        env::set_var("RANLIB_aarch64_linux_android", &ranlib);

        // Set generic CC/AR for Rust's build scripts that don't use target-specific vars
        env::set_var("CC", &cc);
        env::set_var("CXX", &cxx);
        env::set_var("AR", &ar);
        env::set_var("RANLIB", &ranlib);

        // Force Cargo to use the Android linker
        env::set_var("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", &cc);
    }

    println!("cargo:rustc-link-lib=dylib=android");
    println!("cargo:rustc-link-lib=dylib=log");
    println!("cargo:rustc-link-lib=dylib=OpenSLES");
    println!("cargo:rustc-link-lib=dylib=c++_shared");

    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}/21",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}/31",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}",
        sysroot, target_clone
    );

    let (_, openmp_arch) = get_android_arch_from_target(&target_clone);

    let openmp_lib_path = format!(
        "{}/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/{}",
        android_ndk, openmp_arch
    );
    println!("cargo:rustc-link-search=native={}", openmp_lib_path);
    println!("cargo:rustc-link-lib=static=omp");

    // Also set target-specific linker environment variables
    println!("cargo:rustc-env=CC={}", cc);
    println!("cargo:rustc-env=CXX={}", cxx);
    println!("cargo:rustc-env=AR={}", ar);
    println!("cargo:rustc-env=RANLIB={}", ranlib);

    // Force the use of Android NDK clang for all linking
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-Wl,--undefined-version");

    // Force Rust to use the Android NDK linker directly
    // Set the linker path in the environment so clang can find it
    let android_ld_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);

    // Get the current system PATH and append Android NDK path to preserve system tools like bash
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", current_path, android_ld_path);
    println!("cargo:rustc-env=PATH={}", new_path);
    // Let Android NDK clang use its default linker
    // println!("cargo:rustc-link-arg=-fuse-ld=lld");

    // Set linker environment variables that BearSSL will inherit
    unsafe {
        // Force BearSSL to use Android NDK linker
        let android_linker = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/bin/ld.lld",
            android_ndk
        );

        env::set_var("LD", &android_linker);
        env::set_var("BEARSSL_LD", &android_linker);

        // Add linker flags to force Android linker usage
        let linker_flags = format!("-fuse-ld={}", android_linker);
        env::set_var("LDFLAGS", linker_flags);
    }

    println!(
        "Android cross-compilation setup complete for {}",
        target_clone
    );
}

fn clean_android_build_artifacts() {
    let nim_codex_dir = PathBuf::from("vendor/nim-codex");

    // Clean problematic pre-built libraries that cause architecture conflicts
    let artifacts_to_clean = [
        "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build",
        "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/libnatpmp.a",
        "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/*.o",
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target",
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/aarch64-linux-android",
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release",
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/debug",
        "nimcache",
    ];

    for artifact in &artifacts_to_clean {
        if artifact.contains("*.o") {
            // Handle glob patterns for .o files
            let dir_path = nim_codex_dir.join(artifact.replace("/*.o", ""));
            if dir_path.exists() && dir_path.is_dir() {
                println!(
                    "cargo:warning=Cleaning .o files in directory: {}",
                    artifact.replace("/*.o", "")
                );
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "o") {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        } else {
            let path = nim_codex_dir.join(artifact);
            if path.exists() {
                println!(
                    "cargo:warning=Removing Android build artifact: {}",
                    artifact
                );
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    // Also clean any Cargo build artifacts that might be architecture-specific
    let cargo_target_dirs =
        [nim_codex_dir.join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target")];

    for target_dir in &cargo_target_dirs {
        if target_dir.exists() {
            println!(
                "cargo:warning=Cleaning Cargo target directory: {}",
                target_dir.display()
            );
            let _ = std::fs::remove_dir_all(&target_dir);
        }
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

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        "-j12",
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "STATIC=1",
        "libcodex",
    ]);

    // For Android builds, override USE_LIBBACKTRACE to avoid -d:release
    if is_android {
        // CRITICAL: Set NIM_PARAMS FIRST to prevent .DEFAULT target from running
        // This must be set before any other environment variables to prevent git submodule update
        make_cmd.env("NIM_PARAMS", &codex_params); // This prevents the .DEFAULT target from running

        make_cmd.env("USE_LIBBACKTRACE", "0");
        make_cmd.env("CODEX_ANDROID_CPU", "arm64");
        // CRITICAL: Prevent Makefile from updating submodules after patches are applied
        // This ensures our patches don't get overwritten by git submodule update
        make_cmd.env("CODEX_LIB_PARAMS", &codex_params);

        // CRITICAL: Ensure NIM_PARAMS is also set as CODEX_LIB_PARAMS for consistency
        // The Makefile adds CODEX_LIB_PARAMS to NIM_PARAMS, so this double-ensures NIM_PARAMS is set
        if !codex_params.is_empty() {
            make_cmd.env("NIM_PARAMS", &codex_params);
        }
    } else {
        make_cmd.env("USE_LIBBACKTRACE", "1");
        // For desktop static builds, ensure we don't use Android CPU
        make_cmd.env("CODEX_ANDROID_CPU", "");
        if !codex_params.is_empty() {
            make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
        }
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
        println!("Building libcodex with make for Android...");

        let cpu = env::var("CODEX_ANDROID_CPU").unwrap_or_default();
        let cc = env::var("CODEX_ANDROID_CC").unwrap_or_default();
        let cxx = env::var("CXX_").unwrap_or_else(|_| cc.replace("-clang", "-clang++"));
        let ar = env::var("CODEX_ANDROID_AR").unwrap_or_default();
        let ranlib = env::var("CODEX_ANDROID_RANLIB").unwrap_or_default();
        let android_defines = env::var("CODEX_ANDROID_DEFINES").unwrap_or_default();
        let arch_flag = env::var("CODEX_ANDROID_ARCH_FLAG").unwrap_or_default();

        let mut make_cmd = Command::new("make");
        make_cmd.args(&["-j12", "-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

        make_cmd.env("NIM_PARAMS", &android_defines);

        make_cmd.env("USE_LIBBACKTRACE", "0");
        make_cmd.env("ANDROID", "1");
        make_cmd.env("CODEX_ANDROID_CPU", &cpu);
        make_cmd.env("CODEX_ANDROID_CC", &cc);
        make_cmd.env("CODEX_ANDROID_AR", &ar);
        make_cmd.env("CODEX_ANDROID_RANLIB", &ranlib);
        make_cmd.env("CODEX_ANDROID_DEFINES", &android_defines);
        make_cmd.env("CODEX_ANDROID_ARCH_FLAG", &arch_flag);
        make_cmd.env("V", "1");

        make_cmd.env("CODEX_LIB_PARAMS", &android_defines);

        make_cmd.env("NO_X86_INTRINSICS", "1");
        make_cmd.env("BR_NO_X86_INTRINSICS", "1");
        make_cmd.env("BR_NO_X86", "1");
        make_cmd.env("BR_NO_ASM", "1");

        match target.as_str() {
            "aarch64-linux-android" => {
                make_cmd.env("ANDROID_ARM64_BUILD", "1");
                make_cmd.env("TARGET_ARCH", "arm64");
            }
            _ => {}
        }

        let android_ndk = env::var("ANDROID_NDK_ROOT")
            .or_else(|_| env::var("ANDROID_NDK_HOME"))
            .expect("ANDROID_NDK_ROOT or ANDROID_NDK_HOME must be set for Android builds");
        let sysroot = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
            android_ndk
        );

        make_cmd.env("CMAKE_C_COMPILER", &cc);
        make_cmd.env("CMAKE_CXX_COMPILER", &cxx);
        make_cmd.env("CMAKE_AR", &ar);
        make_cmd.env("CMAKE_RANLIB", &ranlib);

        let cmake_android_defines = format!(
            "-include -DNO_TERMIOS -DNO_TERMINFO -DNO_X86_INTRINSICS -DBR_NO_X86_INTRINSICS -DBR_NO_X86 -DBR_NO_ASM",
        );
        make_cmd.env("CMAKE_C_FLAGS", &cmake_android_defines);
        make_cmd.env("CMAKE_CXX_FLAGS", &cmake_android_defines);
        make_cmd.env("CMAKE_SYSTEM_NAME", "Android");
        make_cmd.env("CMAKE_SYSTEM_PROCESSOR", &cpu);
        make_cmd.env("CMAKE_ANDROID_STANDALONE_TOOLCHAIN", &android_ndk);
        make_cmd.env("CMAKE_FIND_ROOT_PATH", &sysroot);
        make_cmd.env("CMAKE_FIND_ROOT_PATH_MODE_PROGRAM", "NEVER");
        make_cmd.env("CMAKE_FIND_ROOT_PATH_MODE_LIBRARY", "ONLY");
        make_cmd.env("CMAKE_FIND_ROOT_PATH_MODE_INCLUDE", "ONLY");

        make_cmd.env("CC", &cc);
        make_cmd.env("CXX", &cxx);
        make_cmd.env("LD", &cc);
        make_cmd.env("LINKER", &cc);
        make_cmd.env("AR", &ar);
        make_cmd.env("RANLIB", &ranlib);

        make_cmd.env("NIM_TARGET", "android");
        make_cmd.env("NIM_ARCH", &cpu);
        make_cmd.env("OS", "android");
        make_cmd.env("detected_OS", "android");

        make_cmd.env("CFLAGS", "-O2 -fPIC");
        make_cmd.env("CXXFLAGS", "-O2 -fPIC");
        make_cmd.env("LDFLAGS", "-O2 -fPIC");

        println!("Running make command: {:?}", make_cmd);
        let output = make_cmd.output().expect("Failed to execute make command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            eprintln!("Make build failed with stderr:");
            eprintln!("{}", stderr);
            eprintln!("Make build stdout:");
            eprintln!("{}", stdout);

            panic!("Failed to build libcodex for Android");
        }

        println!("Successfully built libcodex (dynamic) for Android");
    } else {
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

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/vendor/libbacktrace-upstream/.libs")
            .display()
    );

    let circom_dir = if is_android {
        let target_arch = match target.as_str() {
            "aarch64-linux-android" => "aarch64-linux-android",
            _ => "aarch64-linux-android",
        };
        nim_codex_dir.join(format!(
            "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/{}/release",
            target_arch
        ))
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

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/install/usr/lib")
            .display()
    );

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

    println!("cargo:rustc-link-lib=static=backtrace");
    println!("cargo:rustc-link-lib=static=circom_compat_ffi");
    println!("cargo:rustc-link-lib=static=natpmp");
    println!("cargo:rustc-link-lib=static=miniupnpc");
    println!("cargo:rustc-link-lib=static=backtracenim");
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

    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdCount=0");
    println!("cargo:rustc-link-arg=-Wl,--defsym=cmdLine=0");
}

fn link_dynamic_library(lib_dir: &PathBuf) {
    println!("cargo:rustc-link-lib=dylib=codex");

    let lib_dir_abs = std::fs::canonicalize(lib_dir).unwrap_or_else(|_| lib_dir.to_path_buf());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir_abs.display());
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

    generate_bindings(&nim_codex_dir);
}

pub fn apply_android_patches_during_build() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let target = env::var("TARGET").unwrap_or_default();
    let (arch, _) = get_android_arch_from_target(&target);

    println!(
        "🔧 Applying Android patches for target: {} (arch: {})",
        target, arch
    );

    let engine = PatchEngine::new(true)?;

    let applied_patches = engine.apply_patches_for_arch(arch)?;

    println!(
        "✅ Successfully applied {} patches for architecture {}",
        applied_patches.len(),
        arch
    );

    Ok(applied_patches)
}

/// Set up cargo rerun triggers for patch system files
pub fn setup_cargo_rerun_triggers() {
    println!("cargo:rerun-if-changed=android-patches/");
    println!("cargo:rerun-if-changed=src/patch_system.rs");
}
