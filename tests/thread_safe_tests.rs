use codex_bindings::{CodexConfig, CodexNode};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_thread_safe_node_creation() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let node = CodexNode::new(config).unwrap();
    assert!(!node.is_started());
}

#[tokio::test]
async fn test_thread_safe_node_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node = CodexNode::new(config).unwrap();

    node.start().unwrap();
    assert!(node.is_started());

    let version = node.version().unwrap();
    assert!(!version.is_empty());

    let peer_id = node.peer_id().unwrap();
    assert!(!peer_id.is_empty());

    node.stop().unwrap();
    assert!(!node.is_started());
}

#[tokio::test]
async fn test_node_cloning() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node1 = CodexNode::new(config).unwrap();
    let node2 = node1.clone();

    assert!(!node1.is_started());
    assert!(!node2.is_started());

    node1.start().unwrap();

    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test]
async fn test_concurrent_access() {
    use tokio::task::JoinSet;

    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let node = Arc::new(CodexNode::new(config).unwrap());
    node.start_async().await.unwrap();

    let mut set = JoinSet::new();

    for _ in 0..5 {
        let node_clone = node.clone();
        set.spawn(async move {
            let version = node_clone.version().unwrap();
            assert!(!version.is_empty());
        });
    }

    while let Some(result) = set.join_next().await {
        result.unwrap();
    }
}

#[test]
fn test_send_sync_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let _node = CodexNode::new(config).unwrap();

    assert_send::<CodexNode>();
    assert_sync::<CodexNode>();

    assert_send::<Arc<CodexNode>>();
}

#[test]
fn test_clone_trait() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node1 = CodexNode::new(config).unwrap();
    let node2 = node1.clone();

    assert!(!node1.is_started());
    assert!(!node2.is_started());

    node1.start().unwrap();
    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test]
async fn test_send_between_threads() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = CodexNode::new(config).unwrap();

    let result = tokio::task::spawn(async move {
        let _version = node.version().unwrap();
        "success"
    })
    .await;

    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_async_file_upload() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    node.start_async().await.unwrap();

    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, b"Hello, Codex!").unwrap();

    let options = codex_bindings::UploadOptions::new().filepath(&file_path);

    let result = codex_bindings::upload_file(&node, options).await;

    assert!(result.is_ok(), "Upload should succeed");

    node.stop_async().await.unwrap();
}

#[tokio::test]
async fn test_multiple_concurrent_operations() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    node.start_async().await.unwrap();

    let mut handles = Vec::new();

    for i in 0..5 {
        let node_clone = node.clone();
        let handle = tokio::task::spawn(async move {
            let version = node_clone.version().unwrap();
            let peer_id = node_clone.peer_id().unwrap();
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

    node.stop_async().await.unwrap();
}

#[tokio::test]
async fn test_shared_node_across_tasks() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    struct AppState {
        node: Arc<CodexNode>,
    }

    let state = AppState {
        node: Arc::new(CodexNode::new(config).unwrap()),
    };

    let mut handles = Vec::new();

    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let version = node_clone.version().unwrap();
        format!("Node version: {}", version)
    }));

    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let peer_id = node_clone.peer_id().unwrap();
        format!("Peer ID: {}", peer_id)
    }));

    handles.push(tokio::task::spawn(async move {
        tokio::task::spawn_blocking(move || {
            let mut node = CodexNode::new(CodexConfig::new()).unwrap();
            node.start().unwrap();
            node
        })
        .await
        .unwrap();
        "Node started".to_string()
    }));

    for handle in handles {
        let result = handle.await.unwrap();
        println!("Task result: {}", result);
    }
}

#[tokio::test]
async fn test_send_future_compatibility() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    let future = async move {
        node.start_async().await.unwrap();

        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"Hello, Codex!").unwrap();

        let options = codex_bindings::UploadOptions::new().filepath(&file_path);
        let _result = codex_bindings::upload_file(&node, options).await.unwrap();

        "success"
    };

    let result = tokio::task::spawn(future).await.unwrap();
    assert_eq!(result, "success");
}

#[tokio::test]
async fn test_async_upload_download() {
    use codex_bindings::{DownloadStreamOptions, UploadOptions};

    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    node.start_async().await.unwrap();

    let file_path = temp_dir.path().join("test.txt");
    let test_content = b"Hello, Codex async API!";
    std::fs::write(&file_path, test_content).unwrap();

    let upload_options = UploadOptions::new().filepath(&file_path);
    let upload_result = codex_bindings::upload_file(&node, upload_options)
        .await
        .unwrap();

    let download_path = temp_dir.path().join("downloaded.txt");
    let download_options = DownloadStreamOptions::new(&upload_result.cid).filepath(&download_path);

    let _download_result =
        codex_bindings::download_stream(&node, &upload_result.cid, download_options)
            .await
            .unwrap();

    let downloaded_content = std::fs::read(&download_path).unwrap();
    assert_eq!(downloaded_content, test_content);

    node.stop_async().await.unwrap();
}
