# Codex Rust Bindings

This repository provides Rust bindings for the Codex library, enabling seamless integration with Rust projects.

## Usage

Include in your Cargo project:

```toml
[dependencies]
codex-bindings = "0.2.0"
```

To learn how to use those bindings, take a look at the [example project](https://github.com/nipsysdev/example-codex-rust-bindings) or the [integration tests](./tests/) directory.

## Building

Building will automatically:

1. Fetch the latest prebuilt libstorage binary for your platform from GitHub
2. Generate Rust bindings and compile the crate

**Note**: The first build will download the prebuilt binary (~50MB). Subsequent builds will use the cached version.

### Supported Platforms

- Linux x86_64 (x86_64-unknown-linux-gnu)
- Linux ARM64 (aarch64-unknown-linux-gnu)

### Libstorage Version Pinning

**Option 1: Cargo.toml metadata**

Add to your `Cargo.toml`:

```toml
[package.metadata.prebuilt]
libstorage = "master-60861d6a"
```

**Option 2: Environment variable (for local overrides)**

```bash
export LOGOS_STORAGE_VERSION=master-60861d6a
cargo build
```

Available versions can be found at: https://github.com/nipsysdev/logos-storage-nim-bin/releases

### Building from source

```bash
cargo build --release
# or, for debug
cargo build
```

### Testing

The library includes comprehensive integration tests that demonstrate all major functionality.

#### Running All Tests

```bash
# Run all tests (unit tests + integration tests)
cargo test
```

#### Running Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test basic_usage
cargo test --test chunk_operations
cargo test --test debug_operations
cargo test --test p2p_networking
cargo test --test storage_management
cargo test --test two_node_network
cargo test --test thread_safe_tests
```

#### Available Integration Tests

- **basic_usage**: Demonstrates basic upload/download functionality
- **chunk_operations**: Shows chunk-based upload and download operations
- **debug_operations**: Demonstrates debug operations and logging
- **p2p_networking**: Shows P2P networking operations
- **storage_management**: Demonstrates storage management operations
- **two_node_network**: Shows two-node network setup and data transfer
- **thread_safe_tests**: Tests thread-safe node lifecycle and concurrent operations

## License

[MIT](./LICENSE)
