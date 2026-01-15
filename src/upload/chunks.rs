//! Chunk upload operations for Storage
//!
//! This module provides functionality for uploading individual chunks of data
//! as part of an upload session. Chunks are the basic unit of data transfer
//! in the Storage network.

use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{free_c_string, storage_upload_chunk, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use libc::c_void;

/// Upload a chunk of data as part of an ongoing upload session
///
/// Uploads a single chunk of data to the Storage network. The chunk will be
/// associated with the specified session ID.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the upload
/// * `session_id` - The session ID returned by `upload_init`
/// * `chunk` - The chunk data to upload
///
/// # Returns
///
/// Ok(()) if the chunk was uploaded successfully
///
/// # Errors
///
/// Returns an error if:
/// - The session ID is empty
/// - The chunk is empty
/// - The upload fails for any reason
pub async fn upload_chunk(node: &StorageNode, session_id: &str, chunk: Vec<u8>) -> Result<()> {
    let node = node.clone();
    let session_id = session_id.to_string();

    tokio::task::spawn_blocking(move || {
        if session_id.is_empty() {
            return Err(StorageError::invalid_parameter(
                "session_id",
                "Session ID cannot be empty",
            ));
        }

        if chunk.is_empty() {
            return Err(StorageError::invalid_parameter(
                "chunk",
                "Chunk cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let chunk_ptr = chunk.as_ptr() as *mut u8;
        let chunk_len = chunk.len();
        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libstorage_lock(|| unsafe {
            node.with_ctx(|ctx| {
                let c_session_id = string_to_c_string(&session_id);
                let result = storage_upload_chunk(
                    ctx as *mut _,
                    c_session_id,
                    chunk_ptr,
                    chunk_len,
                    Some(c_callback),
                    context_ptr,
                );

                free_c_string(c_session_id);

                result
            })
        });

        if result != 0 {
            return Err(StorageError::upload_error("Failed to upload chunk"));
        }

        future.wait()?;
        Ok(())
    })
    .await?
}

/// Upload multiple chunks in sequence
///
/// Convenience function to upload multiple chunks one after another.
/// This is useful when you have all chunks ready and want to upload them
/// in a single operation.
///
/// # Arguments
///
/// * `node` - The Storage node to use for the upload
/// * `session_id` - The session ID returned by `upload_init`
/// * `chunks` - A vector of chunks to upload
///
/// # Returns
///
/// Ok(()) if all chunks were uploaded successfully
///
/// # Errors
///
/// Returns an error if any chunk fails to upload
pub async fn upload_chunks(
    node: &StorageNode,
    session_id: &str,
    chunks: Vec<Vec<u8>>,
) -> Result<()> {
    for (index, chunk) in chunks.into_iter().enumerate() {
        upload_chunk(node, session_id, chunk).await.map_err(|e| {
            StorageError::upload_error(format!("Failed to upload chunk {}: {}", index, e))
        })?;
    }
    Ok(())
}
