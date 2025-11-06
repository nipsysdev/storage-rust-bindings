//! Debug configuration JSON generation

use codex_rust_bindings::{CodexConfig, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration like in the basic usage example
    let config = CodexConfig::new()
        .log_level(LogLevel::Info)
        .data_dir("/tmp/test_codex_data")
        .storage_quota(100 * 1024 * 1024) // 100 MB
        .max_peers(50);

    // Convert to JSON
    let json = config.to_json()?;
    println!("Generated JSON configuration:");
    println!("{}", json);

    Ok(())
}
