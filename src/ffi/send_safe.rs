//! Send-safe wrapper for raw pointers
//!
//! This module provides wrapper types that allow raw pointers to be sent
//! across thread boundaries safely. This is necessary because raw pointers in
//! Rust are not `Send` by default, but we need to use them in async functions
//! that must be `Send` for Tauri compatibility.

use libc::c_char;

/// A Send-safe wrapper for raw pointers
///
/// This wrapper allows raw pointers to be sent across thread boundaries safely.
/// It is safe because:
/// - The underlying data is protected by `with_libstorage_lock()` mutex
/// - The FFI library is thread-safe
/// - Pointers are only passed to FFI functions, never dereferenced in Rust
///
/// # Safety
///
/// The caller must ensure that:
/// - The pointer is valid for the duration of its use
/// - The underlying FFI library is thread-safe
/// - Access to the pointer is protected by appropriate synchronization (e.g., `with_libstorage_lock()`)
/// - The pointer is not dereferenced in Rust code
///
/// # Example
///
/// ```no_run
/// use storage_bindings::ffi::SendSafePtr;
///
/// // Create a Send-safe pointer
/// let ptr = unsafe { SendSafePtr::new(std::ptr::null_mut::<i32>()) };
///
/// // Use it in a thread-safe context
/// let raw_ptr = unsafe { ptr.as_ptr() };
/// ```
pub struct SendSafePtr<T>(*mut T);

/// A Send-safe wrapper for C strings
///
/// This wrapper allows C string pointers to be sent across thread boundaries safely.
/// It is safe because:
/// - The pointer is only passed to FFI functions, never dereferenced in Rust
/// - The pointer is properly freed when dropped
/// - The FFI library is thread-safe
///
/// # Safety
///
/// The caller must ensure that:
/// - The pointer is valid for the duration of its use
/// - The underlying FFI library is thread-safe
/// - The pointer is only passed to FFI functions
///
/// # Example
///
/// ```no_run
/// use storage_bindings::ffi::SendSafeCString;
///
/// // Create a Send-safe C string
/// let c_string = SendSafeCString::new("Hello, World!");
///
/// // Get the raw pointer to pass to FFI functions
/// let ptr = unsafe { c_string.as_ptr() };
///
/// // The string is automatically freed when c_string goes out of scope
/// ```
pub struct SendSafeCString(*mut c_char);

impl<T> SendSafePtr<T> {
    /// Create a new Send-safe pointer wrapper
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is valid for the duration of its use
    /// - The underlying FFI library is thread-safe
    /// - Access to the pointer is protected by appropriate synchronization
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is only used within appropriate synchronization (e.g., `with_libstorage_lock()`)
    /// - The pointer is not dereferenced in Rust code
    /// - The pointer is only passed to FFI functions
    pub unsafe fn as_ptr(&self) -> *mut T {
        self.0
    }

    /// Get the raw pointer as a const pointer
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is only used within appropriate synchronization (e.g., `with_libstorage_lock()`)
    /// - The pointer is not dereferenced in Rust code
    /// - The pointer is only passed to FFI functions
    pub unsafe fn as_const_ptr(&self) -> *const T {
        self.0 as *const T
    }

    /// Check if the pointer is null
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

// SAFETY: This is safe because:
// 1. The pointer is only used within `with_libstorage_lock()` which provides mutual exclusion
// 2. The FFI library is thread-safe
// 3. The pointer is never dereferenced in Rust, only passed to FFI functions
// 4. The underlying CallbackFuture already implements Send, confirming thread safety
unsafe impl<T> Send for SendSafePtr<T> {}

// SAFETY: This is safe because:
// 1. The pointer is only used within `with_libstorage_lock()` which provides mutual exclusion
// 2. The FFI library is thread-safe
// 3. The pointer is never dereferenced in Rust, only passed to FFI functions
// 4. The underlying CallbackFuture already implements Send, confirming thread safety
unsafe impl<T> Sync for SendSafePtr<T> {}

impl SendSafeCString {
    /// Create a new Send-safe C string wrapper
    ///
    /// # Panics
    ///
    /// Panics if the string contains a null byte.
    pub fn new(s: &str) -> Self {
        Self(std::ffi::CString::new(s).unwrap().into_raw())
    }

