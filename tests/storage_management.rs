//! Storage management integration test for the Storage Rust bindings
//!
//! This test demonstrates how to manage storage operations:
//! - List manifests
//! - Check storage space
//! - Fetch manifest information
//! - Delete content
//! - Check content existence

use std::fs::File;
use std::io::Write;
use storage_bindings::{LogLevel, StorageConfig, StorageNode};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_storage_management() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");

    // Create a test file to upload
    let mut file = File::create(&file_path)?;
    file.write_all(b"This is a test file for storage management example.")?;
    file.sync_all()?;

    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage_data"))
        .block_retries(3000)
        .discovery_port(8097);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    // Get initial storage information
    println!("Initial storage information:");
    let space_info = storage_bindings::space(&node).await?;
    println!("  Quota: {} bytes", space_info.quota_max_bytes);
    println!("  Used: {} bytes", space_info.quota_used_bytes);
    println!("  Reserved: {} bytes", space_info.quota_reserved_bytes);
    println!("  Total blocks: {}", space_info.total_blocks);

    // List initial manifests (should be empty)
    println!("\nInitial manifests:");
    let manifests = storage_bindings::manifests(&node).await?;
    println!("  Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 0, "Should start with no manifests");

    // Upload a file to have some content
    println!("\nUploading test file:");
    let upload_options = storage_bindings::UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "  Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = storage_bindings::upload_file(&node, upload_options).await?;
    println!("  CID: {}", upload_result.cid);
    println!("  Size: {} bytes", upload_result.size);

    // Check if content exists
    println!("\nChecking content existence:");
    let exists = storage_bindings::exists(&node, &upload_result.cid).await?;
    assert!(exists, "Uploaded content should exist");
    println!("  Content exists: {}", exists);

    // Check non-existent content
    let non_existent_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
    let non_existent = storage_bindings::exists(&node, non_existent_cid).await?;
    assert!(!non_existent, "Non-existent content should not exist");
    println!("  Non-existent content exists: {}", non_existent);

    // Fetch manifest information
    println!("\nFetching manifest information:");
    let manifest = storage_bindings::fetch(&node, &upload_result.cid).await?;
    println!("  CID: {}", manifest.cid);
    println!("  Size: {} bytes", manifest.dataset_size);
    println!("  Block size: {} bytes", manifest.block_size);
    println!("  Filename: {}", manifest.filename);
    println!("  Mimetype: {}", manifest.mimetype);
    println!("  Protected: {}", manifest.protected);

    // List manifests after upload
    println!("\nManifests after upload:");
    let manifests = storage_bindings::manifests(&node).await?;
    println!("  Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 1, "Should have 1 manifest after upload");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Get updated storage information
    println!("\nUpdated storage information:");
    let space_info = storage_bindings::space(&node).await?;
    println!("  Quota: {} bytes", space_info.quota_max_bytes);
    println!("  Used: {} bytes", space_info.quota_used_bytes);
    println!("  Reserved: {} bytes", space_info.quota_reserved_bytes);
    println!("  Total blocks: {}", space_info.total_blocks);

    // Upload another file for more content
    println!("\nUploading second test file:");
    let file_path2 = temp_dir.path().join("test_file2.txt");
    let mut file2 = File::create(&file_path2)?;
    file2.write_all(b"This is a second test file for storage management.")?;
    file2.sync_all()?;

    let upload_options2 = storage_bindings::UploadOptions::new().filepath(&file_path2);

    let upload_result2 = storage_bindings::upload_file(&node, upload_options2).await?;
    println!("  CID: {}", upload_result2.cid);
    println!("  Size: {} bytes", upload_result2.size);

    // List manifests after second upload
    println!("\nManifests after second upload:");
    let manifests = storage_bindings::manifests(&node).await?;
    println!("  Number of manifests: {}", manifests.len());
    assert_eq!(
        manifests.len(),
        2,
        "Should have 2 manifests after second upload"
    );

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Delete the first file
    println!("\nDeleting first file:");
    storage_bindings::delete(&node, &upload_result.cid).await?;
    println!("  First file deleted successfully");

    // Check if deleted content still exists
    println!("\nChecking deleted content:");
    let exists_after_delete = storage_bindings::exists(&node, &upload_result.cid).await?;
    assert!(!exists_after_delete, "Deleted content should not exist");
    println!("  Deleted content exists: {}", exists_after_delete);

    // List manifests after deletion
    println!("\nManifests after deletion:");
    let manifests = storage_bindings::manifests(&node).await?;
    println!("  Number of manifests: {}", manifests.len());
    assert_eq!(manifests.len(), 1, "Should have 1 manifest after deletion");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i, manifest.cid, manifest.dataset_size
        );
    }

    // Get final storage information
    println!("\nFinal storage information:");
    let space_info = storage_bindings::space(&node).await?;
    println!("  Quota: {} bytes", space_info.quota_max_bytes);
    println!("  Used: {} bytes", space_info.quota_used_bytes);
    println!("  Reserved: {} bytes", space_info.quota_reserved_bytes);
    println!("  Total blocks: {}", space_info.total_blocks);

    // Cleanup
    node.stop().await?;
    node.destroy().await?;

    Ok(())
}
