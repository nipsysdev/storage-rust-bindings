//! P2P networking integration test for the Storage Rust bindings
//!
//! This test demonstrates how to use P2P operations:
//! - Connect to peers
//! - Get peer information
//! - Debug peer connections

use storage_bindings::{LogLevel, StorageConfig, StorageNode};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn test_p2p_networking() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();

    let temp_dir = tempdir()?;

    let config = StorageConfig::new()
        .log_level(LogLevel::Error)
        .data_dir(temp_dir.path().join("storage_data"))
        .max_peers(50)
        .block_retries(3000)
        .discovery_port(8091);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    // Get node information
    let peer_id = node.peer_id().await?;
    println!("Peer ID: {}", peer_id);

    let version = node.version().await?;
    println!("Node version: {}", version);

    let repo = node.repo().await?;
    println!("Repository path: {}", repo);

    // Test P2P operations
    let our_peer_id = storage_bindings::get_peer_id(&node).await?;
    assert_eq!(peer_id, our_peer_id, "Peer IDs should match");
    println!("Peer ID from P2P function: {}", our_peer_id);

    // Test connecting to a peer
    println!("\nTesting peer connection:");
    let test_peer_id = "12D3KooWExamplePeer123456789";
    let test_addresses = vec![
        "/ip4/192.168.1.100/tcp/8080".to_string(),
        "/ip4/192.168.1.100/udp/8080/quic".to_string(),
        "/ip6/::1/tcp/8080".to_string(),
    ];

    println!("Attempting to connect to peer: {}", test_peer_id);
    for (i, addr) in test_addresses.iter().enumerate() {
        println!("  Address {}: {}", i + 1, addr);
    }

    let connect_result = storage_bindings::connect(&node, test_peer_id, &test_addresses).await;
    match connect_result {
        Ok(()) => println!("✓ Successfully connected to peer"),
        Err(e) => println!("✗ Failed to connect to peer: {}", e),
    }

    // Test getting peer information
    println!("\nTesting peer information:");
    let peer_info_result = storage_bindings::get_peer_info(&node, test_peer_id).await;
    match peer_info_result {
        Ok(peer_info) => {
            println!("✓ Successfully retrieved peer information:");
            println!("  Peer ID: {}", peer_info.id);
            println!("  Connected: {}", peer_info.connected);
            println!("  Addresses: {:?}", peer_info.addresses);
            println!("  Protocols: {:?}", peer_info.protocols);
        }
        Err(e) => println!("✗ Failed to get peer information: {}", e),
    }

    // Test with various peer ID formats
    println!("\nTesting various peer ID formats:");
    let test_peer_ids = vec![
        "12D3KooWExamplePeer123456789",
        "QmSomePeerId123456789",
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    ];

    for peer_id in test_peer_ids {
        let peer_info_result = storage_bindings::get_peer_info(&node, peer_id).await;
        match peer_info_result {
            Ok(_) => println!("  ✓ Successfully retrieved peer info for {}", peer_id),
            Err(_) => println!(
                "  ✗ Failed to retrieve peer info for {} (expected for test peer)",
                peer_id
            ),
        }
    }

    // Test invalid parameters
    println!("\nTesting invalid parameters:");

    let empty_peer_result = storage_bindings::connect(&node, "", &test_addresses).await;
    assert!(empty_peer_result.is_err(), "Should fail with empty peer ID");
    println!("  ✓ Correctly failed with empty peer ID");

    let empty_addr_result = storage_bindings::connect(&node, test_peer_id, &[]).await;
    assert!(
        empty_addr_result.is_err(),
        "Should fail with empty addresses"
    );
    println!("  ✓ Correctly failed with empty addresses");

    let empty_info_result = storage_bindings::get_peer_info(&node, "").await;
    assert!(empty_info_result.is_err(), "Should fail with empty peer ID");
    println!("  ✓ Correctly failed with empty peer ID");

    // Test concurrent P2P operations
    println!("\nTesting concurrent P2P operations:");
    let peer_id_future1 = storage_bindings::get_peer_id(&node);
    let peer_info_future1 = storage_bindings::get_peer_info(&node, "12D3KooWTestPeer1");
    let peer_info_future2 = storage_bindings::get_peer_info(&node, "12D3KooWTestPeer2");

    let (peer_id_result, peer_info_result1, peer_info_result2) =
        tokio::join!(peer_id_future1, peer_info_future1, peer_info_future2);

    assert!(peer_id_result.is_ok(), "get_peer_id should succeed");
    println!("  ✓ get_peer_id: {}", peer_id_result.unwrap());

    match peer_info_result1 {
        Ok(_) => println!("  ✓ get_peer_info (peer1): succeeded"),
        Err(_) => println!("  ✗ get_peer_info (peer1): failed (expected)"),
    }

    match peer_info_result2 {
        Ok(_) => println!("  ✓ get_peer_info (peer2): succeeded"),
        Err(_) => println!("  ✗ get_peer_info (peer2): failed (expected)"),
    }

    // Cleanup
    node.stop().await?;
    node.destroy().await?;

    Ok(())
}
