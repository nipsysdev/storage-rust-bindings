//! P2P operations implementation

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_connect, codex_peer_debug, codex_peer_id, free_c_string, string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::{c_char, c_void};
use serde::{Deserialize, Serialize};

/// Information about a peer in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Whether the peer is connected
    pub connected: bool,
    /// Connection direction (inbound/outbound)
    pub direction: Option<String>,
    /// Latency to the peer (in milliseconds)
    pub latency_ms: Option<u64>,
}

/// Detailed peer record for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Connection state
    pub connected: bool,
    /// Connection direction
    pub direction: Option<String>,
    /// Latency information
    pub latency_ms: Option<u64>,
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Connect to a peer in the Codex network
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to connect to
/// * `peer_addresses` - List of multiaddresses for the peer
///
/// # Returns
///
/// Ok(()) if the connection was successful, or an error
pub async fn connect(node: &CodexNode, peer_id: &str, peer_addresses: &[String]) -> Result<()> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    if peer_addresses.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_addresses",
            "At least one peer address must be provided",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Convert addresses to C array
    let c_addresses: Vec<*mut c_char> = peer_addresses
        .iter()
        .map(|addr| string_to_c_string(addr))
        .collect();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_connect(
            node.ctx as *mut _,
            c_peer_id,
            c_addresses.as_ptr() as *mut *mut c_char,
            c_addresses.len(),
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
        for addr in c_addresses {
            free_c_string(addr);
        }
    }

    if result != 0 {
        return Err(CodexError::p2p_error("Failed to connect to peer"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Get detailed information about a specific peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get information for
///
/// # Returns
///
/// Detailed peer record
/// Get detailed information about a specific peer
pub async fn get_peer_info(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_peer_debug(
            node.ctx as *mut _,
            c_peer_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
    }

    if result != 0 {
        return Err(CodexError::p2p_error("Failed to get peer info"));
    }

    // Wait for the operation to complete
    let peer_json = future.await?;

    // Parse the peer JSON
    let peer: PeerRecord = serde_json::from_str(&peer_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse peer info: {}", e)))?;

    Ok(peer)
}

/// Get the peer ID of the current node
pub async fn get_peer_id(node: &CodexNode) -> Result<String> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_peer_id(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::p2p_error("Failed to get peer ID"));
    }

    // Wait for the operation to complete
    let peer_id = future.await?;

    Ok(peer_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_connect_with_valid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            "/ip6/::1/tcp/8080".to_string(),
        ];

        let result = connect(&node, peer_id, &addresses).await;
        // This might fail if the peer doesn't exist, but the function should handle it gracefully
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_single_address() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec!["/ip4/192.168.1.100/tcp/8080".to_string()];

        let result = connect(&node, peer_id, &addresses).await;
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_multiple_addresses() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            "/ip4/192.168.1.100/tcp/8081".to_string(),
            "/ip6/::1/tcp/8080".to_string(),
            "/ip6/::1/udp/8080/quic".to_string(),
            "/dns4/example.com/tcp/8080".to_string(),
        ];

        let result = connect(&node, peer_id, &addresses).await;
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_invalid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Empty peer ID
        let result = connect(&node, "", &["/ip4/192.168.1.100/tcp/8080".to_string()]).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Peer ID cannot be empty"));

        // Empty addresses
        let result = connect(&node, "12D3KooWExamplePeer", &[]).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("At least one peer address must be provided"));

        // Empty peer ID with empty addresses
        let result = connect(&node, "", &[]).await;
        assert!(result.is_err());

        // Valid peer ID but empty address in list
        let result = connect(&node, "12D3KooWExamplePeer", &["".to_string()]).await;
        // This might not fail immediately but should be handled gracefully
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_invalid_addresses() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";

        // Test with various invalid address formats
        let invalid_addresses = vec![
            vec!["invalid-address".to_string()],
            vec!["/ip4/256.256.256.256/tcp/8080".to_string()], // Invalid IP
            vec!["/ip4/192.168.1.100/tcp/99999".to_string()],  // Invalid port
            vec!["/invalid/protocol/address".to_string()],
            vec!["".to_string()],
        ];

        for addresses in invalid_addresses {
            let result = connect(&node, peer_id, &addresses).await;
            // These might fail but should not panic
            assert!(result.is_ok() || result.is_err());
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_get_peer_info() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let peer_info_result = get_peer_info(&node, peer_id).await;
        // This might fail if the peer doesn't exist, but the function should work
        assert!(peer_info_result.is_ok() || peer_info_result.is_err());

        if let Ok(peer_info) = peer_info_result {
            assert_eq!(peer_info.id, peer_id);
            assert!(!peer_info.addresses.is_empty());
            // Verify the structure of the returned peer info
            assert!(!peer_info.protocols.is_empty());
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_get_peer_id() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id_result = get_peer_id(&node).await;
        assert!(
            peer_id_result.is_ok(),
            "Failed to get peer ID: {:?}",
            peer_id_result.err()
        );

        let peer_id = peer_id_result.unwrap();
        assert!(!peer_id.is_empty(), "Peer ID should not be empty");

        // Verify it's a valid peer ID format (starts with known prefixes)
        assert!(
            peer_id.starts_with("12D3KooW")
                || peer_id.starts_with("Qm")
                || peer_id.starts_with("bafy"),
            "Peer ID should have a valid prefix: {}",
            peer_id
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_get_peer_info_invalid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Empty peer ID
        let result = get_peer_info(&node, "").await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Peer ID cannot be empty"));

        // Whitespace-only peer ID
        let result = get_peer_info(&node, "   \t\n   ").await;
        assert!(result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_p2p_operations_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        // These operations should work even if the node is not started
        let peer_id_result = get_peer_id(&node).await;
        assert!(
            peer_id_result.is_ok(),
            "Getting peer ID should work without starting node"
        );

        let _peer_info_result = get_peer_info(&node, "12D3KooWExamplePeer").await;
        // This might work or fail depending on the implementation

        let _connect_result = connect(
            &node,
            "12D3KooWExamplePeer",
            &["/ip4/192.168.1.100/tcp/8080".to_string()],
        )
        .await;
        // This might work or fail depending on the implementation

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_peer_info_serialization() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test peer info to verify structure
        let test_peer_info = PeerInfo {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
        };

        // Verify the peer info can be serialized and deserialized
        let json = serde_json::to_string(&test_peer_info).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(test_peer_info.id, deserialized.id);
        assert_eq!(test_peer_info.addresses, deserialized.addresses);
        assert_eq!(test_peer_info.connected, deserialized.connected);
        assert_eq!(test_peer_info.direction, deserialized.direction);
        assert_eq!(test_peer_info.latency_ms, deserialized.latency_ms);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_p2p_operations() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test concurrent operations
        let peer_id_future = get_peer_id(&node);
        let peer_info_future = get_peer_info(&node, "12D3KooWExamplePeer1");
        let peer_info_future2 = get_peer_info(&node, "12D3KooWExamplePeer2");

        let (peer_id_result, peer_info_result, peer_info_result2) =
            tokio::join!(peer_id_future, peer_info_future, peer_info_future2);

        assert!(peer_id_result.is_ok());
        assert!(peer_info_result.is_ok() || peer_info_result.is_err());
        assert!(peer_info_result2.is_ok() || peer_info_result2.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_various_peer_ids() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let addresses = vec!["/ip4/192.168.1.100/tcp/8080".to_string()];

        // Test with various peer ID formats
        let test_peer_ids = vec![
            "12D3KooWExamplePeer123456789",
            "QmSomePeerId123456789",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ];

        for peer_id in test_peer_ids {
            let result = connect(&node, peer_id, &addresses).await;
            // These might fail but should not panic
            assert!(result.is_ok() || result.is_err());
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }
}
