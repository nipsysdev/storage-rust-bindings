use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

/// Downloads a file to a specified path
pub fn download_file(url: &str, dest_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [DOWNLOAD] Starting download_file");
    println!("  [DOWNLOAD] URL: {}", url);
    println!("  [DOWNLOAD] Destination: {}", dest_path.display());

    println!("  [DOWNLOAD] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("storage-rust-bindings")
        .timeout(std::time::Duration::from_secs(900)) // 15 minutes timeout for download
        .build()?;
    println!("  [DOWNLOAD] ✓ HTTP client created");

    println!("  [DOWNLOAD] Starting HTTP GET request...");
    let mut response = client.get(url).send()?;
    println!("  [DOWNLOAD] ✓ HTTP response received");
    println!("  [DOWNLOAD]   Status: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }

    println!("  [DOWNLOAD] Creating destination file...");
    let mut dest_file = File::create(dest_path)?;
    println!("  [DOWNLOAD] ✓ Destination file created");

    println!("  [DOWNLOAD] Copying response body to file...");
    let bytes_copied = copy(&mut response, &mut dest_file)?;
    println!("  [DOWNLOAD] ✓ Copied {} bytes", bytes_copied);

    println!("  [DOWNLOAD] ✓ download_file completed successfully");
    Ok(())
}
