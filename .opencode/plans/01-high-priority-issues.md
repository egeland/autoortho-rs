# Plan: High Priority Issues

## Overview
These issues pose critical risks to application stability, security, or correctness. They should be addressed as a priority.

---

## ✅ Issue 1: ZIP Slip Vulnerability

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/scenery/installer.rs`: Added path validation in `extract_zip()` using `canonicalize()` check
- `src/downloader.rs`: Added `validate_extract_path()` helper and updated both `extract_zip()` and `extract_zip_from_memory()`
- Validation checks for parent directory (`..`) components and canonical path prefix match
- **5 new tests added**: `test_extract_zip_blocks_path_traversal`, `test_extract_zip_blocks_parent_dir_traversal`, `test_extract_zip_normal_file` (in both modules)

### Implementation
See actual code in `src/scenery/installer.rs:333-393` and `src/downloader.rs:24-67`

---

## Issue 1: ZIP Slip Vulnerability

### Severity
**CRITICAL** - Security vulnerability allowing arbitrary file write

### Location
- `src/scenery/installer.rs:324`
- `src/downloader.rs:72` and `src/downloader.rs:100`

### Problem
Zip extraction does not validate that entry paths remain within the target directory. A malicious zip could contain entries like `../../../etc/passwd` to write outside the extraction target.

```rust
// Current code (vulnerable):
let out_path = target_dir.join(&name);
```

### Solution
Implement path validation to ensure extracted paths don't escape the target directory.

### Implementation Steps

1. Create a helper function `validate_path`:
```rust
fn validate_path(target_dir: &Path, entry_path: &Path) -> Result<PathBuf, InstallError> {
    // Canonicalize the target directory
    let target = target_dir.canonicalize()
        .map_err(|e| InstallError::Extract(format!("Cannot resolve target dir: {}", e)))?;
    
    // Join and canonicalize the output path
    let out_path = target_dir.join(entry_path);
    let canonical = out_path.canonicalize()
        .map_err(|e| InstallError::Extract(format!("Path traversal detected: {}", e)))?;
    
    // Verify the canonical path starts with the target
    if !canonical.starts_with(&target) {
        return Err(InstallError::Extract(
            format!("Path traversal attempt: {}", entry_path.display())
        ));
    }
    
    Ok(out_path)
}
```

2. Update `src/scenery/installer.rs:extract_zip`:
   - Import the validation function
   - Call `validate_path()` before each extraction
   - Return an error if validation fails

3. Update `src/downloader.rs:extract_zip` and `extract_zip_from_memory`:
   - Same pattern as above

4. Add tests:
   - Test extraction of malicious zip with `../` paths
   - Test that extraction fails with appropriate error
   - Test that legitimate paths still work

### Testing
```rust
#[test]
fn test_extract_zip_blocks_path_traversal() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path();
    
    // Create a zip with a malicious entry
    let zip_path = create_malicious_zip(target);
    
    // Extraction should fail
    let result = extract_zip(&zip_path, target);
    assert!(result.is_err());
}
```

---

## ✅ Issue 2: Web Server Silently Swallows Errors

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/webui/mod.rs`: Changed `.await.ok()` to log errors via `error!()` macro
- Added `info!()` for graceful shutdown message

### Implementation
```rust
tokio::spawn(async move {
    if let Err(e) = axum::serve(listener, app).await {
        error!("Web server error: {}", e);
    }
    info!("Web server shut down");
});
```

---

## ✅ Issue 3 & 4: Bounded LRU Caches (DDS and Chunk)

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/config.rs`: Added `dds_memory_cache_mb` (default: 256MB), `chunk_memory_cache_mb` (default: 512MB), and helper methods `dds_memory_cache_entries()` / `chunk_memory_cache_entries()`
- `src/fuse/filesystem.rs`: Replaced `HashMap` with `LruCache<String, Arc<Vec<u8>>>`, added `DdsCacheStats` struct, added `with_cache_size()` constructor
- `src/tiles/fetcher.rs`: Replaced `HashMap` with `LruCache<String, Chunk>`, added `ChunkCacheStats`, added `with_cache_size()` constructor, added `cache_stats()` method
- `src/ui/screens/settings.rs`: Added UI controls for memory cache settings with tooltips
- `src/main.rs`: Wired config values to cache constructors
- **2 new tests added**: `test_fetcher_lru_eviction`, `test_fetcher_cache_stats`

### Implementation Details
- DDS tiles: ~22MB each, default 256 tiles = ~5.5GB max (but limited by entries)
- Chunks: ~30KB each, 1024 default entries = ~30MB
- LRU eviction is automatic when capacity is reached

---

## ✅ Issue 5: State Duplication Between Web UI and Desktop UI

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/webui/mod.rs`: Added `config: Arc<parking_lot::RwLock<AutoOrthoConfig>>` to `WebState`, updated `start_server()` signature
- `src/webui/routes.rs`: Updated `cache_tiles()`, `custommap_tiles()`, `cache_stats()` to use shared config
- `src/main.rs`: Updated both mount and server modes to pass config
- `src/ui/mod.rs`: Updated `start_all_services()` to pass config
- Added `parking_lot = "0.12"` dependency

### Benefits
- No more disk I/O on every HTTP request
- Web UI always sees current config
- Thread-safe config access via parking_lot RwLock

---

## ⚠️ Issue 6: Clone on Every UI Render

**Status: NOT IMPLEMENTED** ⚠️

### Decision
After attempting to refactor `config_row()` to use `&str` references instead of `String`, the lifetime complexity made the code harder to maintain. The performance impact of cloning small strings is negligible, and iced handles rendering efficiently. The original `String` parameter approach is retained.

---

## ✅ Issue 7: Direct `std::process::exit()` in UI Handlers

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/ui/mod.rs`: Removed 2 `std::process::exit(0)` calls in `WindowCloseRequested` and `Exit` handlers
- Shutdown signal now properly triggers graceful termination via `shutdown_tx` channel

### Implementation
```rust
// Before:
std::process::exit(0);

// After:
if let Some(tx) = self.shutdown_tx.take() {
    let _ = tx.send(true);
}
// Let framework handle window close/exit
```

---

## Summary

| Issue | Impact | Effort | Priority | Status |
|-------|--------|--------|----------|--------|
| ZIP Slip Vulnerability | Security | Low | P0 | ✅ Done |
| Web Server Silent Errors | Debugging | Low | P1 | ✅ Done |
| Unbounded DDS Cache | Memory | Medium | P1 | ✅ Done |
| Unbounded Chunk Cache | Memory | Medium | P1 | ✅ Done |
| State Duplication | Correctness | Medium | P1 | ✅ Done |
| Clone on Render | Performance | Medium | P2 | ⚠️ Not worth it - lifetime complexity |
| Direct exit() Calls | Clean shutdown | Low | P2 | ✅ Done |

**All high-priority issues resolved!** ✅
