use std::path::PathBuf;

use super::prebuilt;

pub const LOCAL_LIBS_ENV_VAR: &str = "STORAGE_BINDINGS_LOCAL_LIBS";

pub const LIBSTORAGE_H: &str = "libstorage.h";
pub const CACHE_MARKER: &str = ".prebuilt_cached";

/// Attempts to use local libraries if environment variable is set
/// Returns Ok with the output directory path if successful, Err otherwise
pub fn try_local_development_mode(
    out_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_libs_path = match std::env::var(LOCAL_LIBS_ENV_VAR) {
        Ok(path) => path,
        Err(_) => {
            prebuilt::log_info("Local development mode not enabled");
            return Err("Local development mode not enabled".into());
        }
    };

    prebuilt::log_info("🚀 LOCAL DEVELOPMENT MODE DETECTED");
    prebuilt::log_info(&format!("Local libs path: {}", local_libs_path));

    let local_path = PathBuf::from(&local_libs_path);

    if !local_path.exists() {
        return Err(format!(
            "Local library path does not exist: {}. \
             Please check the {} environment variable.",
            local_libs_path, LOCAL_LIBS_ENV_VAR
        )
        .into());
    }

    prebuilt::validate_required_files(&local_path)?;

    prebuilt::log_info(&format!(
        "✓ Using local libraries from: {}",
        local_libs_path
    ));
    prebuilt::log_info("✓ All required files found");

    copy_all_files(&local_path, out_dir)?;
    prebuilt::create_cache_marker(out_dir, "local")?;

    prebuilt::log_info("✓ Local libraries copied successfully");
    Ok(out_dir.clone())
}

/// Copies all files from local path to output directory
fn copy_all_files(
    local_path: &PathBuf,
    out_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    prebuilt::log_info("Copying all files from local path to OUT_DIR...");

    for entry in std::fs::read_dir(local_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let dest = out_dir.join(&file_name);
            std::fs::copy(&path, &dest)?;
            prebuilt::log_info(&format!("  Copied: {}", file_name));
        }
    }

    Ok(())
}
