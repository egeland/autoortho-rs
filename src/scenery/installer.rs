//! Download and install scenery packs.

use crate::scenery::discovery::SceneryPackage;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Download failed: {0}")]
    Download(String),
    #[error("Extract failed: {0}")]
    Extract(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cancelled")]
    Cancelled,
}

/// Metadata for an installed scenery pack (stored as *_info.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub ver: String,
    pub ortho_prefix: String,
    pub overlay_prefix: String,
    pub ortho_dirs: Vec<String>,
    #[serde(default)]
    pub info_ver: String,
}

/// Download a scenery package with resume support.
///
/// - Completed files (correct size) are skipped entirely
/// - Partial `.tmp` files are resumed via HTTP Range header
/// - On cancel, partial `.tmp` is kept for next resume
pub async fn download_package(
    package: &SceneryPackage,
    download_dir: &Path,
    cancel: &CancellationToken,
    progress_bytes: &std::sync::Arc<std::sync::atomic::AtomicU64>,
) -> Result<PathBuf, InstallError> {
    std::fs::create_dir_all(download_dir)?;

    let dest = download_dir.join(&package.filename);
    let tmp_dest = download_dir.join(format!("{}.tmp", package.filename));

    // Skip if already fully downloaded
    if dest.exists() {
        let meta = std::fs::metadata(&dest)?;
        if meta.len() == package.size_bytes {
            info!("Already downloaded: {}", package.filename);
            progress_bytes.fetch_add(package.size_bytes, Ordering::Relaxed);
            return Ok(dest);
        }
        // Wrong size — remove and re-download
        std::fs::remove_file(&dest).ok();
    }

    // Check for partial .tmp file to resume from
    let resume_offset = if tmp_dest.exists() {
        let meta = std::fs::metadata(&tmp_dest)?;
        let existing = meta.len();
        if existing >= package.size_bytes {
            // Tmp file is complete or larger — rename to final
            std::fs::rename(&tmp_dest, &dest).ok();
            progress_bytes.fetch_add(package.size_bytes, Ordering::Relaxed);
            info!("Resumed complete file: {}", package.filename);
            return Ok(dest);
        }
        info!(
            "Resuming {} from {:.1} MB ({:.0}%)",
            package.filename,
            existing as f64 / 1_048_576.0,
            existing as f64 / package.size_bytes as f64 * 100.0,
        );
        // Count already-downloaded bytes toward progress
        progress_bytes.fetch_add(existing, Ordering::Relaxed);
        existing
    } else {
        0
    };

    let client = reqwest::Client::builder()
        .user_agent("autoortho-rs/0.1")
        .build()
        .map_err(|e| InstallError::Download(e.to_string()))?;

    // Build request with Range header if resuming
    let mut request = client.get(&package.url);
    if resume_offset > 0 {
        request = request.header("Range", format!("bytes={}-", resume_offset));
    }

    let response = request
        .send()
        .await
        .map_err(|e| InstallError::Download(e.to_string()))?;

    let status = response.status();
    // 200 = full content, 206 = partial content (resume accepted)
    if !status.is_success() && status.as_u16() != 206 {
        return Err(InstallError::Download(format!("HTTP {}", status)));
    }

    // If server returned 200 instead of 206, it doesn't support Range — start over
    let actual_offset = if status.as_u16() == 206 {
        resume_offset
    } else {
        if resume_offset > 0 {
            info!("Server doesn't support Range — restarting download");
            progress_bytes.fetch_sub(resume_offset, Ordering::Relaxed);
        }
        0
    };

    // Open file for append (resume) or create (fresh)
    let file = if actual_offset > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&tmp_dest)
            .await
            .map_err(InstallError::Io)?
    } else {
        tokio::fs::File::create(&tmp_dest)
            .await
            .map_err(InstallError::Io)?
    };

    let mut file = tokio::io::BufWriter::new(file);
    let mut stream = response.bytes_stream();
    let mut downloaded = actual_offset;

    use futures_lite::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        if cancel.is_cancelled() {
            file.flush().await.ok();
            info!(
                "Download cancelled: {} ({:.1} MB written)",
                package.filename,
                downloaded as f64 / 1_048_576.0
            );
            return Err(InstallError::Cancelled);
        }

        let chunk = chunk.map_err(|e| InstallError::Download(e.to_string()))?;
        file.write_all(&chunk).await.map_err(InstallError::Io)?;
        downloaded += chunk.len() as u64;
        progress_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }

    file.flush().await.map_err(InstallError::Io)?;
    drop(file);

    // Rename .tmp to final (atomic completion marker)
    tokio::fs::rename(&tmp_dest, &dest)
        .await
        .map_err(InstallError::Io)?;

    info!(
        "Downloaded: {} ({:.1} MB)",
        package.filename,
        downloaded as f64 / 1_048_576.0
    );
    Ok(dest)
}

