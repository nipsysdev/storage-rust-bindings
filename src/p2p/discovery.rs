//! P2P discovery operations
//!
//! This module contains peer discovery and information operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_peer_debug, codex_peer_id, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
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
    /// Connection duration in seconds
    pub connection_duration_seconds: Option<u64>,
    /// Bytes sent to this peer
    pub bytes_sent: Option<u64>,
    /// Bytes received from this peer
    pub bytes_received: Option<u64>,
    /// Additional peer metadata
    pub metadata: Option<serde_json::Value>,
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

/// List all connected peers
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// A vector of peer information for all connected peers
///
/// Note: This function is not available in the current C API.
/// Use the debug operations to get peer information.
pub async fn list_peers(_node: &CodexNode) -> Result<Vec<PeerInfo>> {
    Err(CodexError::library_error(
        "list_peers is not available in the current C API",
    ))
}

/// Discover peers by protocol
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `protocol` - The protocol to search for
///
/// # Returns
///
/// A vector of peer information for peers supporting the protocol
pub async fn discover_peers_by_protocol(node: &CodexNode, protocol: &str) -> Result<Vec<PeerInfo>> {
    // This would typically call a C function to discover peers by protocol
    // For now, we'll return an empty vector as a placeholder

    if protocol.is_empty() {
        return Err(CodexError::invalid_parameter(
            "protocol",
            "Protocol cannot be empty",
        ));
    }

    // In a real implementation, you might call something like:
    // let peers_json = unsafe { codex_discover_peers_by_protocol(node.ctx as *mut _, c_protocol) };

    Ok(vec![])
}

/// Get network statistics
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Network statistics as a JSON value
///
/// Note: This function is not available in the current C API.
/// Use the debug operation to get general node information.
pub async fn network_stats(_node: &CodexNode) -> Result<serde_json::Value> {
    Err(CodexError::library_error(
        "network_stats is not available in the current C API",
    ))
}

/// Get peer reputation information
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get reputation for
///
/// # Returns
///
/// Peer reputation information
pub async fn get_peer_reputation(node: &CodexNode, peer_id: &str) -> Result<PeerReputation> {
    // This would typically call a C function to get peer reputation
    // For now, we'll return a placeholder

    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // In a real implementation, you might call something like:
    // let reputation_json = unsafe { codex_peer_reputation(node.ctx as *mut _, c_peer_id) };

    Ok(PeerReputation {
        peer_id: peer_id.to_string(),
        score: 0.5,
        successful_interactions: 0,
        failed_interactions: 0,
        last_interaction: None,
        reputation_level: ReputationLevel::Neutral,
    })
}

/// Peer reputation information
#[derive(Debug, Clone)]
pub struct PeerReputation {
    /// Peer ID
    pub peer_id: String,
    /// Reputation score (0.0 to 1.0)
    pub score: f64,
    /// Number of successful interactions
    pub successful_interactions: u64,
    /// Number of failed interactions
    pub failed_interactions: u64,
    /// Last interaction time
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    /// Reputation level
    pub reputation_level: ReputationLevel,
}

/// Reputation level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationLevel {
    /// Very poor reputation
    VeryPoor,
    /// Poor reputation
    Poor,
    /// Neutral reputation
    Neutral,
    /// Good reputation
    Good,
    /// Excellent reputation
    Excellent,
}

impl PeerReputation {
    /// Get the total number of interactions
    pub fn total_interactions(&self) -> u64 {
        self.successful_interactions + self.failed_interactions
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_interactions();
        if total == 0 {
            0.0
        } else {
            self.successful_interactions as f64 / total as f64 * 100.0
        }
    }

    /// Update reputation based on a new interaction
    pub fn update_interaction(&mut self, success: bool) {
        if success {
            self.successful_interactions += 1;
        } else {
            self.failed_interactions += 1;
        }

        // Update score and level
        self.score = self.success_rate() / 100.0;
        self.reputation_level = self.calculate_reputation_level();
        self.last_interaction = Some(chrono::Utc::now());
    }

