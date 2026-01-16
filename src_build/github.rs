use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Fetches release information from GitHub API
pub fn fetch_release(version: &str) -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    println!("  [GITHUB] Starting fetch_release");
    println!("  [GITHUB] Version: {}", version);

    let url = if version == "latest" {
        println!("  [GITHUB] Using latest release endpoint");
        "https://api.github.com/repos/nipsysdev/logos-storage-nim-bin/releases/latest".to_string()
    } else {
        println!("  [GITHUB] Using tagged release endpoint");
        format!(
            "https://api.github.com/repos/nipsysdev/logos-storage-nim-bin/releases/tags/{}",
            version
        )
    };

    println!("  [GITHUB] URL: {}", url);

    println!("  [GITHUB] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("storage-rust-bindings")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    println!("  [GITHUB] ✓ HTTP client created");

    println!("  [GITHUB] Sending GET request to GitHub API...");
    let response = client.get(&url).send()?;
    println!("  [GITHUB] ✓ Response received");
    println!("  [GITHUB]   Status: {}", response.status());

    if !response.status().is_success() {
        println!("  [GITHUB] ✗ GitHub API request failed");
        return Err(format!("GitHub API returned status: {}", response.status()).into());
    }

    println!("  [GITHUB] Parsing JSON response...");
    let release: GitHubRelease = response.json()?;
    println!("  [GITHUB] ✓ JSON parsed successfully");
    println!("  [GITHUB]   Tag name: {}", release.tag_name);
    println!("  [GITHUB]   Number of assets: {}", release.assets.len());

    for (i, asset) in release.assets.iter().enumerate() {
        println!("  [GITHUB]   Asset {}: {}", i + 1, asset.name);
    }

    println!("  [GITHUB] ✓ fetch_release completed successfully");
    Ok(release)
}

/// Finds the matching asset for the given platform
pub fn find_matching_asset<'a>(
    release: &'a GitHubRelease,
    platform: &str,
) -> Option<&'a GitHubAsset> {
    println!("  [GITHUB] Starting find_matching_asset");
    println!("  [GITHUB] Platform: {}", platform);
    println!(
        "  [GITHUB] Total assets available: {}",
        release.assets.len()
    );

    let search_pattern = format!("linux-{}", platform.replace("linux-", ""));
    println!("  [GITHUB] Search pattern: {}", search_pattern);

    let matching_asset = release
        .assets
        .iter()
        .find(|asset| asset.name.contains(&search_pattern));

    match &matching_asset {
        Some(asset) => {
            println!("  [GITHUB] ✓ Found matching asset: {}", asset.name);
        }
        None => {
            println!("  [GITHUB] ✗ No matching asset found");
            println!("  [GITHUB] Available assets:");
            for (i, asset) in release.assets.iter().enumerate() {
                println!("  [GITHUB]   {}: {}", i + 1, asset.name);
            }
        }
    }

    println!("  [GITHUB] ✓ find_matching_asset completed");
    matching_asset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_asset() {
        let release = GitHubRelease {
            tag_name: "test".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "logos-storage-nim-test-test-linux-amd64.tar.gz".to_string(),
                    browser_download_url: "http://example.com/amd64.tar.gz".to_string(),
                },
                GitHubAsset {
                    name: "logos-storage-nim-test-test-linux-arm64.tar.gz".to_string(),
                    browser_download_url: "http://example.com/arm64.tar.gz".to_string(),
                },
            ],
        };

        let asset = find_matching_asset(&release, "linux-amd64");
        assert!(asset.is_some());
        assert!(asset.unwrap().name.contains("amd64"));

        let asset = find_matching_asset(&release, "linux-arm64");
        assert!(asset.is_some());
        assert!(asset.unwrap().name.contains("arm64"));
    }
}
