use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Import patch_system from the main module
use crate::patch_system::{get_android_arch_from_target, PatchEngine};

// Import the parallelism module for get_parallel_jobs function
use super::parallelism::get_parallel_jobs;

/// Detects the Clang version in the Android NDK
fn detect_clang_version(android_ndk: &str) -> Result<String, Box<dyn std::error::Error>> {
    let clang_lib_path =
        PathBuf::from(android_ndk).join("toolchains/llvm/prebuilt/linux-x86_64/lib/clang");

    if !clang_lib_path.exists() {
        return Err(format!("Clang lib path not found: {}", clang_lib_path.display()).into());
    }

    // Read the directory and find the highest version number
    let mut versions = Vec::new();
    for entry in fs::read_dir(clang_lib_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(version_str) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(version_num) = version_str.parse::<u32>() {
                    versions.push((version_num, version_str.to_string()));
                }
            }
        }
    }

    if versions.is_empty() {
        return Err("No Clang version directories found".into());
    }

    // Sort by version number and take the highest
    versions.sort_by_key(|(num, _)| *num);
    let (_, latest_version) = versions.last().unwrap();

    println!("cargo:warning=Detected Clang version: {}", latest_version);
    Ok(latest_version.to_string())
}

/// Sets up Android cross-compilation environment
pub fn setup_android_cross_compilation(target: String) {
    println!(
        "cargo:warning=Setting up Android cross-compilation for target: {}",
        target
    );

    // Try multiple environment variable names and fallback paths for Android SDK
    let android_sdk = env::var("ANDROID_SDK_ROOT")
        .or_else(|_| env::var("ANDROID_HOME"))
        .expect("ANDROID_SDK_ROOT or ANDROID_HOME environment variable must be set");

    let android_ndk = env::var("NDK_HOME")
        .or_else(|_| env::var("ANDROID_NDK_HOME"))
        .or_else(|_| env::var("ANDROID_NDK_ROOT"))
        .expect("NDK_HOME, ANDROID_NDK_HOME or ANDROID_NDK_ROOT environment variable must be set");

    if !std::path::Path::new(&android_sdk).exists() {
        panic!("Android SDK not found at {}.", android_sdk);
    }
    if !std::path::Path::new(&android_ndk).exists() {
        panic!("Android NDK not found at {}.", android_ndk);
    }

    // Clean architecture-specific build artifacts to prevent cross-architecture contamination
    // This will be handled by the main build.rs clean_architecture_specific_artifacts() function
    // which is called before Android setup

    let target_clone = target.clone();

    unsafe {
        env::set_var(&format!("CARGO_TARGET_{}", target), "1");
        env::set_var(&format!("CARGO_LINKER_{}", target), "clang");

        env::set_var("CARGO_TARGET_{}_LINKER", target);
    }

    let (arch, _) = get_android_arch_from_target(&target_clone);

    // Detect Clang version dynamically
    let clang_version =
        detect_clang_version(&android_ndk).expect("Failed to detect Clang version in Android NDK");

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

        // Set architecture-specific environment variables
        match target_clone.as_str() {
            "aarch64-linux-android" => {
                env::set_var("CC_aarch64_linux_android", &cc);
                env::set_var("CXX_aarch64_linux_android", &cxx);
                env::set_var("AR_aarch64_linux_android", &ar);
                env::set_var("RANLIB_aarch64_linux_android", &ranlib);
            }
            "x86_64-linux-android" => {
                env::set_var("CC_x86_64_linux_android", &cc);
                env::set_var("CXX_x86_64_linux_android", &cxx);
                env::set_var("AR_x86_64_linux_android", &ar);
                env::set_var("RANLIB_x86_64_linux_android", &ranlib);
            }
            _ => panic!("Unsupported Android target: {}", target_clone),
        }
    }

    let sysroot = format!(
        "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
        android_ndk
    );

    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}/21",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-arg=-L{}/usr/lib/{}",
        sysroot, target_clone
    );

    let arch_flag = match target_clone.as_str() {
        "aarch64-linux-android" => "-march=armv8-a",
        "x86_64-linux-android" => "-march=x86-64",
        _ => panic!("Unsupported Android target: {}", target_clone),
    };

    let arch_define = match target_clone.as_str() {
        "aarch64-linux-android" => "-d:arm64",
        "x86_64-linux-android" => "-d:x86_64",
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
            "x86_64-linux-android" => {
                env::set_var("ANDROID_X86_64_BUILD", "1");
                env::set_var("TARGET_ARCH", "x86_64");
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

        // CRITICAL: Set the environment variables that the build.nims patch expects
        env::set_var("ANDROID_SDK_ROOT", &android_sdk);
        env::set_var("ANDROID_NDK_HOME", &android_ndk);
        env::set_var("ANDROID_NDK_ROOT", &android_ndk);
        env::set_var("ANDROID_CLANG_VERSION", &clang_version);
    }

    // Set Rust/Cargo cross-compilation environment variables for circom-compat-ffi
    unsafe {
        env::set_var("CARGO_BUILD_TARGET", &target_clone);

        // Set architecture-specific environment variables
        match target_clone.as_str() {
            "aarch64-linux-android" => {
                env::set_var("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", &cc);
                env::set_var("CC_aarch64_linux_android", &cc);
                env::set_var("CXX_aarch64_linux_android", &cxx);
                env::set_var("AR_aarch64_linux_android", &ar);
                env::set_var("RANLIB_aarch64_linux_android", &ranlib);
            }
            "x86_64-linux-android" => {
                env::set_var("CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER", &cc);
                env::set_var("CC_x86_64_linux_android", &cc);
                env::set_var("CXX_x86_64_linux_android", &cxx);
                env::set_var("AR_x86_64_linux_android", &ar);
                env::set_var("RANLIB_x86_64_linux_android", &ranlib);
            }
            _ => panic!("Unsupported Android target: {}", target_clone),
        }

        // Set generic CC/AR for Rust's build scripts that don't use target-specific vars
        env::set_var("CC", &cc);
        env::set_var("CXX", &cxx);
        env::set_var("AR", &ar);
        env::set_var("RANLIB", &ranlib);
    }

    println!("cargo:rustc-link-lib=dylib=android");
    println!("cargo:rustc-link-lib=dylib=log");
    println!("cargo:rustc-link-lib=dylib=OpenSLES");
    println!("cargo:rustc-link-lib=dylib=c++_shared");

    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}/21",
        sysroot, target_clone
    );
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/{}",
        sysroot, target_clone
    );

    let (_, openmp_arch) = get_android_arch_from_target(&target_clone);

    let openmp_lib_path = format!(
        "{}/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/{}/lib/linux/{}",
        android_ndk, clang_version, openmp_arch
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

    // Disable LSE atomics detection for Android to prevent getauxval() crash on Android 16
    // LSE atomics are a performance optimization that can be safely disabled
    println!("cargo:rustc-env=RUSTFLAGS=-C target-feature=-lse");

    // Force Rust to use the Android NDK linker directly
    // Set the linker path in the environment so clang can find it
    let android_ld_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);

    // Get the current system PATH and append Android NDK path to preserve system tools like bash
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", current_path, android_ld_path);
    println!("cargo:rustc-env=PATH={}", new_path);
    // Let Android NDK clang use its default linker
    // println!("cargo:rustc-link-arg=-fuse-ld=lld");

    // FIX: Force the use of Android NDK lld linker and use shared libc
    // This prevents the PIC linking error with __cpu_model symbol
    println!("cargo:rustc-link-arg=-fuse-ld=lld");

    // Add linker flags to avoid the problematic static libraries
    println!("cargo:rustc-link-arg=-Wl,--as-needed");
    println!("cargo:rustc-link-arg=-Wl,--gc-sections");

    // Explicitly link against the shared libc from API level 21
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}/usr/lib/{}/21",
        sysroot, target_clone
    );

    // Force dynamic linking for libc to avoid PIC issues
    println!("cargo:rustc-link-lib=dylib=c");
    println!("cargo:rustc-link-lib=dylib=m");

    // Manually add the Android runtime libraries that would normally be included
    println!("cargo:rustc-link-lib=dylib=dl");

    // pthread is built into libc on all Android architectures, no separate linking needed

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

