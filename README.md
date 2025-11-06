# Codex Rust Bindings

This repository provides Rust bindings for the Codex library, enabling seamless integration with Rust projects.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
codex-rust-bindings = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## API

### Init

First you need to create a Codex node:

```rust
use codex_rust_bindings::{CodexNode, CodexConfig};

let config = CodexConfig::new()
    .data_dir("/path/to/data")
    .block_retries(10);

let mut node = CodexNode::new(config)?;
// ...
node.destroy()?;
```

The `CodexConfig` object provides several options to configure your node. You should at least
adjust the `data_dir` folder and the `block_retries` setting to avoid long retrieval times when
the data is unavailable.

When you are done with your node, you **have to** call `destroy()` method to free resources.

### Start / Stop

Use `start()` method to start your node. You **have to** call `stop()` before `destroy()` when you are done
with your node.

```rust
node.start()?;
node.stop()?;
node.destroy()?;
```

### Info

You can get the version and revision without starting the node:

```rust
let version = node.version()?;
let revision = node.revision()?;
```

Other information is available after the node is started:

```rust
let version = node.version()?;
let spr = node.spr()?;
let peer_id = node.peer_id()?;
```

### Upload

There are 3 strategies for uploading: `reader`, `file` or `chunks`. Each one requires its own upload session.

#### reader

The `reader` strategy is the easiest option when you already have a Rust `Read`.
It handles creating the upload session and cancels it if an error occurs.

The `filepath` should contain the data's name with its extension, because Codex uses that to
infer the MIME type.

An `on_progress` callback is available to receive progress updates and notify the user.

The `upload_reader` returns the cid of the content uploaded.

```rust
use codex_rust_bindings::{upload_reader, UploadOptions};
use std::io::Cursor;

let data = b"Hello World!";
let reader = Cursor::new(data);
let options = UploadOptions::new()
    .filepath("hello.txt")
    .on_progress(|read, total, percent, error| {
        // Do something with the data
    });

let cid = upload_reader(&node, options, reader)?;
```

#### file

The `file` strategy allows you to upload a file on Codex using the path.
It handles creating the upload session and cancels it if an error occurs.

The `on_progress` callback is the same as for `reader` strategy.

The `upload_file` returns the cid of the content uploaded.

```rust
use codex_rust_bindings::{upload_file, UploadOptions};

let options = UploadOptions::new()
    .filepath("./testdata/hello.txt")
    .on_progress(|read, total, percent, error| {
        // Do something with the data
    });

let cid = upload_file(&node, options)?;
```

#### chunks

The `chunks` strategy allows you to manage the upload by yourself. It requires more code
but provides more flexibility. You have to create the upload session, send the chunks
and then finalize to get the cid.

```rust
use codex_rust_bindings::{upload_init, upload_chunk, upload_finalize, UploadOptions};

let session_id = upload_init(&node, &UploadOptions::new().filepath("hello.txt"))?;

upload_chunk(&node, &session_id, b"Hello ")?;

upload_chunk(&node, &session_id, b"World!")?;

let cid = upload_finalize(&node, &session_id)?;
```

Using this strategy, you can handle resumable uploads and cancel the upload
whenever you want!

### Download

When you receive a cid, you can download the `Manifest` to get information about the data:

```rust
use codex_rust_bindings::download_manifest;

let manifest = download_manifest(&node, "QmExampleCID...")?;
```

It is not mandatory for downloading the data but it is really useful.

There are 2 strategies for downloading: `stream` and `chunks`.

#### stream

The `stream` strategy is the easiest to use.

It provides an `on_progress` callback to receive progress updates and notify the user.
The percentage is calculated from the `dataset_size` (taken from the manifest).
If you don't provide it, you can enable `dataset_size_auto` so `download_stream` fetches the
manifest first and uses its `dataset_size`.

You can pass a `writer` and/or a `filepath` as destinations. They are not mutually exclusive,
letting you write the content to two places for the same download.

```rust
use codex_rust_bindings::{download_stream, DownloadStreamOptions};
use std::fs::File;

let file = File::create("testdata/hello.downloaded.writer.txt")?;
let options = DownloadStreamOptions::new("QmExampleCID...")
    .writer(file)
    .dataset_size(len)
    .on_progress(|read, total, percent, error| {
        // Handle progress
    });

download_stream(&node, "QmExampleCID...", options)?;
```

#### chunks

The `chunks` strategy allows to manage the download by yourself. It requires more code
but provide more flexibility.

This strategy **assumes you already know the total size to download** (from the manifest).
After you believe all chunks have been retrieved, you **must** call `download_cancel`
to terminate the download session.

```rust
use codex_rust_bindings::{download_init, download_chunk, download_cancel, DownloadInitOptions};

let cid = "QmExampleCID...";
download_init(&node, cid, &DownloadInitOptions::new())?;

let chunk = download_chunk(&node, cid)?;

download_cancel(&node, cid)?;
```

Using this strategy, you can handle resumable downloads and cancel the download
whenever you want!

### Storage

Several methods are available to manage the data on your node:

```rust
use codex_rust_bindings::{manifests, space, delete, fetch};

let manifests = manifests(&node)?;
let space_info = space(&node)?;

let cid = "QmExampleCID...";
delete(&node, cid)?;
fetch(&node, cid)?;
```

The `fetch` method downloads remote data into your local node.

### P2P

You can connect to a node using the `peer_id` or the `listen_addresses`:

```rust
use codex_rust_bindings::connect;

let peer_id = "12D3KooWExamplePeerId";
let addrs = vec![
    "/ip4/192.168.1.100/tcp/8080".to_string(),
    "/ip4/192.168.1.100/udp/8080/quic".to_string(),
];

connect(&node, peer_id, &addrs)?;
```

### Debug

Several methods are available to debug your node:

```rust
use codex_rust_bindings::{debug, update_log_level, peer_debug, LogLevel};

// Get node info
let info = debug(&node)?;

// Update the chronicles level log on runtime
update_log_level(&node, LogLevel::Debug)?;

let peer_id = "12D3KooWExamplePeerId";
let record = peer_debug(&node, peer_id)?;
```

`peer_debug` is only available if you built with the appropriate debug flags.

### Async Support

All operations have async versions available:

```rust
use codex_rust_bindings::{CodexNode, CodexConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CodexConfig::default();
    let mut node = CodexNode::new(config)?;

    // Start the node asynchronously
    node.start_async().await?;

    // Use async operations...

    // Stop the node asynchronously
    node.stop_async().await?;
    node.destroy()?;

    Ok(())
}
```

## Building

To build the library, you need to have the libcodex C library installed. The build system will automatically detect it and generate the necessary bindings.

```bash
# Build the library
cargo build

# Build with examples
cargo build --examples

# Run tests
cargo test

# Run the basic example
cargo run --example basic_usage
```

## Requirements

- Rust 1.70 or later
- libcodex C library
- pkg-config (for detecting libcodex)

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Related Projects

- [Codex Go Bindings](https://github.com/codex-storage/codex-go-bindings) - Go bindings for the Codex library
- [Codex](https://github.com/codex-storage/codex) - The main Codex project
