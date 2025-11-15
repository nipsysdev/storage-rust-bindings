//! Chunk download operations for Codex
//!
//! This module provides functionality for downloading individual chunks of data
//! from the Codex network. Chunks are the basic unit of data transfer
//! and can be downloaded individually or as part of a larger download.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_download_chunk, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use std::sync::{Arc, Mutex};

/// Download a single chunk of data
///
/// Downloads a single chunk of data for the specified content ID (CID).
/// This is useful for downloading specific parts of content or for
/// implementing custom download strategies.
///
/// # Arguments
///
/// * `node` - The Codex node to use for the download
/// * `cid` - The content ID of the chunk to download
///
/// # Returns
///
/// The chunk data as a vector of bytes
///
/// # Errors
///
/// Returns an error if:
/// - The CID is empty
/// - The chunk download fails
pub async fn download_chunk(node: &CodexNode, cid: &str) -> Result<Vec<u8>> {
    let node = node.clone();
    let cid = cid.to_string();

    tokio::task::spawn_blocking(move || {
        if cid.is_empty() {
            return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
        }

        let chunk_data = Arc::new(Mutex::new(Vec::<u8>::new()));
        let chunk_data_clone = chunk_data.clone();

        let future = CallbackFuture::new();

        future.context.set_progress_callback(move |_len, chunk| {
            if let Some(chunk_bytes) = chunk {
                let mut data = chunk_data_clone.lock().unwrap();
                data.clear();
                data.extend_from_slice(chunk_bytes);
            }
        });

        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libcodex_lock(|| unsafe {
            let ctx = node.ctx();
            let c_cid = string_to_c_string(&cid);
            let result = codex_download_chunk(ctx as *mut _, c_cid, Some(c_callback), context_ptr);

            free_c_string(c_cid);

            result
        });

        if result != 0 {
            return Err(CodexError::download_error("Failed to download chunk"));
        }

        future.wait()?;

        let data = chunk_data.lock().unwrap().clone();
        Ok(data)
    })
    .await?
}

/// Download multiple chunks in parallel
///
/// Downloads multiple chunks concurrently for better performance.
/// This is useful when you need to download multiple parts of content
/// or when implementing parallel download strategies.
///
/// # Arguments
///
/// * `node` - The Codex node to use for the download
/// * `cids` - A vector of content IDs to download
///
/// # Returns
///
/// A vector of chunk data in the same order as the input CIDs
///
/// # Errors
///
/// Returns an error if any chunk download fails
pub async fn download_chunks(node: &CodexNode, cids: Vec<String>) -> Result<Vec<Vec<u8>>> {
    let node = node.clone();

    let futures: Vec<_> = cids
        .into_iter()
        .enumerate()
        .map(|(index, cid)| {
            let node = node.clone();
            async move { download_chunk(&node, &cid).await.map_err(|e| (index, e)) }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    let mut chunks = Vec::with_capacity(results.len());
    for result in results {
        match result {
            Ok(chunk) => chunks.push(chunk),
            Err((index, e)) => {
                return Err(CodexError::download_error(format!(
                    "Failed to download chunk {}: {}",
                    index, e
                )));
            }
        }
    }

    Ok(chunks)
}

/// Download a chunk with progress callback
///
/// Downloads a single chunk and calls the provided progress callback
/// with the downloaded data. This is useful for streaming large chunks
/// or implementing custom progress tracking.
///
/// # Arguments
///
/// * `node` - The Codex node to use for the download
/// * `cid` - The content ID of the chunk to download
/// * `progress_callback` - Callback function called with chunk data
///
/// # Returns
///
/// Ok(()) if the chunk was downloaded successfully
///
/// # Errors
///
/// Returns an error if:
/// - The CID is empty
/// - The chunk download fails
pub async fn download_chunk_with_progress<F>(
    node: &CodexNode,
    cid: &str,
    progress_callback: F,
) -> Result<()>
where
    F: Fn(&[u8]) + Send + Sync + 'static,
{
    let node = node.clone();
    let cid = cid.to_string();
    let progress_callback = Arc::new(progress_callback);

    tokio::task::spawn_blocking(move || {
        if cid.is_empty() {
            return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
        }

        let future = CallbackFuture::new();
        let progress_callback_clone = progress_callback.clone();

        future.context.set_progress_callback(move |_len, chunk| {
            if let Some(chunk_bytes) = chunk {
                progress_callback_clone(chunk_bytes);
            }
        });

        let context_ptr = future.context_ptr() as *mut c_void;

        let result = with_libcodex_lock(|| unsafe {
            let ctx = node.ctx();
            let c_cid = string_to_c_string(&cid);
            let result = codex_download_chunk(ctx as *mut _, c_cid, Some(c_callback), context_ptr);

            free_c_string(c_cid);

            result
        });

        if result != 0 {
            return Err(CodexError::download_error("Failed to download chunk"));
        }

        future.wait()?;
        Ok(())
    })
    .await?
}
