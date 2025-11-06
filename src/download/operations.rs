//! Download operations implementation

use crate::callback::{c_callback, CallbackFuture};
use crate::download::types::{DownloadOptions, DownloadResult, DownloadStreamOptions, Manifest};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_download_cancel, codex_download_chunk, codex_download_init, codex_download_manifest,
    free_c_string, string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use std::io::Write;
use std::ptr;
use std::sync::Arc;

/// Download manifest information for a content
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to get manifest for
///
/// # Returns
///
/// The manifest information for the content
pub async fn download_manifest(node: &CodexNode, cid: &str) -> Result<Manifest> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_manifest(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to download manifest"));
    }

    // Wait for the operation to complete
    let manifest_json = future.await?;

    // Parse the manifest JSON
    let manifest: Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

/// Initialize a download operation
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to download
/// * `options` - Download options
///
/// # Returns
///
/// Ok(()) if the download was initialized successfully, or an error
pub async fn download_init(node: &CodexNode, cid: &str, options: &DownloadOptions) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    options.validate()?;

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Convert options to JSON
    let options_json = serde_json::to_string(options).map_err(CodexError::from)?;
    let _c_options_json = string_to_c_string(&options_json);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_init(
            node.ctx as *mut _,
            c_cid,
            options.chunk_size.unwrap_or(1024 * 1024),
            false, // local flag
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to initialize download"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Download a chunk of data
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID
///
/// # Returns
///
/// The chunk data
pub async fn download_chunk(node: &CodexNode, cid: &str) -> Result<Vec<u8>> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Use a shared container to store the chunk data
    use std::sync::Mutex;
    let chunk_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let chunk_data_clone = chunk_data.clone();

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Set up a progress callback to capture the chunk data
    future.context.set_progress_callback(move |_len, chunk| {
        println!(
            "Download progress callback: len={}, chunk={:?}",
            _len,
            chunk.is_some()
        );
        if let Some(chunk_bytes) = chunk {
            let mut data = chunk_data_clone.lock().unwrap();
            data.extend_from_slice(chunk_bytes);
            println!("Added {} bytes to chunk data", chunk_bytes.len());
        }
    });

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_chunk(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to download chunk"));
    }

    // Wait for the operation to complete
    future.await?;

    // Extract the chunk data
    let data = chunk_data.lock().unwrap().clone();
    Ok(data)
}

