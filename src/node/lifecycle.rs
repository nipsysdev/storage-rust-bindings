use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{
    free_c_string, storage_close, storage_destroy, storage_new, storage_peer_id, storage_repo,
    storage_revision, storage_spr, storage_start, storage_stop, storage_version,
    string_to_c_string,
};
use crate::node::config::StorageConfig;
use libc::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StorageNode {
    inner: Arc<Mutex<StorageNodeInner>>,
}

struct StorageNodeInner {
    ctx: *mut c_void,
    started: bool,
}

unsafe impl Send for StorageNodeInner {}
unsafe impl Sync for StorageNodeInner {}

unsafe impl Send for StorageNode {}
unsafe impl Sync for StorageNode {}

impl StorageNode {
    pub fn new(config: StorageConfig) -> Result<Self> {
        with_libstorage_lock(|| {
            let json_config = config.to_json()?;
            let c_json_config = string_to_c_string(&json_config);

            let future = CallbackFuture::new();

            let node_ctx = unsafe {
                let node_ctx = storage_new(
                    c_json_config,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                );

                free_c_string(c_json_config);

                if node_ctx.is_null() {
                    return Err(StorageError::node_error("new", "Failed to create node"));
                }

                node_ctx
            };

            let _result = future.wait()?;

            Ok(StorageNode {
                inner: Arc::new(Mutex::new(StorageNodeInner {
                    ctx: node_ctx,
                    started: false,
                })),
            })
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.started {
            return Err(StorageError::node_error("start", "Node is already started"));
        }

        let future = CallbackFuture::new();

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

    pub async fn start_async(&self) -> Result<()> {
        let node = self.clone();

        tokio::task::spawn_blocking(move || {
            {
                let inner = node.inner.lock().unwrap();
                if inner.started {
                    return Err(StorageError::node_error(
                        "start_async_send",
                        "Node is already started",
                    ));
                }
            }

            let future = CallbackFuture::new();

            let ctx = {
                let inner = node.inner.lock().unwrap();
                inner.ctx as *mut _
            };

            let result = unsafe {
                storage_start(ctx, Some(c_callback), future.context_ptr() as *mut c_void)
            };

            if result != 0 {
                return Err(StorageError::node_error(
                    "start_async_send",
                    "Failed to start node",
                ));
            }

            let _result = future.wait()?;

            {
                let mut inner = node.inner.lock().unwrap();
                inner.started = true;
            }

            Ok(())
        })
        .await?
    }

    pub fn stop(&mut self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.started {
            return Err(StorageError::node_error("stop", "Node is not started"));
        }

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_stop(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("stop", "Failed to stop node"));
        }

        inner.started = false;
        Ok(())
    }

    pub async fn stop_async(&self) -> Result<()> {
        let node = self.clone();

        tokio::task::spawn_blocking(move || {
            {
                let inner = node.inner.lock().unwrap();
                if !inner.started {
                    return Err(StorageError::node_error(
                        "stop_async_send",
                        "Node is not started",
                    ));
                }
            }

            let future = CallbackFuture::new();

            let ctx = {
                let inner = node.inner.lock().unwrap();
                inner.ctx as *mut _
            };

            let result =
                unsafe { storage_stop(ctx, Some(c_callback), future.context_ptr() as *mut c_void) };

            if result != 0 {
                return Err(StorageError::node_error(
                    "stop_async_send",
                    "Failed to stop node",
                ));
            }

            let _result = future.wait()?;

            {
                let mut inner = node.inner.lock().unwrap();
                inner.started = false;
            }

            Ok(())
        })
        .await?
    }

    pub fn destroy(self) -> Result<()> {
        if Arc::strong_count(&self.inner) != 1 {
            return Err(StorageError::node_error(
                "destroy",
                "Cannot destroy: multiple references exist",
            ));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.started {
            return Err(StorageError::node_error("destroy", "Node is still started"));
        }

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_close(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("destroy", "Failed to close node"));
        }

        future.wait()?;

        unsafe { storage_destroy(inner.ctx as *mut _, None, ptr::null_mut()) };

        inner.ctx = ptr::null_mut();
        Ok(())
    }

    pub fn version(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_version(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("version", "Failed to get version"));
        }

        let version = future.wait()?;

        Ok(version)
    }

    pub fn revision(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_revision(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error(
                "revision",
                "Failed to get revision",
            ));
        }

        let revision = future.wait()?;

        Ok(revision)
    }

    pub fn repo(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_repo(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("repo", "Failed to get repo path"));
        }

        let repo = future.wait()?;

        Ok(repo)
    }

    pub fn spr(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_spr(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("spr", "Failed to get SPR"));
        }

        let spr = future.wait()?;

        Ok(spr)
    }

    pub fn peer_id(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        let future = CallbackFuture::new();

        let result = unsafe {
            storage_peer_id(
                inner.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(StorageError::node_error("peer_id", "Failed to get peer ID"));
        }

        let peer_id = future.wait()?;

        Ok(peer_id)
    }

    pub fn is_started(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.started
    }

    #[allow(dead_code)]
    pub(crate) fn ctx(&self) -> *mut c_void {
        let inner = self.inner.lock().unwrap();
        inner.ctx
    }

    pub(crate) fn with_ctx<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut c_void) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(inner.ctx)
    }

    pub(crate) fn with_ctx_locked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut c_void) -> R,
    {
        with_libstorage_lock(|| {
            let inner = self.inner.lock().unwrap();
            f(inner.ctx)
        })
    }
}

impl Drop for StorageNode {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            let mut inner = self.inner.lock().unwrap();
            if !inner.ctx.is_null() && inner.started {
                let _ = unsafe {
                    storage_stop(inner.ctx as *mut _, None, ptr::null_mut());
                };
                inner.started = false;
            }

            if !inner.ctx.is_null() {
                let _ = unsafe {
                    storage_destroy(inner.ctx as *mut _, None, ptr::null_mut());
                };
                inner.ctx = ptr::null_mut();
            }
        }
    }
}
