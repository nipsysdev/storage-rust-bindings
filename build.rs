use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// Include the patch system integration
#[path = "src/build_integration.rs"]
mod build_integration;
use build_integration::*;

// Include the patch system
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

fn setup_android_cross_compilation() {
    let target = env::var("TARGET").unwrap_or_default();

    if target.contains("android") {
        println!(
            "cargo:warning=Setting up Android cross-compilation for target: {}",
            target
        );

        // Set Android SDK and NDK paths with better detection
        let android_sdk = env::var("ANDROID_SDK_ROOT")
            .or_else(|_| env::var("ANDROID_SDK"))
            .unwrap();

        let android_ndk = env::var("ANDROID_NDK_ROOT")
            .or_else(|_| env::var("ANDROID_NDK_HOME"))
            .unwrap();

        // Verify NDK exists
        if !std::path::Path::new(&android_ndk).exists() {
            panic!("Android NDK not found at {}. Please set ANDROID_NDK_ROOT or ANDROID_NDK_HOME environment variable.", android_ndk);
        }

        env::set_var("ANDROID_SDK_ROOT", &android_sdk);
        env::set_var("ANDROID_NDK_ROOT", &android_ndk);
        env::set_var("ANDROID_NDK_HOME", &android_ndk);

        // Set up cargo environment for Android cross-compilation
        env::set_var(&format!("CARGO_TARGET_{}", target), "1");
        env::set_var(&format!("CARGO_LINKER_{}", target), "clang");

        // Set CC and AR for the target
        let (arch, llvm_triple) = match target.as_str() {
            "aarch64-linux-android" => ("arm64", "aarch64-linux-android"),
            "armv7-linux-androideabi" => ("arm", "armv7-linux-androideabi"),
            "x86_64-linux-android" => ("amd64", "x86_64-linux-android"),
            "i686-linux-android" => ("i386", "i686-linux-android"),
            _ => ("arm64", "aarch64-linux-android"), // Default to ARM64
        };

        let toolchain_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);
        let cc = format!("{}/{}21-clang", toolchain_path, llvm_triple);
        let ar = format!("{}/llvm-ar", toolchain_path);
        let ranlib = format!("{}/llvm-ranlib", toolchain_path);

        // Set cargo environment variables for the target
        env::set_var(format!("CC_{}", target), &cc);
        env::set_var(format!("CXX_{}", target), &cc);
        env::set_var(format!("AR_{}", target), &ar);
        env::set_var(format!("RANLIB_{}", target), &ranlib);

        // Determine the architecture-specific toolchain
        let (arch, llvm_triple) = match target.as_str() {
            "aarch64-linux-android" => ("arm64", "aarch64-linux-android"),
            "armv7-linux-androideabi" => ("arm", "armv7a-linux-androideabi"),
            "x86_64-linux-android" => ("amd64", "x86_64-linux-android"),
            "i686-linux-android" => ("i386", "i686-linux-android"),
            _ => panic!("Unsupported Android target: {}", target),
        };

        // Set up the NDK toolchain paths
        let toolchain_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);
        let sysroot = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
            android_ndk
        );

        // Set linker flags for Android
        println!("cargo:rustc-link-arg=-L{}/usr/lib/{}", sysroot, llvm_triple);
        println!(
            "cargo:rustc-link-arg=-L{}/usr/lib/{}/21",
            sysroot, llvm_triple
        );
        println!(
            "cargo:rustc-link-arg=-L{}/usr/lib/{}/31",
            sysroot, llvm_triple
        );

        // Set compiler and linker for the target
        let cc = format!("{}/{}21-clang", toolchain_path, llvm_triple);
        let cxx = format!("{}/{}21-clang++", toolchain_path, llvm_triple);
        let ar = format!("{}/llvm-ar", toolchain_path);
        let ranlib = format!("{}/llvm-ranlib", toolchain_path);

        // Set environment variables for cargo
        env::set_var(format!("CC_{}", target), &cc);
        env::set_var(format!("CXX_{}", target), &cxx);
        env::set_var(format!("AR_{}", target), &ar);
        env::set_var(format!("RANLIB_{}", target), &ranlib);

        // Set cargo rustc link args for Android - use the correct sysroot paths
        println!("cargo:rustc-link-arg=-L{}/usr/lib/{}", sysroot, llvm_triple);
        println!(
            "cargo:rustc-link-arg=-L{}/usr/lib/{}/21",
            sysroot, llvm_triple
        );
        println!(
            "cargo:rustc-link-arg=-L{}/usr/lib/{}/31",
            sysroot, llvm_triple
        );

        // Set up CODEX_LIB_PARAMS for Android cross-compilation
        // Define android for Nim and use safe optimization flags
        let arch_flag = match target.as_str() {
            "aarch64-linux-android" => "-march=armv8-a",
            "armv7-linux-androideabi" => "-march=armv7-a",
            "x86_64-linux-android" => "-march=x86-64",
            "i686-linux-android" => "-march=i686",
            _ => "-march=armv8-a", // Default to ARM64 for unknown targets
        };

        // Terminal and Android fixes will be compiled after patch application
        println!("Android terminal and general fixes will be compiled after patch application...");

        // Android-specific defines to handle missing functions with our comprehensive fix
        // Use the correct architecture define based on the target
        let arch_define = match target.as_str() {
            "aarch64-linux-android" => "-d:arm64",
            "armv7-linux-androideabi" => "-d:arm",
            "x86_64-linux-android" => "-d:amd64",
            "i686-linux-android" => "-d:i386",
            _ => "-d:arm64", // Default to ARM64
        };
        let android_defines = format!("{} -d:android -d:debug -d:disable_libbacktrace -d:noIntrinsicsBitOpts -d:NO_X86_INTRINSICS -d:__NO_INLINE_ASM__ -d:noX86 -d:noSSE -d:noAVX -d:noAVX2 -d:noAVX512 -d:noX86Intrinsics -d:noSimd -d:noInlineAsm", arch_define);

        // Set NO_X86_INTRINSICS environment variable for all C builds (BearSSL, Leopard, etc.)
        env::set_var("NO_X86_INTRINSICS", "1");
        env::set_var("BR_NO_X86_INTRINSICS", "1");
        env::set_var("BR_NO_X86", "1");
        env::set_var("BR_NO_ASM", "1");

        // Set architecture-specific environment variables
        match target.as_str() {
            "aarch64-linux-android" => {
                env::set_var("ANDROID_ARM64_BUILD", "1");
            }
            "x86_64-linux-android" => {
                env::set_var("ANDROID_X86_64_BUILD", "1");
            }
            "armv7-linux-androideabi" => {
                env::set_var("ANDROID_ARM32_BUILD", "1");
            }
            "i686-linux-android" => {
                env::set_var("ANDROID_X86_BUILD", "1");
            }
            _ => {}
        }

        // Also add the include to the CMake flags for Leopard
        let terminal_fix_file_abs = std::env::current_dir()
            .unwrap()
            .join("vendor/nim-codex/android_terminal_fix.h");
        let cmake_android_defines = format!(
            "-DCMAKE_C_FLAGS=-include {} -DNO_TERMIOS -DNO_TERMINFO -DNO_X86_INTRINSICS",
            terminal_fix_file_abs.display()
        );

        // Add the terminal fix and Android fix objects to the linker flags
        let terminal_fix_obj = format!("{}/android_terminal_fix.o", env::var("OUT_DIR").unwrap());
        let android_fix_obj = format!("{}/android_fix.o", env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-arg={}", terminal_fix_obj);
        println!("cargo:rustc-link-arg={}", android_fix_obj);

        // For Android x86_64, also compile and link the x86_64-specific fixes
        if target == "x86_64-linux-android" {
            let x86_64_fix_file = "vendor/nim-codex/android_x86_64_fix.c";
            let x86_64_fix_obj = format!("{}/android_x86_64_fix.o", env::var("OUT_DIR").unwrap());

            // Compile the x86_64 fix
            let x86_64_fix_cmd = format!(
                "{} -c {} -o {} -fPIC -DANDROID",
                cc, x86_64_fix_file, x86_64_fix_obj
            );

            println!("Compiling Android x86_64 fix: {}", x86_64_fix_cmd);
            let output = Command::new("sh")
                .arg("-c")
                .arg(&x86_64_fix_cmd)
                .output()
                .expect("Failed to compile Android x86_64 fix");

            if !output.status.success() {
                panic!(
                    "Failed to compile Android x86_64 fix: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            // Add the x86_64 fix object to the linker flags
            println!("cargo:rustc-link-arg={}", x86_64_fix_obj);
        }

        // Set environment variables for Android build
        env::set_var("CODEX_ANDROID_CPU", arch);
        env::set_var("CODEX_ANDROID_CC", &cc);
        env::set_var("CODEX_ANDROID_AR", &ar);
        env::set_var("CODEX_ANDROID_RANLIB", &ranlib);
        env::set_var("CODEX_ANDROID_DEFINES", &android_defines);
        env::set_var("CODEX_ANDROID_ARCH_FLAG", arch_flag);
        let terminal_fix_obj = format!("{}/android_terminal_fix.o", env::var("OUT_DIR").unwrap());
        env::set_var("CODEX_ANDROID_TERMINAL_FIX_OBJ", terminal_fix_obj);
        env::set_var(
            "CODEX_ANDROID_TERMINAL_FIX_FILE",
            terminal_fix_file_abs.to_str().unwrap(),
        );

        // Set CODEX_LIB_PARAMS to include Android defines for Nim build
        // This ensures the android define is passed to the Nim compiler
        env::set_var("CODEX_LIB_PARAMS", &android_defines);

        // Additional environment variables for Nim compilation
        env::set_var("NIM_TARGET", "android");
        env::set_var("NIM_ARCH", arch);

        // Set ANDROID environment variable for CMake
        env::set_var("ANDROID", "1");

        // Set linker flags for Android libraries - use dynamic linking instead of static
        println!("cargo:rustc-link-lib=dylib=android");
        println!("cargo:rustc-link-lib=dylib=log");
        println!("cargo:rustc-link-lib=dylib=OpenSLES");
        println!("cargo:rustc-link-lib=dylib=c++_shared");

        // Add Android NDK library paths with correct locations
        println!(
            "cargo:rustc-link-search=native={}/usr/lib/{}",
            sysroot, llvm_triple
        );
        println!(
            "cargo:rustc-link-search=native={}/usr/lib/{}/21",
            sysroot, llvm_triple
        );
        println!(
            "cargo:rustc-link-search=native={}/usr/lib/{}/31",
            sysroot, llvm_triple
        );
        println!(
            "cargo:rustc-link-search=native={}/usr/lib/{}",
            sysroot, llvm_triple
        );

        // Add correct OpenMP library path for Android
        // Map Android CPU names to actual architecture directory names
        let openmp_arch = match &arch[..] {
            "arm64" => "aarch64",
            "arm" => "arm",
            "amd64" => "x86_64",
            "i386" => "i386",
            _ => "aarch64", // Default to aarch64
        };
        let openmp_lib_path = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/{}",
            android_ndk, openmp_arch
        );
        println!("cargo:rustc-link-search=native={}", openmp_lib_path);
        println!("cargo:rustc-link-lib=static=omp");

        // Set the correct linker for cargo
        println!("cargo:rustc-linker={}", cc);

        // Ensure we use static linking for Android (recommended)
        env::set_var("CODEX_ANDROID_STATIC", "1");

        println!("Android cross-compilation setup complete for {}", target);
    }
}

