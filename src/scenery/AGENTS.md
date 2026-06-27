# Scenery Module

## Purpose

Scenery pack discovery, download, installation, and management. Handles SimHeaven integration.

## Ownership

- `mod.rs` — Module re-exports
- `discovery.rs` — Find installed scenery packs on disk
- `download.rs` — HTTP downloads with resume support, SHA256 verification, progress tracking
- `extract.rs` — ZIP extraction with progress reporting, path traversal protection
- `installer.rs` — Pack metadata (`PackInfo`), save/load/list, re-exports from download/extract
- `orchestrator.rs` — Business logic: region discovery, download+install workflow, progress tracking
- `packs_ini.rs` — Parse `packs.ini` configuration
- `paths.rs` — `mount_dir()`, scenery path resolution
- `simheaven.rs` — SimHeaven X-World integration

## Local Contracts

- `mount_dir()` is the FUSE mount point for X-Plane
- Packs distributed via GitHub releases from `autoortho-scenery` repo
- `packs.ini` defines installed pack metadata
- DSF files define tile coverage; DDS textures generated on-the-fly
- `installer.rs` re-exports `download` and `extract` for backward compatibility

## Work Guidance

- `download.rs` — HTTP logic, resume via Range header, SHA256 verification
- `extract.rs` — ZIP extraction, uses `extract_unwrapped_root_dir` for single-root archives
- `installer.rs` — `PackInfo` metadata, `save_pack_info()`, `load_pack_info()`, `list_installed_packs()`
- `orchestrator.rs` — `download_and_install_region()` coordinates the full workflow
- `discovery.rs` scans X-Plane directories for scenery packs
- `paths.rs` provides cross-platform path resolution
- `simheaven.rs` is optional integration

## Verification

- `cargo test --lib scenery`
- `download.rs` tests: has_partial_downloads, clean_downloads
- `extract.rs` tests: extract_zip, extract_with_progress, traversal protection
- `installer.rs` tests: save/load pack info, list installed, uninstall, migrate
- `orchestrator.rs` tests: DownloadProgress tracking

## Child DOX Index

None — flat module.
