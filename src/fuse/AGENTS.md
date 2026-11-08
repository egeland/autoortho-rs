# FUSE Module

## Purpose

FUSE/Dokan virtual filesystem that exposes DDS tile images to X-Plane. Parses tile paths, serves DDS content on demand.

## Ownership

- `mod.rs` — `DdsPathParser`, virtual dir constants, shared types
- `filesystem.rs` — Thin filesystem router: path parsing, directory listing, cache lookup, delegation to TileGenerator (`resolution_arc()` for background tasks)
- `pass_through.rs` — Real filesystem pass-through: get_attr, list_dir, is_dir_in_root for scenery root
- `tile_generator.rs` — Tile generation pipeline: fetch chunks → decode → compose → compress → DDS bytes
- `mount.rs` — Unix FUSE mount (unifuse) — `#[cfg(not(windows))]`
- `mount_win.rs` — Windows Dokan2 mount — `#[cfg(windows)]`
- `platform.rs` — `platform_name()`, `is_fuse_available()`, `cleanup_mount()`, `prepare_mount_dir()`

## Local Contracts

- DDS path regex: `r".*/(\d+)[-_](\d+)[-_](\S*)(\d{2})\.dds"`
- `VIRTUAL_DIRS`: `["textures", "terrain"]`
- `MARKER_FILE`: `"AOISWORKING"`
- `DdsFileSystem::read()` DDS branch records a mount tile request via the
  stats service (dashboard "X-Plane Tile Requests" indicator, bead-e64);
  marker and pass-through reads don't count
- Poison path: `is_poison_path()` for shutdown signaling
- `calculate_dds_size()` must match pipeline's `dds_file_size_4096_bc3()`

## Work Guidance

- Windows uses Dokan2 (GitHub dokan230), not crates.io version
- `Dokan.dll` must be distributed with Windows builds
- `DdsPathParser` is shared across modules — changes affect `fuse`, `tiles`, `pipeline`
- Platform-gated: `mount.rs` (Unix) vs `mount_win.rs` (Windows)
- Windows mounts must call `prepare_mount_dir()` before mounting: a killed
  process leaves a dangling Dokan mount-point junction that blocks restart
  (`AlreadyExists`)
- Dokan `MountOptions.timeout` must stay well above worst-case tile
  generation time (default 15 s aborts reads with "Insufficient system
  resources" and may unmount the device)

## Verification

- `cargo test --lib fuse`
- Cross-platform availability test in `mod.rs`
- Manual: mount + read a DDS path

## Child DOX Index

None — flat module.