/// Builds libcodex with static linking for Android
pub fn build_libcodex_static_android(nim_codex_dir: &PathBuf, codex_params: &str) {
    println!("Building libcodex with static linking for Android...");

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        &format!("-j{}", get_parallel_jobs()),
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "STATIC=1",
        "libcodex",
    ]);

    // CRITICAL: Set NIM_PARAMS FIRST to prevent .DEFAULT target from running
    // This must be set before any other environment variables to prevent git submodule update
    make_cmd.env("NIM_PARAMS", codex_params); // This prevents the .DEFAULT target from running

    make_cmd.env("USE_LIBBACKTRACE", "0");
    // CRITICAL: Prevent Makefile from updating submodules after patches are applied
    // This ensures our patches don't get overwritten by git submodule update
    make_cmd.env("CODEX_LIB_PARAMS", codex_params);

    // CRITICAL: Ensure NIM_PARAMS is also set as CODEX_LIB_PARAMS for consistency
    // The Makefile adds CODEX_LIB_PARAMS to NIM_PARAMS, so this double-ensures NIM_PARAMS is set
    if !codex_params.is_empty() {
        make_cmd.env("NIM_PARAMS", codex_params);
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

        panic!("Failed to build libcodex with static linking for Android.");
    }

    println!("Successfully built libcodex (static) for Android");
}

