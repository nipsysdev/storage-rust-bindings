//! Peer debugging operations
//!
//! This module contains peer-specific debugging operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_peer_debug, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use crate::p2p::types::PeerRecord;

/// Get detailed debug information about a specific peer
///
/// # Arguments
///
/// * `node` - The Storage node to use
/// * `peer_id` - The peer ID to get debug information for
///
/// # Returns
///
/// Detailed peer record for debugging
pub async fn peer_debug(node: &StorageNode, peer_id: &str) -> Result<PeerRecord> {
    if peer_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_peer_id = string_to_c_string(peer_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        node.with_ctx(|ctx| {
            storage_peer_debug(
                ctx as *mut _,
                c_peer_id.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    };

    if result != 0 {
        return Err(StorageError::library_error("Failed to get peer debug info"));
    }

    // Wait for the operation to complete
    let peer_json = future.await?;

    // Parse the peer JSON
    let peer: PeerRecord = serde_json::from_str(&peer_json).map_err(|e| {
        StorageError::library_error(format!("Failed to parse peer debug info: {}", e))
    })?;

    Ok(peer)
}