/// Cancel a download operation
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID
///
/// # Returns
///
/// Ok(()) if the download was cancelled successfully, or an error
pub async fn download_cancel(node: &CodexNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_cancel(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to cancel download"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Stream download content to a file or writer
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `cid` - The content ID to download
/// * `options` - Download stream options
///
/// # Returns
///
/// The result of the download operation
pub async fn download_stream(
    node: &CodexNode,
    cid: &str,
    options: DownloadStreamOptions,
) -> Result<DownloadResult> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    options.validate()?;

    let start_time = std::time::Instant::now();
    let chunk_size = options.chunk_size.unwrap_or(1024 * 1024); // 1 MB default

    // Get dataset size if auto-detect is enabled
    let _dataset_size = if options.dataset_size_auto {
        match download_manifest(node, cid).await {
            Ok(manifest) => Some(manifest.size),
            Err(_) => None,
        }
    } else {
        options.dataset_size
    };

    // Use a shared container to store the downloaded data
    use std::sync::Mutex;
    let total_bytes = Arc::new(Mutex::new(0usize));
    let total_bytes_clone = total_bytes.clone();

    // Create file handle if filepath is specified
    let file_handle = if let Some(ref filepath) = options.filepath {
        match std::fs::File::create(filepath) {
            Ok(file) => Some(Arc::new(Mutex::new(Some(file)))),
            Err(e) => {
                return Err(CodexError::Io(e));
            }
        }
    } else {
        None
    };

    // Create a callback future for the operation (similar to Go's bridge context)
    let future = CallbackFuture::new();
    let context = future.context.clone();

    // Set up a progress callback to capture the downloaded data and write it
    let _context_clone = context.clone();
    let file_handle_clone = file_handle.clone();

    // For the writer, we need to handle it differently since we can't store it directly
    // We'll use a channel to send the data to be written
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let tx_clone = tx.clone();
    let writer_task = if options.writer.is_some() {
        let mut writer = options.writer.unwrap();
        Some(std::thread::spawn(move || {
            while let Ok(chunk) = rx.recv() {
                if let Err(e) = writer.write_all(&chunk) {
                    eprintln!("Failed to write to writer: {}", e);
                    break;
                }
            }
        }))
    } else {
        None
    };

    context.set_progress_callback(move |len, chunk| {
        println!(
            "Download stream progress: len={}, chunk={:?}",
            len,
            chunk.is_some()
        );

        if let Some(chunk_bytes) = chunk {
            let mut total = total_bytes_clone.lock().unwrap();
            *total += chunk_bytes.len();

            // Write to file if specified
            if let Some(ref file_handle) = file_handle_clone {
                if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
                    use std::io::Write;
                    if let Err(e) = file.write_all(chunk_bytes) {
                        eprintln!("Failed to write to file: {}", e);
                    }
                }
            }

            // Send to writer thread if available
            if let Err(_) = tx_clone.send(chunk_bytes.to_vec()) {
                eprintln!("Failed to send data to writer thread");
            }
        }
    });

    let c_cid = string_to_c_string(cid);
    let c_filepath = if let Some(ref filepath) = options.filepath {
        string_to_c_string(filepath.to_str().unwrap_or(""))
    } else {
        ptr::null_mut()
    };

    // Initialize the download first (required by the C library)
    println!(
        "[{}] Initializing download for CID: {}",
        chrono::Utc::now(),
        cid
    );
    let download_options = DownloadOptions::new(cid)
        .chunk_size(chunk_size)
        .timeout(options.timeout.unwrap_or(300))
        .verify(options.verify);

    download_init(node, cid, &download_options).await?;
    println!("[{}] Download initialized successfully", chrono::Utc::now());

    // Call the C function to stream the download
    println!("[{}] Starting download stream...", chrono::Utc::now());
    println!("[{}] Calling codex_download_stream...", chrono::Utc::now());
    let result = unsafe {
        crate::ffi::codex_download_stream(
            node.ctx as *mut _,
            c_cid,
            chunk_size,
            options.local,
            c_filepath,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };
    println!(
        "[{}] codex_download_stream returned: {}",
        chrono::Utc::now(),
        result
    );

    // Clean up
    unsafe {
        free_c_string(c_cid);
        if !c_filepath.is_null() {
            free_c_string(c_filepath);
        }
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to download stream"));
    }

    // Wait for the operation to complete
    println!(
        "[{}] Waiting for download to complete...",
        chrono::Utc::now()
    );
    future.await?;
    println!("[{}] Download completed!", chrono::Utc::now());

    // Close the channel to signal the writer thread to finish
    drop(tx);

    // Wait for the writer thread to finish
    if let Some(handle) = writer_task {
        if let Err(e) = handle.join() {
            eprintln!("Writer thread failed: {:?}", e);
        }
    }

    // Flush file handles
    if let Some(file_handle) = file_handle {
        if let Some(ref mut file) = file_handle.lock().unwrap().as_mut() {
            use std::io::Write;
            if let Err(e) = file.flush() {
                eprintln!("Failed to flush file: {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    let bytes_downloaded = *total_bytes.lock().unwrap();

    let mut result = DownloadResult::new(cid.to_string(), bytes_downloaded)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify);

    if let Some(filepath) = options.filepath {
        result = result.filepath(filepath);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_download_manifest() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let manifest = download_manifest(&node, "QmExample").await;
        assert!(manifest.is_ok());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.cid, "QmExample");
        assert_eq!(manifest.size, 1024);
        assert_eq!(manifest.blocks, 4);
    }

    #[tokio::test]
    async fn test_download_manifest_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_manifest(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_init_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new("QmExample");
        let result = download_init(&node, "", &options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_init_invalid_options() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new(""); // Invalid empty CID
        let result = download_init(&node, "QmExample", &options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_chunk_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_chunk(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_cancel_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_cancel(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_stream_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let buffer = Cursor::new(Vec::new());
        let options = DownloadStreamOptions::new("").writer(buffer);

        let result = download_stream(&node, "", options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_stream_invalid_options() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadStreamOptions::new("QmExample");
        // No filepath or writer - should fail validation
        let result = download_stream(&node, "QmExample", options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "filepath/writer");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_download_stream_with_invalid_chunk_size() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let buffer = Cursor::new(Vec::new());
        let options = DownloadStreamOptions::new("QmExample")
            .writer(buffer)
            .chunk_size(0); // Invalid chunk size

        let result = download_stream(&node, "QmExample", options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "chunk_size");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_download_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let callback = move |len: usize, chunk: Option<&[u8]>| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            println!("Progress: {} bytes, chunk: {:?}", len, chunk.is_some());
        };

        // Test the callback with some data
        let test_data = b"test data";
        callback(test_data.len(), Some(test_data));

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_download_result_creation() {
        let result = DownloadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true)
            .filepath("/test/downloaded.txt");

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
        assert_eq!(
            result.filepath,
            Some(std::path::PathBuf::from("/test/downloaded.txt"))
        );
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024,
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: Some("2023-01-02T00:00:00Z".to_string()),
            content_type: Some("text/plain".to_string()),
            metadata: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.cid, deserialized.cid);
        assert_eq!(manifest.size, deserialized.size);
        assert_eq!(manifest.blocks, deserialized.blocks);
        assert_eq!(manifest.created, deserialized.created);
        assert_eq!(manifest.accessed, deserialized.accessed);
        assert_eq!(manifest.content_type, deserialized.content_type);
        assert_eq!(manifest.metadata, deserialized.metadata);
    }

    #[tokio::test]
    async fn test_download_init() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = DownloadOptions::new("QmExample");
        let result = download_init(&node, "QmExample", &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_chunk() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let chunk = download_chunk(&node, "QmExample").await;
        assert!(chunk.is_ok());

        let data = chunk.unwrap();
        assert!(!data.is_empty());
    }

    #[tokio::test]
    async fn test_download_cancel() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_cancel(&node, "QmExample").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_stream_to_writer() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let buffer = Cursor::new(Vec::new());
        let options = DownloadStreamOptions::new("QmExample")
            .writer(buffer)
            .chunk_size(1024);

        let result = download_stream(&node, "QmExample", options).await;
        assert!(result.is_ok());

        let download_result = result.unwrap();
        assert_eq!(download_result.cid, "QmExample");
        assert!(download_result.size > 0);
    }

    #[tokio::test]
    async fn test_download_invalid_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_manifest(&node, "").await;
        assert!(result.is_err());

        let options = DownloadOptions::new("QmExample");
        let result = download_init(&node, "", &options).await;
        assert!(result.is_err());

        let result = download_chunk(&node, "").await;
        assert!(result.is_err());

        let result = download_cancel(&node, "").await;
        assert!(result.is_err());
    }
}
