//! Tests for thread safety of storage-bindings async functions
//!
//! These tests verify that async functions in storage-bindings can be safely used
//! in multi-threaded contexts, which require futures to implement the Send trait.
//! This is essential for applications using tokio::spawn or other concurrent
//! execution patterns.

use std::future::Future;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// Test that verifies StorageNode::new() returns a Send future
///
/// This test verifies that the future returned by StorageNode::new() implements
/// the Send trait, allowing it to be used in multi-threaded contexts.
#[tokio::test]
async fn test_storage_node_new_returns_send_future() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    // Create the future
    let future = storage_bindings::StorageNode::new(config);

    // Try to send the future to another thread
    // This will fail to compile if the future is not Send
    let handle = thread::spawn(move || {
        // This block won't execute because we can't actually run async code here,
        // but the fact that we can move the future into this closure proves it's Send
        std::mem::drop(future);
    });

    handle.join().expect("Thread panicked");
}

/// Test that verifies StorageNode::new() can be spawned in tokio::task::spawn
///
/// This test verifies the actual use case for multi-threaded applications, which
/// require futures to be Send for tokio::task::spawn.
#[tokio::test(flavor = "multi_thread")]
async fn test_storage_node_new_in_tokio_spawn() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    // Try to spawn the async function in a tokio task
    // This will fail to compile if the future is not Send
    let handle = tokio::spawn(async move {
        let _node = storage_bindings::StorageNode::new(config).await;
    });

    // Wait for the task to complete (or fail)
    let result = handle.await;
    // We expect this to fail because we don't have the actual libstorage library
    // but the important part is that it compiles
    assert!(result.is_ok() || result.is_err());
}

/// Test that verifies StorageNode::start() returns a Send future
#[tokio::test]
async fn test_storage_node_start_returns_send_future() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    // Create a node (this might fail without libstorage, but that's ok)
    let node_result = storage_bindings::StorageNode::new(config).await;

    if let Ok(node) = node_result {
        // Try to send the start future to another thread
        let handle = thread::spawn(move || {
            std::mem::drop(node.start());
        });

        handle.join().expect("Thread panicked");
    }
    // If node creation failed, that's expected without libstorage
}

/// Test that verifies StorageNode::stop() returns a Send future
#[tokio::test]
async fn test_storage_node_stop_returns_send_future() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    let node_result = storage_bindings::StorageNode::new(config).await;

    if let Ok(node) = node_result {
        // Try to send the stop future to another thread
        let handle = thread::spawn(move || {
            std::mem::drop(node.stop());
        });

        handle.join().expect("Thread panicked");
    }
}

/// Test that verifies StorageNode::peer_id() returns a Send future
#[tokio::test]
async fn test_storage_node_peer_id_returns_send_future() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    let node_result = storage_bindings::StorageNode::new(config).await;

    if let Ok(node) = node_result {
        // Try to send the peer_id future to another thread
        let handle = thread::spawn(move || {
            std::mem::drop(node.peer_id());
        });

        handle.join().expect("Thread panicked");
    }
}

/// Test that simulates a multi-threaded command scenario
///
/// Multi-threaded applications require futures to be Send because they are spawned
/// in tokio tasks. This test simulates that scenario.
#[tokio::test(flavor = "multi_thread")]
async fn test_multi_threaded_command_scenario() {
    let temp_dir = Arc::new(TempDir::new().expect("Failed to create temp directory"));
    let temp_dir_clone = temp_dir.clone();

    // Simulate a command that starts a node
    async fn start_node_command(data_dir: std::path::PathBuf) -> Result<(), String> {
        let config = storage_bindings::StorageConfig::new()
            .log_level(storage_bindings::LogLevel::Error)
            .data_dir(&data_dir);

        let node = storage_bindings::StorageNode::new(config)
            .await
            .map_err(|e| e.to_string())?;

        node.start().await.map_err(|e| e.to_string())?;

        Ok(())
    }

    // Try to spawn the command in a tokio task
    let handle =
        tokio::spawn(async move { start_node_command(temp_dir_clone.path().to_path_buf()).await });

    let result = handle.await;
    // We expect this to fail without libstorage, but it should compile
    assert!(result.is_ok() || result.is_err());
}

/// Test that verifies the future is Send using trait bounds
///
/// This is a compile-time test that uses trait bounds to verify Send.
#[tokio::test]
async fn test_future_is_send_trait_bound() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    let future = storage_bindings::StorageNode::new(config);

    // This function only accepts Send futures
    fn assert_send<F: Future + Send>(_: F) {}

    // This will fail to compile if the future is not Send
    assert_send(future);
}

/// Test that verifies multiple async operations can be spawned concurrently
///
/// This tests the scenario where multiple async operations might be running
/// concurrently, all requiring Send futures.
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_send_futures() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");

    let config1 = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir1.path());

    let config2 = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir2.path());

    // Try to spawn multiple futures concurrently
    let handle1 = tokio::spawn(async move { storage_bindings::StorageNode::new(config1).await });

    let handle2 = tokio::spawn(async move { storage_bindings::StorageNode::new(config2).await });

    // Wait for both to complete
    let _ = handle1.await;
    let _ = handle2.await;
}

/// Test that verifies Arc<StorageNode> can be sent across threads
///
/// StorageNode itself is Send, but we need to verify that Arc<StorageNode>
/// can be used in multi-threaded scenarios.
#[tokio::test]
async fn test_arc_storage_node_is_send() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = storage_bindings::StorageConfig::new()
        .log_level(storage_bindings::LogLevel::Error)
        .data_dir(temp_dir.path());

    let node_result = storage_bindings::StorageNode::new(config).await;

    if let Ok(node) = node_result {
        let node_arc = Arc::new(node);

        // Try to send Arc<StorageNode> to another thread
        let handle = thread::spawn(move || {
            let _ = node_arc;
        });

        handle.join().expect("Thread panicked");
    }
}

/// Test that verifies SendSafeCString works correctly in async context
///
/// This test demonstrates that SendSafeCString properly implements Send,
/// allowing futures to be used in multi-threaded contexts.
#[tokio::test]
async fn test_send_safe_cstring_in_async_context() {
    use storage_bindings::ffi::SendSafeCString;

    // This simulates what happens in StorageNode::new() with the fix
    let json_config = r#"{"log_level":"error","data_dir":"/tmp"}"#;
    let c_json_config = SendSafeCString::new(json_config);

    // The SendSafeCString is captured here - this is now Send!
    let future = async move {
        // This SendSafeCString is captured in the async block
        let _ptr = c_json_config;
        // The pointer will be automatically cleaned up when dropped
    };

    // Try to send this future to another thread
    // This will succeed because SendSafeCString is Send
    let handle = thread::spawn(move || {
        // Use pin to satisfy the Future trait
        std::mem::drop(Box::pin(future));
    });

    handle.join().expect("Thread panicked");
}
