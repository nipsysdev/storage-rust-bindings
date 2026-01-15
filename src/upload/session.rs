//! Upload session management for Codex
//!
//! This module provides low-level session management operations for uploads.
//! These functions handle the lifecycle of upload sessions including initialization,
//! finalization, and cancellation.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    free_c_string, storage_upload_cancel, storage_upload_finalize, storage_upload_init,
    string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use crate::upload::types::UploadOptions;
use libc::c_void;

/// Initialize an upload session
///
/// Creates a new upload session with the specified options. Returns a session ID
/// that can be used for subsequent chunk uploads.
///
/// # Arguments
///
/// * `node` - The Codex node to use for the upload
/// * `options` - Upload configuration options
///
/// # Returns
///
/// A session ID string that identifies this upload session
pub async fn upload_init(node: &CodexNode, options: &UploadOptions) -> Result<String> {
    let node = node.clone();
    let options = options.clone();

    tokio::task::spawn_blocking(move || {
        options.validate()?;

        let future = CallbackFuture::new();

        let filepath_str = options
            .filepath
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        let chunk_size = options.chunk_size.unwrap_or(1024 * 1024);
        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libcodex_lock(|| unsafe {
            node.with_ctx(|ctx| {
                let c_filepath = string_to_c_string(filepath_str);
                let result = storage_upload_init(
                    ctx as *mut _,
                    c_filepath,
                    chunk_size,
                    Some(c_callback),
                    context_ptr,
                );

                if !c_filepath.is_null() {
                    free_c_string(c_filepath);
                }

                result
            })
        });

        if result != 0 {
            return Err(CodexError::upload_error("Failed to initialize upload"));
        }

        let session_id = future.wait()?;
        Ok(session_id)
    })
    .await?
}

/// Finalize an upload session
///
/// Completes an upload session and returns the content ID (CID) of the uploaded data.
/// This should be called after all chunks have been uploaded.
///
/// # Arguments
///
/// * `node` - The Codex node used for the upload
/// * `session_id` - The session ID returned by `upload_init`
///
/// # Returns
///
/// The CID of the uploaded content
pub async fn upload_finalize(node: &CodexNode, session_id: &str) -> Result<String> {
    let node = node.clone();
    let session_id = session_id.to_string();

    tokio::task::spawn_blocking(move || {
        if session_id.is_empty() {
            return Err(CodexError::invalid_parameter(
                "session_id",
                "Session ID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libcodex_lock(|| unsafe {
            node.with_ctx(|ctx| {
                let c_session_id = string_to_c_string(&session_id);
                let result = storage_upload_finalize(
                    ctx as *mut _,
                    c_session_id,
                    Some(c_callback),
                    context_ptr,
                );

                free_c_string(c_session_id);

                result
            })
        });

        if result != 0 {
            return Err(CodexError::upload_error("Failed to finalize upload"));
        }

        let cid = future.wait()?;
        Ok(cid)
    })
    .await?
}

/// Cancel an upload session
///
/// Cancels an ongoing upload session and cleans up any resources associated with it.
/// This should be called if an upload needs to be aborted.
///
/// # Arguments
///
/// * `node` - The Codex node used for the upload
/// * `session_id` - The session ID returned by `upload_init`
pub async fn upload_cancel(node: &CodexNode, session_id: &str) -> Result<()> {
    let node = node.clone();
    let session_id = session_id.to_string();

    tokio::task::spawn_blocking(move || {
        if session_id.is_empty() {
            return Err(CodexError::invalid_parameter(
                "session_id",
                "Session ID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libcodex_lock(|| unsafe {
            node.with_ctx(|ctx| {
                let c_session_id = string_to_c_string(&session_id);
                let result = storage_upload_cancel(
                    ctx as *mut _,
                    c_session_id,
                    Some(c_callback),
                    context_ptr,
                );

                free_c_string(c_session_id);

                result
            })
        });

        if result != 0 {
            return Err(CodexError::upload_error("Failed to cancel upload"));
        }

        future.wait()?;
        Ok(())
    })
    .await?
}