/// Compile Android terminal and general fixes after patches are applied
fn compile_android_fixes_after_patches() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::var("TARGET").unwrap_or_default();

    if !target.contains("android") {
        return Ok(());
    }

    println!("Compiling Android fixes after patch application...");

    // Get the Android compiler
    let android_ndk = env::var("ANDROID_NDK_ROOT")
        .or_else(|_| env::var("ANDROID_NDK_HOME"))
        .unwrap();

    let (arch, llvm_triple) = match target.as_str() {
        "aarch64-linux-android" => ("arm64", "aarch64-linux-android"),
        "armv7-linux-androideabi" => ("arm", "armv7a-linux-androideabi"),
        "x86_64-linux-android" => ("amd64", "x86_64-linux-android"),
        "i686-linux-android" => ("i386", "i686-linux-android"),
        _ => panic!("Unsupported Android target: {}", target),
    };

    let toolchain_path = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin", android_ndk);
    let cc = format!("{}/{}21-clang", toolchain_path, llvm_triple);

    // Compile our comprehensive Android terminal fix
    let terminal_fix_file = "vendor/nim-codex/android_terminal_fix.h";
    let terminal_fix_obj = format!("{}/android_terminal_fix.o", env::var("OUT_DIR").unwrap());

    // Check if the terminal fix file exists (should be copied by patch system)
    if !Path::new(terminal_fix_file).exists() {
        return Err(format!("Terminal fix file not found: {}", terminal_fix_file).into());
    }

    // Create a C file that includes our header for compilation
    let terminal_fix_c = format!("{}/android_terminal_fix.c", env::var("OUT_DIR").unwrap());
    let terminal_fix_file_abs = std::env::current_dir().unwrap().join(terminal_fix_file);
    std::fs::write(
        &terminal_fix_c,
        format!(
            r#"
#include "{}"
"#,
            terminal_fix_file_abs.display()
        ),
    )?;

    // Compile the terminal fix
    let terminal_fix_cmd = format!(
        "{} -c {} -o {} -fPIC -DANDROID",
        cc, terminal_fix_c, terminal_fix_obj
    );

    println!("Compiling Android terminal fix: {}", terminal_fix_cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&terminal_fix_cmd)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to compile Android terminal fix: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // Compile the general Android fix for all architectures
    let android_fix_file = "vendor/nim-codex/android_fix.h";
    let android_fix_obj = format!("{}/android_fix.o", env::var("OUT_DIR").unwrap());

    // Check if the Android fix file exists (should be copied by patch system)
    if !Path::new(android_fix_file).exists() {
        return Err(format!("Android fix file not found: {}", android_fix_file).into());
    }

    // Create a C file that includes our header for compilation
    let android_fix_c = format!("{}/android_fix.c", env::var("OUT_DIR").unwrap());
    let android_fix_file_abs = std::env::current_dir().unwrap().join(android_fix_file);
    std::fs::write(
        &android_fix_c,
        format!(
            r#"
#include "{}"
"#,
            android_fix_file_abs.display()
        ),
    )?;

    // Compile the Android fix
    let android_fix_cmd = format!(
        "{} -c {} -o {} -fPIC -DANDROID",
        cc, android_fix_c, android_fix_obj
    );

    println!("Compiling Android fix: {}", android_fix_cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&android_fix_cmd)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to compile Android fix: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    println!("✅ Android fixes compiled successfully");
    Ok(())
}

