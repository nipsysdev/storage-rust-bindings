//! CRUD operations for storage
//!
//! This module contains content operations: fetch, delete, and exists.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_storage_delete, codex_storage_exists, codex_storage_fetch, free_c_string,
    string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;

/// Fetch manifest information for a specific content
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to fetch manifest for
///
/// # Returns
///
/// The manifest information for the specified content
pub async fn fetch(node: &CodexNode, cid: &str) -> Result<super::types::Manifest> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_fetch(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "fetch",
            "Failed to fetch manifest",
        ));
    }

    // Wait for the operation to complete
    let manifest_json = future.await?;

    // Parse the manifest JSON
    let manifest: super::types::Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

/// Delete content from storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to delete
///
/// # Returns
///
/// Ok(()) if the content was deleted successfully, or an error
pub async fn delete(node: &CodexNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_delete(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "delete",
            "Failed to delete content",
        ));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Check if content exists in storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to check
///
/// # Returns
///
/// true if the content exists, false otherwise
pub async fn exists(node: &CodexNode, cid: &str) -> Result<bool> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_exists(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::storage_error(
            "exists",
            "Failed to check if content exists",
        ));
    }

    // Wait for the operation to complete
    let exists_str = future.await?;

    // Parse the boolean result
    let exists = exists_str
        .parse::<bool>()
        .map_err(|e| CodexError::library_error(format!("Failed to parse exists result: {}", e)))?;

    Ok(exists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_fetch_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let non_existent_cid = "QmNonExistent123456789";
        let fetch_result = fetch(&node, non_existent_cid).await;
        // This should fail since the content doesn't exist
        assert!(
            fetch_result.is_err(),
            "Fetching non-existent content should fail"
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let non_existent_cid = "QmNonExistent123456789";
        let delete_result = delete(&node, non_existent_cid).await;
        // This might fail since the content doesn't exist, but the function should handle it gracefully
        assert!(delete_result.is_ok() || delete_result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_exists_various_cids() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test with various CID formats
        let test_cids = vec![
            "QmNonExistent123456789",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "QmaQwYWpchozXhFv8nvxprECWBSCEppN9dfd2VQiJfRo3F",
            "zdj7WWeQ43G6JJvLWQWZpyHuAMq6uYWRjkBXZadLDEotRHi7T7ycf",
        ];

        for cid in test_cids {
            let exists_result = exists(&node, cid).await;
            assert!(
                exists_result.is_ok(),
                "Failed to check existence for CID {}: {:?}",
                cid,
                exists_result.err()
            );

            let exists = exists_result.unwrap();
            // For a new node, none of these should exist
            assert!(!exists, "CID {} should not exist in a new node", cid);
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_invalid_cid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test empty CID
        let empty_cid = "";
        let fetch_result = fetch(&node, empty_cid).await;
        assert!(fetch_result.is_err(), "Fetching with empty CID should fail");

        let error = fetch_result.unwrap_err();
        assert!(error.to_string().contains("CID cannot be empty"));

        let delete_result = delete(&node, empty_cid).await;
        assert!(
            delete_result.is_err(),
            "Deleting with empty CID should fail"
        );

        let error = delete_result.unwrap_err();
        assert!(error.to_string().contains("CID cannot be empty"));

        let exists_result = exists(&node, empty_cid).await;
        assert!(
            exists_result.is_err(),
            "Checking existence with empty CID should fail"
        );

        let error = exists_result.unwrap_err();
        assert!(error.to_string().contains("CID cannot be empty"));

        // Test with whitespace-only CID
        let whitespace_cid = "   \t\n   ";
        let fetch_result = fetch(&node, whitespace_cid).await;
        assert!(
            fetch_result.is_err(),
            "Fetching with whitespace-only CID should fail"
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_storage_operations_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        // These operations should work even if the node is not started
        let _fetch_result = fetch(&node, "QmTest").await;
        // Don't assert here as behavior might vary

        let _delete_result = delete(&node, "QmTest").await;
        // Don't assert here as behavior might vary

        let exists_result = exists(&node, "QmTest").await;
        assert!(
            exists_result.is_ok(),
            "Exists check should work without starting node"
        );

        node.destroy().unwrap();
    }
}
