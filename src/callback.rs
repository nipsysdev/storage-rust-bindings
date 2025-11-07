//! Callback mechanism for handling async operations
//!
//! This module provides a safe way to handle callbacks from the C library
//! and convert them to Rust futures or streams.

use crate::error::{CodexError, Result};
use crate::ffi::{c_str_to_string, CallbackReturn};
use libc::{c_char, c_int, c_void, size_t};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

// Global registry for callback contexts
static CALLBACK_REGISTRY: LazyLock<Mutex<HashMap<u64, Arc<CallbackContext>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// A callback context that manages the state of an async operation
pub struct CallbackContext {
    /// The result of the operation
    result: Mutex<Option<Result<String>>>,
    /// The waker to notify when the operation completes
    waker: Mutex<Option<Waker>>,
    /// Progress callback
    progress_callback: Mutex<Option<Box<dyn Fn(usize, Option<&[u8]>) + Send>>>,
    /// Whether the operation has completed
    completed: Mutex<bool>,
    /// The unique ID for this context
    id: u64,
}

impl CallbackContext {
    /// Create a new callback context
    pub fn new() -> Self {
        let id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        Self {
            result: Mutex::new(None),
            waker: Mutex::new(None),
            progress_callback: Mutex::new(None),
            completed: Mutex::new(false),
            id,
        }
    }

