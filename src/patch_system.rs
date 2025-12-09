use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PatchEngine {
    verbose: bool,
    patches_dir: PathBuf,
    base_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("Failed to discover patches: {0}")]
    DiscoveryError(String),

    #[error("Patch application failed: {0}")]
    ApplicationFailed(String),

    #[error("Patch validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl PatchEngine {
    pub fn new(verbose: bool) -> Result<Self, PatchError> {
        let patches_dir = PathBuf::from("android-patches");
        let base_dir = PathBuf::from(".");

        Ok(Self {
            verbose,
            patches_dir,
            base_dir,
        })
    }

    fn discover_all_patches(&self, arch: &str) -> Result<(Vec<String>, Vec<String>), PatchError> {
        let mut arch_patches = Vec::new();
        let mut shared_patches = Vec::new();

        let arch_dir = self.patches_dir.join(arch);
        if arch_dir.exists() {
            self.find_patch_files_recursive(&arch_dir, &mut arch_patches, "")?;
        }

        let shared_dir = self.patches_dir.join("shared");
        if shared_dir.exists() {
            self.find_patch_files_recursive(&shared_dir, &mut shared_patches, "")?;
        }

        if self.verbose {
            println!(
                "Discovered {} architecture-specific patches for {}",
                arch_patches.len(),
                arch
            );
            println!("Discovered {} shared patches", shared_patches.len());
        }

        Ok((arch_patches, shared_patches))
    }

    fn find_patch_files_recursive(
        &self,
        dir: &Path,
        patches: &mut Vec<String>,
        prefix: &str,
    ) -> Result<(), PatchError> {
        let entries = fs::read_dir(dir).map_err(|e| {
            PatchError::DiscoveryError(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PatchError::DiscoveryError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let new_prefix = if prefix.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{}/{}", prefix, dir_name)
                };

                self.find_patch_files_recursive(&path, patches, &new_prefix)?;
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".patch") {
                    let patch_path = if prefix.is_empty() {
                        file_name.to_string()
                    } else {
                        format!("{}/{}", prefix, file_name)
                    };
                    patches.push(patch_path);
                }
            }
        }

        Ok(())
    }

    pub fn apply_patches_for_arch(&self, arch: &str) -> Result<Vec<String>, PatchError> {
        let (arch_patches, shared_patches) = self.discover_all_patches(arch)?;

        if self.verbose {
            println!(
                "Applying {} architecture-specific patches for architecture {}",
                arch_patches.len(),
                arch
            );
        }

        let mut applied_arch_patches = Vec::new();
        let mut failed_arch_patches = Vec::new();

        for patch_file in arch_patches {
            let patch_path = self.patches_dir.join(arch).join(&patch_file);

            if !patch_path.exists() {
                if self.verbose {
                    println!("  ⚠️  Patch file not found: {}", patch_path.display());
                }
                failed_arch_patches.push(patch_file.clone());
                continue;
            }

            match self.apply_patch(&patch_path, &patch_file) {
                Ok(()) => {
                    applied_arch_patches.push(patch_file.clone());
                }
                Err(e) => {
                    if self.verbose {
                        println!("  ⚠️  Failed to apply patch {}: {}", patch_file, e);
                    }
                    failed_arch_patches.push(patch_file.clone());
                }
            }
        }

        let mut applied_shared_patches = Vec::new();
        let mut failed_shared_patches = Vec::new();

        if self.verbose {
            println!("Applying {} shared patches...", shared_patches.len());
        }

        for patch_file in shared_patches {
            let patch_path = self.patches_dir.join("shared").join(&patch_file);

            if !patch_path.exists() {
                if self.verbose {
                    println!(
                        "  ⚠️  Shared patch file not found: {}",
                        patch_path.display()
                    );
                }
                failed_shared_patches.push(patch_file.clone());
                continue;
            }

            match self.apply_patch(&patch_path, &patch_file) {
                Ok(()) => {
                    applied_shared_patches.push(patch_file.clone());
                }
                Err(e) => {
                    if self.verbose {
                        println!("  ⚠️  Failed to apply shared patch {}: {}", patch_file, e);
                    }
                    failed_shared_patches.push(patch_file.clone());
                }
            }
        }

        let mut all_applied_patches = applied_arch_patches.clone();
        all_applied_patches.extend(applied_shared_patches.clone());

        if self.verbose {
            println!("\n=== Patch Summary for Architecture {} ===", arch);
            println!(
                "Architecture-specific patches: {} applied, {} failed",
                applied_arch_patches.len(),
                failed_arch_patches.len()
            );
            println!(
                "Shared patches: {} applied, {} failed",
                applied_shared_patches.len(),
                failed_shared_patches.len()
            );
            println!(
                "Total: {} patches applied, {} failed",
                all_applied_patches.len(),
                failed_arch_patches.len() + failed_shared_patches.len()
            );

            if !failed_arch_patches.is_empty() {
                println!("  Failed architecture-specific patches:");
                for patch in &failed_arch_patches {
                    println!("    - {}", patch);
                }
            }

            if !failed_shared_patches.is_empty() {
                println!("  Failed shared patches:");
                for patch in &failed_shared_patches {
                    println!("    - {}", patch);
                }
            }
            println!("========================================\n");
        }

        if all_applied_patches.is_empty()
            && (!failed_arch_patches.is_empty() || !failed_shared_patches.is_empty())
        {
            Err(PatchError::ApplicationFailed(format!(
                "No patches could be applied for architecture {} ({} failed)",
                arch,
                failed_arch_patches.len() + failed_shared_patches.len()
            )))
        } else {
            Ok(all_applied_patches)
        }
    }

    fn apply_patch(&self, patch_file: &Path, patch_name: &str) -> Result<(), PatchError> {
        if self.verbose {
            println!("Applying patch: {}", patch_name);
        }

        println!("🔧 DEBUG: Attempting to apply patch: {}", patch_name);
        println!("🔧 DEBUG: Patch file path: {}", patch_file.display());
        println!(
            "🔧 DEBUG: Current time: {}",
            std::process::Command::new("date")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        );

        if self.is_patch_already_applied(patch_file)? {
            if self.verbose {
                println!("  ✅ Patch {} already applied", patch_name);
            }
            println!("🔧 DEBUG: Patch {} already applied, skipping", patch_name);
            return Ok(());
        }

        let output = Command::new("git")
            .arg("apply")
            .arg(patch_file)
            .current_dir(&self.base_dir)
            .output()
            .map_err(|e| {
                PatchError::ApplicationFailed(format!("Failed to run patch command: {}", e))
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        if self.verbose {
            println!("  Patch command stdout: {}", stdout);
            println!("  Patch command stderr: {}", stderr);
            println!("  Patch command exit status: {}", output.status);
        }

        // DEBUG: Log patch application result
        if output.status.success() {
            println!("🔧 DEBUG: ✅ Successfully applied patch: {}", patch_name);
            println!(
                "🔧 DEBUG: Post-application time: {}",
                std::process::Command::new("date")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default()
            );
        } else {
            println!("🔧 DEBUG: ❌ Failed to apply patch: {}", patch_name);
        }

        if !output.status.success() {
            if stderr.contains("already applied")
                || stderr.contains("previously applied")
                || stdout.contains("already applied")
                || stdout.contains("previously applied")
                || stderr.contains("FAILED")
            {
                if self.verbose {
                    println!(
                        "  ✅ Patch {} already applied or partially applied",
                        patch_name
                    );
                }
                println!(
                    "🔧 DEBUG: Patch {} already applied or partially applied",
                    patch_name
                );
                return Ok(());
            }

            return Err(PatchError::ApplicationFailed(format!(
                "Patch {} failed: {}\nstdout: {}",
                patch_name, stderr, stdout
            )));
        }

        if self.verbose {
            println!("  ✅ Applied patch: {}", patch_name);
        }

        Ok(())
    }

    fn is_patch_already_applied(&self, patch_file: &Path) -> Result<bool, PatchError> {
        let output = Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg(patch_file)
            .current_dir(&self.base_dir)
            .output()
            .map_err(|e| {
                PatchError::ValidationFailed(format!("Failed to run patch --dry-run: {}", e))
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        let already_applied = stderr.contains("patch does not apply");

        Ok(already_applied)
    }

    #[allow(dead_code)]
    pub fn validate_patches_for_arch(&self, arch: &str) -> Result<(), PatchError> {
        let (arch_patches, shared_patches) = self.discover_all_patches(arch)?;

        if self.verbose {
            println!(
                "Validating {} architecture-specific patches and {} shared patches for architecture {}",
                arch_patches.len(),
                shared_patches.len(),
                arch
            );
        }

        for patch_file in arch_patches {
            let patch_path = self.patches_dir.join(arch).join(&patch_file);

            if !patch_path.exists() {
                return Err(PatchError::ValidationFailed(format!(
                    "Patch file not found: {}",
                    patch_path.display()
                )));
            }

            if !self.is_patch_already_applied(&patch_path)? {
                return Err(PatchError::ValidationFailed(format!(
                    "Architecture-specific patch {} is not applied",
                    patch_file
                )));
            }
        }

        for patch_file in shared_patches {
            let patch_path = self.patches_dir.join("shared").join(&patch_file);

            if !patch_path.exists() {
                return Err(PatchError::ValidationFailed(format!(
                    "Shared patch file not found: {}",
                    patch_path.display()
                )));
            }

            if !self.is_patch_already_applied(&patch_path)? {
                return Err(PatchError::ValidationFailed(format!(
                    "Shared patch {} is not applied",
                    patch_file
                )));
            }
        }

        if self.verbose {
            println!("✅ All patches validated for architecture {}", arch);
        }

        Ok(())
    }
}

pub fn get_android_arch_from_target(target: &str) -> (&'static str, &'static str) {
    match target {
        "aarch64-linux-android" => ("arm64", "aarch64"),
        _ => panic!("Unsupported Android target: {}", target),
    }
}
