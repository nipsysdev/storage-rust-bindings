use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_peer_debug, storage_peer_id, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use crate::p2p::types::PeerRecord;

pub async fn get_peer_info(node: &StorageNode, peer_id: &str) -> Result<PeerRecord> {
    if peer_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_peer_id = string_to_c_string(peer_id);

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
        return Err(StorageError::p2p_error("Failed to get peer info"));
    }

    let peer_json = future.await?;

    let peer: PeerRecord = serde_json::from_str(&peer_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse peer info: {}", e)))?;

    Ok(peer)
}

pub async fn get_peer_id(node: &StorageNode) -> Result<String> {
    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| storage_peer_id(ctx as *mut _, Some(c_callback), context_ptr.as_ptr()))
    });

    if result != 0 {
        return Err(StorageError::p2p_error("Failed to get peer ID"));
    }

    let peer_id = future.await?;

    Ok(peer_id)
}
