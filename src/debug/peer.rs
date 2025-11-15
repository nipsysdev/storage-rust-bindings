//! Peer debugging operations
//!
//! This module contains peer-specific debugging operations.

use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_peer_debug, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use crate::p2p::types::PeerRecord;
use libc::c_void;

/// Get detailed debug information about a specific peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get debug information for
///
/// # Returns
///
/// Detailed peer record for debugging
pub async fn peer_debug(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
    let node = node.clone();
    let peer_id = peer_id.to_string();

    tokio::task::spawn_blocking(move || {
        if peer_id.is_empty() {
            return Err(CodexError::invalid_parameter(
                "peer_id",
                "Peer ID cannot be empty",
            ));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        with_libcodex_lock(|| {
            let c_peer_id = string_to_c_string(&peer_id);

            // Call the C function with the context pointer directly
            let result = unsafe {
                node.with_ctx(|ctx| {
                    codex_peer_debug(
                        ctx as *mut _,
                        c_peer_id,
                        Some(c_callback),
                        future.context_ptr() as *mut c_void,
                    )
                })
            };

            // Clean up
            unsafe {
                free_c_string(c_peer_id);
            }

            if result != 0 {
                return Err(CodexError::library_error("Failed to get peer debug info"));
            }

            Ok(())
        })?;

        // Wait for the operation to complete
        let peer_json = future.wait()?;

        // Parse the peer JSON
        let peer: PeerRecord = serde_json::from_str(&peer_json).map_err(|e| {
            CodexError::library_error(format!("Failed to parse peer debug info: {}", e))
        })?;

        Ok(peer)
    })
    .await?
}
