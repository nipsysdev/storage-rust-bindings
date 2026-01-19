use std::env;
use std::fs;
use std::path::PathBuf;

pub const CACHE_DIR_NAME: &str = "storage-bindings";
pub const FORCE_DOWNLOAD_ENV_VAR: &str = "STORAGE_BINDINGS_FORCE_DOWNLOAD";
pub const CLEAN_CACHE_ENV_VAR: &str = "STORAGE_BINDINGS_CLEAN_CACHE";

/// Gets the base cache directory (platform-specific)
/// - Linux/macOS: ~/.cache/storage-bindings/
/// - Windows: %LOCALAPPDATA%\storage-bindings\cache\
pub fn get_cache_base_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = dirs::cache_dir().ok_or("Failed to determine cache directory")?;

    Ok(cache_dir.join(CACHE_DIR_NAME))
}

/// Gets the version-specific cache directory
/// Structure: <cache-base>/<version>/<platform>/
pub fn get_version_cache_dir(
    version: &str,
    platform: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_base = get_cache_base_dir()?;
    Ok(cache_base.join(version).join(platform))
}

/// Checks if force download is requested via environment variable
pub fn should_force_download() -> bool {
    env::var(FORCE_DOWNLOAD_ENV_VAR).is_ok()
}

/// Checks if cache cleanup is requested via environment variable
pub fn should_clean_cache() -> bool {
    env::var(CLEAN_CACHE_ENV_VAR).is_ok()
}

/// Cleans the entire cache directory
pub fn clean_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache_base = get_cache_base_dir()?;

    if cache_base.exists() {
        println!(
            "  [CACHE] Removing cache directory: {}",
            cache_base.display()
        );
        fs::remove_dir_all(&cache_base)?;
        println!("  [CACHE] ✓ Cache cleaned successfully");
    } else {
        println!("  [CACHE] Cache directory does not exist, nothing to clean");
    }

    Ok(())
}

/// Copies files from cache to output directory
pub fn copy_from_cache(
    cache_dir: &PathBuf,
    out_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [CACHE] Copying files from cache to OUT_DIR...");

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("Invalid filename")?;

            let dest = out_dir.join(file_name);
            fs::copy(&path, &dest)?;
            println!("  [CACHE]   Copied: {}", file_name);
        }
    }

    println!("  [CACHE] ✓ All files copied from cache");
    Ok(())
}

/// Validates that cache directory contains required files
pub fn validate_cache(cache_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !cache_dir.exists() {
        return Err("Cache directory does not exist".into());
    }

    // Check for at least one .a library file
    let has_library = cache_dir
        .read_dir()?
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().map_or(false, |ext| ext == "a"));

    if !has_library {
        return Err("No library files (.a) found in cache".into());
    }

    // Check for required header file
    let libstorage_h = cache_dir.join("libstorage.h");
    if !libstorage_h.exists() {
        return Err("Required header file libstorage.h not found in cache".into());
    }

    Ok(())
}
