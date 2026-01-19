use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::urls;

/// Downloads a file to a specified path
pub fn download_file(url: &str, dest_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [DOWNLOAD] Starting download_file");
    println!("  [DOWNLOAD] URL: {}", url);
    println!("  [DOWNLOAD] Destination: {}", dest_path.display());

    println!("  [DOWNLOAD] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent(urls::USER_AGENT)
        .timeout(std::time::Duration::from_secs(
            urls::DOWNLOAD_TIMEOUT_SECONDS,
        ))
        .build()?;
    println!("  [DOWNLOAD] ✓ HTTP client created");

    println!("  [DOWNLOAD] Starting HTTP GET request...");
    let response = client.get(url).send()?;
    println!("  [DOWNLOAD] ✓ HTTP response received");
    println!("  [DOWNLOAD]   Status: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    println!("  [DOWNLOAD]   Total size: {} bytes", total_size);

    println!("  [DOWNLOAD] Creating destination file...");
    let mut dest_file = File::create(dest_path)?;
    println!("  [DOWNLOAD] ✓ Destination file created");

    println!("  [DOWNLOAD] Copying response body to file...");
    let mut reader = response;
    let mut buffer = vec![0u8; urls::DOWNLOAD_BUFFER_SIZE];
    let mut downloaded = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dest_file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            println!(
                "  [DOWNLOAD]   Progress: {:.1}% ({}/{} bytes)",
                percent, downloaded, total_size
            );
        } else {
            println!("  [DOWNLOAD]   Downloaded: {} bytes", downloaded);
        }
    }

    println!("  [DOWNLOAD] ✓ Copied {} bytes", downloaded);
    println!("  [DOWNLOAD] ✓ download_file completed successfully");
    Ok(())
}
