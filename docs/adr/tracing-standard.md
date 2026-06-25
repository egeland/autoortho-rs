# ADR: Standardize on `tracing` for Observability

**Date:** 2026-06-25  
**Status:** Accepted  
**Supersedes:** Hybrid `log` + `tracing` usage across the codebase

---

## Context

The codebase uses the `tracing` ecosystem (`tracing`, `tracing-subscriber`, `tracing-appender`) with a `tracing-log` bridge for log compatibility. However, ~15 files still use `log::{debug, warn, info}` directly instead of `tracing::{debug, warn, info}`.

This creates two problems:

1. **Filter bypass**: `log::` calls go through `tracing-log`'s `LogTracer`, which converts them to tracing events — but they bypass the `LevelFilter` configured in `init_logger()`. Log lines at any level leak through.
2. **Inconsistent idiom**: Two logging APIs in one codebase. New contributors don't know which to use.

## Decision

**All logging in autoortho-rs uses `tracing` macros directly.** No `use log::*` in production code. The `tracing-log` bridge remains only for third-party crates that still use `log` internally.

### Scope

| Crate | Rule |
|-------|------|
| `autoortho-rs` (this crate) | `tracing::{debug, info, warn, error, trace}` only |
| Third-party deps | Bridge handles automatically via `tracing-log` |
| `log` crate | Not imported directly. `tracing-log` re-exports nothing. |

### Migration

All existing `use log::{debug, warn, info}` → `use tracing::{debug, warn, info}`. Mechanical find-and-replace. No behavior change — `tracing` macros have the same syntax.

Files affected (non-exhaustive):
- `src/tiles/provider.rs`
- `src/tiles/fetcher.rs`
- `src/tiles/tile_generator.rs`
- `src/fuse/filesystem.rs`
- `src/fuse/platform.rs`
- `src/scenery/*.rs`
- `src/webui/*.rs`
- `src/services/*.rs`

### Verification

After migration:
- `cargo clippy` passes (no unused import warnings)
- `cargo test` passes (no behavioral change)
- Grep for `use log::` returns zero hits in `src/` (excluding test code if any)

## Consequences

- All log output respects the configured tracing filter (debug mode, RUST_LOG)
- Structured tracing features (spans, fields) become available if needed later
- Single logging idiom across the codebase
- `log` crate remains as a transitive dependency (via `tracing-log`) but is never imported directly

## Revisit If

- A major dependency switches from `log` to `tracing` natively — then `tracing-log` bridge can be removed
- Performance profiling reveals tracing overhead is unacceptable (unlikely for this use case)
