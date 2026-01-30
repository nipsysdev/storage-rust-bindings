use crate::error::{Result, StorageError};
use crate::ffi::{c_str_to_string, CallbackReturn};
use libc::{c_char, c_int, c_void, size_t};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

/// Type alias for the progress callback function type
type ProgressCallback = Box<dyn Fn(usize, Option<&[u8]>) + Send>;

static LIBSTORAGE_MUTEX: Mutex<()> = Mutex::new(());

static CALLBACK_REGISTRY: LazyLock<Mutex<HashMap<u64, Arc<CallbackContext>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_CALLBACK_ID: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(1));

pub struct CallbackContext {
    result: Mutex<Option<Result<String>>>,
    waker: Mutex<Option<Waker>>,
    progress_callback: Mutex<Option<ProgressCallback>>,
    completed: Mutex<bool>,
    id: u64,
}

impl Default for CallbackContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackContext {
    pub fn new() -> Self {
        let id = {
            let mut next_id = NEXT_CALLBACK_ID.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        Self {
            result: Mutex::new(None),
            waker: Mutex::new(None),
            progress_callback: Mutex::new(None),
            completed: Mutex::new(false),
            id,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(usize, Option<&[u8]>) + Send + 'static,
    {
        *self.progress_callback.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn set_waker(&self, waker: Waker) {
        *self.waker.lock().unwrap() = Some(waker);
    }

    pub fn get_result(&self) -> Option<Result<String>> {
        match &*self.result.lock().unwrap() {
            Some(Ok(s)) => Some(Ok(s.clone())),
            Some(Err(e)) => Some(Err(e.clone())),
            None => None,
        }
    }

    /// Handles a callback from the C library
    ///
    /// # Safety
    ///
    /// The `msg` pointer must be either:
    /// - A valid pointer to a null-terminated C string (for Ok/Error callbacks)
    /// - A valid pointer to a byte array of length `len` (for Progress callbacks)
    /// - A null pointer (which will be handled appropriately)
    ///
    /// The memory pointed to by `msg` must remain valid for the duration of this call.
    pub unsafe fn handle_callback(&self, ret: i32, msg: *const c_char, len: size_t) {
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

                *self.result.lock().unwrap() = Some(Err(StorageError::library_error(message)));
                *self.completed.lock().unwrap() = true;

                if let Some(waker) = self.waker.lock().unwrap().take() {
                    waker.wake();
                }
            }
            CallbackReturn::Progress => {
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
}

impl Drop for CallbackContext {
    fn drop(&mut self) {
        if let Ok(mut registry) = CALLBACK_REGISTRY.lock() {
            registry.remove(&self.id);
        }
    }
}

pub struct CallbackFuture {
    pub(crate) context: Arc<CallbackContext>,
}

impl Default for CallbackFuture {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackFuture {
    pub fn new() -> Self {
        let context = Arc::new(CallbackContext::new());

        {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry.insert(context.id(), context.clone());
        }

        Self { context }
    }

    pub fn context_ptr(&self) -> *const c_void {
        self.context.id() as *const c_void
    }

    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(usize, Option<&[u8]>) + Send + 'static,
    {
        self.context.set_progress_callback(callback);
    }

    /// Wait for the callback to complete with a timeout (async)
    ///
    /// This method provides a timeout wrapper around the Future implementation.
    /// If the callback does not complete within the specified duration, a timeout error is returned.
    ///
    /// # Arguments
    ///
    /// * `duration` - The maximum time to wait for the callback to complete
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage_bindings::callback::CallbackFuture;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let future = CallbackFuture::new();
    ///     let result = future.wait_with_timeout(Duration::from_secs(30)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn wait_with_timeout(self, duration: Duration) -> Result<String> {
        tokio::time::timeout(duration, self)
            .await
            .map_err(|_| StorageError::timeout("callback operation"))?
    }
}

impl std::future::Future for CallbackFuture {
    type Output = Result<String>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.context.set_waker(cx.waker().clone());

        if let Some(result) = self.context.get_result() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}

unsafe impl Send for CallbackFuture {}

pub fn with_libstorage_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _lock = LIBSTORAGE_MUTEX.lock().unwrap();
    f()
}

/// C callback function that is called from the libstorage library
///
/// # Safety
///
/// The `msg` pointer must be either:
/// - A valid pointer to a null-terminated C string (for Ok/Error callbacks)
/// - A valid pointer to a byte array of length `len` (for Progress callbacks)
/// - A null pointer (which will be handled appropriately)
///
/// The `resp` pointer must be either:
/// - A valid pointer to a u64 callback ID that was previously registered
/// - A null pointer (which will cause the function to return early)
///
/// The memory pointed to by `msg` and `resp` must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn c_callback(
    ret: c_int,
    msg: *const c_char,
    len: size_t,
    resp: *mut c_void,
) {
    if resp.is_null() {
        return;
    }

    let callback_id = resp as u64;

    let context = {
        if let Ok(registry) = CALLBACK_REGISTRY.lock() {
            registry.get(&callback_id).cloned()
        } else {
            None
        }
    };

    if let Some(context) = context {
        unsafe {
            context.handle_callback(ret, msg, len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_callback_context_creation() {
        let context = CallbackContext::new();
        assert!(context.get_result().is_none());
    }

    #[test]
    fn test_callback_context_success() {
        let context = CallbackContext::new();
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
        unsafe {
            context.handle_callback(1, std::ptr::null_mut(), 0);
        }
        let result = context.get_result().unwrap();
        assert!(result.is_err());

        match result {
            Err(StorageError::LibraryError { message, .. }) => {
                assert_eq!(message, "Unknown error");
            }
            other => {
                assert!(
                    matches!(other, Err(StorageError::LibraryError { .. })),
                    "Expected LibraryError with message 'Unknown error', got: {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn test_callback_context_progress() {
        let context = CallbackContext::new();
        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        context.set_progress_callback(move |_len, _chunk| {
            progress_called_clone.store(true, Ordering::SeqCst);
        });

        let test_data = b"test data";
        unsafe {
            context.handle_callback(3, test_data.as_ptr() as *mut c_char, test_data.len());
        }

        assert!(progress_called.load(Ordering::SeqCst));
        assert!(context.get_result().is_none());
    }

    #[test]
    fn test_callback_future_creation() {
        let future = CallbackFuture::new();
        let ptr = future.context_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_callback_future_progress_callback() {
        let future = CallbackFuture::new();
        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        future.set_progress_callback(move |_len, _chunk| {
            progress_called_clone.store(true, Ordering::SeqCst);
        });

        let test_data = b"test data";
        unsafe {
            future
                .context
                .handle_callback(3, test_data.as_ptr() as *mut c_char, test_data.len());
        }

        assert!(progress_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_callback_future_success() {
        let future = CallbackFuture::new();
        unsafe {
            future.context.handle_callback(0, std::ptr::null_mut(), 0);
        }
        let result = future.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_callback_future_error() {
        let future = CallbackFuture::new();
        unsafe {
            future.context.handle_callback(1, std::ptr::null_mut(), 0);
        }
        let result = future.await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_callback_wait_success() {
        let future = CallbackFuture::new();
        unsafe {
            future.context.handle_callback(0, std::ptr::null_mut(), 0);
        }
        let result = future.await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_callback_null_context() {
        unsafe {
            c_callback(0, std::ptr::null_mut(), 0, std::ptr::null_mut());
        }
    }

    #[test]
    fn test_c_callback_with_valid_context() {
        let future = CallbackFuture::new();
        let context_id = future.context.id();
        let context_ptr = context_id as *mut c_void;

        unsafe {
            c_callback(0, std::ptr::null_mut(), 0, context_ptr);
        }

        let result = future.context.get_result();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());
    }
}
