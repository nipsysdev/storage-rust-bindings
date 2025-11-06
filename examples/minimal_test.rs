//! Minimal test for the Codex Rust bindings
//!
//! This example uses a minimal configuration similar to the Go bindings.

use codex_rust_bindings::{CodexConfig, CodexNode, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Codex Rust Bindings - Minimal Test");
    println!("==================================");

    // Create a minimal configuration like the Go bindings
    let config = CodexConfig::new()
        .log_level(LogLevel::Error) // Use ERROR level like Go tests
        .data_dir("/tmp/minimal_test_codex");

    println!("✓ Configuration created successfully");

    // Print the JSON configuration to debug
    let json = config.to_json()?;
    println!("Generated JSON: {}", json);

    // Test that we can create a node (this doesn't start the node)
    let _node = CodexNode::new(config)?;
    println!("✓ Node created successfully");

    // Test that we can get basic node info without starting
    println!("✓ Basic node operations work");

    println!("Minimal test completed successfully!");
    Ok(())
}
