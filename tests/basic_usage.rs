//! Basic usage integration test for the Storage Rust bindings
//!
//! This test demonstrates how to create a Storage node, start it,
//! upload a file, download it, and then clean up.

use std::fs::File;
use std::io::Write;
use storage_bindings::{
    download_stream, upload_file, DownloadStreamOptions, LogLevel, StorageConfig, StorageNode,
    UploadOptions,
};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("example.txt");
    let download_path = temp_dir.path().join("downloaded.txt");

    // Create a test file to upload
    let mut file = File::create(&file_path)?;
    file.write_all(b"Hello, Storage! This is a test file for the Rust bindings.")?;
    file.sync_all()?;

    // Create and start a Storage node
    let config = StorageConfig::new()
        .log_level(LogLevel::Info)
        .data_dir(temp_dir.path().join("storage_data"))
        .storage_quota(100 * 1024 * 1024)
        .max_peers(50)
        .discovery_port(8090);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    // Upload the file
    let upload_options = UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = upload_file(&node, upload_options).await?;
    println!(
        "Uploaded: CID={}, Size={} bytes",
        upload_result.cid, upload_result.size
    );

    // Download the file
    let download_options = DownloadStreamOptions::new(&upload_result.cid)
        .filepath(&download_path)
        .on_progress(|progress| {
            println!(
                "Download progress: {} bytes ({}%)",
                progress.bytes_downloaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let download_result = download_stream(&node, &upload_result.cid, download_options).await?;
    println!("Downloaded: Size={} bytes", download_result.size);

    // Verify the downloaded content
    let original_content = std::fs::read_to_string(&file_path)?;
    let downloaded_content = std::fs::read_to_string(&download_path)?;
    assert_eq!(
        original_content, downloaded_content,
        "Downloaded content should match original"
    );

    // Cleanup
    node.stop().await?;
    node.destroy().await?;

    Ok(())
}
