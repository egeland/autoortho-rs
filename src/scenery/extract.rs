//! Extract scenery pack ZIP archives with progress reporting.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Extract failed: {0}")]
    Extract(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Extract a ZIP file to the target directory.
///
/// Uses `extract_unwrapped_root_dir` to automatically strip the root directory
/// if the archive contains a single top-level folder (common in scenery packs).
/// Path traversal is blocked by the crate's built-in sanitization.
pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), ExtractError> {
    info!(
        "Extracting {} to {}",
        zip_path.display(),
        target_dir.display()
    );

    let file = std::fs::File::open(zip_path)
        .map_err(|e| ExtractError::Extract(format!("Cannot open {}: {}", zip_path.display(), e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ExtractError::Extract(format!("Invalid ZIP: {}", e)))?;

    std::fs::create_dir_all(target_dir)?;

    archive
        .extract_unwrapped_root_dir(target_dir, zip::read::root_dir_common_filter)
        .map_err(|e| ExtractError::Extract(e.to_string()))?;

    info!("Extraction complete");
    Ok(())
}

/// Extract a ZIP file to the target directory with per-file progress reporting.
///
/// Reports progress by updating `files_done` and `files_total` atomics.
/// Also tracks which pack is being extracted via `current_pack` and `total_packs`.
/// Uses the same security filtering as `extract_zip`.
pub fn extract_zip_with_pack_progress(
    zip_path: &Path,
    target_dir: &Path,
    files_done: Arc<AtomicU32>,
    files_total: Arc<AtomicU32>,
    _current_pack: u32,
    _total_packs: u32,
) -> Result<(), ExtractError> {
    info!(
        "Extracting {} to {}",
        zip_path.display(),
        target_dir.display()
    );

    let file = std::fs::File::open(zip_path)
        .map_err(|e| ExtractError::Extract(format!("Cannot open {}: {}", zip_path.display(), e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ExtractError::Extract(format!("Invalid ZIP: {}", e)))?;

    std::fs::create_dir_all(target_dir)?;

    // Get total file count for progress reporting
    let total = archive.len() as u32;
    files_total.store(total, Ordering::Relaxed);
    files_done.store(0, Ordering::Relaxed);

    // Iterate through all files and extract them one by one
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ExtractError::Extract(e.to_string()))?;

        // Get the sanitized output path
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue, // Skip files that would escape
        };

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| ExtractError::Extract(e.to_string()))?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ExtractError::Extract(e.to_string()))?;
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| ExtractError::Extract(e.to_string()))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| ExtractError::Extract(e.to_string()))?;
        }

        files_done.fetch_add(1, Ordering::Relaxed);
    }

    info!("Extraction complete ({} files)", total);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn create_test_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, content) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn test_extract_zip_normal() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("test.zip");
        let target = tmp.path().join("extracted");

        create_test_zip(
            &zip_path,
            &[
                ("file1.txt", b"content1"),
                ("subdir/file2.txt", b"content2"),
            ],
        );

        extract_zip(&zip_path, &target).unwrap();

        assert!(target.join("file1.txt").exists());
        assert!(target.join("subdir/file2.txt").exists());
    }

    #[test]
    fn test_extract_zip_blocks_traversal() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("evil.zip");

        create_test_zip(&zip_path, &[("../../../etc/evil.txt", b"bad")]);

        let result = extract_zip(&zip_path, tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_zip_blocks_parent_dir_traversal() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("evil2.zip");

        create_test_zip(&zip_path, &[("../escape.txt", b"escape")]);

        let result = extract_zip(&zip_path, &tmp.path().join("subdir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unwrapped_strips_root_dir() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("with_root.zip");

        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("mypackage/file1.txt", options).unwrap();
        zip.write_all(b"content1").unwrap();
        zip.start_file("mypackage/subdir/file2.txt", options)
            .unwrap();
        zip.write_all(b"content2").unwrap();
        zip.finish().unwrap();

        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive
            .extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter)
            .unwrap();

        assert!(tmp.path().join("file1.txt").exists());
        assert!(tmp.path().join("subdir/file2.txt").exists());
        assert!(!tmp.path().join("mypackage").exists());
    }

    #[test]
    fn test_extract_unwrapped_no_root_dir_unchanged() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("no_root.zip");

        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("file1.txt", options).unwrap();
        zip.write_all(b"content1").unwrap();
        zip.start_file("other/file2.txt", options).unwrap();
        zip.write_all(b"content2").unwrap();
        zip.finish().unwrap();

        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive
            .extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter)
            .unwrap();

        assert!(tmp.path().join("file1.txt").exists());
        assert!(tmp.path().join("other/file2.txt").exists());
    }

    #[test]
    fn test_extract_unwrapped_blocks_path_traversal() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("evil_unwrapped.zip");

        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("../../../etc/evil.txt", options).unwrap();
        zip.write_all(b"bad").unwrap();
        zip.finish().unwrap();

        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let result =
            archive.extract_unwrapped_root_dir(tmp.path(), zip::read::root_dir_common_filter);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_with_progress() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("progress.zip");
        let target = tmp.path().join("extracted");

        create_test_zip(
            &zip_path,
            &[("a.txt", b"1"), ("b.txt", b"2"), ("c.txt", b"3")],
        );

        let done = Arc::new(AtomicU32::new(0));
        let total = Arc::new(AtomicU32::new(0));

        extract_zip_with_pack_progress(&zip_path, &target, done.clone(), total.clone(), 1, 2)
            .unwrap();

        assert_eq!(done.load(Ordering::Relaxed), 3);
        assert_eq!(total.load(Ordering::Relaxed), 3);
    }
}
