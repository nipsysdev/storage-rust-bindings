use storage_bindings::{
    download_cancel, download_chunk, download_init, upload_cancel, upload_chunk, upload_finalize,
    upload_init, LogLevel, StorageConfig, StorageNode, UploadOptions,
};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_chunk_operations() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;

    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage_data"))
        .storage_quota(100 * 1024 * 1024)
        .block_retries(3000)
        .discovery_port(8094);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    let test_data = b"Hello, Storage! This is a test file for chunk-based upload. ";
    let test_data2 = b"It contains multiple chunks that will be uploaded separately. ";
    let test_data3 = b"This demonstrates the chunk-based upload functionality.";

    println!(
        "Total data size: {} bytes",
        test_data.len() + test_data2.len() + test_data3.len()
    );

    // Chunk-based upload
    let upload_options = UploadOptions::new().filepath("test_chunks.txt");
    let session_id = upload_init(&node, &upload_options).await?;
    println!("Upload session created: {}", session_id);

    upload_chunk(&node, &session_id, test_data.to_vec()).await?;
    println!("Chunk 1 uploaded ({} bytes)", test_data.len());

    upload_chunk(&node, &session_id, test_data2.to_vec()).await?;
    println!("Chunk 2 uploaded ({} bytes)", test_data2.len());

    upload_chunk(&node, &session_id, test_data3.to_vec()).await?;
    println!("Chunk 3 uploaded ({} bytes)", test_data3.len());

    let cid = upload_finalize(&node, &session_id).await?;
    println!("Upload finalized: CID={}", cid);

    // Verify upload
    let exists = storage_bindings::exists(&node, &cid).await?;
    assert!(exists, "Content should exist after upload");

    let manifest = storage_bindings::fetch(&node, &cid).await?;
    println!(
        "Manifest: CID={}, Size={} bytes",
        manifest.cid, manifest.dataset_size
    );

    // Chunk-based download
    let download_options = storage_bindings::DownloadOptions::new(&cid);
    download_init(&node, &cid, &download_options).await?;

    let mut downloaded_data = Vec::new();
    let mut chunk_count = 0;

    loop {
        match download_chunk(&node, &cid).await {
            Ok(chunk) => {
                chunk_count += 1;
                println!("Downloaded chunk {} ({} bytes)", chunk_count, chunk.len());
                downloaded_data.extend_from_slice(&chunk);

                if chunk_count >= 10 {
                    println!("Stopping after 10 chunks to avoid infinite loop");
                    break;
                }
            }
            Err(e) => {
                println!("Download completed or error: {}", e);
                break;
            }
        }

        if downloaded_data.len() >= manifest.dataset_size {
            println!("All expected data downloaded");
            break;
        }
    }

    download_cancel(&node, &cid).await?;
    println!("Download session canceled");

    // Verify downloaded data
    let mut expected_data = Vec::new();
    expected_data.extend_from_slice(test_data);
    expected_data.extend_from_slice(test_data2);
    expected_data.extend_from_slice(test_data3);

    println!(
        "Expected: {} bytes, Downloaded: {} bytes",
        expected_data.len(),
        downloaded_data.len()
    );
    assert!(
        downloaded_data.len() >= expected_data.len(),
        "Should download at least expected data size"
    );

    let downloaded_str = String::from_utf8_lossy(&downloaded_data[..expected_data.len()]);
    let expected_str = String::from_utf8_lossy(&expected_data);
    assert_eq!(
        downloaded_str, expected_str,
        "Downloaded data should match expected data"
    );
    println!("Downloaded data matches expected data");

    // Test upload cancellation
    let cancel_options = UploadOptions::new().filepath("cancel_test.txt");
    let cancel_session_id = upload_init(&node, &cancel_options).await?;
    upload_chunk(
        &node,
        &cancel_session_id,
        b"This upload will be canceled".to_vec(),
    )
    .await?;
    upload_cancel(&node, &cancel_session_id).await?;
    println!("Upload canceled");

    let finalize_result = upload_finalize(&node, &cancel_session_id).await;
    assert!(
        finalize_result.is_err(),
        "Finalize should fail after cancellation"
    );
    println!("Finalize correctly failed after cancellation");

    // Test small chunks
    let small_options = UploadOptions::new().filepath("small_chunks.txt");
    let small_session_id = upload_init(&node, &small_options).await?;

    for (i, chunk) in [b"A", b"B", b"C", b"D", b"E"].iter().enumerate() {
        upload_chunk(&node, &small_session_id, chunk.to_vec()).await?;
        println!(
            "Uploaded small chunk {}: '{}' (1 byte)",
            i + 1,
            String::from_utf8_lossy(&chunk[..])
        );
    }

    let small_cid = upload_finalize(&node, &small_session_id).await?;
    println!("Small chunks upload finalized: {}", small_cid);

    // Final storage information
    let space_info = storage_bindings::space(&node).await?;
    println!(
        "Storage: Used={} bytes, Available={} bytes, Blocks={}",
        space_info.quota_used_bytes,
        space_info.quota_max_bytes - space_info.quota_used_bytes,
        space_info.total_blocks
    );

    let manifests = storage_bindings::manifests(&node).await?;
    println!("Total manifests: {}", manifests.len());
    assert!(manifests.len() >= 2, "Should have at least 2 manifests");

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "  Manifest {}: CID={}, Size={} bytes",
            i + 1,
            manifest.cid,
            manifest.dataset_size
        );
    }

    // Cleanup
    node.stop().await?;
    node.destroy().await?;

    Ok(())
}
