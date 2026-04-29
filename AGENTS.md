# Project-Specific Details

## Project Identity

- **Name**: autoortho-rs
- **Description**: Pure Rust AutoOrtho reimplementation: X-Plane satellite scenery, high-performance tile imagery, cross-platform.
- **Repository**: <https://github.com/egeland/autoortho-rs>
- **License**: Apache-2.0 OR GPL-3.0-or-later
- **Author**: Frode Egeland
- **Rust Edition**: 2024 (stable ≥1.85; no toolchain pinning, check upgrades)

## Codebase Structure

- **Entry Points**:
  - `src/main.rs`: Binary entry point (CLI/GUI)
  - `src/lib.rs`: Library crate (`autoortho_lib`)
- **Core Modules**:
  - `src/pipeline/`: Image processing: JPEG decode, DDS gen, cache, budget
  - `src/tiles/`: Tile engine: coords, chunks, assembly, prefetch, providers, fetcher, fallback
  - `src/fuse/`: FUSE (Linux/macOS: unifuse; Windows: winfsp)
  - `src/xplane/`: X-Plane integration: dataref, simbrief, UDP
  - `src/webui/`: Web UI: axum, WebSocket, REST API
  - `src/ui/`: Desktop UI (iced): setup, settings, dashboard, dev tools
  - `src/scenery/`: Scenery management: discovery, packs.ini, installer, SimHeaven
- **Config**: `src/config.rs`, `config.toml` (platform-specific paths)
- **Tests**: Unit tests in module files, integration tests in `tests/`, benches in `benches/`

## Platform-Specific Notes

- **Windows**: WinFsp required, `winfsp-x64.dll` must be distributed (GPL-3.0 license)
- **macOS**: macFUSE required for FUSE mounting
- **Linux**: libfuse-dev required for building
- **Cross-Compilation**: Cross-compile via `cargo-dist`: x86_64 Linux (gnu/musl), macOS (x86_64/aarch64), Windows (x86_64 msvc)

## Common Commands

| Task | Command |
|------|--------|
| Debug build | `cargo build` |
| Release build | `cargo build --release` |
| Run (GUI) | `./target/release/autoortho --gui` |
| Run (CLI) | `./target/release/autoortho --xplane /path/to/X-Plane` |
| Format | `cargo fmt` |
| Lint | `cargo clippy -- -D warnings` |
| Test | `cargo test` |
| Library tests | `cargo test --lib` |
| Integration tests | `cargo test --test integration_test` |

## Workflow Specifics

1. **Branch**: Use `wt` (worktrunk tool) for tasks; see `wt --help`.

   ```bash
   git switch main && git pull && wt switch --create fix/some-fix-wt
   ```

2. **Pre-Commit Checks**:

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test --lib
   ```

3. **Commit**: Conventional commits (`fix:`, `feat:`, `chore:`)

## CI/CD Details

- **ci.yml**: Cross-platform tests: Linux/macOS/Windows
- **release.yml**: Release builds; Inno Setup deprecated
- **release-plz.yml**: Version bumps + release PRs via release-plz
- **security.yml**: PR + main push: cargo audit, cargo deny

## References

- Original AutoOrtho: <https://github.com/ProgrammingDinosaur/autoortho>
- PLAN.md
- CHANGELOG.md
- deny.toml: license restrictions
