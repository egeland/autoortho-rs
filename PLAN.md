# AutoOrtho Rust - Development Plan

This document outlines planned work, organized by priority and effort.

---

## High Priority

### 1. Error Handling Consistency
**File**: Multiple (`src/**/*.rs`)

- **Task**: Standardize error handling using `thiserror` or `anyhow`
- **Current State**: Mixed usage of `Box<dyn Error>`, custom enums, `unwrap()`/`expect()`
- **Goal**: Single robust error type throughout; use `?` operator at all logic levels
- **Effort**: ~4-6 hours
- **Status**: ⚠️ Partially done - `thiserror` in 21 modules, but some `unwrap()` remain

---

### 2. Add Image Decode Failure Logging
**File**: `src/tiles/assembler.rs:107-133`

- **Task**: Replace silent `.ok()` with proper logging
- **Current State**: Failures are silently ignored
- **Impact**: Debugging is difficult without visibility into which images fail
- **Effort**: ~1 hour
- **Status**: 📋 To do

---

## Medium Priority

### 3. Fetcher Constructor Consolidation
**File**: `src/tiles/fetcher.rs`

- **Task**: Reduce 4 constructor variants to unified API
- **Current State**: `new`, `with_cache_size`, `with_provider_and_cache_size`, `with_rate_limit`
- **Goal**: Builder pattern or single constructor with optional parameters
- **Effort**: ~2-3 hours
- **Status**: 📋 To do

---

## Low Priority / Future

### 4. Vertical Speed Calculation
**File**: `src/dataref.rs:156`

- **Task**: Compute `vertical_speed_fpm` from altitude delta
- **Current State**: Hardcoded to `0.0`
- **Note**: Requires additional sensor data or interpolation logic
- **Effort**: ~2-4 hours depending on algorithm choice
- **Status**: 📋 To do

---

## Completed (Historical)

| Feature | File/Location | Status |
|---------|---------------|--------|
| TileCoord newtype struct | `src/coords.rs` | ✅ Done |
| Zero-copy slice_range() | `src/filesystem.rs:690` | ✅ Done |
| Rate limiting | `src/tiles/rate_limiter.rs` | ✅ Done |
| Error handling standardization | Multiple modules | ✅ Done |
| Async/await pattern | `src/**/*.rs` | ✅ Done |

---

## Review Notes

### Architecture Decisions to Verify
- [ ] Confirm `Arc<RwLock<T>>` pattern is optimal for shared state
- [ ] Review FUSE mount cleanup with `Drop` trait
- [ ] Verify LRU cache size thresholds in production workloads

---

*Last updated: 2026-04-13 | Keep this section sorted by priority*
