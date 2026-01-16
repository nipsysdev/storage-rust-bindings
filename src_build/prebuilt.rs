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
    println!("  [PREBUILT] Starting ensure_prebuilt_binary");
    println!("  [PREBUILT] Target: {}", target);
    println!("  [PREBUILT] Output directory: {}", out_dir.display());

    // Map target to platform
    let platform = map_target_to_platform(target).ok_or_else(|| {
        format!(
            "Unsupported target: {}. Supported platforms: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu. \
             For Android support, please use the previous version or build from source.",
            target
        )
    })?;

    println!("  [PREBUILT] Mapped target to platform: {}", platform);

    // Check cache
    let cache_marker = out_dir.join(".prebuilt_cached");
    let lib_path = out_dir.join("libstorage.a");
    let header_path = out_dir.join("libstorage.h");
    let checksum_path = out_dir.join("libstorage.a.sha256");

    println!("  [PREBUILT] Checking cache:");
    println!(
        "  [PREBUILT]   Cache marker: {} (exists: {})",
        cache_marker.display(),
        cache_marker.exists()
    );
    println!(
        "  [PREBUILT]   Library path: {} (exists: {})",
        lib_path.display(),
        lib_path.exists()
    );
    println!(
        "  [PREBUILT]   Header path: {} (exists: {})",
        header_path.display(),
        header_path.exists()
    );
    println!(
        "  [PREBUILT]   Checksum path: {} (exists: {})",
        checksum_path.display(),
        checksum_path.exists()
    );

    if cache_marker.exists() && lib_path.exists() && header_path.exists() {
        println!("  [PREBUILT] ✓ Using cached prebuilt binaries");

        // Verify checksum even for cached files
        if checksum_path.exists() {
            println!("  [PREBUILT] Verifying checksum of cached files...");
            if let Err(e) = checksum::verify_checksum(&lib_path, &checksum_path) {
                println!(
                    "  [PREBUILT] ⚠ Checksum verification failed for cached files: {}",
                    e
                );
                println!("  [PREBUILT] Re-downloading prebuilt binaries...");
                let _ = fs::remove_file(&cache_marker);
            } else {
                println!("  [PREBUILT] ✓ Checksum verification passed");
                println!(
                    "  [PREBUILT] ✓ Returning cached directory: {}",
                    out_dir.display()
                );
                return Ok(out_dir.clone());
            }
        }
    }

    // Get release version
    println!("  [PREBUILT] Getting release version...");
    let release_version = version::get_release_version()?;
    println!("  [PREBUILT] Release version: {}", release_version);

    // Fetch release
    println!("  [PREBUILT] Fetching release from GitHub...");
    let release = github::fetch_release(&release_version)?;
    println!("  [PREBUILT] ✓ Release fetched: {}", release.tag_name);
    println!("  [PREBUILT]   Number of assets: {}", release.assets.len());

    // Find matching asset
    println!(
        "  [PREBUILT] Looking for asset matching platform: {}",
        platform
    );
    let asset = github::find_matching_asset(&release, platform).ok_or_else(|| {
        format!(
            "No prebuilt binary found for platform: {} in release: {}. \
             Please check the GitHub releases page for available platforms.",
            platform, release.tag_name
        )
    })?;

    println!("  [PREBUILT] ✓ Found matching asset:");
    println!("  [PREBUILT]   Name: {}", asset.name);
    println!(
        "  [PREBUILT]   Download URL: {}",
        asset.browser_download_url
    );

    // Download and extract
    println!("  [PREBUILT] Starting download and extraction...");
    download::download_and_extract(&asset.browser_download_url, out_dir)?;
    println!("  [PREBUILT] ✓ Download and extraction complete");

    // Verify files exist
    println!("  [PREBUILT] Verifying extracted files...");
    if !lib_path.exists() {
        return Err(format!(
            "libstorage.a not found after extraction at {}. \
             This should not happen - please report this issue.",
            lib_path.display()
        )
        .into());
    }
    println!("  [PREBUILT] ✓ Library file exists: {}", lib_path.display());

    if !header_path.exists() {
        return Err(format!(
            "libstorage.h not found after extraction at {}. \
             This should not happen - please report this issue.",
            header_path.display()
        )
        .into());
    }
    println!(
        "  [PREBUILT] ✓ Header file exists: {}",
        header_path.display()
    );

    // Verify checksum
    if checksum_path.exists() {
        println!("  [PREBUILT] Verifying checksum...");
        checksum::verify_checksum(&lib_path, &checksum_path)?;
        println!("  [PREBUILT] ✓ Checksum verification passed");
    } else {
        println!("  [PREBUILT] ⚠ Warning: Checksum file not found, skipping verification");
    }

    // Create cache marker
    println!("  [PREBUILT] Creating cache marker...");
    fs::write(&cache_marker, "")?;
    println!("  [PREBUILT] ✓ Cache marker created");

    println!("  [PREBUILT] ✓ Successfully extracted prebuilt binaries");
    println!("  [PREBUILT] ✓ Returning directory: {}", out_dir.display());
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