/// Builds libcodex with dynamic linking for Android
pub fn build_libcodex_dynamic_android(nim_codex_dir: &PathBuf, target: &str) {
    println!("Building libcodex with make for Android...");

    let cpu = env::var("CODEX_ANDROID_CPU").unwrap_or_default();
    let cc = env::var("CODEX_ANDROID_CC").unwrap_or_default();
    let cxx = env::var("CXX_").unwrap_or_else(|_| cc.replace("-clang", "-clang++"));
    let ar = env::var("CODEX_ANDROID_AR").unwrap_or_default();
    let ranlib = env::var("CODEX_ANDROID_RANLIB").unwrap_or_default();
    let android_defines = env::var("CODEX_ANDROID_DEFINES").unwrap_or_default();
    let arch_flag = env::var("CODEX_ANDROID_ARCH_FLAG").unwrap_or_default();

    let mut make_cmd = Command::new("make");
    make_cmd.args(&[
        &format!("-j{}", get_parallel_jobs()),
        "-C",
        &nim_codex_dir.to_string_lossy(),
        "libcodex",
    ]);

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

    match target {
        "aarch64-linux-android" => {
            make_cmd.env("ANDROID_ARM64_BUILD", "1");
            make_cmd.env("TARGET_ARCH", "arm64");
        }
        "x86_64-linux-android" => {
            make_cmd.env("ANDROID_X86_64_BUILD", "1");
            make_cmd.env("TARGET_ARCH", "x86_64");
        }
        _ => {}
    }

    let android_ndk = env::var("NDK_HOME")
        .or_else(|_| env::var("ANDROID_NDK_HOME"))
        .or_else(|_| env::var("ANDROID_NDK_ROOT"))
        .expect("NDK_HOME, ANDROID_NDK_HOME or ANDROID_NDK_ROOT environment variable must be set");
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
}

/// Gets the Android-specific circom directory
pub fn get_android_circom_dir(nim_codex_dir: &PathBuf, target: &str) -> PathBuf {
    let circom_dir = match target {
        "aarch64-linux-android" => "aarch64-linux-android",
        "x86_64-linux-android" => "x86_64-linux-android",
        _ => "aarch64-linux-android",
    };

    nim_codex_dir.join(format!(
        "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/{}/release",
        circom_dir
    ))
}

/// Forces LevelDB rebuild for Android to ensure proper cross-compilation
pub fn force_leveldb_rebuild_android(nim_codex_dir: &PathBuf) {
    println!("🔧 Forcing LevelDB rebuild for Android...");

    let leveldb_dir = nim_codex_dir.join("vendor/nim-leveldbstatic");
    let leveldb_build_dir = leveldb_dir.join("build");

    // Remove LevelDB build directory
    if leveldb_build_dir.exists() {
        println!(
            "  Removing LevelDB build directory: {:?}",
            leveldb_build_dir
        );
        let _ = fs::remove_dir_all(&leveldb_build_dir);
    }

    // Remove any compiled LevelDB libraries
    let leveldb_lib_patterns = ["libleveldb.a", "libleveldb.so", "leveldb"];

    for pattern in &leveldb_lib_patterns {
        let lib_path = leveldb_dir.join(pattern);
        if lib_path.exists() {
            println!("  Removing LevelDB library: {:?}", lib_path);
            let _ = fs::remove_file(&lib_path);
        }
    }

    // Remove any cached Nim files for LevelDB
    let home_dir = env::var("HOME").unwrap_or_default();
    let nim_cache_leveldb = PathBuf::from(home_dir)
        .join(".cache/nim")
        .join("leveldbstatic");

    if nim_cache_leveldb.exists() {
        println!("  Removing Nim LevelDB cache: {:?}", nim_cache_leveldb);
        let _ = fs::remove_dir_all(&nim_cache_leveldb);
    }

    println!("  ✅ LevelDB rebuild preparation complete");
}

/// Applies Android patches during build
pub fn apply_android_patches_during_build(
    nim_codex_dir: &PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let target = env::var("TARGET").unwrap_or_default();
    let (arch, _) = get_android_arch_from_target(&target);

    println!(
        "🔧 Applying Android patches for target: {} (arch: {})",
        target, arch
    );

    // Force LevelDB rebuild for Android to ensure proper cross-compilation
    force_leveldb_rebuild_android(nim_codex_dir);

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
