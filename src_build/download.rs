use std::path::PathBuf;

/// Downloads and extracts a tar.gz archive
pub fn download_and_extract(
    url: &str,
    dest_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [DOWNLOAD] Starting download_and_extract");
    println!("  [DOWNLOAD] URL: {}", url);
    println!("  [DOWNLOAD] Destination: {}", dest_dir.display());

    println!("  [DOWNLOAD] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("storage-rust-bindings")
        .timeout(std::time::Duration::from_secs(900)) // 15 minutes timeout for download
        .build()?;
    println!("  [DOWNLOAD] ✓ HTTP client created");

    println!("  [DOWNLOAD] Starting HTTP GET request...");
    let response = client.get(url).send()?;
    println!("  [DOWNLOAD] ✓ HTTP response received");
    println!("  [DOWNLOAD]   Status: {}", response.status());
    println!("  [DOWNLOAD]   Headers: {:?}", response.headers());

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }

    let reader = response;

    // Extract tar.gz
    println!("  [DOWNLOAD] Creating GzDecoder...");
    let gz_decoder = flate2::read::GzDecoder::new(reader);
    println!("  [DOWNLOAD] ✓ GzDecoder created");

    println!("  [DOWNLOAD] Creating tar archive...");
    let mut tar_archive = tar::Archive::new(gz_decoder);
    println!("  [DOWNLOAD] ✓ Tar archive created");

    println!(
        "  [DOWNLOAD] Starting extraction to: {}",
        dest_dir.display()
    );
    let result = tar_archive.unpack(dest_dir);

    match &result {
        Ok(_) => {
            println!("  [DOWNLOAD] ✓ Extraction completed successfully");
        }
        Err(e) => {
            println!("  [DOWNLOAD] ✗ Extraction failed: {}", e);
        }
    }

    result?;

    println!("  [DOWNLOAD] ✓ download_and_extract completed successfully");
    Ok(())
}
