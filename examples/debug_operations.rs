//! Debug operations example for the Codex Rust bindings
//!
//! This example demonstrates how to use debug operations:
//! - Get node debug information
//! - Update log levels
//! - Get peer debug information

use codex_rust_bindings::debug::LogLevel;
use codex_rust_bindings::{CodexConfig, CodexNode};
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Codex Rust Bindings - Debug Operations Example");
    println!("=============================================");

    // Create a temporary directory for our example
    let temp_dir = tempdir()?;

    // Create a Codex configuration
    println!("Creating Codex configuration...");
    let config = CodexConfig::new()
        .log_level(codex_rust_bindings::LogLevel::Info)
        .data_dir(temp_dir.path().join("codex_data"))
        .storage_quota(100 * 1024 * 1024); // 100 MB

    // Create and start a Codex node
    println!("Creating and starting Codex node...");
    let mut node = CodexNode::new(config)?;
    node.start()?;
    println!("Node started successfully!");

    // Get initial debug information
    println!("\n=== Initial Debug Information ===");
    let debug_info = codex_rust_bindings::debug(&node).await?;
    println!("Node version: {}", debug_info.version);
    println!("Node revision: {}", debug_info.revision);
    println!("Peer ID: {}", debug_info.peer_id);
    println!("Repository path: {}", debug_info.repo);
    println!("SPR: {}", debug_info.spr);
    println!("Current log level: {}", debug_info.log_level);
    println!("Connected peers: {}", debug_info.connected_peers);
    println!("Uptime: {} seconds", debug_info.uptime_seconds);
    println!("Memory usage: {} bytes", debug_info.memory_usage_bytes);

    if let Some(extra) = &debug_info.extra {
        println!("Extra debug info: {}", extra);
    }

    // Test updating log levels
    println!("\n=== Testing Log Level Updates ===");
    let log_levels = vec![
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Notice,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Fatal,
    ];

    for log_level in log_levels {
        println!("Setting log level to: {:?}", log_level);
        let update_result = codex_rust_bindings::update_log_level(&node, log_level).await;
        match update_result {
            Ok(()) => {
                println!("  ✓ Successfully updated log level to {:?}", log_level);

                // Verify the change by getting debug info again
                let debug_info = codex_rust_bindings::debug(&node).await?;
                println!(
                    "  Current log level in debug info: {}",
                    debug_info.log_level
                );
            }
            Err(e) => {
                println!("  ✗ Failed to update log level to {:?}: {}", log_level, e);
            }
        }
    }

    // Reset to Info level for further testing
    codex_rust_bindings::update_log_level(&node, LogLevel::Info).await?;

    // Test peer debug information
    println!("\n=== Testing Peer Debug Information ===");
    let test_peer_ids = vec![
        "12D3KooWExamplePeer123456789",
        "QmSomePeerId123456789",
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        "zdj7WWeQ43G6JJvLWQWZpyHuAMq6uYWRjkBXZadLDEotRHi7T7ycf",
    ];

    for peer_id in test_peer_ids {
        println!("Getting debug info for peer: {}", peer_id);
        let peer_record = codex_rust_bindings::peer_debug(&node, peer_id);
        match peer_record {
            Ok(record) => {
                println!("  ✓ Successfully retrieved peer debug info:");
                println!("    Peer ID: {}", record.id);
                println!("    Connected: {}", record.connected);
                println!("    Addresses: {:?}", record.addresses);
                println!("    Protocols: {:?}", record.protocols);
                if let Some(direction) = &record.direction {
                    println!("    Direction: {}", direction);
                }
                if let Some(latency) = record.latency_ms {
                    println!("    Latency: {} ms", latency);
                }
                if let Some(user_agent) = &record.user_agent {
                    println!("    User Agent: {}", user_agent);
                }
                if let Some(last_seen) = &record.last_seen {
                    println!("    Last Seen: {}", last_seen);
                }
                if let Some(duration) = record.connection_duration_seconds {
                    println!("    Connection Duration: {} seconds", duration);
                }
                if let Some(bytes_sent) = record.bytes_sent {
                    println!("    Bytes Sent: {}", bytes_sent);
                }
                if let Some(bytes_received) = record.bytes_received {
                    println!("    Bytes Received: {}", bytes_received);
                }
                if let Some(metadata) = &record.metadata {
                    println!("    Metadata: {}", metadata);
                }
            }
            Err(e) => {
                println!("  ✗ Failed to get peer debug info: {}", e);
            }
        }
    }

    // Test invalid peer ID
    println!("\n=== Testing Invalid Peer ID ===");
    let empty_peer_result = codex_rust_bindings::peer_debug(&node, "");
    match empty_peer_result {
        Ok(_) => println!("  ✗ Unexpectedly succeeded with empty peer ID"),
        Err(e) => println!("  ✓ Correctly failed with empty peer ID: {}", e),
    }

    // Test whitespace-only peer ID
    let whitespace_peer_result = codex_rust_bindings::peer_debug(&node, "   \t\n   ");
    match whitespace_peer_result {
        Ok(_) => println!("  ✗ Unexpectedly succeeded with whitespace-only peer ID"),
        Err(e) => println!("  ✓ Correctly failed with whitespace-only peer ID: {}", e),
    }

    // Test debug operations without starting node
    println!("\n=== Testing Debug Operations Without Starting Node ===");
    let config2 = CodexConfig::new()
        .log_level(codex_rust_bindings::LogLevel::Error)
        .data_dir(temp_dir.path().join("codex_data2"));

    let node2 = CodexNode::new(config2)?;
    // Don't start the node

    // These should work even if the node is not started
    let debug_info_result = codex_rust_bindings::debug(&node2).await;
    match debug_info_result {
        Ok(info) => {
            println!("  ✓ Debug info works without starting node:");
            println!("    Version: {}", info.version);
            println!("    Peer ID: {}", info.peer_id);
        }
        Err(e) => println!("  ✗ Debug info failed without starting node: {}", e),
    }

    let update_result = codex_rust_bindings::update_log_level(&node2, LogLevel::Debug).await;
    match update_result {
        Ok(()) => println!("  ✓ Log level update works without starting node"),
        Err(e) => println!("  ✗ Log level update failed without starting node: {}", e),
    }

    let peer_debug_result = codex_rust_bindings::peer_debug(&node2, "12D3KooWTestPeer");
    match peer_debug_result {
        Ok(_) => println!("  ✓ Peer debug works without starting node"),
        Err(e) => println!("  ✗ Peer debug failed without starting node: {}", e),
    }

    // Clean up the second node
    node2.destroy()?;

    // Test concurrent debug operations
    println!("\n=== Testing Concurrent Debug Operations ===");
    let debug_future1 = codex_rust_bindings::debug(&node);
    let debug_future2 = codex_rust_bindings::debug(&node);
    let peer_debug_future1 = async { codex_rust_bindings::peer_debug(&node, "12D3KooWTestPeer1") };
    let peer_debug_future2 = async { codex_rust_bindings::peer_debug(&node, "12D3KooWTestPeer2") };

    let (debug_result1, debug_result2, peer_debug_result1, peer_debug_result2) = tokio::join!(
        debug_future1,
        debug_future2,
        peer_debug_future1,
        peer_debug_future2
    );

    println!("Concurrent operations results:");
    match debug_result1 {
        Ok(_) => println!("  ✓ debug (1): succeeded"),
        Err(e) => println!("  ✗ debug (1) failed: {}", e),
    }

    match debug_result2 {
        Ok(_) => println!("  ✓ debug (2): succeeded"),
        Err(e) => println!("  ✗ debug (2) failed: {}", e),
    }

    match peer_debug_result1 {
        Ok(_) => println!("  ✓ peer_debug (1): succeeded"),
        Err(_) => println!("  ✗ peer_debug (1): failed (expected for test peer)"),
    }

    match peer_debug_result2 {
        Ok(_) => println!("  ✓ peer_debug (2): succeeded"),
        Err(_) => println!("  ✗ peer_debug (2): failed (expected for test peer)"),
    }

    // Get final debug information
    println!("\n=== Final Debug Information ===");
    let final_debug_info = codex_rust_bindings::debug(&node).await?;
    println!("Final node state:");
    println!("  Version: {}", final_debug_info.version);
    println!("  Peer ID: {}", final_debug_info.peer_id);
    println!("  Connected peers: {}", final_debug_info.connected_peers);
    println!("  Uptime: {} seconds", final_debug_info.uptime_seconds);
    println!(
        "  Memory usage: {} bytes",
        final_debug_info.memory_usage_bytes
    );

    // Stop and destroy the node
    println!("\n=== Cleanup ===");
    node.stop()?;
    node.destroy()?;
    println!("Node stopped and destroyed.");

    println!("\nDebug operations example completed successfully!");
    Ok(())
}
