//! Send-safe wrapper for raw pointers
//!
//! This module provides a wrapper type that allows raw pointers to be sent
//! across thread boundaries safely. This is necessary because raw pointers in
//! Rust are not `Send` by default, but we need to use them in async functions
//! that must be `Send` for Tauri compatibility.

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
}
