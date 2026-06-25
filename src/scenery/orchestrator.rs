// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Scenery orchestration — business logic for region discovery, download, and installation.
//!
//! Extracted from `ui/mod.rs` to keep UI layer thin and scenery logic testable
//! independently of the iced framework.

use crate::scenery::discovery;
use crate::scenery::installer;
use crate::ui::scenery_state::{InstalledPackInfo, SceneryRegionInfo};
use tracing::info;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use tokio_util::sync::CancellationToken;

/// Progress tracking for a scenery region download.
///
/// All fields are atomic so the download task can update them
/// while the UI polls for display.
pub struct DownloadProgress {
    pub cancel: CancellationToken,
    pub bytes_downloaded: Arc<AtomicU64>,
    pub current_file: Arc<parking_lot::Mutex<String>>,
    pub files_done: Arc<AtomicU32>,
    pub extract_done: Arc<AtomicU32>,
    pub extract_total: Arc<AtomicU32>,
    pub extracting: Arc<AtomicBool>,
    pub pack_current: Arc<AtomicU32>,
    pub pack_total: Arc<AtomicU32>,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            cancel: CancellationToken::new(),
            bytes_downloaded: Arc::new(AtomicU64::new(0)),
            current_file: Arc::new(parking_lot::Mutex::new(String::new())),
            files_done: Arc::new(AtomicU32::new(0)),
            extract_done: Arc::new(AtomicU32::new(0)),
            extract_total: Arc::new(AtomicU32::new(0)),
            extracting: Arc::new(AtomicBool::new(false)),
            pack_current: Arc::new(AtomicU32::new(0)),
            pack_total: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch available regions from GitHub and list installed packs.
///
/// Returns `(regions, installed_packs)` — regions for the UI to display,
/// installed packs for status display.
pub async fn fetch_regions_and_installed(
    data_dir: &str,
    download_dir: &str,
) -> Result<(Vec<SceneryRegionInfo>, Vec<InstalledPackInfo>), String> {
    let regions = discovery::discover_regions()
        .await
        .map_err(|e| e.to_string())?;

    let dl_path = Path::new(download_dir);
    let ui_regions: Vec<SceneryRegionInfo> = regions
        .iter()
        .map(|r| SceneryRegionInfo {
            id: r.id.clone(),
            name: r.name.clone(),
            version: r.version.clone(),
            package_count: r.packages.len(),
            total_size_bytes: r.packages.iter().map(|p| p.size_bytes).sum(),
            has_partial_download: installer::has_partial_downloads(dl_path, &r.id),
        })
        .collect();

    let packs = installer::list_installed_packs(Path::new(data_dir));
    let ui_packs: Vec<InstalledPackInfo> = packs
        .into_iter()
        .map(|p| InstalledPackInfo {
            id: p.id,
            name: p.name,
            version: p.ver,
        })
        .collect();

    Ok((ui_regions, ui_packs))
}

/// Download and install a scenery region.
///
/// Downloads all packages for the region, verifies SHA256 hashes,
/// extracts ZIPs, and saves pack metadata.
#[allow(clippy::too_many_arguments)]
pub async fn download_and_install_region(
    region_id: &str,
    download_dir: &str,
    data_dir: &str,
    progress: &DownloadProgress,
) -> Result<String, String> {
    // Discover to get download URLs
    let regions = discovery::discover_regions()
        .await
        .map_err(|e| e.to_string())?;

    let region = regions
        .iter()
        .find(|r| r.id == region_id)
        .ok_or_else(|| format!("Region '{}' not found", region_id))?;

    let download_path = Path::new(download_dir);
    let data_path = Path::new(data_dir);
    let mut downloaded_files = Vec::new();

    // Download all packages in the region
    let mut verified = 0u32;
    let mut unverified = 0u32;

    for package in &region.packages {
        if progress.cancel.is_cancelled() {
            return Err("Cancelled".to_string());
        }

        *progress.current_file.lock() = package.filename.clone();

        let path = installer::download_package(
            package,
            download_path,
            &progress.cancel,
            &progress.bytes_downloaded,
        )
        .await
        .map_err(|e| format!("{}", e))?;

        // Download and verify SHA256 hash if available
        let has_hash = installer::download_hash_file(package, download_path)
            .await
            .unwrap_or(false);

        if has_hash {
            match installer::verify_file_hash(&path) {
                Ok(true) => verified += 1,
                Ok(false) => unverified += 1,
                Err(e) => {
                    return Err(format!(
                        "Verification failed for {}: {}",
                        package.filename, e
                    ));
                }
            }
        } else {
            unverified += 1;
        }

        downloaded_files.push(path);
        progress.files_done.fetch_add(1, Ordering::Relaxed);
    }

    // Extract ZIP files
    let total_packs = downloaded_files
        .iter()
        .filter(|p| p.to_string_lossy().ends_with(".zip"))
        .count() as u32;
    progress.pack_total.store(total_packs, Ordering::Relaxed);
    progress.extracting.store(true, Ordering::Relaxed);

    for (pack_idx, path) in downloaded_files.iter().enumerate() {
        if progress.cancel.is_cancelled() {
            return Err("Cancelled".to_string());
        }

        if path.to_string_lossy().ends_with(".zip") {
            let current_pack = pack_idx as u32 + 1;
            progress.pack_current.store(current_pack, Ordering::Relaxed);

            let target = data_path
                .join("scenery")
                .join(format!("z_ao_{}", region_id));
            installer::extract_zip_with_pack_progress(
                path,
                &target,
                progress.extract_done.clone(),
                progress.extract_total.clone(),
                current_pack,
                total_packs,
            )
            .map_err(|e| format!("Extract failed: {}", e))?;
        }
    }
    progress.extracting.store(false, Ordering::Relaxed);
    progress.pack_current.store(total_packs, Ordering::Relaxed);

    // Save metadata
    let info = installer::PackInfo {
        id: region_id.to_string(),
        name: region.name.clone(),
        ver: region.version.clone(),
        ortho_prefix: format!("z_{}_", region_id),
        overlay_prefix: format!("y_{}_overlays", region_id),
        ortho_dirs: vec![],
        info_ver: "v1".to_string(),
    };
    installer::save_pack_info(&info, data_path)
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    let verify_msg = if verified > 0 && unverified == 0 {
        format!(", {} verified", verified)
    } else if verified > 0 {
        format!(", {} verified, {} unverified", verified, unverified)
    } else {
        String::new()
    };

    info!(
        "Installed {} v{} ({} packages{})",
        region.name,
        region.version,
        region.packages.len(),
        verify_msg
    );

    Ok(format!(
        "Installed {} v{} ({} packages{})",
        region.name,
        region.version,
        region.packages.len(),
        verify_msg
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress_defaults() {
        let p = DownloadProgress::new();
        assert!(!p.cancel.is_cancelled());
        assert_eq!(p.bytes_downloaded.load(Ordering::Relaxed), 0);
        assert!(p.current_file.lock().is_empty());
        assert_eq!(p.files_done.load(Ordering::Relaxed), 0);
        assert_eq!(p.extract_done.load(Ordering::Relaxed), 0);
        assert_eq!(p.extract_total.load(Ordering::Relaxed), 0);
        assert!(!p.extracting.load(Ordering::Relaxed));
        assert_eq!(p.pack_current.load(Ordering::Relaxed), 0);
        assert_eq!(p.pack_total.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_download_progress_cancel() {
        let p = DownloadProgress::new();
        assert!(!p.cancel.is_cancelled());
        p.cancel.cancel();
        assert!(p.cancel.is_cancelled());
    }

    #[test]
    fn test_download_progress_files_done() {
        let p = DownloadProgress::new();
        p.files_done.fetch_add(1, Ordering::Relaxed);
        p.files_done.fetch_add(1, Ordering::Relaxed);
        assert_eq!(p.files_done.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_download_progress_extract_tracking() {
        let p = DownloadProgress::new();
        p.extract_total.store(10, Ordering::Relaxed);
        p.extracting.store(true, Ordering::Relaxed);
        p.extract_done.store(5, Ordering::Relaxed);

        assert!(p.extracting.load(Ordering::Relaxed));
        assert_eq!(p.extract_done.load(Ordering::Relaxed), 5);
        assert_eq!(p.extract_total.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_download_progress_pack_tracking() {
        let p = DownloadProgress::new();
        p.pack_total.store(3, Ordering::Relaxed);
        p.pack_current.store(2, Ordering::Relaxed);
        assert_eq!(p.pack_current.load(Ordering::Relaxed), 2);
        assert_eq!(p.pack_total.load(Ordering::Relaxed), 3);
    }
}
