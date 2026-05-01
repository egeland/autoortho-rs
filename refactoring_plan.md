# AutoOrtho-RS Refactoring Plan

## Overview
Code duplication elimination and quality improvements for the autoortho-rs codebase. (Downloader module removed in `d6459fc`, so all downloader-specific items are deprecated.)

---

## Priority 1: Code Duplication Elimination

### Pattern 1: StatsStore Lock Pattern (HIGH - 7 occurrences)
**Location:** `src/stats.rs`

All methods follow identical pattern:
```rust
pub fn method(&self, ...) {
    let mut snap = self.snapshot.lock();
    snap.field = value;
}
```

**Occurrences:**
- `record_download()` (2 field updates)
- `record_cache_hit()`
- `record_cache_miss()`
- `set_pending_tiles()`
- `set_completed_tiles()`
- `clear()` (4 field resets)
- `hit_ratio()` (read-only variant)

**Suggested refactor:** Add helper method with closure:
```rust
fn with_snapshot<F, R>(&self, f: F) -> R
where F: FnOnce(&mut StatsSnapshot) -> R
{
    f(&mut self.snapshot.lock())
}
```

**Estimated savings:** ~30 lines, improved consistency.

---

### Pattern 2: Config Extraction Pattern (MEDIUM - 2+ locations)
**Location:** `src/main.rs` (`run_with_mount`, `run_simbrief_prefetch`)

```rust
let (field1, field2, ...) = {
    let c = config.read();
    (c.field1, c.field2.clone(), ...)
};
```

**Suggested refactor:** Create `ConfigSnapshot` struct:
```rust
struct ConfigSnapshot {
    tile_provider: String,
    max_zoom: u32,
    zoom_rules: Vec<ZoomRule>,
    enable_dynamic_zoom: bool,
    // ... fields needed for prefetch/dynamic zoom
}
impl From<&AutoOrthoConfig> for ConfigSnapshot { ... }
```

**Estimated savings:** ~20 lines, clearer intent.

---

### Pattern 3: DynamicZoom Instantiation (LOW-MEDIUM - 2 locations)
**Locations:** `src/main.rs`, `src/ui/screens/settings.rs`

```rust
DynamicZoom::new(config.zoom_rules.clone(), &config.tile_provider)
```

**Suggested refactor:** Add constructor to `AppContext` or use `ConfigSnapshot`.

**Estimated savings:** ~5 lines, single point of configuration.

---

### Pattern 4: Test TempDir Setup (MEDIUM - 6+ files)
**Files:** `app_context.rs`, `config.rs`, `custommap.rs`, `installer.rs`, `packs_ini.rs`, `filesystem.rs`

Common pattern:
```rust
use tempfile::TempDir;
let tmp = TempDir::new().unwrap();
```

**Suggested refactor:** Create test helper module `src/test_utils.rs` with common setup functions.

**Estimated savings:** ~40 lines across test files, improved consistency.

---

### Pattern 5: Night Exclusion Polling (LOW - 1 location)
**Location:** `src/main.rs` - night exclusion loop

Could extract into `start_night_exclusion_monitor()` helper.

**Estimated savings:** ~10 lines, improved readability.

---

## Priority 2: Minor Improvements

### 1. Error Message Clarity
- Update "Unsupported format" to include actual extension (if applicable).
- Update "Empty URL" to specify which URL was empty (if applicable).

### 2. Test Coverage Gaps
- Add tests for refactored StatsStore patterns.
- Add tests for `ConfigSnapshot` extraction.
- Improve test consistency via `test_utils.rs`.

---

## Execution Plan

### Phase 1: High-Impact StatsStore Refactor
1. Add `with_snapshot()` helper to `StatsStore`
2. Refactor all 7 methods to use it
3. Test: `cargo test stats` (9 tests)

### Phase 2: Config Extraction
1. Create `ConfigSnapshot` struct in `config.rs`
2. Implement `From<&AutoOrthoConfig>`
3. Refactor `run_with_mount()` and `run_simbrief_prefetch()`
4. Test: `cargo test --lib`

### Phase 3: Test Utilities
1. Create `src/test_utils.rs` with common test helpers
2. Migrate 2-3 test modules as proof of concept
3. Test: `cargo test --lib`

### Phase 4: Minor Extractions
1. Extract night exclusion monitor
2. Simplify DynamicZoom creation
3. Apply minor improvements

---

## Estimated Impact

| Phase | Lines Saved | Risk | Tests |
|-------|-------------|------|-------|
| Phase 1 (StatsStore) | ~30 | Low | 9 existing |
| Phase 2 (Config) | ~20 | Low | 364 existing |
| Phase 3 (Test Utils) | ~40 | Low | Existing + new |
| Phase 4 (Minor) | ~15 | Low | Add 2+ new |

**Total estimated savings:** 80-105 lines of duplicated/boilerplate code

**Quality improvements:**
- Reduced boilerplate and cognitive load
- Single point of truth for patterns
- Improved test consistency

---

## Next Steps
1. **Start with Phase 1** (StatsStore `with_snapshot()` refactor) - highest impact, lowest risk
2. Run `cargo fmt`, `cargo clippy`, `cargo test --lib` after each change
3. Commit with conventional commits: `refactor: ...`
4. Progress through phases sequentially
