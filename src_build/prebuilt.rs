use std::path::{Path, PathBuf};

use super::local;
use super::remote;

pub use super::local::LIBSTORAGE_H;

/// Ensures prebuilt binary is available in OUT_DIR
/// Downloads and extracts if not cached
pub fn ensure_prebuilt_binary(
    out_dir: &Path,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    log_info("Starting ensure_prebuilt_binary");
    log_info(&format!("Target: {}", target));
    log_info(&format!("Output directory: {}", out_dir.display()));

    if let Ok(result) = local::try_local_development_mode(out_dir) {
        return Ok(result);
    }

    remote::download_from_github(out_dir, target)
}

/// Validates that required files exist in the given path
pub fn validate_required_files(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Check for at least one .a library file
    let has_library = path
        .read_dir()?
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().is_some_and(|ext| ext == "a"));

    if !has_library {
        return Err("No library files (.a) found in the directory.".into());
    }

    // Check for required header file
    let libstorage_h = path.join(LIBSTORAGE_H);
    if !libstorage_h.exists() {
        return Err(format!(
            "Required header file not found: {}. \
             Please ensure the folder contains {}",
            libstorage_h.display(),
            LIBSTORAGE_H
        )
        .into());
    }

    Ok(())
}

pub fn log_info(message: &str) {
    println!("  [PREBUILT] {}", message);
}
