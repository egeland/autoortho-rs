# Project-Specific Details

## Project Identity

- **Name**: autoortho-rs
- **Description**: Pure Rust reimplementation of AutoOrtho for X-Plane satellite scenery, high-performance tile-based imagery with cross-platform support.
- **Repository**: <https://github.com/egeland/autoortho-rs>
- **License**: Apache-2.0 OR GPL-3.0-or-later
- **Author**: Frode Egeland
- **Rust Edition**: 2024

## Codebase Structure

- **Entry Points**:
  - `src/main.rs`: Binary entry point (CLI/GUI)
  - `src/lib.rs`: Library crate (`autoortho_lib`)
- **Core Modules**:
  - `src/pipeline/`: Image processing (JPEG decode, DDS generation, cache, budget)
  - `src/tiles/`: Tile engine (coords, chunks, tile assembly, prefetch, providers, fetcher, fallback)
  - `src/fuse/`: FUSE filesystem (Linux/macOS: unifuse; Windows: winfsp)
  - `src/xplane/`: X-Plane integration (dataref, simbrief, UDP)
  - `src/webui/`: Web UI (axum, WebSocket, REST API)
  - `src/ui/`: Desktop UI (iced: setup, settings, dashboard, developer tools)
  - `src/scenery/`: Scenery management (discovery, packs.ini, installer, SimHeaven)
- **Config**: `src/config.rs`, `config.toml` (platform-specific paths)
- **Tests**: Unit tests in module files, integration tests in `tests/`, benches in `benches/`

## Platform-Specific Notes

- **Windows**: WinFsp required, `winfsp-x64.dll` must be distributed (GPL-3.0 license)
- **macOS**: macFUSE required for FUSE mounting
- **Linux**: libfuse-dev required for building
- **Cross-Compilation**: Uses `cargo-dist` for releases, supports x86_64 Linux (gnu/musl), macOS (x86_64/aarch64), Windows (x86_64 msvc)

## Common Commands

| Task | Command |
|------|--------|
| Debug build | `cargo build` |
| Release build | `cargo build --release` |
| Run (GUI) | `./target/release/autoortho --gui` |
| Run (CLI) | `./target/release/autoortho --xplane /path/to/X-Plane` |
| Format | `cargo fmt` |
| Lint | `cargo clippy -- -D warnings` |
| Lint (all features) | `cargo clippy --all-features -- -D warnings` |
| Test | `cargo test` |
| Library tests | `cargo test --lib` |
| Integration tests | `cargo test --test integration_test` |

## Workflow Specifics

1. **Branch**: Use `worktree-work` skill for separate tasks

   ```bash
   git checkout main && git pull && wt switch --create fix/some-fix-wt
   ```

2. **Pre-Commit Checks**:

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test --lib
   ```

3. **Commit**: Conventional commits (`fix:`, `feat:`, `chore:`)
4. **PR**: Squash merge only
5. **CI**: Auto-bump version after merge to `main`

## CI/CD Details

- **release.yml**: Inno Setup install hangs with wrong flags
- **security.yml**: Runs on PR + main push (cargo audit, cargo deny)
- **version.yml**: Auto-creates version bump PR via `release-please`
- **cross-platform.yml**: Tests on Linux/macOS/Windows after main push

## References

- Original AutoOrtho: <https://github.com/ProgrammingDinosaur/autoortho>
- PLAN.md
- CHANGELOG.md
- deny.toml (license restrictions)
- Global [AGENTS.md](~/.config/opencode/AGENTS.md)
