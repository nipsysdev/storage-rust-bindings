//! Node configuration structures for Codex

use crate::error::{CodexError, Result};
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

/// Serialize u64 as string for compatibility with C library
/// Note: storage-quota must be a string due to C library requirements
fn serialize_u64_as_string<S>(
    value: &Option<u64>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

/// Log level for the Codex node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Fatal,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Notice => write!(f, "notice"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

/// Log format for the Codex node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Auto,
    Colors,
    NoColors,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Auto
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Auto => write!(f, "auto"),
            LogFormat::Colors => write!(f, "colors"),
            LogFormat::NoColors => write!(f, "nocolors"),
            LogFormat::Json => write!(f, "json"),
        }
    }
}

/// Repository kind for storage backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoKind {
    Fs,
    Sqlite,
    LevelDb,
}

impl Default for RepoKind {
    fn default() -> Self {
        RepoKind::Fs
    }
}

impl std::fmt::Display for RepoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoKind::Fs => write!(f, "fs"),
            RepoKind::Sqlite => write!(f, "sqlite"),
            RepoKind::LevelDb => write!(f, "leveldb"),
        }
    }
}

/// Configuration for a Codex node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// Log level (default: INFO)
    #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    /// Log format (default: auto)
    #[serde(rename = "log-format", skip_serializing_if = "Option::is_none")]
    pub log_format: Option<LogFormat>,

    /// Enable the metrics server (default: false)
    #[serde(rename = "metrics", skip_serializing_if = "Option::is_none")]
    pub metrics_enabled: Option<bool>,

    /// Listening address of the metrics server (default: 127.0.0.1)
    #[serde(rename = "metrics-address", skip_serializing_if = "Option::is_none")]
    pub metrics_address: Option<String>,

    /// Listening HTTP port of the metrics server (default: 8008)
    #[serde(rename = "metrics-port", skip_serializing_if = "Option::is_none")]
    pub metrics_port: Option<u16>,

    /// The directory where codex will store configuration and data
    #[serde(rename = "data-dir", skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,

    /// Multi Addresses to listen on (default: ["/ip4/0.0.0.0/tcp/0"])
    #[serde(rename = "listen-addrs", skip_serializing_if = "Vec::is_empty")]
    pub listen_addrs: Vec<String>,

    /// Specify method to use for determining public address
    #[serde(rename = "nat", skip_serializing_if = "Option::is_none")]
    pub nat: Option<String>,

    /// Discovery (UDP) port (default: 8090)
    #[serde(rename = "disc-port", skip_serializing_if = "Option::is_none")]
    pub discovery_port: Option<u16>,

    /// Source of network (secp256k1) private key file path or name (default: "key")
    #[serde(rename = "net-privkey", skip_serializing_if = "Option::is_none")]
    pub net_priv_key_file: Option<PathBuf>,

    /// Specifies one or more bootstrap nodes to use when connecting to the network
    #[serde(rename = "bootstrap-node", skip_serializing_if = "Vec::is_empty")]
    pub bootstrap_nodes: Vec<String>,

    /// The maximum number of peers to connect to (default: 160)
    #[serde(rename = "max-peers", skip_serializing_if = "Option::is_none")]
    pub max_peers: Option<u32>,

    /// Number of worker threads (default: 0 = use as many threads as there are CPU cores available)
    #[serde(rename = "num-threads", skip_serializing_if = "Option::is_none")]
    pub num_threads: Option<u32>,

    /// Node agent string which is used as identifier in network (default: "Codex")
    #[serde(rename = "agent-string", skip_serializing_if = "Option::is_none")]
    pub agent_string: Option<String>,

    /// Backend for main repo store (fs, sqlite, leveldb) (default: fs)
    #[serde(rename = "repo-kind", skip_serializing_if = "Option::is_none")]
    pub repo_kind: Option<RepoKind>,

    /// The size of the total storage quota dedicated to the node (default: 20 GiBs)
    #[serde(
        rename = "storage-quota",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_u64_as_string"
    )]
    pub storage_quota: Option<u64>,

    /// Default block timeout in seconds - 0 disables the ttl (default: 30 days)
    #[serde(rename = "block-ttl", skip_serializing_if = "Option::is_none")]
    pub block_ttl: Option<u32>,

    /// Time interval in seconds - determines frequency of block maintenance cycle (default: 10 minutes)
    #[serde(rename = "block-mi", skip_serializing_if = "Option::is_none")]
    pub block_maintenance_interval: Option<u32>,

    /// Number of blocks to check every maintenance cycle (default: 1000)
    #[serde(rename = "block-mn", skip_serializing_if = "Option::is_none")]
    pub block_maintenance_number_of_blocks: Option<u32>,

    /// Number of times to retry fetching a block before giving up (default: 3000)
    #[serde(rename = "block-retries", skip_serializing_if = "Option::is_none")]
    pub block_retries: Option<u32>,

    /// The size of the block cache, 0 disables the cache (default: 0)
    #[serde(rename = "cache-size", skip_serializing_if = "Option::is_none")]
    pub cache_size: Option<u64>,

    /// Log file path (default: "" - no log file)
    #[serde(rename = "log-file", skip_serializing_if = "Option::is_none")]
    pub log_file: Option<PathBuf>,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: Some(LogFormat::Auto),
            metrics_enabled: Some(false),
            metrics_address: Some("127.0.0.1".to_string()),
            metrics_port: Some(8008),
            data_dir: None,
            listen_addrs: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
            nat: Some("any".to_string()),
            discovery_port: Some(8090),
            net_priv_key_file: None,
            bootstrap_nodes: vec![],
            max_peers: Some(160),
            num_threads: Some(0),
            agent_string: Some("Codex".to_string()),
            repo_kind: Some(RepoKind::Fs),
            storage_quota: Some(20 * 1024 * 1024 * 1024), // 20 GiB
            block_ttl: Some(30 * 24 * 60 * 60),           // 30 days in seconds
            block_maintenance_interval: Some(10 * 60),    // 10 minutes in seconds
            block_maintenance_number_of_blocks: Some(1000),
            block_retries: Some(3000),
            cache_size: Some(0),
            log_file: None,
        }
    }
}

