pub mod callback;
pub mod error;
pub mod ffi;

pub mod debug;
pub mod download;
pub mod node;
pub mod p2p;
pub mod storage;
pub mod upload;

// Patch system
pub mod build_integration;
pub mod patch_system;

// Debug operations and types
pub use debug::{debug, peer_debug, update_log_level, DebugInfo};

pub use download::{
    download_cancel, download_chunk, download_init, download_manifest, download_stream,
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions,
};

pub use error::{CodexError, Result};

pub use node::{CodexConfig, CodexNode, LogFormat, LogLevel};

pub use p2p::{
    connect, connect_to_multiple, get_peer_id, get_peer_info, validate_addresses, validate_peer_id,
    ConnectionQuality, PeerInfo, PeerRecord,
};

pub use storage::{delete, exists, fetch, manifests, space, Manifest as StorageManifest, Space};

pub use upload::{
    upload_cancel, upload_chunk, upload_file, upload_finalize, upload_init, upload_reader,
    UploadOptions, UploadProgress, UploadResult, UploadStrategy,
};

pub use upload::{
    create_streaming_reader, AsyncStreamingUploadReader, StreamingUploadReader, UploadProgressExt,
};