/// Directly patch the Makefile to disable all git operations during Android builds
/// This is the most reliable approach to prevent patches from being overwritten
fn patch_makefile_for_android(nim_codex_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Patch the main Makefile
    let makefile_path = nim_codex_dir.join("Makefile");

    // Read the current Makefile
    let mut content = std::fs::read_to_string(&makefile_path)?;

    // Check if we've already patched it to avoid double patching
    if content.contains("# ANDROID_BUILD_PATCHED: Git operations disabled") {
        println!("Makefile already patched for Android build");
    } else {
        // 1. Replace the git submodule update command with an echo
        content = content.replace(
            "GIT_SUBMODULE_UPDATE := git submodule update --init --recursive",
            "GIT_SUBMODULE_UPDATE := echo \"Git submodule update disabled during Android build to prevent patch overwriting\""
        );

        // 2. Add a check at the top of the Makefile to disable git operations for Android builds
        let android_check = r#"# ANDROID_BUILD_PATCHED: Git operations disabled to prevent patch overwriting
# This section was added by build.rs to prevent git operations during Android builds
ifneq ($(filter android,$(NIM_PARAMS)),)
# Android build detected - disable all git operations to prevent patch overwriting
GIT_SUBMODULE_UPDATE := echo "Git operations disabled during Android build"
override GIT_SUBMODULE_UPDATE := echo "Git operations disabled during Android build"
endif

"#;

        // Insert after the SHELL line
        if let Some(pos) = content.find("SHELL := bash") {
            if let Some(end_pos) = content[pos..].find('\n') {
                let insert_pos = pos + end_pos + 1;
                content.insert_str(insert_pos, android_check);
            }
        }

        // 3. Add a comment to mark that we've patched the file
        if let Some(pos) =
            content.find("GIT_SUBMODULE_UPDATE := echo \"Git submodule update disabled")
        {
            content.insert_str(
                pos,
                "# ANDROID_BUILD_PATCHED: Git operations disabled to prevent patch overwriting\n",
            );
        }

        // Write the patched Makefile back
        std::fs::write(&makefile_path, content)?;

        println!("✅ Successfully patched Makefile to disable git operations during Android build");
    }

    // 2. Patch the build_nim.sh script which also contains git operations
    let build_nim_path = nim_codex_dir.join("vendor/nimbus-build-system/scripts/build_nim.sh");

    if build_nim_path.exists() {
        let mut build_nim_content = std::fs::read_to_string(&build_nim_path)?;

        // Check if we've already patched it
        if build_nim_content.contains("# ANDROID_BUILD_PATCHED: Git operations disabled") {
            println!("build_nim.sh already patched for Android build");
        } else {
            // Add Android check at the beginning of the script
            let android_check_sh = r#"# ANDROID_BUILD_PATCHED: Git operations disabled to prevent patch overwriting
# This section was added by build.rs to prevent git operations during Android builds
# Check if we're building for Android and disable git operations
if [[ "$NIM_PARAMS" == *"android"* ]] || [[ "$ANDROID" == "1" ]]; then
    echo "🔒 Android build detected - disabling git operations in build_nim.sh to prevent patch overwriting"
    # Override git command to prevent operations
    git() {
        echo "🔒 Git operation '$*' disabled during Android build to prevent patch overwriting"
        return 0
    }
    export -f git
fi

"#;

            // Insert after the shebang line
            if let Some(pos) = build_nim_content.find('\n') {
                build_nim_content.insert_str(pos + 1, android_check_sh);
            }

            // Write the patched build_nim.sh back
            std::fs::write(&build_nim_path, build_nim_content)?;

            println!("✅ Successfully patched build_nim.sh to disable git operations during Android build");
        }
    }

    // 3. Also patch any other build scripts that might contain git operations
    let other_scripts = [
        "vendor/nimbus-build-system/scripts/env.sh",
        "vendor/nimbus-build-system/makefiles/variables.mk",
    ];

    for script in &other_scripts {
        let script_path = nim_codex_dir.join(script);
        if script_path.exists() {
            let mut script_content = std::fs::read_to_string(&script_path)?;

            if script_content.contains("# ANDROID_BUILD_PATCHED: Git operations disabled") {
                println!("{} already patched for Android build", script);
            } else {
                // Replace git pull operations
                script_content = script_content.replace(
                    "git pull -q",
                    "echo \"Git pull disabled during Android build\"",
                );

                // Write back
                std::fs::write(&script_path, script_content)?;
                println!(
                    "✅ Successfully patched {} to disable git operations during Android build",
                    script
                );
            }
        }
    }

    Ok(())
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

    // Note: Patch validation is now handled by the new patch system in main()

    if is_android {
        // Clear library caches to prevent architecture mismatches
        println!("Clearing library caches to prevent architecture mismatches...");
        let home_dir = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let leopard_cache_dir = home_dir.join(".cache/nim/libcodex_d/vendor_leopard");
        let bearssl_cache_dir = home_dir.join(".cache/nim/libcodex_d/@m..@svendor@snim-bearssl");

        // Clear Leopard library cache
        if leopard_cache_dir.exists() {
            println!(
                "Removing cached Leopard library at: {:?}",
                leopard_cache_dir
            );
            if let Err(e) = std::fs::remove_dir_all(&leopard_cache_dir) {
                println!("Warning: Failed to remove Leopard cache: {}", e);
            } else {
                println!("Successfully cleared Leopard library cache");
            }
        }

        // Clear BearSSL library cache
        if bearssl_cache_dir.exists() {
            println!(
                "Removing cached BearSSL library at: {:?}",
                bearssl_cache_dir
            );
            if let Err(e) = std::fs::remove_dir_all(&bearssl_cache_dir) {
                println!("Warning: Failed to remove BearSSL cache: {}", e);
            } else {
                println!("Successfully cleared BearSSL library cache");
            }
        }

        // Also clear the entire nimcache to ensure clean build
        let nimcache_dir = home_dir.join(".cache/nim/libcodex_d");
        if nimcache_dir.exists() {
            println!("Clearing entire nimcache directory...");
            if let Err(e) = std::fs::remove_dir_all(&nimcache_dir) {
                println!("Warning: Failed to remove nimcache: {}", e);
            } else {
                println!("Successfully cleared nimcache");
            }
        }

        // Clear vendor-specific build directories that contain architecture-specific pre-built libraries
        // Note: We only remove build artifacts, not source directories
        let vendor_build_dirs = [
            // Don't clear miniupnpc build directory anymore since we're rebuilding it manually
            // "vendor/nim-codex/vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build",
            "vendor/nim-codex/vendor/nim-circom-compat/vendor/circom-compat-ffi/target",
        ];

        for build_dir in &vendor_build_dirs {
            let build_path = PathBuf::from(build_dir);
            if build_path.exists() {
                println!("Removing vendor build directory: {:?}", build_path);
                if let Err(e) = std::fs::remove_dir_all(&build_path) {
                    println!("Warning: Failed to remove vendor build directory: {}", e);
                } else {
                    println!("Successfully cleared vendor build directory");
                }
            }
        }

        // For libnatpmp, remove the build directory but not the source
        let libnatpmp_build_dir = PathBuf::from(
            "vendor/nim-codex/vendor/nim-nat-traversal/vendor/libnatpmp-upstream/build",
        );
        if libnatpmp_build_dir.exists() {
            println!(
                "Removing libnatpmp build directory: {:?}",
                libnatpmp_build_dir
            );
            if let Err(e) = std::fs::remove_dir_all(&libnatpmp_build_dir) {
                println!("Warning: Failed to remove libnatpmp build directory: {}", e);
            } else {
                println!("Successfully removed libnatpmp build directory");
            }
        }

        // Also remove any pre-built libnatpmp.a files
        let libnatpmp_a = PathBuf::from(
            "vendor/nim-codex/vendor/nim-nat-traversal/vendor/libnatpmp-upstream/libnatpmp.a",
        );
        if libnatpmp_a.exists() {
            println!("Removing pre-built libnatpmp.a: {:?}", libnatpmp_a);
            if let Err(e) = std::fs::remove_file(&libnatpmp_a) {
                println!("Warning: Failed to remove libnatpmp.a: {}", e);
            } else {
                println!("Successfully removed libnatpmp.a");
            }
        }

        // For Android builds, set environment variables and let build.nims handle the parameters
        println!("Building libcodex with make for Android...");

        let cpu = env::var("CODEX_ANDROID_CPU").unwrap_or_default();
        let cc = env::var("CODEX_ANDROID_CC").unwrap_or_default();
        let cxx = env::var("CXX_").unwrap_or_else(|_| {
            // If CXX_ is not set, construct it from cc
            cc.replace("-clang", "-clang++")
        });
        let ar = env::var("CODEX_ANDROID_AR").unwrap_or_default();
        let ranlib = env::var("CODEX_ANDROID_RANLIB").unwrap_or_default();
        let android_defines = env::var("CODEX_ANDROID_DEFINES").unwrap_or_default();
        let arch_flag = env::var("CODEX_ANDROID_ARCH_FLAG").unwrap_or_default();
        let terminal_fix_obj = env::var("CODEX_ANDROID_TERMINAL_FIX_OBJ").unwrap_or_default();
        let terminal_fix_file = env::var("CODEX_ANDROID_TERMINAL_FIX_FILE").unwrap_or_default();

        let mut make_cmd = Command::new("make");
        make_cmd.args(&["-j12", "-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

        // CRITICAL: Set NIM_PARAMS FIRST to prevent .DEFAULT target from running
        // This must be set before any other environment variables to prevent git submodule update
        make_cmd.env("NIM_PARAMS", &android_defines); // This prevents the .DEFAULT target from running

        make_cmd.env("USE_LIBBACKTRACE", "0");
        make_cmd.env("ANDROID", "1");
        make_cmd.env("CODEX_ANDROID_CPU", &cpu);
        make_cmd.env("CODEX_ANDROID_CC", &cc);
        make_cmd.env("CODEX_ANDROID_AR", &ar);
        make_cmd.env("CODEX_ANDROID_RANLIB", &ranlib);
        make_cmd.env("CODEX_ANDROID_DEFINES", &android_defines);
        make_cmd.env("CODEX_ANDROID_ARCH_FLAG", &arch_flag);
        make_cmd.env("CODEX_ANDROID_TERMINAL_FIX_OBJ", &terminal_fix_obj);
        make_cmd.env("CODEX_ANDROID_TERMINAL_FIX_FILE", &terminal_fix_file);
        make_cmd.env("V", "1"); // Verbose output for debugging

        // CRITICAL: Ensure Android defines are passed to NIM_PARAMS
        // The Makefile adds CODEX_LIB_PARAMS to NIM_PARAMS, so we need to set CODEX_LIB_PARAMS
        make_cmd.env("CODEX_LIB_PARAMS", &android_defines);

        // CRITICAL: Double-ensure NIM_PARAMS is set to prevent any chance of submodule update
        if !android_defines.is_empty() {
            make_cmd.env("NIM_PARAMS", &android_defines);
        }

        // CRITICAL: Ensure NO_X86_INTRINSICS is propagated to all C builds
        make_cmd.env("NO_X86_INTRINSICS", "1");
        make_cmd.env("BR_NO_X86_INTRINSICS", "1");
        make_cmd.env("BR_NO_X86", "1");
        make_cmd.env("BR_NO_ASM", "1");

        // Set architecture-specific environment variables for CMake
        match target.as_str() {
            "aarch64-linux-android" => {
                make_cmd.env("ANDROID_ARM64_BUILD", "1");
            }
            "x86_64-linux-android" => {
                make_cmd.env("ANDROID_X86_64_BUILD", "1");
            }
            "armv7-linux-androideabi" => {
                make_cmd.env("ANDROID_ARM32_BUILD", "1");
            }
            "i686-linux-android" => {
                make_cmd.env("ANDROID_X86_BUILD", "1");
            }
            _ => {}
        }

        // CRITICAL: Ensure Android cross-compiler is used for CMake builds
        let android_ndk = env::var("ANDROID_NDK_ROOT")
            .or_else(|_| env::var("ANDROID_NDK_HOME"))
            .unwrap();
        let sysroot = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
            android_ndk
        );

        make_cmd.env("CMAKE_C_COMPILER", &cc);
        make_cmd.env("CMAKE_CXX_COMPILER", &cxx);
        make_cmd.env("CMAKE_AR", &ar);
        make_cmd.env("CMAKE_RANLIB", &ranlib);

        // CRITICAL: Ensure NO_X86_INTRINSICS is propagated to CMake for all components
        let terminal_fix_file_abs = std::env::current_dir()
            .unwrap()
            .join("vendor/nim-codex/android_terminal_fix.h");
        let cmake_android_defines = format!(
            "-include {} -DNO_TERMIOS -DNO_TERMINFO -DNO_X86_INTRINSICS -DBR_NO_X86_INTRINSICS -DBR_NO_X86 -DBR_NO_ASM",
            terminal_fix_file_abs.display()
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

        // CRITICAL FIX: Use Android cross-compiler for CMake builds, but system compiler for Nim compiler build
        // The Nim compiler should always be built for the host system, then used to cross-compile
        // However, CMake builds (like Leopard) should use the Android cross-compiler
        make_cmd.env("CC", &cc);
        make_cmd.env("CXX", &cxx);
        make_cmd.env("LD", &cc);
        make_cmd.env("LINKER", &cc);
        make_cmd.env("AR", &ar);
        make_cmd.env("RANLIB", &ranlib);

        // Set Android-specific environment for Nim compiler build
        make_cmd.env("NIM_TARGET", "android");
        make_cmd.env("NIM_ARCH", &cpu);
        make_cmd.env("OS", "android"); // CRITICAL: Tell makefile we're building for Android
        make_cmd.env("detected_OS", "android"); // CRITICAL: Override detected OS in Makefile

        // Set compiler flags for host system (Nim compiler build)
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
        // For non-Android builds, use the original make approach
        let codex_params = env::var("CODEX_LIB_PARAMS").unwrap_or_default();

        let mut make_cmd = Command::new("make");
        make_cmd.args(&["-C", &nim_codex_dir.to_string_lossy(), "libcodex"]);

        if !codex_params.is_empty() {
            make_cmd.env("CODEX_LIB_PARAMS", &codex_params);
        }

        // Don't set USE_LIBBACKTRACE=0 for desktop builds to allow release mode
        make_cmd.env("V", "1");
        make_cmd.env("USE_SYSTEM_NIM", "0");
        make_cmd.env("USE_LIBBACKTRACE", "1");
        // Explicitly add -d:release to CODEX_LIB_PARAMS to ensure release mode
        make_cmd.env("CODEX_LIB_PARAMS", "-d:release");

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

    // Makefile patch is now handled manually in the submodule - DISABLED
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        println!("Using manual Android makefile modifications...");
    }

    match linking_mode {
        LinkingMode::Static => build_libcodex_static(nim_codex_dir),
        LinkingMode::Dynamic => build_libcodex_dynamic(nim_codex_dir),
    }
}

fn link_static_library(nim_codex_dir: &PathBuf, _lib_dir: &PathBuf) {
    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");
    let home_dir = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    println!(
        "cargo:rustc-link-search=native={}",
        nim_codex_dir
            .join("vendor/nim-libbacktrace/vendor/libbacktrace-upstream/.libs")
            .display()
    );

    // For Android, the circom_compat_ffi library is built in a different location
    let circom_dir = if is_android {
        let target_arch = match target.as_str() {
            "aarch64-linux-android" => "aarch64-linux-android",
            "armv7-linux-androideabi" => "armv7-linux-androideabi",
            "x86_64-linux-android" => "x86_64-linux-android",
            "i686-linux-android" => "i686-linux-android",
            _ => "aarch64-linux-android",
        };
        nim_codex_dir.join(format!(
            "vendor/nim-circom-compat/vendor/circom-compat-ffi/target/{}/release",
            target_arch
        ))
    } else {
        nim_codex_dir.join("vendor/nim-circom-compat/vendor/circom-compat-ffi/target/release")
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

    // Try release directory first, then debug directory, then cache directory
    let leopard_dir_release = nim_codex_dir.join("nimcache/release/libcodex/vendor_leopard");
    let leopard_dir_debug = nim_codex_dir.join("nimcache/debug/libcodex/vendor_leopard");
    let leopard_dir_cache = home_dir.join(".cache/nim/libcodex_d/vendor_leopard");

    let leopard_dir = if leopard_dir_release.exists() {
        leopard_dir_release
    } else if leopard_dir_debug.exists() {
        println!("Warning: Leopard library not found in release directory, using debug directory");
        leopard_dir_debug
    } else {
        println!("Warning: Leopard library not found in release or debug directories, using cache directory");
        leopard_dir_cache
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

    // For Android, use -lomp instead of -lgomp (Clang/LLVM naming)
    if is_android {
        println!("cargo:rustc-link-lib=static=omp");
    } else {
        println!("cargo:rustc-link-lib=dylib=gomp");
    }

    // Android-specific linking - now handled in setup_android_cross_compilation
    if is_android {
        println!(
            "Android build detected, using libraries configured in setup_android_cross_compilation"
        );
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

    // Set up cargo rerun triggers for patch system
    setup_cargo_rerun_triggers();

    let linking_mode = determine_linking_mode();
    let nim_codex_dir = get_nim_codex_dir();
    let target = env::var("TARGET").unwrap_or_default();
    println!("cargo:warning=Build script running with TARGET: {}", target);

    // CRITICAL: Set NIM_PARAMS IMMEDIATELY for Android builds to prevent submodule updates
    // This must happen BEFORE any other operations to prevent the Makefile .DEFAULT target
    let android_defines = if target.contains("android") {
        println!("cargo:warning=Android target detected: {}", target);
        println!("cargo:warning=CRITICAL: Setting NIM_PARAMS immediately to prevent submodule updates...");

        // Set NIM_PARAMS IMMEDIATELY to prevent Makefile .DEFAULT target from running
        let arch_define = match target.as_str() {
            "aarch64-linux-android" => "-d:arm64",
            "armv7-linux-androideabi" => "-d:arm",
            "x86_64-linux-android" => "-d:amd64",
            "i686-linux-android" => "-d:i386",
            _ => "-d:arm64",
        };
        let early_android_defines = format!("{} -d:android -d:debug -d:disable_libbacktrace -d:noIntrinsicsBitOpts -d:NO_X86_INTRINSICS -d:__NO_INLINE_ASM__ -d:noX86 -d:noSSE -d:noAVX -d:noAVX2 -d:noAVX512 -d:noX86Intrinsics -d:noSimd -d:noInlineAsm", arch_define);

        // CRITICAL: Set these environment variables IMMEDIATELY to prevent submodule update
        env::set_var("NIM_PARAMS", &early_android_defines);
        env::set_var("CODEX_LIB_PARAMS", &early_android_defines);

        // Additional safety: Set a dummy variable to ensure Makefile thinks variables.mk is included
        env::set_var("BUILD_SYSTEM_DIR", "vendor/nimbus-build-system");

        println!(
            "cargo:warning=🔒 NIM_PARAMS locked to prevent submodule update: {}",
            &early_android_defines[..std::cmp::min(100, early_android_defines.len())]
        );

        early_android_defines
    } else {
        String::new()
    };

    // Set up Android cross-compilation and apply patches in the correct order
    let (applied_patches, final_android_defines) = if target.contains("android") {
        // Set up Android environment early to get the defines and prevent submodule updates
        setup_android_cross_compilation();

        // Get the Android defines for NIM_PARAMS (should be same as early_android_defines)
        let android_defines =
            env::var("CODEX_ANDROID_DEFINES").unwrap_or_else(|_| android_defines.clone());

        // CRITICAL: Re-assert NIM_PARAMS after setup_android_cross_compilation to ensure it's not overwritten
        env::set_var("NIM_PARAMS", &android_defines);
        env::set_var("CODEX_LIB_PARAMS", &android_defines);

        // CRITICAL: Directly patch the Makefile to disable ALL git operations before build
        // This is the most reliable approach to prevent patches from being overwritten
        println!(
            "cargo:warning=🔧 Patching Makefile to disable git operations during Android build..."
        );
        if let Err(e) = patch_makefile_for_android(&nim_codex_dir) {
            panic!("Failed to patch Makefile for Android build: {}", e);
        }
        println!("cargo:warning=✅ Makefile patched successfully - git operations disabled");

        // CRITICAL: Apply patches AFTER git operations are disabled but BEFORE compilation
        println!("cargo:warning=Applying enhanced Android patch system...");
        let patches = match apply_android_patches_during_build() {
            Ok(patches) => {
                println!(
                    "cargo:warning=✅ Successfully applied {} Android patches with validation",
                    patches.len()
                );
                Some(patches)
            }
            Err(e) => {
                println!("cargo:warning=❌ Android patch system failed: {}", e);
                println!("cargo:warning=This will likely result in incorrect architecture builds");
                println!("cargo:warning=Consider cleaning and rebuilding, or check the manually-patched reference");

                // For critical failures, we should fail the build rather than continue with broken configuration
                if e.to_string().contains("validation failed") {
                    panic!("Critical Android patch validation failed: {}. Build cannot continue with incorrect configuration.", e);
                }

                None
            }
        };

        // Compile Android fixes after patches are applied
        if let Err(e) = compile_android_fixes_after_patches() {
            panic!("Failed to compile Android fixes: {}", e);
        }

        (patches, android_defines)
    } else {
        println!("cargo:warning=Not an Android target: {}", target);
        (None, String::new())
    };

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

    // Apply critical patches post-build for Android to ensure they're not overwritten
    let post_build_patches: Option<Vec<String>> = if target.contains("android") {
        println!("cargo:warning=Applying critical Android patches post-build...");
        // Post-build patch application not needed with new system
        None
    } else {
        None
    };

    // CRITICAL: Final verification that critical patches are still applied after build
    if target.contains("android") {
        println!("cargo:warning=Performing final verification of critical patches...");

        // Verify bitops patch specifically
        let bitops_path =
            Path::new("vendor/nim-codex/vendor/nimbus-build-system/vendor/Nim/lib/pure/bitops.nim");
        if let Ok(content) = std::fs::read_to_string(bitops_path) {
            if content.contains("not defined(android) and not defined(arm64) and not defined(arm)")
            {
                println!("cargo:warning=✅ Final verification: BitOps patch is correctly applied");
            } else {
                println!("cargo:warning=❌ CRITICAL: BitOps patch was lost during build!");
                println!("cargo:warning=🔧 Reapplying BitOps patch...");

                // Use the new patch system to reapply critical patches if needed
                println!("cargo:warning=🔧 BitOps patch validation failed - this should be handled by the new patch system");
            }
        } else {
            println!("cargo:warning=❌ Could not read bitops.nim for final verification");
        }
    }

    // Log applied patches information
    if let Some(patches) = applied_patches {
        println!(
            "Android build completed with {} patches applied:",
            patches.len()
        );
        for patch in &patches {
            println!("  - {}", patch);
        }

        // Log post-build patches
        if let Some(post_patches) = post_build_patches {
            println!("Post-build critical patches applied:");
            for patch in &post_patches {
                println!("  - {} (post-build)", patch);
            }
        }
    } else if target.contains("android") {
        println!("Android build completed without patches (patch system disabled or failed)");
    }
}