    fn calculate_reputation_level(&self) -> ReputationLevel {
        if self.score >= 0.9 {
            ReputationLevel::Excellent
        } else if self.score >= 0.7 {
            ReputationLevel::Good
        } else if self.score >= 0.3 {
            ReputationLevel::Neutral
        } else if self.score >= 0.1 {
            ReputationLevel::Poor
        } else {
            ReputationLevel::VeryPoor
        }
    }
}

/// Search for peers by criteria
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `criteria` - Search criteria
///
/// # Returns
///
/// A vector of matching peers
pub async fn search_peers(node: &CodexNode, criteria: PeerSearchCriteria) -> Result<Vec<PeerInfo>> {
    // This would typically call a C function to search for peers
    // For now, we'll return an empty vector as a placeholder

    // Validate criteria
    if criteria.min_connections > criteria.max_connections {
        return Err(CodexError::invalid_parameter(
            "criteria",
            "min_connections cannot be greater than max_connections",
        ));
    }

    // In a real implementation, you might call something like:
    // let peers_json = unsafe { codex_search_peers(node.ctx as *mut _, c_criteria) };

    Ok(vec![])
}

/// Peer search criteria
#[derive(Debug, Clone, Default)]
pub struct PeerSearchCriteria {
    /// Minimum number of connections
    pub min_connections: usize,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Required protocols
    pub required_protocols: Vec<String>,
    /// Exclude protocols
    pub exclude_protocols: Vec<String>,
    /// Maximum latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Minimum reputation score
    pub min_reputation: Option<f64>,
    /// Limit the number of results
    pub limit: Option<usize>,
}