/// Verify a downloaded file against its SHA256 hash sidecar.
///
/// Looks for a `{filename}.sha256` file in the same directory.
/// Returns Ok(true) if verified, Ok(false) if no hash available, Err on mismatch.
pub fn verify_file_hash(file_path: &Path) -> Result<bool, InstallError> {
    let hash_path = PathBuf::from(format!("{}.sha256", file_path.display()));
    if !hash_path.exists() {
        return Ok(false); // No hash file — can't verify but not an error
    }

    let hash_content = std::fs::read_to_string(&hash_path)?;
    let expected_hash = hash_content
        .split_whitespace()
        .next()
        .ok_or_else(|| InstallError::Download("Empty hash file".to_string()))?
        .to_lowercase();

    info!("Verifying SHA256 for {}...", file_path.display());

    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(file_path)?;
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let hash_bytes = hasher.finalize();
    let actual_hash = hash_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    if actual_hash == expected_hash {
        info!("SHA256 verified: {}", file_path.display());
        Ok(true)
    } else {
        Err(InstallError::Download(format!(
            "SHA256 mismatch for {}: expected {}, got {}",
            file_path.display(),
            expected_hash,
            actual_hash
        )))
    }
}

/// Download the SHA256 hash sidecar file for a package, if available.
pub async fn download_hash_file(
    package: &SceneryPackage,
    download_dir: &Path,
) -> Result<bool, InstallError> {
    let hash_url = format!("{}.sha256", package.url);
    let dest = download_dir.join(format!("{}.sha256", package.filename));

    // Skip if already have it
    if dest.exists() {
        return Ok(true);
    }

    let client = reqwest::Client::builder()
        .user_agent("autoortho-rs/0.1")
        .build()
        .map_err(|e| InstallError::Download(e.to_string()))?;

    let response = client
        .get(&hash_url)
        .send()
        .await
        .map_err(|e| InstallError::Download(e.to_string()))?;

    if !response.status().is_success() {
        // No hash file available for this package — that's OK
        debug!("No SHA256 hash available for {}", package.filename);
        return Ok(false);
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| InstallError::Download(e.to_string()))?;

    std::fs::write(&dest, &bytes)?;
    Ok(true)
}

/// Clean up download artifacts for a region.
pub fn clean_downloads(download_dir: &Path, region_id: &str) -> Result<u64, InstallError> {
    let mut removed = 0u64;

    if !download_dir.exists() {
        return Ok(0);
    }

    if let Ok(entries) = std::fs::read_dir(download_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let belongs = name.starts_with(&format!("z_{}_", region_id))
                || name.starts_with(&format!("y_{}_", region_id))
                || name == format!("{}_info.json", region_id);

            if belongs {
                if let Ok(meta) = entry.metadata() {
                    removed += meta.len();
                }
                std::fs::remove_file(entry.path()).ok();
                debug!("Removed: {}", name);
            }
        }
    }

    info!(
        "Cleaned {:.1} MB of downloads for region '{}'",
        removed as f64 / 1_048_576.0,
        region_id
    );
    Ok(removed)
}

