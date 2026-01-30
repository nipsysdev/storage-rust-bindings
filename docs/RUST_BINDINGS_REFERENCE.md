# Rust Bindings Reference Documentation

This document provides comprehensive guidance for Rust developers on how to safely use the `libstorage` FFI bindings. It covers type mappings, memory safety rules, usage patterns, and concurrency considerations.

## Table of Contents

1. [Type Mappings](#type-mappings)
2. [Memory Safety Rules](#memory-safety-rules)
3. [Usage Pattern / Lifecycle](#usage-pattern--lifecycle)
4. [Concurrency Model](#concurrency-model)
5. [Error Handling](#error-handling)
6. [Best Practices](#best-practices)

---

## Type Mappings

### Opaque Pointers

The Nim-based `libstorage` library uses opaque pointers to represent complex objects. These are wrapped in Rust structs for type safety.

#### Storage Node

```rust
// Internal representation
struct StorageNodeInner {
    ctx: *mut c_void,  // Opaque pointer to Nim storage context
    started: bool,
}

// Public wrapper
pub struct StorageNode {
    inner: Arc<Mutex<StorageNodeInner>>,
}
```

**Key Points:**

- The `ctx` field is an opaque pointer (`*mut c_void`) to the Nim storage context
- Never access `ctx` directly - use the provided methods
- The pointer is managed internally and automatically cleaned up

### String Conversions

Strings are converted between Nim (GC-managed) and Rust (owned) using helper functions.

#### Rust to C String

```rust
use crate::ffi::string_to_c_string;

// Convert a Rust string to a C string
let rust_str = "Hello, Storage!";
let c_str = string_to_c_string(rust_str);

// c_str is now a *mut c_char that can be passed to C functions
// IMPORTANT: Must be freed with free_c_string() when done
```

#### C String to Rust String

```rust
use crate::ffi::c_str_to_string;

// Convert a C string back to Rust
unsafe {
    let rust_str = c_str_to_string(c_str_ptr)?;
    // rust_str is now a String
}
```

**Memory Ownership:**

- **Rust → C**: The C string is allocated by Rust and must be freed by Rust
- **C → Rust**: The C string is borrowed and converted to an owned Rust String
- Always call `free_c_string()` on strings created with `string_to_c_string()`

### Arrays and Binary Data

Large binary data and CIDs are handled through callback mechanisms with progress tracking.

#### Progress Callbacks

```rust
// Progress callback receives chunks of data
type ProgressCallback = Box<dyn Fn(usize, Option<&[u8]>) + Send>;

// Example: Upload progress tracking
let upload_options = UploadOptions::new()
    .on_progress(|progress| {
        println!("Uploaded {} bytes", progress.bytes_uploaded);
    });
```

**Data Flow:**

1. Binary data is split into chunks
2. Each chunk is passed to the progress callback
3. The callback receives the chunk size and optional byte slice
4. Progress is tracked without copying large amounts of data

---

## Memory Safety Rules

### Memory Ownership

#### Who Owns What?

| Resource             | Owner                   | Cleanup Method                   |
| -------------------- | ----------------------- | -------------------------------- |
| Storage Node Context | Rust (`StorageNode`)    | Automatic via `Drop` trait       |
| C Strings (Rust→C)   | Rust                    | Must call `free_c_string()`      |
| C Strings (C→Rust)   | Rust                    | Automatic conversion to `String` |
| Callback Context     | Rust (`CallbackFuture`) | Automatic via `Drop` trait       |

#### Manual Memory Management

**C Strings Created by Rust:**

```rust
// ✅ CORRECT: Always free C strings created by Rust
let c_str = string_to_c_string("test");
// ... use c_str ...
unsafe { free_c_string(c_str); }

// ❌ WRONG: Forgetting to free causes memory leak
let c_str = string_to_c_string("test");
// ... use c_str ...
// Memory leak! c_str is never freed
```

**Null Pointer Safety:**

```rust
// ✅ CORRECT: Check for null pointers
unsafe {
    if !ptr.is_null() {
        let s = c_str_to_string(ptr)?;
    }
}

// ✅ CORRECT: Helper functions handle null pointers safely
unsafe {
    let s = c_str_to_string(ptr)?;  // Returns empty string if null
}
```

### Unsafe Blocks

The bindings use `unsafe` blocks to interact with C code. These are safe because:

1. **Pointer Validity**: All pointers are checked for null before use
2. **Lifetime Management**: Pointers are only used within their valid lifetime
3. **Thread Safety**: Global mutex prevents concurrent access to libstorage
4. **Error Handling**: All C return codes are checked and converted to Rust errors

**Example of Safe Unsafe Code:**

```rust
pub fn start(&mut self) -> Result<()> {
    let mut inner = self.inner.lock().unwrap();
    if inner.started {
        return Err(StorageError::node_error("start", "Node is already started"));
    }

    let future = CallbackFuture::new();

    // SAFE: inner.ctx is guaranteed to be valid and non-null
    // SAFE: Callback is registered and will be called exactly once
    let result = unsafe {
        storage_start(
            inner.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(StorageError::node_error("start", "Failed to start node"));
    }

    let _result = future.wait()?;
    inner.started = true;
    Ok(())
}
```

### Automatic Cleanup

The `Drop` trait ensures resources are cleaned up automatically:

```rust
impl Drop for StorageNode {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            let mut inner = self.inner.lock().unwrap();

            // Stop the node if it's running
            if !inner.ctx.is_null() && inner.started {
                unsafe {
                    storage_stop(inner.ctx as *mut _, None, ptr::null_mut());
                }
                inner.started = false;
            }

            // Destroy the node context
            if !inner.ctx.is_null() {
                unsafe {
                    storage_destroy(inner.ctx as *mut _, None, ptr::null_mut());
                }
                inner.ctx = ptr::null_mut();
            }
        }
    }
}
```

**Important:** Always call `destroy()` explicitly when you're done with a node to ensure proper cleanup, even though `Drop` provides a safety net.

---

## Usage Pattern / Lifecycle

### Complete Lifecycle Example

```rust
use storage_bindings::{
    download_stream, upload_file, LogLevel, StorageConfig, StorageNode,
    UploadOptions, DownloadStreamOptions,
};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ========================================
    // 1. Configuration
    // ========================================
    let config = StorageConfig::new()
        .log_level(LogLevel::Info)
        .data_dir("/path/to/storage/data")
        .storage_quota(100 * 1024 * 1024) // 100 MB
        .max_peers(50)
        .discovery_port(8090);

    // ========================================
    // 2. Node Creation
    // ========================================
    let mut node = StorageNode::new(config)?;
    println!("Node created successfully");

    // ========================================
    // 3. Node Startup
    // ========================================
    node.start()?;
    println!("Node started successfully");

    // Get node information
    println!("Version: {}", node.version()?);
    println!("Peer ID: {}", node.peer_id()?);

    // ========================================
    // 4. Upload File
    // ========================================
    let file_path = "/path/to/file.txt";
    let upload_options = UploadOptions::new()
        .filepath(file_path)
        .on_progress(|progress| {
            println!(
                "Upload: {} bytes ({}%)",
                progress.bytes_uploaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let upload_result = upload_file(&node, upload_options).await?;
    println!("File uploaded! CID: {}", upload_result.cid);

    // ========================================
    // 5. Download File
    // ========================================
    let download_path = "/path/to/downloaded.txt";
    let download_options = DownloadStreamOptions::new(&upload_result.cid)
        .filepath(download_path)
        .on_progress(|progress| {
            println!(
                "Download: {} bytes ({}%)",
                progress.bytes_downloaded,
                (progress.percentage * 100.0) as u32
            );
        });

    let download_result = download_stream(&node, &upload_result.cid, download_options).await?;
    println!("File downloaded! Size: {} bytes", download_result.size);

    // ========================================
    // 6. Node Shutdown
    // ========================================
    node.stop()?;
    println!("Node stopped");

    // ========================================
    // 7. Node Destruction
    // ========================================
    node.destroy()?;
    println!("Node destroyed");

    Ok(())
}
```

### Error Handling

All operations return `Result<T>` for proper error handling:

```rust
use storage_bindings::{StorageError, StorageNode, StorageConfig};

fn example() -> Result<(), StorageError> {
    // Create node with error handling
    let config = StorageConfig::new()
        .log_level(LogLevel::Info)
        .data_dir("/path/to/data");

    let mut node = match StorageNode::new(config) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to create node: {}", e);
            return Err(e);
        }
    };

    // Start node with error handling
    if let Err(e) = node.start() {
        eprintln!("Failed to start node: {}", e);
        // Attempt cleanup even on error
        let _ = node.destroy();
        return Err(e);
    }

    // Use the node...

    // Always cleanup
    let _ = node.stop();
    let _ = node.destroy();

    Ok(())
}
```

### Error Types

The `StorageError` enum provides detailed error information:

```rust
pub enum StorageError {
    LibraryError { message: String },
    NodeError { operation: String, message: String },
    UploadError { message: String },
    DownloadError { message: String },
    StorageError { operation: String, message: String },
    P2PError { message: String },
    ConfigError { message: String },
    InvalidParameter { parameter: String, message: String },
    Timeout { operation: String },
    Cancelled { operation: String },
    Io(std::io::Error),
    Json(serde_json::Error),
    Utf8(std::str::Utf8Error),
    NullPointer { context: String },
    JoinError(tokio::task::JoinError),
}
```

**Handling Specific Errors:**

```rust
match result {
    Ok(cid) => println!("Upload successful: {}", cid),
    Err(StorageError::UploadError { message }) => {
        eprintln!("Upload failed: {}", message);
    }
    Err(StorageError::Timeout { operation }) => {
        eprintln!("Operation timed out: {}", operation);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

---

## Concurrency Model

### Thread Safety

The `StorageNode` is designed to be thread-safe:

```rust
unsafe impl Send for StorageNode {}
unsafe impl Sync for StorageNode {}
```

**Implementation:**

```rust
pub struct StorageNode {
    inner: Arc<Mutex<StorageNodeInner>>,
}
```

- **`Arc`**: Allows multiple ownership across threads
- **`Mutex`**: Ensures exclusive access to the internal state
- **`Send`/`Sync`**: Safe to share across threads

### Sharing Across Threads

```rust
use std::sync::Arc;
use tokio::task::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = StorageConfig::new().log_level(LogLevel::Info);
    let node = StorageNode::new(config)?;
    node.start()?;

    // Wrap in Arc for sharing across threads
    let node = Arc::new(node);

    // Spawn multiple tasks that share the node
    let mut handles = vec![];

    for i in 0..5 {
        let node_clone = Arc::clone(&node);
        let handle = spawn(async move {
            // Each task can safely use the node
            let peer_id = node_clone.peer_id()?;
            println!("Task {}: Peer ID = {}", i, peer_id);
            Ok::<_, StorageError>(())
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await??;
    }

    node.stop()?;
    node.destroy()?;

    Ok(())
}
```

### Global Mutex

A global mutex ensures thread-safe access to the underlying C library:

```rust
static LIBSTORAGE_MUTEX: Mutex<()> = Mutex::new();

pub fn with_libstorage_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _lock = LIBSTORAGE_MUTEX.lock().unwrap();
    f()
}
```

**Why This Matters:**

- The Nim `libstorage` library is not thread-safe
- The global mutex prevents concurrent calls to C functions
- All FFI operations are wrapped in `with_libstorage_lock()`

### Async Operations

Async operations use `tokio::task::spawn_blocking` to avoid blocking the async runtime:

```rust
pub async fn upload_init(node: &StorageNode, options: &UploadOptions) -> Result<String> {
    let node = node.clone();
    let options = options.clone();

    tokio::task::spawn_blocking(move || {
        // This runs on a blocking thread pool
        // Safe to call blocking C functions here
        let future = CallbackFuture::new();

        let result = with_libstorage_lock(|| unsafe {
            node.with_ctx(|ctx| {
                // Call C function
                storage_upload_init(ctx, /* ... */)
            })
        });

        if result != 0 {
            return Err(StorageError::upload_error("Failed to initialize upload"));
        }

        let session_id = future.wait()?;
        Ok(session_id)
    })
    .await?
}
```

**Key Points:**

- Async functions are non-blocking from the caller's perspective
- Blocking C calls run on a dedicated thread pool
- The global mutex ensures only one C call at a time

### Concurrent Uploads/Downloads

Multiple concurrent operations are safe:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = StorageConfig::new().log_level(LogLevel::Info);
    let node = StorageNode::new(config)?;
    node.start()?;

    let node = Arc::new(node);

    // Upload multiple files concurrently
    let files = vec!["file1.txt", "file2.txt", "file3.txt"];
    let mut upload_tasks = vec![];

    for file in files {
        let node_clone = Arc::clone(&node);
        let task = spawn(async move {
            let options = UploadOptions::new().filepath(file);
            upload_file(&node_clone, options).await
        });
        upload_tasks.push(task);
    }

    // Wait for all uploads to complete
    for task in upload_tasks {
        let result = task.await??;
        println!("Uploaded: {}", result.cid);
    }

    node.stop()?;
    node.destroy()?;

    Ok(())
}
```

---

## Error Handling

### Checking Return Codes

All C function return codes are checked and converted to Rust errors:

```rust
// C function returns 0 on success, non-zero on error
let result = unsafe {
    storage_start(
        ctx,
        Some(c_callback),
        future.context_ptr() as *mut c_void,
    )
};

if result != 0 {
    return Err(StorageError::node_error("start", "Failed to start node"));
}
```

### Callback Error Handling

Errors from the C library are delivered through callbacks:

```rust
unsafe fn handle_callback(&self, ret: i32, msg: *const c_char, len: size_t) {
    match CallbackReturn::from(ret) {
        CallbackReturn::Ok => {
            // Success - store result
            let message = unsafe {
                if msg.is_null() {
                    String::new()
                } else {
                    c_str_to_string(msg).unwrap_or_else(|_| String::new())
                }
            };
            *self.result.lock().unwrap() = Some(Ok(message));
        }
        CallbackReturn::Error => {
            // Error - store error message
            let message = unsafe {
                if msg.is_null() {
                    "Unknown error".to_string()
                } else {
                    c_str_to_string(msg)
                        .unwrap_or_else(|_| "Invalid UTF-8 in error message".to_string())
                }
            };
            *self.result.lock().unwrap() = Some(Err(StorageError::library_error(message)));
        }
        CallbackReturn::Progress => {
            // Progress update - call progress callback
            // ...
        }
    }
}
```

### Timeout Handling

Operations have built-in timeout protection:

```rust
pub fn wait(&self) -> Result<String> {
    // Wait up to 60 seconds (600 * 100ms)
    for _ in 0..600 {
        {
            let completed = self.completed.lock().unwrap();
            if *completed {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    if let Some(result) = self.get_result() {
        result
    } else {
        Err(StorageError::timeout("callback operation"))
    }
}
```

---

## Best Practices

### 1. Always Explicitly Destroy Nodes

```rust
// ✅ GOOD: Explicit cleanup
let mut node = StorageNode::new(config)?;
node.start()?;
// ... use node ...
node.stop()?;
node.destroy()?;

// ⚠️ ACCEPTABLE: Rely on Drop (but not recommended)
{
    let mut node = StorageNode::new(config)?;
    node.start()?;
    // ... use node ...
    // Drop trait will clean up, but errors won't be reported
}
```

### 2. Handle Errors Properly

```rust
// ✅ GOOD: Proper error handling
match StorageNode::new(config) {
    Ok(node) => {
        if let Err(e) = node.start() {
            eprintln!("Failed to start: {}", e);
            let _ = node.destroy();
            return Err(e);
        }
        // ... use node ...
    }
    Err(e) => {
        eprintln!("Failed to create node: {}", e);
        return Err(e);
    }
}

// ❌ BAD: Ignoring errors
let node = StorageNode::new(config).unwrap();
node.start().unwrap();
// ... use node ...
```

### 3. Use Arc for Shared Nodes

```rust
// ✅ GOOD: Use Arc for sharing
let node = Arc::new(StorageNode::new(config)?);
let node_clone = Arc::clone(&node);
spawn(async move {
    // Use node_clone in another thread
});

// ❌ BAD: Trying to move node into multiple threads
let node = StorageNode::new(config)?;
spawn(async move {
    // Can't use node here - it was moved
});
```

### 4. Free C Strings Promptly

```rust
// ✅ GOOD: Free C strings immediately after use
let c_str = string_to_c_string("test");
unsafe {
    // ... use c_str ...
    free_c_string(c_str);
}

// ❌ BAD: Forgetting to free
let c_str = string_to_c_string("test");
unsafe {
    // ... use c_str ...
}
// Memory leak!
```

### 5. Use Progress Callbacks for Large Files

```rust
// ✅ GOOD: Track progress for large uploads
let options = UploadOptions::new()
    .filepath("/path/to/large/file.bin")
    .on_progress(|progress| {
        println!(
            "Progress: {}/{} bytes ({}%)",
            progress.bytes_uploaded,
            progress.total_bytes,
            (progress.percentage * 100.0) as u32
        );
    });

upload_file(&node, options).await?;
```

### 6. Validate Configuration

```rust
// ✅ GOOD: Validate configuration before use
let config = StorageConfig::new()
    .data_dir("/path/to/data")
    .storage_quota(100 * 1024 * 1024);

// Ensure data directory exists
if let Some(ref data_dir) = config.data_dir {
    std::fs::create_dir_all(data_dir)?;
}

let node = StorageNode::new(config)?;
```

### 7. Use Appropriate Log Levels

```rust
// ✅ GOOD: Use appropriate log levels for different environments
let config = if cfg!(debug_assertions) {
    StorageConfig::new()
        .log_level(LogLevel::Debug)
        .log_format(LogFormat::Colors)
} else {
    StorageConfig::new()
        .log_level(LogLevel::Info)
        .log_format(LogFormat::Json)
};
```

### 8. Handle Timeouts Gracefully

```rust
// ✅ GOOD: Handle timeouts with retry logic
async fn upload_with_retry(
    node: &StorageNode,
    options: UploadOptions,
    max_retries: u32,
) -> Result<UploadResult> {
    for attempt in 0..max_retries {
        match upload_file(node, options.clone()).await {
            Ok(result) => return Ok(result),
            Err(StorageError::Timeout { .. }) if attempt < max_retries - 1 => {
                eprintln!("Upload timed out, retrying ({}/{})", attempt + 1, max_retries);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(StorageError::timeout("upload"))
}
```

---

## Summary

### Key Takeaways

1. **Type Safety**: Opaque pointers are wrapped in Rust structs for type safety
2. **Memory Management**: C strings must be manually freed; other resources are auto-cleaned
3. **Thread Safety**: Nodes are `Send` and `Sync`, protected by `Arc<Mutex<>>`
4. **Async Support**: Use async functions for non-blocking operations
5. **Error Handling**: All operations return `Result<T>` for proper error handling
6. **Global Mutex**: Ensures thread-safe access to the underlying C library

### Quick Reference

```rust
// Create and start a node
let config = StorageConfig::new().log_level(LogLevel::Info);
let mut node = StorageNode::new(config)?;
node.start()?;

// Upload a file
let options = UploadOptions::new().filepath("file.txt");
let result = upload_file(&node, options).await?;

// Download a file
let options = DownloadStreamOptions::new(&result.cid).filepath("output.txt");
let result = download_stream(&node, &result.cid, options).await?;

// Cleanup
node.stop()?;
node.destroy()?;
```

For more examples, see the [integration tests](../tests/) directory.
