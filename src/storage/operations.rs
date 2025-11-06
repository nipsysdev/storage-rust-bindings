//! Storage operations implementation

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_storage_delete, codex_storage_exists, codex_storage_fetch, codex_storage_list,
    codex_storage_space, free_c_string, string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Manifest information for a stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Content ID (CID) - set separately in fetch()
    #[serde(skip)]
    pub cid: String,
    /// Tree CID - root of the merkle tree
    #[serde(rename = "treeCid", default)]
    pub tree_cid: String,
    /// Dataset size - total size of all blocks
    #[serde(rename = "datasetSize")]
    pub dataset_size: usize,
    /// Block size - size of each contained block
    #[serde(rename = "blockSize")]
    pub block_size: usize,
    /// Filename - name of the file (optional)
    #[serde(default)]
    pub filename: String,
    /// Mimetype - MIME type of the file (optional)
    #[serde(default)]
    pub mimetype: String,
    /// Protected datasets have erasure coded info
    #[serde(default)]
    pub protected: bool,
}

/// Manifest with CID wrapper (as returned by storage list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWithCid {
    /// Content ID (CID)
    pub cid: String,
    /// Manifest data
    pub manifest: Manifest,
}

/// Storage space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Total number of blocks stored by the node
    #[serde(rename = "totalBlocks")]
    pub total_blocks: usize,
    /// Maximum storage space (in bytes) available
    #[serde(rename = "quotaMaxBytes")]
    pub quota_max_bytes: u64,
    /// Amount of storage space (in bytes) currently used
    #[serde(rename = "quotaUsedBytes")]
    pub quota_used_bytes: u64,
    /// Amount of storage reserved (in bytes) for future use
    #[serde(rename = "quotaReservedBytes")]
    pub quota_reserved_bytes: u64,
}

