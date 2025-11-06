//! End-to-end test for the Codex Rust bindings
//!
//! This example demonstrates a complete upload/download workflow.

use codex_rust_bindings::{
    download_stream, upload_file, CodexConfig, CodexNode, DownloadStreamOptions, UploadOptions,
};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Codex Rust Bindings - End-to-End Test");
    println!("=================================");

    // Create a temporary directory for our example
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");
    let download_path = temp_dir.path().join("downloaded_file.txt");

    // Create a test file to upload
    println!("Creating test file...");
    let mut file = File::create(&file_path)?;
    file.write_all(b"Hello, Codex! This is a test file for the Rust bindings.")?;
    file.sync_all()?;
    println!("Test file created at: {}", file_path.display());

    // Create a minimal configuration - use the same approach as simple_config_test
    println!("Creating Codex configuration...");
    let config = CodexConfig {
        log_level: Some(codex_rust_bindings::LogLevel::Error),
        data_dir: Some(temp_dir.path().join("codex_data")),
        log_format: None,
        metrics_enabled: None,
        metrics_address: None,
        metrics_port: None,
        listen_addrs: vec![],
        nat: None,
        discovery_port: None,
        net_priv_key_file: None,
        bootstrap_nodes: vec![],
        max_peers: None,
        num_threads: None,
        agent_string: None,
        repo_kind: None,
        storage_quota: None,
        block_ttl: None,
        block_maintenance_interval: None,
        block_maintenance_number_of_blocks: None,
        block_retries: None,
        cache_size: None,
        log_file: None,
    };

    // Print the JSON configuration to debug
    let config_json = config.to_json()?;
    println!("Generated JSON: {}", config_json);
    println!("JSON length: {}", config_json.len());

    // Create a new Codex node with the minimal configuration
    println!("Creating Codex node...");
    let start_time = std::time::Instant::now();
    let mut node = CodexNode::new(config)?;
    println!("✓ Node created in {} ms", start_time.elapsed().as_millis());

    // Start the node
    println!("Starting Codex node...");
    let start_time = std::time::Instant::now();
    node.start()?;
    println!("✓ Node started in {} ms", start_time.elapsed().as_millis());

    // Get node information
    println!("Getting node information...");
    let start_time = std::time::Instant::now();
    let version = node.version()?;
    println!(
        "  Version: {} ({} ms)",
        version,
        start_time.elapsed().as_millis()
    );

    let start_time = std::time::Instant::now();
    let peer_id = node.peer_id()?;
    println!(
        "  Peer ID: {} ({} ms)",
        peer_id,
        start_time.elapsed().as_millis()
    );

    let start_time = std::time::Instant::now();
    let repo = node.repo()?;
    println!(
        "  Repository: {} ({} ms)",
        repo,
        start_time.elapsed().as_millis()
    );

    // Upload the file
    println!("Uploading file...");
    let start_time = std::time::Instant::now();
    let upload_options = UploadOptions::new()
        .filepath(&file_path)
        .on_progress(|progress| {
            println!(
                "  Upload progress: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = upload_file(&node, upload_options).await?;
    println!("✓ File uploaded in {} ms", start_time.elapsed().as_millis());
    println!("  CID: {}", upload_result.cid);
    println!("  Size: {} bytes", upload_result.size);
    println!("  Chunks: {:?}", upload_result.chunks);
    println!("  Duration: {} ms", upload_result.duration_ms);

    // Download the file
    println!("Downloading file...");
    let start_time = std::time::Instant::now();
    let download_options = DownloadStreamOptions::new(&upload_result.cid)
        .filepath(&download_path)
        .on_progress(|progress| {
            println!(
                "  Download progress: {} bytes ({}%)",
                progress.bytes_downloaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let download_result = download_stream(&node, &upload_result.cid, download_options).await?;
    println!(
        "✓ File downloaded in {} ms",
        start_time.elapsed().as_millis()
    );
    println!("  Size: {} bytes", download_result.size);
    println!("  Duration: {} ms", download_result.duration_ms);
    println!("  Saved to: {:?}", download_result.filepath);

    // Verify the downloaded content
    println!("Verifying downloaded content...");
    let start_time = std::time::Instant::now();
    let original_content = std::fs::read_to_string(&file_path)?;
    let downloaded_content = std::fs::read_to_string(&download_path)?;

    if original_content == downloaded_content {
        println!(
            "✓ Content verification successful! ({} ms)",
            start_time.elapsed().as_millis()
        );
    } else {
        println!("✗ Content verification failed!");
    }

    // Stop and destroy the node
    println!("Stopping Codex node...");
    let start_time = std::time::Instant::now();
    node.stop()?;
    println!("✓ Node stopped in {} ms", start_time.elapsed().as_millis());

    println!("Destroying Codex node...");
    let start_time = std::time::Instant::now();
    node.destroy()?;
    println!(
        "✓ Node destroyed in {} ms",
        start_time.elapsed().as_millis()
    );

    println!("End-to-end test completed successfully!");
    Ok(())
}