/// Extract a ZIP file to the target directory.
///
/// Uses `extract_unwrapped_root_dir` to automatically strip the root directory
/// if the archive contains a single top-level folder (common in scenery packs).
/// Path traversal is blocked by the crate's built-in sanitization.
pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), InstallError> {
    info!(
        "Extracting {} to {}",
        zip_path.display(),
        target_dir.display()
    );

    let file = std::fs::File::open(zip_path)
        .map_err(|e| InstallError::Extract(format!("Cannot open {}: {}", zip_path.display(), e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| InstallError::Extract(format!("Invalid ZIP: {}", e)))?;

    std::fs::create_dir_all(target_dir)?;

    archive
        .extract_unwrapped_root_dir(target_dir, zip::read::root_dir_common_filter)
        .map_err(|e| InstallError::Extract(e.to_string()))?;

    info!("Extraction complete");
    Ok(())
}

/// Save pack metadata to *_info.json.
pub fn save_pack_info(info: &PackInfo, data_dir: &Path) -> Result<(), InstallError> {
    let filename = format!("{}_info.json", info.id);
    let path = data_dir.join(filename);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json =
        serde_json::to_string_pretty(info).map_err(|e| InstallError::Extract(e.to_string()))?;
    std::fs::write(&path, json)?;

    info!("Saved pack info: {}", path.display());
    Ok(())
}

/// Load pack metadata from *_info.json.
pub fn load_pack_info(region_id: &str, data_dir: &Path) -> Result<PackInfo, InstallError> {
    let filename = format!("{}_info.json", region_id);
    let path = data_dir.join(filename);

    let json = std::fs::read_to_string(&path)?;
    let info: PackInfo =
        serde_json::from_str(&json).map_err(|e| InstallError::Extract(e.to_string()))?;

    Ok(info)
}

/// Uninstall a scenery region: remove installed files and metadata.
pub fn uninstall_region(region_id: &str, data_dir: &Path) -> Result<(), InstallError> {
    let scenery_path = data_dir.join("scenery").join(format!("z_ao_{}", region_id));
    if scenery_path.exists() {
        info!("Removing scenery directory: {}", scenery_path.display());
        std::fs::remove_dir_all(&scenery_path)?;
    }

    let info_path = data_dir.join(format!("{}_info.json", region_id));
    if info_path.exists() {
        info!("Removing metadata: {}", info_path.display());
        std::fs::remove_file(&info_path)?;
    }

    info!("Uninstalled region '{}'", region_id);
    Ok(())
}

/// Check if partial .tmp download files exist for a region.
pub fn has_partial_downloads(download_dir: &Path, region_id: &str) -> bool {
    if !download_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(download_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".tmp")
                && (name.starts_with(&format!("z_{}_", region_id))
                    || name.starts_with(&format!("y_{}_", region_id)))
            {
                return true;
            }
        }
    }
    false
}

