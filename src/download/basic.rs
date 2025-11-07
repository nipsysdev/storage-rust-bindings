//! Basic download operations
//!
//! This module contains the core download operations: init, chunk, and cancel.

use crate::callback::{c_callback, CallbackFuture};
use crate::download::types::DownloadOptions;
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_download_cancel, codex_download_chunk, codex_download_init, free_c_string,
    string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;

/// Initialize a download operation
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to download
/// * `options` - Download options
///
/// # Returns
///
/// Ok(()) if the download was initialized successfully, or an error
pub async fn download_init(node: &CodexNode, cid: &str, options: &DownloadOptions) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    options.validate()?;

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Convert options to JSON
    let options_json = serde_json::to_string(options).map_err(CodexError::from)?;
    let _c_options_json = string_to_c_string(&options_json);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_init(
            node.ctx as *mut _,
            c_cid,
            options.chunk_size.unwrap_or(1024 * 1024),
            false, // local flag
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to initialize download"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Download a chunk of data
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID
///
/// # Returns
///
/// The chunk data
pub async fn download_chunk(node: &CodexNode, cid: &str) -> Result<Vec<u8>> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Use a shared container to store the chunk data
    use std::sync::Mutex;
    let chunk_data = std::sync::Arc::new(Mutex::new(Vec::<u8>::new()));
    let chunk_data_clone = chunk_data.clone();

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Set up a progress callback to capture the chunk data
    // This follows the same pattern as the Go implementation
    future.context.set_progress_callback(move |_len, chunk| {
        if let Some(chunk_bytes) = chunk {
            let mut data = chunk_data_clone.lock().unwrap();
            data.clear(); // Clear any previous data
            data.extend_from_slice(chunk_bytes);
        }
    });

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_chunk(
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
        return Err(CodexError::download_error("Failed to download chunk"));
    }

    // Wait for the operation to complete
    future.await?;

    // Extract the chunk data
    let data = chunk_data.lock().unwrap().clone();
    Ok(data)
}

/// Cancel a download operation
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID
///
/// # Returns
///
/// Ok(()) if the download was cancelled successfully, or an error
pub async fn download_cancel(node: &CodexNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_cancel(
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
        return Err(CodexError::download_error("Failed to cancel download"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;

    #[tokio::test]
    async fn test_download_init_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new("QmExample");
        let result = download_init(&node, "", &options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_init_invalid_options() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new(""); // Invalid empty CID
        let result = download_init(&node, "QmExample", &options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_chunk_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_chunk(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_cancel_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_cancel(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_init() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new("QmExample");
        let result = download_init(&node, "QmExample", &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_chunk() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let chunk = download_chunk(&node, "QmExample").await;
        assert!(chunk.is_ok());

        let data = chunk.unwrap();
        assert!(!data.is_empty());
    }

    #[tokio::test]
    async fn test_download_cancel() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_cancel(&node, "QmExample").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_invalid_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_init(&node, "", &DownloadOptions::new("QmExample")).await;
        assert!(result.is_err());

        let result = download_chunk(&node, "").await;
        assert!(result.is_err());

        let result = download_cancel(&node, "").await;
        assert!(result.is_err());
    }
}
