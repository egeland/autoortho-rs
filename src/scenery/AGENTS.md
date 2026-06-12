# Scenery Module

## Purpose

Scenery pack discovery, download, installation, and management. Handles SimHeaven integration.

## Ownership

- `mod.rs` — Module re-exports
- `discovery.rs` — Find installed scenery packs on disk
- `installer.rs` — Download and install scenery packs from GitHub releases
- `packs_ini.rs` — Parse `packs.ini` configuration
- `paths.rs` — `mount_dir()`, scenery path resolution
- `simheaven.rs` — SimHeaven X-World integration

## Local Contracts

- `mount_dir()` is the FUSE mount point for X-Plane
- Packs distributed via GitHub releases from `autoortho-scenery` repo
- `packs.ini` defines installed pack metadata
- DSF files define tile coverage; DDS textures generated on-the-fly

## Work Guidance

- `installer.rs` is the most complex file (~800 lines) — handles download + extraction
- `discovery.rs` scans X-Plane directories for scenery packs
- `paths.rs` provides cross-platform path resolution
- `simheaven.rs` is optional integration

## Verification

- `cargo test --lib scenery`
- Discovery tests use temp directories

## Child DOX Index

None — flat module.
