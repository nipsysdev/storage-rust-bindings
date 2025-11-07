//! Basic upload operations
//!
//! This module contains the core upload operations: init, chunk, finalize, and cancel.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_upload_cancel, codex_upload_chunk, codex_upload_finalize, codex_upload_init,
    free_c_string, string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use crate::upload::types::UploadOptions;
use libc::c_void;
use std::ptr;

/// Initialize an upload operation
///
/// # Arguments
///
/// * `node` - The Codex node to use for the upload
/// * `options` - Upload options
///
/// # Returns
///
/// A session ID that can be used to upload chunks and finalize the upload
pub async fn upload_init(node: &CodexNode, options: &UploadOptions) -> Result<String> {
    options.validate()?;

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_filepath = if let Some(ref filepath) = options.filepath {
        string_to_c_string(filepath.to_str().unwrap_or(""))
    } else {
        ptr::null_mut()
    };

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_upload_init(
            node.ctx as *mut _,
            c_filepath,
            options.chunk_size.unwrap_or(1024 * 1024),
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        if !c_filepath.is_null() {
            free_c_string(c_filepath);
        }
    }

    if result != 0 {
        return Err(CodexError::upload_error("Failed to initialize upload"));
    }

    // Wait for the operation to complete
    let session_id = future.await?;
    Ok(session_id)
}

/// Upload a chunk of data
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `session_id` - The session ID returned by upload_init
/// * `chunk` - The chunk data to upload
///
/// # Returns
///
/// Ok(()) if the chunk was uploaded successfully, or an error
pub async fn upload_chunk(node: &CodexNode, session_id: &str, chunk: &[u8]) -> Result<()> {
    if session_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    if chunk.is_empty() {
        return Err(CodexError::invalid_parameter(
            "chunk",
            "Chunk cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_session_id = string_to_c_string(session_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_upload_chunk(
            node.ctx as *mut _,
            c_session_id,
            chunk.as_ptr() as *mut u8,
            chunk.len(),
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_session_id);
    }

    if result != 0 {
        return Err(CodexError::upload_error("Failed to upload chunk"));
    }

    // Wait for the operation to complete
    future.await?;
    Ok(())
}

/// Finalize an upload operation
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `session_id` - The session ID returned by upload_init
///
/// # Returns
///
/// The CID of the uploaded content
pub async fn upload_finalize(node: &CodexNode, session_id: &str) -> Result<String> {
    if session_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_session_id = string_to_c_string(session_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_upload_finalize(
            node.ctx as *mut _,
            c_session_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_session_id);
    }

    if result != 0 {
        return Err(CodexError::upload_error("Failed to finalize upload"));
    }

    // Wait for the operation to complete
    let cid = future.await?;
    Ok(cid)
}

/// Cancel an upload operation
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `session_id` - The session ID returned by upload_init
///
/// # Returns
///
/// Ok(()) if the upload was cancelled successfully, or an error
pub async fn upload_cancel(node: &CodexNode, session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "session_id",
            "Session ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_session_id = string_to_c_string(session_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_upload_cancel(
            node.ctx as *mut _,
            c_session_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_session_id);
    }

    if result != 0 {
        return Err(CodexError::upload_error("Failed to cancel upload"));
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
    async fn test_upload_init() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();
        let options = UploadOptions::new();

        let session_id = upload_init(&node, &options).await;
        assert!(session_id.is_ok());
    }

    #[tokio::test]
    async fn test_upload_init_invalid_options() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = UploadOptions::new().chunk_size(0); // Invalid chunk size
        let session_id = upload_init(&node, &options).await;
        assert!(session_id.is_err());

        match session_id.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "chunk_size");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_upload_chunk() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();
        let options = UploadOptions::new();

        let session_id = upload_init(&node, &options).await.unwrap();
        let chunk = b"Hello, world!";

        let result = upload_chunk(&node, &session_id, chunk).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_finalize() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();
        let options = UploadOptions::new();

        let session_id = upload_init(&node, &options).await.unwrap();
        let cid = upload_finalize(&node, &session_id).await;

        assert!(cid.is_ok());
        assert!(!cid.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_upload_cancel() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();
        let options = UploadOptions::new();

        let session_id = upload_init(&node, &options).await.unwrap();
        let result = upload_cancel(&node, &session_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_invalid_session_id() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = upload_chunk(&node, "", b"data").await;
        assert!(result.is_err());

        let result = upload_finalize(&node, "").await;
        assert!(result.is_err());

        let result = upload_cancel(&node, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upload_invalid_chunk() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();
        let options = UploadOptions::new();

        let session_id = upload_init(&node, &options).await.unwrap();
        let result = upload_chunk(&node, &session_id, &[]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "chunk");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }
}
