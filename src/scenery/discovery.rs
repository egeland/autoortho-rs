//! Discover available scenery packs from the GitHub releases API.
//!
//! Each GitHub release contains packages for a single region (e.g., NA, SA, EUR).
//! The region is identified by a `*_info.json` asset in the release.
//! Multiple releases may exist for the same region — we show the latest.

use log::{debug, info};
use serde::Deserialize;
use thiserror::Error;

const RELEASES_URL: &str = "https://api.github.com/repos/kubilus1/autoortho-scenery/releases";
const USER_AGENT: &str = "autoortho-rs/0.1";

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("No releases found")]
    NoReleases,
}

/// A scenery region available for download.
#[derive(Debug, Clone)]
pub struct SceneryRegion {
    /// Region identifier (e.g., "na", "sa", "eur")
    pub id: String,
    /// Human-readable name (e.g., "North America Ortho Set")
    pub name: String,
    /// Version tag (e.g., "0.0.54")
    pub version: String,
    /// List of downloadable packages in this region
    pub packages: Vec<SceneryPackage>,
    /// URL to the info.json for this region
    pub info_url: String,
}

/// A downloadable package (ZIP file) within a region.
#[derive(Debug, Clone)]
pub struct SceneryPackage {
    /// Package filename (e.g., "z_na_00.zip")
    pub filename: String,
    /// Download URL
    pub url: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Package type: "z" (ortho) or "y" (overlay)
    pub pkg_type: String,
}

/// Metadata from *_info.json
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RegionInfo {
    id: String,
    name: String,
    #[serde(default)]
    ortho_prefix: String,
    #[serde(default)]
    overlay_prefix: String,
}

/// Fetch available scenery regions from GitHub releases.
///
/// Discovers regions by finding `*_info.json` assets, fetching them for
/// the region name, and grouping packages. Shows only the latest version
/// per region.
pub async fn discover_regions() -> Result<Vec<SceneryRegion>, DiscoveryError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| DiscoveryError::Http(e.to_string()))?;

    info!("Fetching releases from GitHub...");

    let response = client
        .get(RELEASES_URL)
        .send()
        .await
        .map_err(|e| DiscoveryError::Http(e.to_string()))?;

    if !response.status().is_success() {
        return Err(DiscoveryError::Http(format!("HTTP {}", response.status())));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| DiscoveryError::Parse(e.to_string()))?;

    if releases.is_empty() {
        return Err(DiscoveryError::NoReleases);
    }

    info!("Found {} releases, identifying regions...", releases.len());

    // For each release, find the *_info.json and extract the region
    let mut regions = std::collections::HashMap::<String, SceneryRegion>::new();

    for release in &releases {
        // Find *_info.json asset (skip test_ prefixed)
        let info_asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with("_info.json") && !a.name.starts_with("test_"));

        let info_asset = match info_asset {
            Some(a) => a,
            None => continue, // Release has no info.json — skip
        };

        // Extract region ID from filename: "na_info.json" → "na"
        let region_id = info_asset.name.trim_end_matches("_info.json").to_string();

        // Skip if we already have a newer version (releases are sorted newest first)
        if regions.contains_key(&region_id) {
            continue;
        }

        // Fetch the info.json to get the human-readable name
        let region_name = match fetch_region_info(&client, &info_asset.browser_download_url).await {
            Ok(info) => info.name,
            Err(e) => {
                debug!("Failed to fetch {}: {}", info_asset.name, e);
                format_region_name(&region_id)
            }
        };

        // Collect ZIP packages for this release
        let packages: Vec<SceneryPackage> = release
            .assets
            .iter()
            .filter(|a| a.name.ends_with(".zip") || a.name.contains(".zip."))
            .filter(|a| !a.name.ends_with(".sha256")) // Skip hash files
            .map(|a| {
                let pkg_type = if a.name.starts_with("y_") {
                    "y".to_string()
                } else {
                    "z".to_string()
                };
                SceneryPackage {
                    filename: a.name.clone(),
                    url: a.browser_download_url.clone(),
                    size_bytes: a.size,
                    pkg_type,
                }
            })
            .collect();

        if packages.is_empty() {
            continue;
        }

        info!(
            "  Region '{}' ({}): {} packages, v{}",
            region_name,
            region_id,
            packages.len(),
            release.tag_name
        );

        regions.insert(
            region_id.clone(),
            SceneryRegion {
                id: region_id,
                name: region_name,
                version: release.tag_name.clone(),
                packages,
                info_url: info_asset.browser_download_url.clone(),
            },
        );
    }

    let mut result: Vec<SceneryRegion> = regions.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));

    info!("Discovered {} scenery regions", result.len());

    Ok(result)
}

/// Fetch and parse a *_info.json file.
async fn fetch_region_info(
    client: &reqwest::Client,
    url: &str,
) -> Result<RegionInfo, DiscoveryError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| DiscoveryError::Http(e.to_string()))?;

    response
        .json::<RegionInfo>()
        .await
        .map_err(|e| DiscoveryError::Parse(e.to_string()))
}

/// Generate a fallback display name from a region ID.
fn format_region_name(id: &str) -> String {
    match id {
        "na" => "North America".to_string(),
        "sa" => "South America".to_string(),
        "eur" => "Europe".to_string(),
        "afr" => "Africa".to_string(),
        "asi" => "Asia".to_string(),
        "aus_pac" => "Australia & Pacific".to_string(),
        "ant" => "Antarctica".to_string(),
        "test" => "Test Region".to_string(),
        other => other.to_uppercase(),
    }
}

// --- GitHub API types ---

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_id_from_filename() {
        let filename = "na_info.json";
        let id = filename.trim_end_matches("_info.json");
        assert_eq!(id, "na");
    }

    #[test]
    fn test_region_id_from_sa() {
        let filename = "sa_info.json";
        let id = filename.trim_end_matches("_info.json");
        assert_eq!(id, "sa");
    }

    #[test]
    fn test_format_region_name() {
        assert_eq!(format_region_name("na"), "North America");
        assert_eq!(format_region_name("sa"), "South America");
        assert_eq!(format_region_name("eur"), "Europe");
        assert_eq!(format_region_name("unknown"), "UNKNOWN");
    }

    #[test]
    fn test_skip_test_regions() {
        let name = "test_eur_info.json";
        assert!(name.starts_with("test_"));
    }

    #[test]
    fn test_package_type_detection() {
        assert!(!"z_na_00.zip".starts_with("y_"));
        assert!("y_na_overlays.zip.00".starts_with("y_"));
    }
}
