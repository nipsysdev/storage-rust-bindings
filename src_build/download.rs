use std::path::PathBuf;

/// Downloads and extracts a tar.gz archive
pub fn download_and_extract(
    url: &str,
    dest_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading from: {}", url);

    let client = reqwest::blocking::Client::builder()
        .user_agent("codex-rust-bindings")
        .timeout(std::time::Duration::from_secs(900)) // 15 minutes timeout for download
        .build()?;

    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }

    let reader = response;

    // Extract tar.gz
    let gz_decoder = flate2::read::GzDecoder::new(reader);
    let mut tar_archive = tar::Archive::new(gz_decoder);

    tar_archive.unpack(dest_dir)?;

    Ok(())
}