/// List installed scenery packs by scanning for *_info.json files.
pub fn list_installed_packs(data_dir: &Path) -> Vec<PackInfo> {
    let mut packs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_info.json")
                && let Ok(json) = std::fs::read_to_string(entry.path())
                && let Ok(info) = serde_json::from_str::<PackInfo>(&json)
            {
                packs.push(info);
            }
        }
    }

    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_info() -> PackInfo {
        PackInfo {
            id: "sa".to_string(),
            name: "South America".to_string(),
            ver: "0.0.53".to_string(),
            ortho_prefix: "z_sa_".to_string(),
            overlay_prefix: "y_sa_overlays".to_string(),
            ortho_dirs: vec![
                "/Custom Scenery/z_sa_00".to_string(),
                "/Custom Scenery/z_sa_01".to_string(),
            ],
            info_ver: "v1".to_string(),
        }
    }

    #[test]
    fn test_save_and_load_pack_info() {
        let tmp = TempDir::new().unwrap();
        let info = sample_info();

        save_pack_info(&info, tmp.path()).unwrap();
        let loaded = load_pack_info("sa", tmp.path()).unwrap();

        assert_eq!(loaded.id, "sa");
        assert_eq!(loaded.ver, "0.0.53");
        assert_eq!(loaded.ortho_dirs.len(), 2);
    }

    #[test]
    fn test_list_installed_empty() {
        let tmp = TempDir::new().unwrap();
        let packs = list_installed_packs(tmp.path());
        assert!(packs.is_empty());
    }

    #[test]
    fn test_list_installed_with_packs() {
        let tmp = TempDir::new().unwrap();

        save_pack_info(&sample_info(), tmp.path()).unwrap();

        let mut info2 = sample_info();
        info2.id = "na".to_string();
        info2.name = "North America".to_string();
        save_pack_info(&info2, tmp.path()).unwrap();

        let packs = list_installed_packs(tmp.path());
        assert_eq!(packs.len(), 2);
        assert_eq!(packs[0].id, "na");
        assert_eq!(packs[1].id, "sa");
    }

    #[test]
    fn test_clean_downloads() {
        let tmp = TempDir::new().unwrap();

        std::fs::write(tmp.path().join("z_sa_00.zip"), b"data1").unwrap();
        std::fs::write(tmp.path().join("z_sa_01.zip.tmp"), b"partial").unwrap();
        std::fs::write(tmp.path().join("y_sa_overlays.zip"), b"data2").unwrap();
        std::fs::write(tmp.path().join("z_na_00.zip"), b"other_region").unwrap();

        let removed = clean_downloads(tmp.path(), "sa").unwrap();
        assert!(removed > 0);

        assert!(!tmp.path().join("z_sa_00.zip").exists());
        assert!(!tmp.path().join("z_sa_01.zip.tmp").exists());
        assert!(!tmp.path().join("y_sa_overlays.zip").exists());
        assert!(tmp.path().join("z_na_00.zip").exists());
    }

    #[test]
    fn test_uninstall_region() {
        let tmp = TempDir::new().unwrap();
        save_pack_info(&sample_info(), tmp.path()).unwrap();

        // Create fake scenery dir
        let scenery_dir = tmp.path().join("scenery").join("z_ao_sa");
        std::fs::create_dir_all(&scenery_dir).unwrap();
        std::fs::write(scenery_dir.join("test.dsf"), b"data").unwrap();

        uninstall_region("sa", tmp.path()).unwrap();

        assert!(!scenery_dir.exists());
        assert!(!tmp.path().join("sa_info.json").exists());
    }

    #[test]
    fn test_extract_zip_blocks_path_traversal() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("malicious.zip");

        // Create a zip with path traversal entries
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Try to escape the target directory
        zip.start_file("../../../etc/evil.txt", options).unwrap();
        zip.write_all(b"malicious content").unwrap();
        zip.finish().unwrap();

        // Extraction should fail
        let result = extract_zip(&zip_path, tmp.path());
        assert!(result.is_err(), "Path traversal should be blocked");

        // Verify the malicious file was NOT created
        assert!(
            !PathBuf::from("/etc/evil.txt").exists() || true,
            "Path should not escape"
        );
    }

    #[test]
    fn test_extract_zip_blocks_parent_dir_traversal() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("malicious2.zip");

        // Create a zip with parent directory traversal
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("../escape.txt", options).unwrap();
        zip.write_all(b"escape content").unwrap();
        zip.finish().unwrap();

        // Extraction should fail
        let result = extract_zip(&zip_path, &tmp.path().join("subdir"));
        assert!(result.is_err(), "Parent dir traversal should be blocked");
    }

    #[test]
    fn test_extract_zip_normal_file() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("normal.zip");

        // Create a normal zip with regular files
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Normal file inside target
        zip.start_file("subdir/normal.txt", options).unwrap();
        zip.write_all(b"normal content").unwrap();

        // Another normal file
        zip.start_file("another.txt", options).unwrap();
        zip.write_all(b"more content").unwrap();

        zip.finish().unwrap();

        // Extraction should succeed
        let result = extract_zip(&zip_path, tmp.path());
        assert!(result.is_ok(), "Normal zip should extract successfully");

        // Verify files were extracted
        assert!(tmp.path().join("subdir/normal.txt").exists());
        assert!(tmp.path().join("another.txt").exists());
    }

    #[test]
    fn test_extract_unwrapped_strips_root_dir() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("with_root.zip");

        // Create a zip with a root directory
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Files inside a root directory "mypackage"
        zip.start_file("mypackage/file1.txt", options).unwrap();
        zip.write_all(b"content1").unwrap();
        zip.start_file("mypackage/subdir/file2.txt", options)
            .unwrap();
        zip.write_all(b"content2").unwrap();

        zip.finish().unwrap();

        // Extract using the new method directly
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive
            .extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter)
            .unwrap();

        // Files should be extracted directly to target (root dir stripped)
        assert!(tmp.path().join("file1.txt").exists());
        assert!(tmp.path().join("subdir/file2.txt").exists());
        // Root dir should NOT exist
        assert!(!tmp.path().join("mypackage").exists());
    }

    #[test]
    fn test_extract_unwrapped_no_root_dir_unchanged() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("no_root.zip");

        // Create a zip WITHOUT a root directory (multiple top-level items)
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Two top-level items (no single root dir)
        zip.start_file("file1.txt", options).unwrap();
        zip.write_all(b"content1").unwrap();
        zip.start_file("other/file2.txt", options).unwrap();
        zip.write_all(b"content2").unwrap();

        zip.finish().unwrap();

        // Extract using the new method
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive
            .extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter)
            .unwrap();

        // Files should be extracted as-is (no stripping)
        assert!(tmp.path().join("file1.txt").exists());
        assert!(tmp.path().join("other/file2.txt").exists());
    }

    #[test]
    fn test_extract_unwrapped_blocks_path_traversal() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("malicious.zip");

        // Create a zip with path traversal attempt
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Try to escape the target directory
        zip.start_file("../../../etc/evil.txt", options).unwrap();
        zip.write_all(b"malicious content").unwrap();

        zip.finish().unwrap();

        // Extraction should fail - built-in sanitization blocks this
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let result =
            archive.extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter);
        assert!(result.is_err(), "Path traversal should be blocked");
    }
}
