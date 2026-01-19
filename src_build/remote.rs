use std::fs;
use std::path::{Path, PathBuf};

use super::cache;
use super::checksum;
use super::download;
use super::github;
use super::prebuilt;
use super::targets;
use super::version;

/// Downloads prebuilt binaries from GitHub
pub fn download_from_github(
    out_dir: &Path,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let platform = targets::map_target_to_platform(target).ok_or_else(|| {
        let supported = targets::supported_targets().join(", ");
        format!(
            "Unsupported target: {}. Supported platforms: {}.",
            target, supported
        )
    })?;

    prebuilt::log_info(&format!("Mapped target to platform: {}", platform));

    // Get the version we're using
    let release_version = version::get_release_version()?;

    // Check if force download is requested
    if cache::should_force_download() {
        prebuilt::log_info("Force download requested, skipping cache");
        download_and_extract_binaries(out_dir, platform, &release_version)?;
        prebuilt::validate_required_files(out_dir)?;
        return Ok(out_dir.to_path_buf());
    }

    // Try to use cached files
    if let Ok(result) = try_use_cached_files(out_dir, &release_version, platform) {
        return Ok(result);
    }

    // Download and cache
    download_and_extract_binaries(out_dir, platform, &release_version)?;
    prebuilt::validate_required_files(out_dir)?;

    // Save to global cache
    save_to_cache(out_dir, &release_version, platform)?;

    prebuilt::log_info("✓ Successfully extracted and cached prebuilt binaries");
    prebuilt::log_info(&format!("✓ Returning directory: {}", out_dir.display()));
    Ok(out_dir.to_path_buf())
}

/// Attempts to use cached files if they exist and are valid
fn try_use_cached_files(
    out_dir: &Path,
    version: &str,
    platform: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = cache::get_version_cache_dir(version, platform)?;

    prebuilt::log_info("Checking cache:");
    prebuilt::log_info(&format!(
        "  Cache directory: {} (exists: {})",
        cache_dir.display(),
        cache_dir.exists()
    ));

    if !cache_dir.exists() {
        return Err("Cache directory not found".into());
    }

    // Validate cache contents
    cache::validate_cache(&cache_dir)?;

    prebuilt::log_info("✓ Using cached prebuilt binaries");

    // Verify checksums of cached files
    prebuilt::log_info("Verifying checksums of cached files...");
    if let Err(e) = checksum::verify_all_checksums(&cache_dir) {
        prebuilt::log_info(&format!(
            "⚠ Checksum verification failed for cached files: {}",
            e
        ));
        prebuilt::log_info("Re-downloading prebuilt binaries...");
        // Remove invalid cache
        let _ = fs::remove_dir_all(&cache_dir);
        return Err("Checksum verification failed".into());
    }
    prebuilt::log_info("✓ Checksum verification passed");

    // Copy from cache to OUT_DIR
    cache::copy_from_cache(&cache_dir, out_dir)?;

    prebuilt::log_info(&format!(
        "✓ Returning cached directory: {}",
        out_dir.display()
    ));
    Ok(out_dir.to_path_buf())
}

/// Downloads and extracts binaries from GitHub
fn download_and_extract_binaries(
    out_dir: &Path,
    platform: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    prebuilt::log_info(&format!("Getting release version: {}", version));

    prebuilt::log_info("Fetching release from GitHub...");
    let release = github::fetch_release(version)?;
    prebuilt::log_info(&format!("✓ Release fetched: {}", release.tag_name));
    prebuilt::log_info(&format!("  Number of assets: {}", release.assets.len()));

    prebuilt::log_info(&format!(
        "Looking for asset matching platform: {}",
        platform
    ));
    let asset = github::find_matching_asset(&release, platform).ok_or_else(|| {
        format!(
            "No prebuilt binary found for platform: {} in release: {}. \
             Please check the GitHub releases page for available platforms.",
            platform, release.tag_name
        )
    })?;

    prebuilt::log_info("✓ Found matching asset:");
    prebuilt::log_info(&format!("  Name: {}", asset.name));
    prebuilt::log_info(&format!("  Download URL: {}", asset.browser_download_url));

    // Download to temporary location
    prebuilt::log_info("Downloading archive to temporary location...");
    let temp_archive = out_dir.join(format!("{}.tar.gz", platform));
    download::download_file(&asset.browser_download_url, &temp_archive)?;
    prebuilt::log_info("✓ Archive downloaded to temporary location");

    // Fetch SHA256SUMS.txt to get expected checksum for the archive
    prebuilt::log_info("Fetching SHA256SUMS.txt from GitHub...");
    let checksums_content = github::fetch_checksums_file(version)?;

    // Parse checksums to find the expected checksum for this archive
    prebuilt::log_info(&format!("Looking for checksum for: {}", asset.name));
    let expected_checksum = checksums_content
        .lines()
        .find_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == asset.name {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Checksum not found for {} in SHA256SUMS.txt", asset.name))?;

    prebuilt::log_info(&format!("✓ Found expected checksum: {}", expected_checksum));

    // Verify archive before extraction
    prebuilt::log_info("Verifying archive checksum before extraction...");
    checksum::verify_archive_checksum(&temp_archive, &expected_checksum)?;
    prebuilt::log_info("✓ Archive checksum verified");

    // Extract the archive
    prebuilt::log_info("Extracting archive...");
    extract_archive(&temp_archive, out_dir)?;
    prebuilt::log_info("✓ Archive extracted");

    // Clean up temporary archive
    prebuilt::log_info("Cleaning up temporary archive...");
    fs::remove_file(&temp_archive)?;
    prebuilt::log_info("✓ Temporary archive removed");

    // Verify all extracted files
    prebuilt::log_info("Verifying checksums of extracted files...");
    checksum::verify_all_checksums(out_dir)?;
    prebuilt::log_info("✓ All checksums verified");

    Ok(())
}

/// Saves downloaded files to global cache
fn save_to_cache(
    out_dir: &Path,
    version: &str,
    platform: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = cache::get_version_cache_dir(version, platform)?;

    prebuilt::log_info(&format!("Saving to cache: {}", cache_dir.display()));

    // Create cache directory if it doesn't exist
    fs::create_dir_all(&cache_dir)?;

    // Copy all files from OUT_DIR to cache
    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("Invalid filename")?;

            let dest = cache_dir.join(file_name);
            fs::copy(&path, &dest)?;
            prebuilt::log_info(&format!("  Cached: {}", file_name));
        }
    }

    prebuilt::log_info("✓ Files saved to cache");
    Ok(())
}

/// Extracts a tar.gz archive to a directory
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    prebuilt::log_info(&format!("Extracting archive: {}", archive_path.display()));
    prebuilt::log_info(&format!("Destination: {}", dest_dir.display()));

    let file = fs::File::open(archive_path)?;
    let gz_decoder = flate2::read::GzDecoder::new(file);
    let mut tar_archive = tar::Archive::new(gz_decoder);

    tar_archive.unpack(dest_dir)?;

    prebuilt::log_info("✓ Archive extraction completed");
    Ok(())
}