impl CodexConfig {
    /// Create a new configuration with minimal values (compatible with C library)
    pub fn new() -> Self {
        Self {
            log_level: Some(LogLevel::Info),
            log_format: None,
            metrics_enabled: None,
            metrics_address: None,
            metrics_port: None,
            data_dir: None,
            listen_addrs: vec![],
            nat: None,
            discovery_port: None,
            net_priv_key_file: None,
            bootstrap_nodes: vec![],
            max_peers: None,
            num_threads: None,
            agent_string: None,
            repo_kind: None,
            storage_quota: None,
            block_ttl: None,
            block_maintenance_interval: None,
            block_maintenance_number_of_blocks: None,
            block_retries: None,
            cache_size: None,
            log_file: None,
        }
    }

    /// Create a new configuration with all default values (for advanced use)
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Set the data directory
    pub fn data_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Set the storage quota in bytes
    pub fn storage_quota(mut self, quota: u64) -> Self {
        self.storage_quota = Some(quota);
        self
    }

    /// Add a bootstrap node
    pub fn add_bootstrap_node<S: Into<String>>(mut self, node: S) -> Self {
        self.bootstrap_nodes.push(node.into());
        self
    }

    /// Set the maximum number of peers
    pub fn max_peers(mut self, max: u32) -> Self {
        self.max_peers = Some(max);
        self
    }

    /// Set the repository kind
    pub fn repo_kind(mut self, kind: RepoKind) -> Self {
        self.repo_kind = Some(kind);
        self
    }

    /// Convert the configuration to a JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(CodexError::from)
    }

    /// Create a configuration from a JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(CodexError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CodexConfig::default();
        assert_eq!(config.log_level, Some(LogLevel::Info));
        assert_eq!(config.log_format, Some(LogFormat::Auto));
        assert_eq!(config.metrics_enabled, Some(false));
        assert_eq!(config.max_peers, Some(160));
    }

    #[test]
    fn test_config_builder() {
        let config = CodexConfig::new()
            .log_level(LogLevel::Debug)
            .data_dir("/tmp/codex")
            .storage_quota(1024 * 1024) // 1 MB
            .max_peers(100)
            .repo_kind(RepoKind::Sqlite);

        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.data_dir, Some(PathBuf::from("/tmp/codex")));
        assert_eq!(config.storage_quota, Some(1024 * 1024));
        assert_eq!(config.max_peers, Some(100));
        assert_eq!(config.repo_kind, Some(RepoKind::Sqlite));
    }

    #[test]
    fn test_config_json() {
        let config = CodexConfig::new().log_level(LogLevel::Debug);
        let json = config.to_json().unwrap();
        let parsed = CodexConfig::from_json(&json).unwrap();
        assert_eq!(parsed.log_level, Some(LogLevel::Debug));
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(LogFormat::Auto.to_string(), "auto");
        assert_eq!(LogFormat::Colors.to_string(), "colors");
        assert_eq!(LogFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_repo_kind_display() {
        assert_eq!(RepoKind::Fs.to_string(), "fs");
        assert_eq!(RepoKind::Sqlite.to_string(), "sqlite");
        assert_eq!(RepoKind::LevelDb.to_string(), "leveldb");
    }
}
