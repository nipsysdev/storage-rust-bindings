use std::sync::Arc;
use storage_bindings::{StorageConfig, StorageNode};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_thread_safe_node_creation() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    let node = StorageNode::new(config).await.unwrap();
    assert!(!node.is_started());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_thread_safe_node_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    let node = StorageNode::new(config).await.unwrap();

    node.start().await.unwrap();
    assert!(node.is_started());

    let version = node.version().await.unwrap();
    assert!(!version.is_empty());

    let peer_id = node.peer_id().await.unwrap();
    assert!(!peer_id.is_empty());

    node.stop().await.unwrap();
    assert!(!node.is_started());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_node_cloning() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    let node1 = StorageNode::new(config).await.unwrap();
    let node2 = node1.clone();

    assert!(!node1.is_started());
    assert!(!node2.is_started());

    node1.start().await.unwrap();

    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_access() {
    use tokio::task::JoinSet;

    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    let node = Arc::new(StorageNode::new(config).await.unwrap());
    node.start().await.unwrap();

    let mut set = JoinSet::new();

    for _ in 0..5 {
        let node_clone = node.clone();
        set.spawn(async move {
            let version = node_clone.version().await.unwrap();
            assert!(!version.is_empty());
        });
    }

    while let Some(result) = set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_send_sync_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let _node = StorageNode::new(config).await.unwrap();

    assert_send::<StorageNode>();
    assert_sync::<StorageNode>();

    assert_send::<Arc<StorageNode>>();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clone_trait() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    let node1 = StorageNode::new(config).await.unwrap();
    let node2 = node1.clone();

    assert!(!node1.is_started());
    assert!(!node2.is_started());

    node1.start().await.unwrap();
    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_send_between_threads() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let node = StorageNode::new(config).await.unwrap();

    let result = tokio::task::spawn(async move {
        let _version = node.version().await.unwrap();
        "success"
    })
    .await;

    assert_eq!(result.unwrap(), "success");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_async_file_upload() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(StorageNode::new(config).await.unwrap());

    node.start().await.unwrap();

    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, b"Hello, Storage!").unwrap();

    let options = storage_bindings::UploadOptions::new().filepath(&file_path);

    let result = storage_bindings::upload_file(&node, options).await;

    assert!(result.is_ok(), "Upload should succeed");

    node.stop().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_concurrent_operations() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(StorageNode::new(config).await.unwrap());

    node.start().await.unwrap();

    let mut handles = Vec::new();

    for i in 0..5 {
        let node_clone = node.clone();
        let handle = tokio::task::spawn(async move {
            let version = node_clone.version().await.unwrap();
            let peer_id = node_clone.peer_id().await.unwrap();
            (i, version, peer_id)
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    assert_eq!(
        results.len(),
        5,
        "All concurrent operations should complete"
    );

    node.stop().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_shared_node_across_tasks() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());

    struct AppState {
        node: Arc<StorageNode>,
    }

    let state = AppState {
        node: Arc::new(StorageNode::new(config).await.unwrap()),
    };

    let mut handles = Vec::new();

    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let version = node_clone.version().await.unwrap();
        format!("Node version: {}", version)
    }));

    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let peer_id = node_clone.peer_id().await.unwrap();
        format!("Peer ID: {}", peer_id)
    }));

    // Create node in a blocking task to avoid Send issues with raw pointers
    handles.push(tokio::task::spawn_blocking(move || {
        let temp_dir3 = tempdir().unwrap();
        let config3 = StorageConfig::new().data_dir(temp_dir3.path());
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let node = StorageNode::new(config3).await.unwrap();
            node.start().await.unwrap();
            "Node started".to_string()
        })
    }));

    for handle in handles {
        let result = handle.await.unwrap();
        println!("Task result: {}", result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_send_future_compatibility() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(StorageNode::new(config).await.unwrap());

    node.start().await.unwrap();

    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, b"Hello, Storage!").unwrap();

    let options = storage_bindings::UploadOptions::new().filepath(&file_path);
    let _result = storage_bindings::upload_file(&node, options).await.unwrap();

    assert_eq!("success", "success");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_async_upload_download() {
    use storage_bindings::{DownloadStreamOptions, UploadOptions};

    let temp_dir = tempdir().unwrap();
    let config = StorageConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(StorageNode::new(config).await.unwrap());

    node.start().await.unwrap();

    let file_path = temp_dir.path().join("test.txt");
    let test_content = b"Hello, Storage async API!";
    std::fs::write(&file_path, test_content).unwrap();

    let upload_options = UploadOptions::new().filepath(&file_path);
    let upload_result = storage_bindings::upload_file(&node, upload_options)
        .await
        .unwrap();

    let download_path = temp_dir.path().join("downloaded.txt");
    let download_options = DownloadStreamOptions::new(&upload_result.cid).filepath(&download_path);

    let _download_result =
        storage_bindings::download_stream(&node, &upload_result.cid, download_options)
            .await
            .unwrap();

    let downloaded_content = std::fs::read(&download_path).unwrap();
    assert_eq!(downloaded_content, test_content);

    node.stop().await.unwrap();
}