    /// Get the unique ID for this context
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Set the progress callback
    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(usize, Option<&[u8]>) + Send + 'static,
    {
        *self.progress_callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Set the waker for this context
    pub fn set_waker(&self, waker: Waker) {
        *self.waker.lock().unwrap() = Some(waker);
    }

    /// Get the result if available
    pub fn get_result(&self) -> Option<Result<String>> {
        match &*self.result.lock().unwrap() {
            Some(Ok(s)) => Some(Ok(s.clone())),
            Some(Err(e)) => Some(Err(e.clone())),
            None => None,
        }
    }

    /// Handle a callback from the C library
    ///
    /// # Safety
    ///
    /// This function dereferences raw pointers from the C library.
    /// The caller must ensure that the pointers are valid.
    pub unsafe fn handle_callback(&self, ret: i32, msg: *mut c_char, len: size_t) {
        match CallbackReturn::from(ret) {
            CallbackReturn::Ok => {
                let message = unsafe {
                    if msg.is_null() {
                        String::new()
                    } else {
                        c_str_to_string(msg).unwrap_or_else(|_| String::new())
                    }
                };

                *self.result.lock().unwrap() = Some(Ok(message));
                *self.completed.lock().unwrap() = true;

                if let Some(waker) = self.waker.lock().unwrap().take() {
                    waker.wake();
                }
            }
            CallbackReturn::Error => {
                let message = unsafe {
                    if msg.is_null() {
                        "Unknown error".to_string()
                    } else {
                        c_str_to_string(msg)
                            .unwrap_or_else(|_| "Invalid UTF-8 in error message".to_string())
                    }
                };

                *self.result.lock().unwrap() = Some(Err(CodexError::library_error(message)));
                *self.completed.lock().unwrap() = true;

                if let Some(waker) = self.waker.lock().unwrap().take() {
                    waker.wake();
                }
            }
            CallbackReturn::Progress => {
                let len = len;

                let chunk = if !msg.is_null() {
                    unsafe { Some(std::slice::from_raw_parts(msg as *const u8, len)) }
                } else {
                    None
                };

                if let Some(callback) = self.progress_callback.lock().unwrap().as_ref() {
                    callback(len, chunk);
                }
            }
        }
    }

    /// Wait for the operation to complete synchronously
    pub fn wait(&self) -> Result<String> {
        // Wait for completion with a timeout
        for _ in 0..600 {
            // 60 seconds timeout
            {
                let completed = self.completed.lock().unwrap();
                if *completed {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        // Check if we have a result
        if let Some(result) = self.get_result() {
            result
        } else {
            Err(CodexError::timeout("callback operation"))
        }
    }
}

impl Drop for CallbackContext {
    fn drop(&mut self) {
        // Remove from registry when dropped
        let mut registry = CALLBACK_REGISTRY.lock().unwrap();
        registry.remove(&self.id);
    }
}

/// A future that represents an async operation with callbacks
pub struct CallbackFuture {
    pub(crate) context: Arc<CallbackContext>,
}

impl CallbackFuture {
    /// Create a new callback future
    pub fn new() -> Self {
        let context = Arc::new(CallbackContext::new());

        // Register the context in the global registry
        {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry.insert(context.id(), context.clone());
        }

        Self { context }
    }

    /// Get a pointer to the callback ID for passing to C
    pub fn context_ptr(&self) -> *const c_void {
        // Return the ID as a pointer value (not a real pointer)
        self.context.id() as *const c_void
    }

    /// Set the progress callback
    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(usize, Option<&[u8]>) + Send + 'static,
    {
        self.context.set_progress_callback(callback);
    }

    /// Wait for the operation to complete synchronously
    pub fn wait(&self) -> Result<String> {
        self.context.wait()
    }
}

impl std::future::Future for CallbackFuture {
    type Output = Result<String>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Set the waker if not already set
        self.context.set_waker(cx.waker().clone());

        // Check if we have a result
        if let Some(result) = self.context.get_result() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}

/// C callback function that forwards to the appropriate Rust callback context
#[no_mangle]
pub unsafe extern "C" fn c_callback(ret: c_int, msg: *mut c_char, len: size_t, resp: *mut c_void) {
    if resp.is_null() {
        return;
    }

    // The resp parameter is now the callback ID (not a pointer)
    let callback_id = resp as u64;

    // Look up the context in the global registry
    let context = {
        let registry = CALLBACK_REGISTRY.lock().unwrap();
        registry.get(&callback_id).cloned()
    };

    if let Some(context) = context {
        // Handle the callback
        unsafe {
            context.handle_callback(ret, msg, len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::Wake;

    struct TestWaker {
        woken: AtomicBool,
    }

    impl Wake for TestWaker {
        fn wake(self: Arc<Self>) {
            self.woken.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_callback_context_creation() {
        let context = CallbackContext::new();

        // Initially no result
        assert!(context.get_result().is_none());
    }

    #[test]
    fn test_callback_context_success() {
        let context = CallbackContext::new();

        // Test success callback
        unsafe {
            context.handle_callback(0, std::ptr::null_mut(), 0);
        }
        let result = context.get_result().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_callback_context_error() {
        let context = CallbackContext::new();

        // Test error callback
        unsafe {
            context.handle_callback(1, std::ptr::null_mut(), 0);
        }
        let result = context.get_result().unwrap();
        assert!(result.is_err());

        match result {
            Err(CodexError::LibraryError { message, .. }) => {
                assert_eq!(message, "Unknown error");
            }
            _ => panic!("Expected LibraryError, got: {:?}", result),
        }
    }

    #[test]
    fn test_callback_context_progress() {
        let context = CallbackContext::new();
        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        // Set progress callback
        context.set_progress_callback(move |_len, _chunk| {
            progress_called_clone.store(true, Ordering::SeqCst);
        });

        // Trigger progress callback
        let test_data = b"test data";
        unsafe {
            context.handle_callback(3, test_data.as_ptr() as *mut c_char, test_data.len());
        }

        // Progress callback should have been called synchronously
        assert!(progress_called.load(Ordering::SeqCst));

        // No result should be available yet
        assert!(context.get_result().is_none());
    }

    #[test]
    fn test_callback_future_creation() {
        let future = CallbackFuture::new();

        // Should be able to get context pointer
        let ptr = future.context_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_callback_future_progress_callback() {
        let future = CallbackFuture::new();
        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        // Set progress callback
        future.set_progress_callback(move |_len, _chunk| {
            progress_called_clone.store(true, Ordering::SeqCst);
        });

        // Trigger progress callback through context
        let test_data = b"test data";
        unsafe {
            future
                .context
                .handle_callback(3, test_data.as_ptr() as *mut c_char, test_data.len());
        }

        // Progress callback should have been called synchronously
        assert!(progress_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_callback_future_success() {
        let future = CallbackFuture::new();

        // Simulate successful callback
        unsafe {
            future.context.handle_callback(0, std::ptr::null_mut(), 0);
        }

        // Future should complete successfully
        let result = future.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_callback_future_error() {
        let future = CallbackFuture::new();

        // Simulate error callback
        unsafe {
            future.context.handle_callback(1, std::ptr::null_mut(), 0);
        }

        // Future should complete with error
        let result = future.await;
        assert!(result.is_err());
    }

    #[test]
    fn test_callback_wait_timeout() {
        let context = CallbackContext::new();

        // Wait without setting any result - should timeout
        let result = context.wait();
        assert!(result.is_err());

        match result {
            Err(CodexError::Timeout { .. }) => {
                // Expected
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_callback_wait_success() {
        let context = CallbackContext::new();

        // Set a result first
        unsafe {
            context.handle_callback(0, std::ptr::null_mut(), 0);
        }

        // Wait should return immediately with the result
        let result = context.wait();
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_callback_null_context() {
        // Test that c_callback handles null context gracefully
        unsafe {
            c_callback(0, std::ptr::null_mut(), 0, std::ptr::null_mut());
        }
        // Should not crash
    }

    #[test]
    fn test_c_callback_with_valid_context() {
        let future = CallbackFuture::new();
        let context_id = future.context.id();
        let context_ptr = context_id as *mut c_void;

        // Test success callback
        unsafe {
            c_callback(0, std::ptr::null_mut(), 0, context_ptr);
        }

        // Check that the callback was handled
        let result = future.context.get_result();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());
    }
}
