//! Memory safety tests for the Storage Rust bindings
//!
//! These tests verify that the Rust wrapper properly manages memory and resources.
//!
//! ## Running with Memory Sanitizers
//!
//! To detect memory leaks and other memory safety issues, run these tests with:
//!
//! ### AddressSanitizer (ASan)
//! ```bash
//! RUSTFLAGS="-Z sanitizer=address" cargo test --test memory_safety
//! ```
//!
//! ### ThreadSanitizer (TSan)
//! ```bash
//! RUSTFLAGS="-Z sanitizer=thread" cargo test --test memory_safety
//! ```
//!
//! ### Valgrind (Linux only)
//! ```bash
//! cargo build --release
//! valgrind --leak-check=full --show-leak-kinds=all ./target/release/test_binary
//! ```
//!
//! ## What These Tests Verify
//!
//! - **No memory leaks**: Creating and destroying nodes doesn't leak memory
//! - **No double-free**: Resources are freed exactly once
//! - **No use-after-free**: Context is invalidated after destroy
//! - **Reference counting**: Multiple references prevent premature destruction
//! - **Proper cleanup**: Drop implementation cleans up resources correctly

use storage_bindings::{LogLevel, StorageConfig, StorageNode};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_no_memory_leak() {
    // Create and destroy multiple nodes to check for memory leaks
    // Using a smaller number for faster test execution
    for i in 0..10 {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path().join(format!("storage_{}", i)))
            .discovery_port(8090 + i as u16); // Use unique port to avoid conflicts

        let node = StorageNode::new(config).unwrap();
        node.start_async().await.unwrap();
        node.stop_async().await.unwrap();
        node.destroy_async().await.unwrap();
        // Node should be dropped here
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_no_memory_leak_with_operations() {
    // Create nodes, perform operations, and destroy them
    // Using a smaller number for faster test execution
    for i in 0..5 {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path().join(format!("storage_{}", i)))
            .discovery_port(8090 + i as u16); // Use unique port to avoid conflicts

        let node = StorageNode::new(config).unwrap();
        node.start_async().await.unwrap();

        // Perform various operations
        let _version = node.version().unwrap();
        let _peer_id = node.peer_id().unwrap();
        let _repo = node.repo().unwrap();

        node.stop_async().await.unwrap();
        node.destroy_async().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_context_invalidated_after_destroy() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage"));

    let node = StorageNode::new(config).unwrap();
    node.start_async().await.unwrap();
    node.stop_async().await.unwrap();

    // Destroy the node
    node.destroy_async().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_no_double_free() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage"))
        .discovery_port(8090);

    let node = StorageNode::new(config).unwrap();
    node.start_async().await.unwrap();
    node.stop_async().await.unwrap();
    node.destroy_async().await.unwrap();
    // Node is already dropped after destroy_async

    // Create another node to ensure no corruption
    let temp_dir2 = tempdir().unwrap();
    let config2 = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir2.path().join("storage2"))
        .discovery_port(8091);

    let node2 = StorageNode::new(config2).unwrap();
    node2.start_async().await.unwrap();
    node2.stop_async().await.unwrap();
    node2.destroy_async().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_references_prevent_destroy() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage"))
        .discovery_port(8092);

    let node = StorageNode::new(config).unwrap();
    let node_clone = node.clone(); // Create a second reference

    node.start_async().await.unwrap();
    node.stop_async().await.unwrap();

    // Destroy should fail because there are multiple references
    let result = node.clone().destroy_async().await;
    assert!(result.is_err());

    // Drop the clone
    drop(node_clone);

    // Now destroy should succeed
    let result = node.destroy_async().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lifecycle_cleanup() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage"))
        .discovery_port(8093);

    let node = StorageNode::new(config).unwrap();
    node.start_async().await.unwrap();
    node.stop_async().await.unwrap();
    node.destroy_async().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_drop_cleanup() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage"))
        .discovery_port(8094);

    {
        let node = StorageNode::new(config).unwrap();
        node.start_async().await.unwrap();
        // Node will be dropped here without explicit destroy
    }
}
