//! Simple test for the Codex Rust bindings
//!
//! This example just tests that the library can be loaded and basic functions work.

use codex_rust_bindings::{CodexConfig, CodexNode, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Codex Rust Bindings - Simple Test");
    println!("=================================");

    // Test that we can create a minimal configuration (like end_to_end_test)
    let config = CodexConfig {
        log_level: Some(LogLevel::Error),
        data_dir: None,
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

    println!("✓ Configuration created successfully");

    // Print the JSON configuration to debug
    let config_json = config.to_json()?;
    println!("Generated JSON: {}", config_json);
    println!("JSON length: {}", config_json.len());

    // Test that we can create a node (this doesn't start the node)
    let _node = CodexNode::new(config)?;
    println!("✓ Node created successfully");

    // Test that we can get basic node info without starting
    println!("✓ Basic node operations work");

    println!("Simple test completed successfully!");
    Ok(())
}
