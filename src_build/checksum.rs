use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Verifies SHA256 checksum of a file
pub fn verify_checksum(
    lib_path: &PathBuf,
    checksum_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [CHECKSUM] Starting verify_checksum");
    println!("  [CHECKSUM] Library path: {}", lib_path.display());
    println!("  [CHECKSUM] Checksum path: {}", checksum_path.display());

    // Read expected checksum
    println!("  [CHECKSUM] Reading expected checksum from file...");
    let checksum_content = fs::read_to_string(checksum_path)?;
    let expected_checksum = checksum_content
        .split_whitespace()
        .next()
        .ok_or("Invalid checksum file format")?;
    println!("  [CHECKSUM] ✓ Expected checksum: {}", expected_checksum);

    // Compute actual checksum
    println!("  [CHECKSUM] Computing actual checksum...");
    let mut file = fs::File::open(lib_path)?;
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

    let actual_checksum = hex::encode(hasher.finalize());
    println!("  [CHECKSUM] ✓ Actual checksum: {}", actual_checksum);
    println!("  [CHECKSUM] ✓ Total bytes processed: {}", total_bytes);

    if expected_checksum != actual_checksum {
        println!("  [CHECKSUM] ✗ Checksum verification failed!");
        println!("  [CHECKSUM]   Expected: {}", expected_checksum);
        println!("  [CHECKSUM]   Actual:   {}", actual_checksum);
        return Err(format!(
            "Checksum verification failed!\nExpected: {}\nActual: {}",
            expected_checksum, actual_checksum
        )
        .into());
    }

    println!("  [CHECKSUM] ✓ Checksum verification passed");
    println!("  [CHECKSUM] ✓ verify_checksum completed successfully");
    Ok(())
}
