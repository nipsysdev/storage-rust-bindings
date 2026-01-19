use std::fs;
use std::path::PathBuf;

use super::checksum;
use super::download;
use super::github;
use super::prebuilt;
use super::version;

/// Maps Rust target triple to platform identifier for prebuilt binaries
pub fn map_target_to_platform(target: &str) -> Option<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" => Some("linux-amd64"),
        "aarch64-unknown-linux-gnu" => Some("linux-arm64"),
        "aarch64-apple-darwin" => Some("darwin-arm64"),
        "x86_64-apple-darwin" => Some("darwin-amd64"),
        _ => None,
    }
}

/// Downloads prebuilt binaries from GitHub
pub fn download_from_github(
    out_dir: &PathBuf,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let platform = map_target_to_platform(target).ok_or_else(|| {
        format!(
            "Unsupported target: {}. Supported platforms: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-apple-darwin.",
            target
        )
    })?;

    prebuilt::log_info(&format!("Mapped target to platform: {}", platform));

    if let Ok(result) = try_use_cached_files(out_dir) {
        return Ok(result);
    }

    download_and_extract_binaries(out_dir, platform)?;
    prebuilt::validate_required_files(out_dir)?;
    prebuilt::create_cache_marker(out_dir, "")?;

    prebuilt::log_info("✓ Successfully extracted prebuilt binaries");
    prebuilt::log_info(&format!("✓ Returning directory: {}", out_dir.display()));
    Ok(out_dir.clone())
}

/// Attempts to use cached files if they exist and are valid
fn try_use_cached_files(out_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_marker = out_dir.join(prebuilt::CACHE_MARKER);

    prebuilt::log_info("Checking cache:");
    prebuilt::log_info(&format!(
        "  Cache marker: {} (exists: {})",
        cache_marker.display(),
        cache_marker.exists()
    ));

    if !cache_marker.exists() {
        return Err("Cache marker not found".into());
    }

    // Validate that required files exist (at least one .a file and libstorage.h)
    prebuilt::validate_required_files(out_dir)?;

    prebuilt::log_info("✓ Using cached prebuilt binaries");

    // Use comprehensive checksum verification
    prebuilt::log_info("Verifying checksums of cached files...");
    if let Err(e) = checksum::verify_all_checksums(out_dir) {
        prebuilt::log_info(&format!(
            "⚠ Checksum verification failed for cached files: {}",
            e
        ));
        prebuilt::log_info("Re-downloading prebuilt binaries...");
        let _ = fs::remove_file(&cache_marker);
        return Err("Checksum verification failed".into());
    }
    prebuilt::log_info("✓ Checksum verification passed");

    prebuilt::log_info(&format!(
        "✓ Returning cached directory: {}",
        out_dir.display()
    ));
    Ok(out_dir.clone())
}

/// Downloads and extracts binaries from GitHub
fn download_and_extract_binaries(
    out_dir: &PathBuf,
    platform: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    prebuilt::log_info("Getting release version...");
    let release_version = version::get_release_version()?;
    prebuilt::log_info(&format!("Release version: {}", release_version));

    prebuilt::log_info("Fetching release from GitHub...");
    let release = github::fetch_release(&release_version)?;
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
    let checksums_content = github::fetch_checksums_file(&release_version)?;

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

/// Extracts a tar.gz archive to a directory
fn extract_archive(
    archive_path: &PathBuf,
    dest_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    prebuilt::log_info(&format!("Extracting archive: {}", archive_path.display()));
    prebuilt::log_info(&format!("Destination: {}", dest_dir.display()));

    let file = fs::File::open(archive_path)?;
    let gz_decoder = flate2::read::GzDecoder::new(file);
    let mut tar_archive = tar::Archive::new(gz_decoder);

    tar_archive.unpack(dest_dir)?;

    prebuilt::log_info("✓ Archive extraction completed");
    Ok(())
}
