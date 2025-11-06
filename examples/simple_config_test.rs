//! Simple config test for the Codex Rust bindings
//!
//! This example uses a very simple configuration.

use codex_rust_bindings::{CodexConfig, CodexNode, LogLevel};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Codex Rust Bindings - Simple Config Test");
    println!("=========================================");

    // Try with an empty config first
    println!("Testing with empty JSON...");
    let empty_json = "{}";
    println!("Empty JSON: {}", empty_json);

    // Create a very simple configuration manually
    println!("Testing with minimal manual JSON...");
    let minimal_json = json!({
        "log-level": "error",
        "data-dir": "/tmp/simple_test_codex"
    });
    println!("Minimal JSON: {}", minimal_json);

    // Try creating a config with only the essential fields
    let config = CodexConfig {
        log_level: Some(LogLevel::Error),
        data_dir: Some("/tmp/simple_test_codex".into()),
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

    println!("✓ Manual configuration created successfully");

    // Print the JSON configuration to debug
    let json = config.to_json()?;
    println!("Generated JSON: {}", json);
    println!("JSON length: {}", json.len());

    // Now try to create a node with this minimal configuration
    println!("Testing node creation with minimal config...");
    match CodexNode::new(config) {
        Ok(_node) => {
            println!("✓ Node created successfully with minimal config!");
        }
        Err(e) => {
            println!("✗ Failed to create node: {}", e);
        }
    }

    println!("Simple config test completed!");
    Ok(())
}
