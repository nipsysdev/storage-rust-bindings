# Storage Rust Bindings

This repository provides Rust bindings for the Storage library, enabling seamless integration with Rust projects.

## Usage

Include in your Cargo project:

```toml
[dependencies]
storage-bindings = "0.2.0"
```

To learn how to use those bindings, take a look at the [example project](https://github.com/nipsysdev/example-storage-rust-bindings) or the [integration tests](./tests/) directory.

## Building

Building will automatically:

1. Fetch the latest prebuilt libstorage binary for your platform from GitHub
2. Generate Rust bindings and compile the crate

**Note**: The first build will download the prebuilt binary (~50MB). Subsequent builds will use the cached version.

## Caching

Prebuilt binaries are automatically cached to improve build performance and reduce network usage.

### Cache Location

Prebuilt binaries are cached in a platform-specific location:

- **Linux**: `~/.cache/storage-bindings/`
- **macOS**: `~/Library/Caches/storage-bindings/`
- **Windows**: `%LOCALAPPDATA%\storage-bindings\cache\`

The cache is organized by version and platform:

```
~/.cache/storage-bindings/
├── master-1acedcf/
│   ├── linux-amd64/
│   │   ├── libstorage.a
│   │   ├── libstorage.h
│   │   └── SHA256SUMS.txt
│   └── darwin-arm64/
│       └── ...
└── master-2b3d4e5/
    └── ...
```

### Managing the Cache

#### Force Re-download

To force a fresh download without clearing the cache:

```bash
STORAGE_BINDINGS_FORCE_DOWNLOAD=1 cargo build
```

#### Clean Entire Cache

To remove all cached binaries:

```bash
# Linux/macOS
rm -rf ~/.cache/storage-bindings/

# Windows
rmdir /s /q %LOCALAPPDATA%\storage-bindings\cache

# Or using the build script
STORAGE_BINDINGS_CLEAN_CACHE=1 cargo build
```

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

### Building using local libraries

To use locally built libraries instead of downloading from GitHub, set the `STORAGE_BINDINGS_LOCAL_LIBS` environment variable to the path of the dist folder:

```bash
export STORAGE_BINDINGS_LOCAL_LIBS=/path/to/logos-storage-nim-bin/dist/master-50bd1839-linux-amd64
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
cargo test --test $test_name
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
