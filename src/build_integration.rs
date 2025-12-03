//! Build System Integration for Patch System
//!
//! Minimal integration with cargo build script for Android patch application.

use crate::patch_system::{get_android_arch_from_target, is_android_build, PatchEngine};
use std::env;

/// Apply Android patches during build using the simple patch system
pub fn apply_android_patches_during_build() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if !is_android_build() {
        println!("Not an Android target, skipping patch application");
        return Ok(Vec::new());
    }

    let target = env::var("TARGET").unwrap_or_default();
    let arch = get_android_arch_from_target(&target).ok_or("Unsupported Android target")?;

    println!(
        "🔧 Applying Android patches for target: {} (arch: {})",
        target, arch
    );

    // Create patch engine
    let engine = PatchEngine::new(true)?;

    // Apply patches for this architecture
    let applied_patches = engine.apply_patches_for_arch(arch)?;

    println!(
        "✅ Successfully applied {} patches for architecture {}",
        applied_patches.len(),
        arch
    );

    Ok(applied_patches)
}

/// Validate Android patches without applying them
#[allow(dead_code)]
pub fn validate_android_patches_for_build() -> Result<(), Box<dyn std::error::Error>> {
    if !is_android_build() {
        return Ok(());
    }

    let target = env::var("TARGET").unwrap_or_default();
    let arch = get_android_arch_from_target(&target).ok_or("Unsupported Android target")?;

    println!("🔧 Validating Android patches for architecture: {}", arch);

    // Create patch engine
    let engine = PatchEngine::new(true)?;

    // Validate patches for this architecture
    engine.validate_patches_for_arch(arch)?;

    println!("✅ All patches validated for architecture {}", arch);

    Ok(())
}

/// Get patch system information for debugging
#[allow(dead_code)]
pub fn get_patch_system_info() -> Result<String, Box<dyn std::error::Error>> {
    let engine = PatchEngine::new(false)?;
    let archs = engine.get_available_architectures()?;

    let mut info = String::new();
    info.push_str("Patch System\n");
    info.push_str("============\n\n");

    for arch in &archs {
        info.push_str(&format!("Architecture: {}\n", arch));
        let patches = engine.get_patches_for_arch(arch)?;
        for (i, patch) in patches.iter().enumerate() {
            info.push_str(&format!("  {}. {}\n", i + 1, patch));
        }
        info.push('\n');
    }

    Ok(info)
}

/// Set up cargo rerun triggers for patch system files
pub fn setup_cargo_rerun_triggers() {
    println!("cargo:rerun-if-changed=android-patches/");
    println!("cargo:rerun-if-changed=src/patch_system.rs");
    println!("cargo:rerun-if-changed=src/build_integration.rs");
}
