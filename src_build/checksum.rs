use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Verifies SHA256 checksum of a file
pub fn verify_checksum(
    lib_path: &PathBuf,
    checksum_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read expected checksum
    let checksum_content = fs::read_to_string(checksum_path)?;
    let expected_checksum = checksum_content
        .split_whitespace()
        .next()
        .ok_or("Invalid checksum file format")?;

    // Compute actual checksum
    let mut file = fs::File::open(lib_path)?;
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
            "Checksum verification failed!\nExpected: {}\nActual: {}",
            expected_checksum, actual_checksum
        )
        .into());
    }

    println!("✓ Checksum verification passed");
    Ok(())
}