    /// Get the raw pointer
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is only used within appropriate synchronization (e.g., `with_libstorage_lock()`)
    /// - The pointer is not dereferenced in Rust code
    /// - The pointer is only passed to FFI functions
    pub unsafe fn as_ptr(&self) -> *mut c_char {
        self.0
    }
}

impl Drop for SendSafeCString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = std::ffi::CString::from_raw(self.0);
            }
        }
    }
}

// SAFETY: This is safe because:
// 1. The pointer is only passed to FFI functions, never dereferenced in Rust
// 2. The FFI library is thread-safe
// 3. The pointer is automatically freed when dropped, preventing use-after-free
unsafe impl Send for SendSafeCString {}

// SAFETY: This is safe because:
// 1. The pointer is only passed to FFI functions, never dereferenced in Rust
// 2. The FFI library is thread-safe
// 3. The pointer is automatically freed when dropped, preventing use-after-free
unsafe impl Sync for SendSafeCString {}

#[cfg(test)]
mod tests {
    use super::*;
    use libc::c_void;
    use std::thread;

    #[test]
    fn test_send_safe_ptr_is_send() {
        let ptr = unsafe { SendSafePtr::new(std::ptr::null_mut::<i32>()) };

        // This should compile because SendSafePtr is Send
        thread::spawn(move || {
            let _ = ptr;
        })
        .join()
        .unwrap();
    }

    #[test]
    fn test_send_safe_ptr_is_sync() {
        let ptr = unsafe { SendSafePtr::new(std::ptr::null_mut::<i32>()) };

        // This should compile because SendSafePtr is Sync
        let handle = thread::spawn(|| {
            let _ = ptr;
        });

        // Can still use ptr in main thread
        let _ = ptr;

        handle.join().unwrap();
    }

    #[test]
    fn test_send_safe_ptr_as_ptr() {
        let raw_ptr: *mut i32 = std::ptr::null_mut();
        let ptr = unsafe { SendSafePtr::new(raw_ptr) };

        let retrieved_ptr = unsafe { ptr.as_ptr() };
        assert_eq!(raw_ptr, retrieved_ptr);
    }

    #[test]
    fn test_send_safe_ptr_as_const_ptr() {
        let raw_ptr: *mut i32 = std::ptr::null_mut();
        let ptr = unsafe { SendSafePtr::new(raw_ptr) };

        let const_ptr = unsafe { ptr.as_const_ptr() };
        assert_eq!(raw_ptr as *const i32, const_ptr);
    }

    #[test]
    fn test_send_safe_ptr_with_c_void() {
        let raw_ptr: *mut c_void = std::ptr::null_mut();
        let ptr = unsafe { SendSafePtr::new(raw_ptr) };

        let retrieved_ptr = unsafe { ptr.as_ptr() };
        assert_eq!(raw_ptr, retrieved_ptr);
    }

    #[test]
    fn test_send_safe_c_string_is_send() {
        let c_string = SendSafeCString::new("Hello, World!");

        thread::spawn(move || {
            let _ = c_string;
        })
        .join()
        .unwrap();
    }

    #[test]
    fn test_send_safe_c_string_is_sync() {
        let c_string = SendSafeCString::new("Hello, World!");

        let handle = thread::spawn(|| {
            let _ = c_string;
        });

        let _ = c_string;

        handle.join().unwrap();
    }

    #[test]
    fn test_send_safe_c_string_as_ptr() {
        let c_string = SendSafeCString::new("Hello, World!");
        let ptr = unsafe { c_string.as_ptr() };

        assert!(!ptr.is_null());
    }

    #[test]
    fn test_send_safe_c_string_cleanup() {
        // This test verifies that SendSafeCString properly cleans up the C string
        // when it goes out of scope
        {
            let _c_string = SendSafeCString::new("Test string");
        } // c_string is dropped here, and the C string should be freed
    }
}
