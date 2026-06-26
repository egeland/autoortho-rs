//! Download scenery packages with resume support and SHA256 verification.

use crate::scenery::discovery::SceneryPackage;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Download failed: {0}")]
    Download(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cancelled")]
    Cancelled,
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
    progress_bytes: &std::sync::Arc<AtomicU64>,
) -> Result<PathBuf, DownloadError> {
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
        .map_err(|e| DownloadError::Download(e.to_string()))?;

    // Build request with Range header if resuming
    let mut request = client.get(&package.url);
    if resume_offset > 0 {
        request = request.header("Range", format!("bytes={}-", resume_offset));
    }

    let response = request
        .send()
        .await
        .map_err(|e| DownloadError::Download(e.to_string()))?;

    let status = response.status();
    // 200 = full content, 206 = partial content (resume accepted)
    if !status.is_success() && status.as_u16() != 206 {
        return Err(DownloadError::Download(format!("HTTP {}", status)));
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
            .map_err(DownloadError::Io)?
    } else {
        tokio::fs::File::create(&tmp_dest)
            .await
            .map_err(DownloadError::Io)?
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
            return Err(DownloadError::Cancelled);
        }

        let chunk = chunk.map_err(|e| DownloadError::Download(e.to_string()))?;
        file.write_all(&chunk).await.map_err(DownloadError::Io)?;
        downloaded += chunk.len() as u64;
        progress_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }

    file.flush().await.map_err(DownloadError::Io)?;
    drop(file);

    // Rename .tmp to final (atomic completion marker)
    tokio::fs::rename(&tmp_dest, &dest)
        .await
        .map_err(DownloadError::Io)?;

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
pub fn verify_file_hash(file_path: &Path) -> Result<bool, DownloadError> {
    let hash_path = PathBuf::from(format!("{}.sha256", file_path.display()));
    if !hash_path.exists() {
        return Ok(false); // No hash file — can't verify but not an error
    }

    let hash_content = std::fs::read_to_string(&hash_path)?;
    let expected_hash = hash_content
        .split_whitespace()
        .next()
        .ok_or_else(|| DownloadError::Download("Empty hash file".to_string()))?
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
        Err(DownloadError::Download(format!(
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
) -> Result<bool, DownloadError> {
    let hash_url = format!("{}.sha256", package.url);
    let dest = download_dir.join(format!("{}.sha256", package.filename));

    // Skip if already have it
    if dest.exists() {
        return Ok(true);
    }

    let client = reqwest::Client::builder()
        .user_agent("autoortho-rs/0.1")
        .build()
        .map_err(|e| DownloadError::Download(e.to_string()))?;

    let response = client
        .get(&hash_url)
        .send()
        .await
        .map_err(|e| DownloadError::Download(e.to_string()))?;

    if !response.status().is_success() {
        // No hash file available for this package — that's OK
        debug!("No SHA256 hash available for {}", package.filename);
        return Ok(false);
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| DownloadError::Download(e.to_string()))?;

    std::fs::write(&dest, &bytes)?;
    Ok(true)
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

/// Clean up download artifacts for a region.
pub fn clean_downloads(download_dir: &Path, region_id: &str) -> Result<u64, DownloadError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_partial_downloads_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(!has_partial_downloads(tmp.path(), "sa"));
    }

    #[test]
    fn test_has_partial_downloads_found() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("z_sa_00.zip.tmp"), b"partial").unwrap();
        assert!(has_partial_downloads(tmp.path(), "sa"));
    }

    #[test]
    fn test_clean_downloads_removes_region_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("z_sa_00.zip"), b"data1").unwrap();
        std::fs::write(tmp.path().join("z_na_00.zip"), b"other").unwrap();

        let removed = clean_downloads(tmp.path(), "sa").unwrap();
        assert!(removed > 0);
        assert!(!tmp.path().join("z_sa_00.zip").exists());
        assert!(tmp.path().join("z_na_00.zip").exists());
    }
}
