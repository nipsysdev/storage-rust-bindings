use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_debug, codex_log_level, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub id: String,
    pub addrs: Vec<String>,
    pub spr: String,
    #[serde(rename = "announceAddresses")]
    pub announce_addresses: Vec<String>,
    pub table: DiscoveryTable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTable {
    #[serde(rename = "localNode")]
    pub local_node: LocalNodeInfo,
    pub nodes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: String,
    pub record: String,
    pub address: String,
    pub seen: bool,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn peer_id(&self) -> &str {
        &self.id
    }

    pub fn address_count(&self) -> usize {
        self.addrs.len()
    }

    pub fn announce_address_count(&self) -> usize {
        self.announce_addresses.len()
    }

    pub fn discovery_node_count(&self) -> usize {
        self.table.nodes.len()
    }

    pub fn is_healthy(&self) -> bool {
        !self.id.is_empty()
            && !self.addrs.is_empty()
            && !self.spr.is_empty()
            && !self.table.local_node.node_id.is_empty()
    }

    pub fn health_status(&self) -> &'static str {
        if self.is_healthy() {
            "Healthy"
        } else {
            "Unhealthy"
        }
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            addrs: Vec::new(),
            spr: String::new(),
            announce_addresses: Vec::new(),
            table: DiscoveryTable {
                local_node: LocalNodeInfo {
                    node_id: String::new(),
                    peer_id: String::new(),
                    record: String::new(),
                    address: String::new(),
                    seen: false,
                },
                nodes: Vec::new(),
            },
        }
    }
}

pub async fn debug(node: &CodexNode) -> Result<DebugInfo> {
    let node = node.clone();

    tokio::task::spawn_blocking(move || {
        let future = CallbackFuture::new();

        let result = with_libcodex_lock(|| unsafe {
            node.with_ctx(|ctx| {
                codex_debug(
                    ctx as *mut _,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                )
            })
        });

        if result != 0 {
            return Err(CodexError::library_error("Failed to get debug info"));
        }

        let debug_json = future.wait()?;

        let debug_info: DebugInfo = serde_json::from_str(&debug_json)
            .map_err(|e| CodexError::library_error(format!("Failed to parse debug info: {}", e)))?;

        Ok(debug_info)
    })
    .await?
}

pub async fn update_log_level(node: &CodexNode, log_level: LogLevel) -> Result<()> {
    let node = node.clone();

    tokio::task::spawn_blocking(move || {
        let future = CallbackFuture::new();

        let c_log_level = string_to_c_string(&log_level.to_string());

        let result = with_libcodex_lock(|| unsafe {
            node.with_ctx(|ctx| {
                codex_log_level(
                    ctx as *mut _,
                    c_log_level,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                )
            })
        });

        unsafe {
            free_c_string(c_log_level);
        }

        if result != 0 {
            return Err(CodexError::library_error("Failed to update log level"));
        }

        future.wait()?;

        Ok(())
    })
    .await?
}
