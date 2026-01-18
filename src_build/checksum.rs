use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Calculates SHA256 checksum of a file
pub fn calculate_sha256(file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    println!(
        "  [CHECKSUM] Calculating SHA256 for: {}",
        file_path.display()
    );

    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_bytes += n as u64;
    }

    let checksum = hex::encode(hasher.finalize());
    println!("  [CHECKSUM] ✓ SHA256 calculated: {}", checksum);
    println!("  [CHECKSUM] ✓ Total bytes processed: {}", total_bytes);

    Ok(checksum)
}

/// Verifies the checksum of an archive file against an expected checksum
pub fn verify_archive_checksum(
    archive_path: &PathBuf,
    expected_checksum: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [CHECKSUM] Verifying archive checksum...");
    println!("  [CHECKSUM] Archive path: {}", archive_path.display());
    println!("  [CHECKSUM] Expected checksum: {}", expected_checksum);

    let actual_checksum = calculate_sha256(archive_path)?;

    if actual_checksum != expected_checksum {
        println!("  [CHECKSUM] ✗ Archive checksum mismatch!");
        println!("  [CHECKSUM]   Expected: {}", expected_checksum);
        println!("  [CHECKSUM]   Actual:   {}", actual_checksum);
        return Err(format!(
            "Archive checksum mismatch:\n  Expected: {}\n  Actual: {}",
            expected_checksum, actual_checksum
        )
        .into());
    }

    println!("  [CHECKSUM] ✓ Archive checksum verified");
    Ok(())
}

/// Parses a SHA256SUMS.txt file and returns a HashMap of filename -> checksum
pub fn parse_checksums_file(
    path: &PathBuf,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    println!("  [CHECKSUM] Parsing checksums file: {}", path.display());

    let content = fs::read_to_string(path)?;
    let mut checksums = HashMap::new();
    let mut line_count = 0;

    for line in content.lines() {
        line_count += 1;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Format: <checksum> <filename>
            checksums.insert(parts[1].to_string(), parts[0].to_string());
        } else {
            println!(
                "  [CHECKSUM] ⚠ Skipping invalid line {}: {}",
                line_count, line
            );
        }
    }

    println!("  [CHECKSUM] ✓ Parsed {} checksum entries", checksums.len());
    Ok(checksums)
}

/// Verifies checksums for all files in a directory against SHA256SUMS.txt
pub fn verify_all_checksums(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [CHECKSUM] Starting comprehensive checksum verification");
    println!("  [CHECKSUM] Directory: {}", dir.display());

    let checksums_file = dir.join("SHA256SUMS.txt");

    if !checksums_file.exists() {
        println!("  [CHECKSUM] ⚠ SHA256SUMS.txt not found, skipping checksum verification");
        return Ok(());
    }

    // Parse checksums file
    let checksums = parse_checksums_file(&checksums_file)?;

    if checksums.is_empty() {
        println!("  [CHECKSUM] ⚠ No checksums found in SHA256SUMS.txt");
        return Ok(());
    }

    // Verify each file
    let mut verified_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_name() != Some(OsStr::new("SHA256SUMS.txt")) {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>");

            if let Some(expected_checksum) = checksums.get(file_name) {
                match verify_single_checksum(&path, expected_checksum) {
                    Ok(_) => {
                        println!("  [CHECKSUM] ✓ Checksum verified: {}", file_name);
                        verified_count += 1;
                    }
                    Err(e) => {
                        println!("  [CHECKSUM] ✗ Checksum failed: {} - {}", file_name, e);
                        failed_count += 1;
                    }
                }
            } else {
                // File exists but no checksum in SHA256SUMS.txt
                println!("  [CHECKSUM] ⚠ No checksum found for: {}", file_name);
                skipped_count += 1;
            }
        }
    }

    println!("  [CHECKSUM] Checksum verification summary:");
    println!("  [CHECKSUM]   Verified: {}", verified_count);
    println!("  [CHECKSUM]   Failed: {}", failed_count);
    println!("  [CHECKSUM]   Skipped (no checksum): {}", skipped_count);

    if failed_count > 0 {
        return Err(format!("{} files failed checksum verification", failed_count).into());
    }

    println!("  [CHECKSUM] ✓ All checksums verified successfully");
    Ok(())
}

/// Verifies a single file's checksum against an expected value
fn verify_single_checksum(
    file_path: &PathBuf,
    expected_checksum: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let actual_checksum = hex::encode(hasher.finalize());

    if expected_checksum != actual_checksum {
        return Err(format!(
            "Checksum mismatch for {}: expected {}, got {}",
            file_path.display(),
            expected_checksum,
            actual_checksum
        )
        .into());
    }

    Ok(())
}
