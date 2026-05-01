# Downloader Module Improvement Plan

## Overview
Based on review of `src/downloader.rs`, this plan outlines critical fixes, code quality improvements, and refactoring steps for the `SceneryDownloader` implementation.

## Priority 1: Critical Security Fixes

### 1. Path Traversal in 7z Extraction
- **Issue**: `extract_7z` uses `sevenz_rust::decompress_file` without validating individual entry paths, unlike zip extraction.
- **Fix**: Replace `decompress_file` with `sevenz_rust`'s entry iteration API to validate each path via `validate_extract_path` before extraction.

### 2. Absolute Path Handling in `validate_extract_path`
- **Issue**: Does not block absolute paths in zip entries (e.g., `/etc/passwd`), which can bypass traversal checks on Unix.
- **Fix**: Add `entry_path.is_absolute()` check at the start of `validate_extract_path`.

## Priority 2: Code Quality Fixes

### 3. Unused `_cache_dir` Field
- **Issue**: `SceneryDownloader` stores `_cache_dir` but never uses it; `new` method accepts unnecessary parameter.
- **Fix**: Remove the field and parameter, or implement caching logic.

### 4. Stubbed `set_progress_callback`
- **Issue**: Accepts callback but does not store it, so progress updates never fire.
- **Fix**: Add `progress_callback: Option<Box<dyn Fn(u64, u64) + 'static>>` field to `SceneryDownloader` and invoke during download/extract.

### 5. Simulated `download` Method
- **Issue**: Async stub returns filename without downloading; does not ensure `download_dir` exists.
- **Fix**: Implement actual HTTP download via `reqwest` with progress reporting; add `tokio::fs::create_dir_all(&self.download_dir).await`.

### 6. Redundant Path Traversal Checks
- **Issue**: `validate_extract_path` checks `..` in string after already checking `ParentDir` components.
- **Fix**: Remove redundant `entry_str.contains("..")` check.

## Priority 3: Minor Improvements

### 7. Error Message Clarity
- Update "Unsupported format" to include actual extension.
- Update "Empty URL" to specify which URL was empty.

### 8. Test Coverage Gaps
- Add tests for 7z extraction (normal/malicious).
- Add test for absolute paths in zip entries.
- Add test for `download` creating actual files (post-implementation).

### 9. Code Duplication
- Extract common extraction logic between `extract_zip` and `extract_zip_from_memory` into a helper.

## Next Steps
1. Implement critical security fixes (Priority 1)
2. Address code quality issues (Priority 2)
3. Add minor improvements and tests (Priority 3)
4. Run `cargo fmt`, `cargo clippy`, `cargo test` after each change.
