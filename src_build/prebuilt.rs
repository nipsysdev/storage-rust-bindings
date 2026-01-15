use std::fs;
use std::path::PathBuf;

// Import sibling modules
use super::checksum;
use super::download;
use super::github;
use super::version;

/// Maps Rust target triple to platform identifier for prebuilt binaries
pub fn map_target_to_platform(target: &str) -> Option<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" => Some("linux-amd64"),
        "aarch64-unknown-linux-gnu" => Some("linux-arm64"),
        _ => None,
    }
}

/// Ensures prebuilt binary is available in OUT_DIR
/// Downloads and extracts if not cached
pub fn ensure_prebuilt_binary(
    out_dir: &PathBuf,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Map target to platform
    let platform = map_target_to_platform(target).ok_or_else(|| {
        format!(
            "Unsupported target: {}. Supported platforms: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu. \
             For Android support, please use the previous version or build from source.",
            target
        )
    })?;

    println!("Target platform: {}", platform);

    // Check cache
    let cache_marker = out_dir.join(".prebuilt_cached");
    let lib_path = out_dir.join("libstorage.a");
    let header_path = out_dir.join("libstorage.h");
    let checksum_path = out_dir.join("libstorage.a.sha256");

    if cache_marker.exists() && lib_path.exists() && header_path.exists() {
        println!("Using cached prebuilt binaries");

        // Verify checksum even for cached files
        if checksum_path.exists() {
            if let Err(e) = checksum::verify_checksum(&lib_path, &checksum_path) {
                println!(
                    "Warning: Checksum verification failed for cached files: {}",
                    e
                );
                println!("Re-downloading prebuilt binaries...");
                let _ = fs::remove_file(&cache_marker);
            } else {
                return Ok(out_dir.clone());
            }
        }
    }

    // Get release version
    let release_version = version::get_release_version()?;

    // Fetch release
    println!("Fetching release from GitHub...");
    let release = github::fetch_release(&release_version)?;
    println!("Release: {}", release.tag_name);

    // Find matching asset
    let asset = github::find_matching_asset(&release, platform).ok_or_else(|| {
        format!(
            "No prebuilt binary found for platform: {} in release: {}. \
             Please check the GitHub releases page for available platforms.",
            platform, release.tag_name
        )
    })?;

    println!("Downloading: {}", asset.name);

    // Download and extract
    download::download_and_extract(&asset.browser_download_url, out_dir)?;

    // Verify files exist
    if !lib_path.exists() {
        return Err(format!(
            "libstorage.a not found after extraction at {}. \
             This should not happen - please report this issue.",
            lib_path.display()
        )
        .into());
    }

    if !header_path.exists() {
        return Err(format!(
            "libstorage.h not found after extraction at {}. \
             This should not happen - please report this issue.",
            header_path.display()
        )
        .into());
    }

    // Verify checksum
    if checksum_path.exists() {
        checksum::verify_checksum(&lib_path, &checksum_path)?;
    } else {
        println!("Warning: Checksum file not found, skipping verification");
    }

    // Create cache marker
    fs::write(&cache_marker, "")?;

    println!("Successfully extracted prebuilt binaries");
    Ok(out_dir.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_target_to_platform() {
        assert_eq!(
            map_target_to_platform("x86_64-unknown-linux-gnu"),
            Some("linux-amd64")
        );
        assert_eq!(
            map_target_to_platform("aarch64-unknown-linux-gnu"),
            Some("linux-arm64")
        );
        assert_eq!(map_target_to_platform("x86_64-pc-windows-msvc"), None);
    }
}
