# FUSE Module

## Purpose

FUSE/Dokan virtual filesystem that exposes DDS tile images to X-Plane. Parses tile paths, serves DDS content on demand.

## Ownership

- `mod.rs` — `DdsPathParser`, virtual dir constants, shared types
- `filesystem.rs` — Filesystem trait implementation, file handle management
- `mount.rs` — Unix FUSE mount (unifuse) — `#[cfg(not(windows))]`
- `mount_win.rs` — Windows Dokan2 mount — `#[cfg(windows)]`
- `platform.rs` — `platform_name()`, `is_fuse_available()`

## Local Contracts

- DDS path regex: `r".*/(\d+)[-_](\d+)[-_](\S*)(\d{2})\.dds"`
- `VIRTUAL_DIRS`: `["textures", "terrain"]`
- `MARKER_FILE`: `"AOISWORKING"`
- Poison path: `is_poison_path()` for shutdown signaling
- `calculate_dds_size()` must match pipeline's `dds_file_size_4096_bc3()`

## Work Guidance

- Windows uses Dokan2 (GitHub dokan230), not crates.io version
- `Dokan.dll` must be distributed with Windows builds
- `DdsPathParser` is shared across modules — changes affect `fuse`, `tiles`, `pipeline`
- Platform-gated: `mount.rs` (Unix) vs `mount_win.rs` (Windows)

## Verification

- `cargo test --lib fuse`
- Cross-platform availability test in `mod.rs`
- Manual: mount + read a DDS path

## Child DOX Index

None — flat module.
