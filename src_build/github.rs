use serde::Deserialize;

use super::urls;

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
        urls::latest_release_url()
    } else {
        println!("  [GITHUB] Using tagged release endpoint");
        urls::tagged_release_url(version)
    };

    println!("  [GITHUB] URL: {}", url);

    println!("  [GITHUB] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent(urls::USER_AGENT)
        .timeout(std::time::Duration::from_secs(urls::API_TIMEOUT_SECONDS))
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

/// Fetches the SHA256SUMS.txt file from a GitHub release
pub fn fetch_checksums_file(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("  [GITHUB] Starting fetch_checksums_file");
    println!("  [GITHUB] Version: {}", version);

    let url = if version == "latest" {
        println!("  [GITHUB] Using latest release endpoint");
        urls::latest_release_url()
    } else {
        println!("  [GITHUB] Using tagged release endpoint");
        urls::tagged_release_url(version)
    };

    println!("  [GITHUB] URL: {}", url);

    println!("  [GITHUB] Creating HTTP client...");
    let client = reqwest::blocking::Client::builder()
        .user_agent(urls::USER_AGENT)
        .timeout(std::time::Duration::from_secs(urls::API_TIMEOUT_SECONDS))
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

    // Find SHA256SUMS.txt asset
    println!("  [GITHUB] Looking for SHA256SUMS.txt asset...");
    let checksums_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS.txt")
        .ok_or("SHA256SUMS.txt not found in release assets")?;

    println!("  [GITHUB] ✓ Found SHA256SUMS.txt asset");
    println!(
        "  [GITHUB]   Download URL: {}",
        checksums_asset.browser_download_url
    );

    // Download the checksums file content
    println!("  [GITHUB] Downloading SHA256SUMS.txt content...");
    let checksums_response = client.get(&checksums_asset.browser_download_url).send()?;

    if !checksums_response.status().is_success() {
        println!("  [GITHUB] ✗ Failed to download SHA256SUMS.txt");
        return Err(format!(
            "Failed to download SHA256SUMS.txt: {}",
            checksums_response.status()
        )
        .into());
    }

    let content = checksums_response.text()?;
    println!("  [GITHUB] ✓ SHA256SUMS.txt downloaded successfully");
    println!("  [GITHUB]   Content length: {} bytes", content.len());

    println!("  [GITHUB] ✓ fetch_checksums_file completed successfully");
    Ok(content)
}