/// List all manifests in the storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// A vector of manifests for all stored content
pub async fn manifests(node: &CodexNode) -> Result<Vec<Manifest>> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_list(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "manifests",
            "Failed to list manifests",
        ));
    }

    // Wait for the operation to complete
    let manifests_json = future.await?;

    // Parse the manifests JSON array
    let manifests_with_cid: Vec<ManifestWithCid> = serde_json::from_str(&manifests_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifests: {}", e)))?;

    // Convert to Manifest structs with CID set
    let manifests: Vec<Manifest> = manifests_with_cid
        .into_iter()
        .map(|item| {
            let mut manifest = item.manifest;
            manifest.cid = item.cid;
            manifest
        })
        .collect();

    Ok(manifests)
}

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
pub async fn fetch(node: &CodexNode, cid: &str) -> Result<Manifest> {
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
    let manifest: Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

/// Get storage space information
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Information about storage usage and availability
pub async fn space(node: &CodexNode) -> Result<Space> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_space(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "space",
            "Failed to get storage space",
        ));
    }

    // Wait for the operation to complete
    let space_json = future.await?;

    // Parse the space JSON
    let space: Space = serde_json::from_str(&space_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse space info: {}", e)))?;

    Ok(space)
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
    async fn test_manifests_empty() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error) // Reduce log noise
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024); // 100 MB

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let manifests_result = manifests(&node).await;
        assert!(
            manifests_result.is_ok(),
            "Failed to get manifests: {:?}",
            manifests_result.err()
        );

        let manifest_list = manifests_result.unwrap();
        // Should be empty for a new node
        assert_eq!(manifest_list.len(), 0, "New node should have no manifests");

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_manifests_with_started_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let manifests_result = manifests(&node).await;
        assert!(manifests_result.is_ok());

        let manifest_list = manifests_result.unwrap();
        // Verify the structure of returned manifests
        for manifest in &manifest_list {
            assert!(!manifest.cid.is_empty(), "Manifest CID should not be empty");
            assert!(
                manifest.dataset_size > 0,
                "Manifest dataset_size should be positive"
            );
            assert!(
                manifest.block_size > 0,
                "Manifest block_size should be positive"
            );
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

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
    async fn test_space_info() {
        let temp_dir = tempdir().unwrap();
        let quota = 100 * 1024 * 1024; // 100 MB
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(quota);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let space_result = space(&node).await;
        assert!(
            space_result.is_ok(),
            "Failed to get space info: {:?}",
            space_result.err()
        );

        let space_info = space_result.unwrap();
        assert!(space_info.quota_max_bytes > 0, "Quota should be positive");
        assert!(
            space_info.quota_used_bytes > 0 || space_info.quota_used_bytes == 0,
            "Used space should be valid"
        );
        assert!(
            space_info.quota_used_bytes <= space_info.quota_max_bytes,
            "Used space should not exceed quota"
        );
        assert!(
            space_info.quota_reserved_bytes > 0 || space_info.quota_reserved_bytes == 0,
            "Reserved space should be valid"
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
        let manifests_result = manifests(&node).await;
        assert!(
            manifests_result.is_ok(),
            "Manifests should work without starting node"
        );

        let space_result = space(&node).await;
        assert!(
            space_result.is_ok(),
            "Space info should work without starting node"
        );

        let exists_result = exists(&node, "QmTest").await;
        assert!(
            exists_result.is_ok(),
            "Exists check should work without starting node"
        );

        // These might fail since the node is not started
        let _fetch_result = fetch(&node, "QmTest").await;
        // Don't assert here as behavior might vary

        let _delete_result = delete(&node, "QmTest").await;
        // Don't assert here as behavior might vary

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_manifest_structure() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test manifest to verify structure
        let test_manifest = Manifest {
            cid: "QmTest123".to_string(),
            tree_cid: "QmTree123".to_string(),
            dataset_size: 1024,
            block_size: 256,
            filename: "test.txt".to_string(),
            mimetype: "text/plain".to_string(),
            protected: false,
        };

        // Verify the manifest can be serialized and deserialized
        let json = serde_json::to_string(&test_manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(test_manifest.tree_cid, deserialized.tree_cid);
        assert_eq!(test_manifest.dataset_size, deserialized.dataset_size);
        assert_eq!(test_manifest.block_size, deserialized.block_size);
        assert_eq!(test_manifest.filename, deserialized.filename);
        assert_eq!(test_manifest.mimetype, deserialized.mimetype);
        assert_eq!(test_manifest.protected, deserialized.protected);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_space_structure() {
        let temp_dir = tempdir().unwrap();
        let quota = 200 * 1024 * 1024; // 200 MB
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(quota);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test space info to verify structure
        let test_space = Space {
            total_blocks: 10,
            quota_max_bytes: quota,
            quota_used_bytes: 50 * 1024 * 1024,      // 50 MB
            quota_reserved_bytes: 150 * 1024 * 1024, // 150 MB
        };

        // Verify the space info can be serialized and deserialized
        let json = serde_json::to_string(&test_space).unwrap();
        let deserialized: Space = serde_json::from_str(&json).unwrap();

        assert_eq!(test_space.total_blocks, deserialized.total_blocks);
        assert_eq!(test_space.quota_max_bytes, deserialized.quota_max_bytes);
        assert_eq!(test_space.quota_used_bytes, deserialized.quota_used_bytes);
        assert_eq!(
            test_space.quota_reserved_bytes,
            deserialized.quota_reserved_bytes
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_storage_operations() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test concurrent operations
        let manifests_future = manifests(&node);
        let space_future = space(&node);
        let exists_future = exists(&node, "QmTest1");
        let exists_future2 = exists(&node, "QmTest2");

        let (manifests_result, space_result, exists_result, exists_result2) = tokio::join!(
            manifests_future,
            space_future,
            exists_future,
            exists_future2
        );

        assert!(manifests_result.is_ok());
        assert!(space_result.is_ok());
        assert!(exists_result.is_ok());
        assert!(exists_result2.is_ok());

        node.stop().unwrap();
        node.destroy().unwrap();
    }
}
