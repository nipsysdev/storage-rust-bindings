//! Advanced upload operations
//!
//! This module contains high-level upload operations like upload_reader and upload_file.

use crate::error::{CodexError, Result};
use crate::node::lifecycle::CodexNode;
use crate::upload::basic::{upload_cancel, upload_chunk, upload_finalize, upload_init};
use crate::upload::types::{UploadOptions, UploadProgress, UploadResult};
use std::io::Read;
use std::path::Path;

/// Upload data from a reader
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `options` - Upload options
/// * `reader` - The reader to read data from
///
/// # Returns
///
/// The result of the upload operation
pub async fn upload_reader<R>(
    node: &CodexNode,
    options: UploadOptions,
    mut reader: R,
) -> Result<UploadResult>
where
    R: Read,
{
    options.validate()?;

    let start_time = std::time::Instant::now();
    let chunk_size = options.chunk_size.unwrap_or(1024 * 1024); // 1 MB default

    // Initialize the upload
    let session_id = upload_init(node, &options).await?;

    // Read and upload chunks
    let mut buffer = vec![0u8; chunk_size];
    let mut total_bytes = 0;
    let mut chunk_count = 0;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                total_bytes += n;
                chunk_count += 1;

                // Upload the chunk
                upload_chunk(node, &session_id, &buffer[..n]).await?;

                // Call progress callback if provided
                if let Some(ref callback) = options.on_progress {
                    let progress = UploadProgress::new_chunked(
                        total_bytes,
                        None, // Unknown total size
                        chunk_count,
                        chunk_count, // Current chunk is also the total so far
                    );
                    callback(progress);
                }
            }
            Err(e) => {
                // Cancel the upload on error
                let _ = upload_cancel(node, &session_id).await;
                return Err(CodexError::from(e));
            }
        }
    }

    // Finalize the upload
    let cid = upload_finalize(node, &session_id).await?;

    let duration = start_time.elapsed();

    Ok(UploadResult::new(cid, total_bytes)
        .chunks(chunk_count)
        .duration_ms(duration.as_millis() as u64)
        .verified(options.verify))
}

/// Upload a file
///
/// # Arguments
///
/// * `node` - The Codex node
/// * `options` - Upload options (must have filepath set)
///
/// # Returns
///
/// The result of the upload operation
pub async fn upload_file(node: &CodexNode, mut options: UploadOptions) -> Result<UploadResult> {
    if options.filepath.is_none() {
        return Err(CodexError::invalid_parameter(
            "filepath",
            "File path must be specified for file upload",
        ));
    }

    let filepath = options.filepath.as_ref().unwrap();

    if !Path::new(filepath).exists() {
        return Err(CodexError::invalid_parameter(
            "filepath",
            format!("File does not exist: {}", filepath.display()),
        ));
    }

    // Get file size for progress tracking
    let file_size = std::fs::metadata(filepath)?.len() as usize;

    // If no total bytes is set in the progress callback, use the file size
    if options.on_progress.is_some() {
        let file_size = file_size;
        let original_callback = options.on_progress.take().unwrap();
        options.on_progress = Some(Box::new(move |mut progress: UploadProgress| {
            if progress.total_bytes.is_none() {
                progress.total_bytes = Some(file_size);
                progress.percentage = progress.bytes_uploaded as f64 / file_size as f64;
            }
            original_callback(progress);
        }));
    }

    // Open the file
    let file = std::fs::File::open(filepath)?;

    // Upload the file
    upload_reader(node, options, file).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;
    use crate::upload::types::UploadStrategy;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_upload_reader() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let data = b"Hello, world!";
        let reader = Cursor::new(data);
        let options = UploadOptions::new().chunk_size(5);

        let result = upload_reader(&node, options, reader).await;
        assert!(result.is_ok());

        let upload_result = result.unwrap();
        assert_eq!(upload_result.size, data.len());
        assert_eq!(upload_result.chunks, Some(3)); // 5 + 5 + 3

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_upload_file_no_filepath() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = UploadOptions::new(); // No filepath set
        let result = upload_file(&node, options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "filepath");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[tokio::test]
    async fn test_upload_file_nonexistent() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let options = UploadOptions::new().filepath("/nonexistent/file.txt");
        let result = upload_file(&node, options).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "filepath");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_upload_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = std::sync::Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let callback = move |progress: UploadProgress| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            println!(
                "Upload progress: {} bytes, {}%",
                progress.bytes_uploaded,
                progress.percentage * 100.0
            );
        };

        // Test the callback with some progress
        let progress = UploadProgress::new(500, Some(1000));
        callback(progress);

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_upload_result_creation() {
        let result = UploadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true);

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
    }

    #[test]
    fn test_upload_strategy_serialization() {
        let strategy = UploadStrategy::Chunked;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: UploadStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_upload_options_default() {
        let options = UploadOptions::default();

        assert_eq!(options.filepath, None);
        assert_eq!(options.chunk_size, Some(1024 * 1024));
        assert_eq!(options.strategy, UploadStrategy::Auto);
        assert!(options.on_progress.is_none());
        assert!(options.verify);
        assert!(options.metadata.is_none());
        assert_eq!(options.timeout, Some(300));
    }

    #[test]
    fn test_upload_progress_percentage() {
        // Test with known total
        let progress = UploadProgress::new(500, Some(1000));
        assert_eq!(progress.percentage, 0.5);

        // Test with zero total
        let progress = UploadProgress::new(500, Some(0));
        assert_eq!(progress.percentage, 0.0);

        // Test with unknown total
        let progress = UploadProgress::new(500, None);
        assert_eq!(progress.percentage, 0.0);

        // Test with bytes exceeding total (should cap at 1.0)
        let progress = UploadProgress::new(1500, Some(1000));
        assert_eq!(progress.percentage, 1.0);
    }

    #[test]
    fn test_upload_progress_chunked() {
        let progress = UploadProgress::new_chunked(250, Some(1000), 1, 4);

        assert_eq!(progress.bytes_uploaded, 250);
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.percentage, 0.25);
        assert_eq!(progress.current_chunk, Some(1));
        assert_eq!(progress.total_chunks, Some(4));
    }
}
