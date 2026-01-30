use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{free_c_string, storage_connect, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use libc::{c_char, c_void};

pub async fn connect(node: &StorageNode, peer_id: &str, peer_addresses: &[String]) -> Result<()> {
    if peer_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    if peer_addresses.is_empty() {
        return Err(StorageError::invalid_parameter(
            "peer_addresses",
            "At least one peer address must be provided",
        ));
    }

    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    let c_addresses: Vec<*mut c_char> = peer_addresses
        .iter()
        .map(|addr| string_to_c_string(addr))
        .collect();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_connect(
                ctx as *mut _,
                c_peer_id,
                c_addresses.as_ptr() as *mut *const c_char,
                c_addresses.len(),
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        })
    });

    unsafe {
        free_c_string(c_peer_id);
        for addr in c_addresses {
            free_c_string(addr);
        }
    }

    if result != 0 {
        return Err(StorageError::p2p_error("Failed to connect to peer"));
    }

    future.await?;

    Ok(())
}

pub async fn connect_to_multiple(
    node: &StorageNode,
    peer_connections: Vec<(String, Vec<String>)>,
) -> Vec<Result<()>> {
    let mut results = Vec::with_capacity(peer_connections.len());

    for (peer_id, addresses) in peer_connections {
        let result = connect(node, &peer_id, &addresses).await;
        results.push(result);
    }

    results
}

pub fn validate_peer_id(peer_id: &str) -> Result<()> {
    if peer_id.is_empty() {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    if peer_id.len() < 10 {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID is too short",
        ));
    }

    if peer_id.len() > 100 {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID is too long",
        ));
    }

    let valid_prefixes = ["12D3KooW", "Qm", "bafy", "bafk"];

    let has_valid_prefix = valid_prefixes
        .iter()
        .any(|&prefix| peer_id.starts_with(prefix));

    if !has_valid_prefix {
        return Err(StorageError::invalid_parameter(
            "peer_id",
            "Peer ID has invalid format or prefix",
        ));
    }

    Ok(())
}

pub fn validate_addresses(addresses: &[String]) -> Result<()> {
    if addresses.is_empty() {
        return Err(StorageError::invalid_parameter(
            "addresses",
            "At least one address must be provided",
        ));
    }

    for (i, address) in addresses.iter().enumerate() {
        if address.is_empty() {
            return Err(StorageError::invalid_parameter(
                format!("addresses[{}]", i),
                "Address cannot be empty",
            ));
        }

        if !address.starts_with('/') {
            return Err(StorageError::invalid_parameter(
                format!("addresses[{}]", i),
                "Address must start with '/'",
            ));
        }

        let valid_protocols = [
            "/ip4", "/ip6", "/dns4", "/dns6", "/dnsaddr", "/tcp", "/udp", "/quic", "/ws", "/wss",
            "/p2p", "/ipfs",
        ];

        let has_valid_protocol = valid_protocols
            .iter()
            .any(|&protocol| address.contains(protocol));

        if !has_valid_protocol {
            return Err(StorageError::invalid_parameter(
                format!("addresses[{}]", i),
                "Address contains invalid protocol",
            ));
        }
    }

    Ok(())
}
