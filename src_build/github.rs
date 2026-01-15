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
    let url = if version == "latest" {
        "https://api.github.com/repos/nipsysdev/logos-storage-nim-bin/releases/latest".to_string()
    } else {
        format!(
            "https://api.github.com/repos/nipsysdev/logos-storage-nim-bin/releases/tags/{}",
            version
        )
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("storage-rust-bindings")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()).into());
    }

    let release: GitHubRelease = response.json()?;
    Ok(release)
}

/// Finds the matching asset for the given platform
pub fn find_matching_asset<'a>(
    release: &'a GitHubRelease,
    platform: &str,
) -> Option<&'a GitHubAsset> {
    release.assets.iter().find(|asset| {
        asset
            .name
            .contains(&format!("linux-{}", platform.replace("linux-", "")))
    })
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