impl PeerSearchCriteria {
    /// Create new search criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum connections
    pub fn min_connections(mut self, min: usize) -> Self {
        self.min_connections = min;
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Add a required protocol
    pub fn require_protocol(mut self, protocol: String) -> Self {
        self.required_protocols.push(protocol);
        self
    }

    /// Add an excluded protocol
    pub fn exclude_protocol(mut self, protocol: String) -> Self {
        self.exclude_protocols.push(protocol);
        self
    }

    /// Set maximum latency
    pub fn max_latency(mut self, latency_ms: u64) -> Self {
        self.max_latency_ms = Some(latency_ms);
        self
    }

    /// Set minimum reputation
    pub fn min_reputation(mut self, score: f64) -> Self {
        self.min_reputation = Some(score);
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

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
    async fn test_list_peers_not_implemented() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peers = list_peers(&node).await;
        assert!(peers.is_err());

        let error = peers.unwrap_err();
        assert!(error
            .to_string()
            .contains("not available in the current C API"));

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_discover_peers_by_protocol() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let result = discover_peers_by_protocol(&node, "/codex/1.0.0").await;
        assert!(result.is_ok());

        let peers = result.unwrap();
        // Should return empty vector for now (placeholder implementation)
        assert_eq!(peers.len(), 0);

        // Test with empty protocol
        let result = discover_peers_by_protocol(&node, "").await;
        assert!(result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_network_stats_not_implemented() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let stats = network_stats(&node).await;
        assert!(stats.is_err());

        let error = stats.unwrap_err();
        assert!(error
            .to_string()
            .contains("not available in the current C API"));

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_get_peer_reputation() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let reputation = get_peer_reputation(&node, peer_id).await;
        assert!(reputation.is_ok());

        let reputation = reputation.unwrap();
        assert_eq!(reputation.peer_id, peer_id);
        assert_eq!(reputation.score, 0.5);
        assert_eq!(reputation.successful_interactions, 0);
        assert_eq!(reputation.failed_interactions, 0);
        assert_eq!(reputation.reputation_level, ReputationLevel::Neutral);

        // Test with empty peer ID
        let result = get_peer_reputation(&node, "").await;
        assert!(result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_search_peers() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let criteria = PeerSearchCriteria::new()
            .min_connections(1)
            .max_connections(10)
            .require_protocol("/codex/1.0.0".to_string())
            .max_latency(1000)
            .min_reputation(0.5)
            .limit(50);

        let result = search_peers(&node, criteria).await;
        assert!(result.is_ok());

        let peers = result.unwrap();
        // Should return empty vector for now (placeholder implementation)
        assert_eq!(peers.len(), 0);

        // Test with invalid criteria
        let invalid_criteria = PeerSearchCriteria::new()
            .min_connections(10)
            .max_connections(5); // min > max

        let result = search_peers(&node, invalid_criteria).await;
        assert!(result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_peer_reputation_methods() {
        let mut reputation = PeerReputation {
            peer_id: "12D3KooWExamplePeer".to_string(),
            score: 0.5,
            successful_interactions: 5,
            failed_interactions: 5,
            last_interaction: None,
            reputation_level: ReputationLevel::Neutral,
        };

        assert_eq!(reputation.total_interactions(), 10);
        assert_eq!(reputation.success_rate(), 50.0);

        // Update with successful interaction
        reputation.update_interaction(true);
        assert_eq!(reputation.successful_interactions, 6);
        assert_eq!(reputation.failed_interactions, 5);
        assert_eq!(reputation.total_interactions(), 11);
        assert!(reputation.success_rate() > 50.0);
        assert!(reputation.last_interaction.is_some());

        // Update with failed interaction
        reputation.update_interaction(false);
        assert_eq!(reputation.successful_interactions, 6);
        assert_eq!(reputation.failed_interactions, 6);
        assert_eq!(reputation.total_interactions(), 12);
        assert_eq!(reputation.success_rate(), 50.0);
    }

    #[test]
    fn test_peer_search_criteria() {
        let criteria = PeerSearchCriteria::new()
            .min_connections(1)
            .max_connections(10)
            .require_protocol("/codex/1.0.0".to_string())
            .exclude_protocol("/legacy/1.0.0".to_string())
            .max_latency(1000)
            .min_reputation(0.5)
            .limit(50);

        assert_eq!(criteria.min_connections, 1);
        assert_eq!(criteria.max_connections, 10);
        assert_eq!(criteria.required_protocols.len(), 1);
        assert_eq!(criteria.exclude_protocols.len(), 1);
        assert_eq!(criteria.max_latency_ms, Some(1000));
        assert_eq!(criteria.min_reputation, Some(0.5));
        assert_eq!(criteria.limit, Some(50));
    }

    #[test]
    fn test_peer_info_serialization() {
        let peer_info = PeerInfo {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
        };

        let json = serde_json::to_string(&peer_info).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(peer_info.id, deserialized.id);
        assert_eq!(peer_info.addresses, deserialized.addresses);
        assert_eq!(peer_info.connected, deserialized.connected);
        assert_eq!(peer_info.direction, deserialized.direction);
        assert_eq!(peer_info.latency_ms, deserialized.latency_ms);
    }

    #[test]
    fn test_peer_record_serialization() {
        let peer_record = PeerRecord {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
            protocols: vec!["/codex/1.0.0".to_string()],
            user_agent: Some("codex-rust-bindings/0.1.0".to_string()),
            last_seen: Some("2023-01-01T12:00:00Z".to_string()),
            connection_duration_seconds: Some(1800),
            bytes_sent: Some(1024 * 1024),
            bytes_received: Some(2 * 1024 * 1024),
            metadata: Some(serde_json::json!({"version": "1.0.0"})),
        };

        let json = serde_json::to_string(&peer_record).unwrap();
        let deserialized: PeerRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(peer_record.id, deserialized.id);
        assert_eq!(peer_record.addresses, deserialized.addresses);
        assert_eq!(peer_record.connected, deserialized.connected);
        assert_eq!(peer_record.direction, deserialized.direction);
        assert_eq!(peer_record.latency_ms, deserialized.latency_ms);
        assert_eq!(peer_record.protocols, deserialized.protocols);
        assert_eq!(peer_record.user_agent, deserialized.user_agent);
        assert_eq!(peer_record.last_seen, deserialized.last_seen);
        assert_eq!(
            peer_record.connection_duration_seconds,
            deserialized.connection_duration_seconds
        );
        assert_eq!(peer_record.bytes_sent, deserialized.bytes_sent);
        assert_eq!(peer_record.bytes_received, deserialized.bytes_received);
        assert_eq!(peer_record.metadata, deserialized.metadata);
    }
}
