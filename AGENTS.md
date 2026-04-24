# AGENTS.md - AutoOrtho-RS

## For AI Agents

This file provides context for AI agents working on the autoortho-rs project. Follow all instructions here and in the global `~/.config/opencode/AGENTS.md`.

---

## Project Identity

- **Name**: autoortho-rs
- **Description**: Pure Rust reimplementation of AutoOrtho for X-Plane satellite scenery, high-performance tile-based imagery with cross-platform support.
- **Repository**: <https://github.com/egeland/autoortho-rs>
- **License**: Apache-2.0 OR GPL-3.0-or-later
- **Author**: Frode Egeland
- **Rust Edition**: 2024

---

## Key Features

- Multiple tile providers (Google Maps, Bing Maps, ArcGIS, USGS NAIP, etc.)
- Cross-platform FUSE filesystem (Linux: libfuse, macOS: macFUSE, Windows: WinFsp)
- Real-time web UI with flight tracking (WebSocket)
- Intelligent zstd-compressed DDS tile cache with LRU eviction
- Dynamic zoom, night exclusion, seasonal adjustments
- SimBrief flight plan integration

---

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

---

## Development Environment

### Prerequisites

- Rust 1.75+ (via rustup)
- **Linux**: libfuse-dev (`sudo apt-get install libfuse-dev`)
- **macOS**: macFUSE 4.x (via Homebrew)
- **Windows**: WinFsp (download from [winfsp GitHub](https://github.com/winfsp/winfsp))

### Setup

```bash
git clone https://github.com/egeland/autoortho-rs.git
cd autoortho-rs
cargo build
```

---

## Common Commands

| Task | Command |
|------|---------|
| Debug build | `cargo build` |
| Release build | `cargo build --release` |
| Run (GUI) | `./target/release/autoortho --gui` |
| Run (CLI) | `./target/release/autoortho --xplane /path/to/X-Plane` |
| Library tests | `cargo test --lib` |
| All tests | `cargo test` |
| Format check | `cargo fmt --check` |
| Lint (default) | `cargo clippy -- -D warnings` |
| Lint (all features) | `cargo clippy --all-features -- -D warnings` |
| Benchmark | `cargo bench` |
| Integration tests | `cargo test --test integration_test` |

---

## Development Workflow

1. **Branch**: Create feature branches from `main` (use `worktree-work` skill for separate tasks)

   ```bash
   git checkout main && git pull && git checkout -b fix/some-fix-commit
   ```

2. **Develop**: Follow TDD (write tests first, run, fail, write code). Do one test at a time, write its code, then proceed to the next. Do not write all the tests up front, then all the code. Do not write the code followed by the tests.
3. **Pre-Commit Checks**:

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test --lib
   ```

4. **Commit**: Use [conventional commits](https://www.conventionalcommits.org/) (e.g., `fix: correct Inno Setup install flags`)
5. **Push**: Push branch, open PR to `main`
6. **PR**: Squash merge only, CI must pass
7. **CI**: After merge to `main`, automatic version bump and release

---

## CI/CD Pipeline

All workflows in `.github/workflows/`:

| Workflow | Trigger | Jobs | Notes |
|----------|----------|------|-------|
| `ci.yml` | PR to main | Format check, Clippy (default + all features), Test (Linux/macOS/Windows) | Uses `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` |
| `security.yml` | PR + main push | `cargo audit`, `cargo deny` | Checks vulnerabilities and license compliance |
| `cross-platform.yml` | Main push | Test on Linux/macOS/Windows | Runs after merge |
| `version.yml` | Main push (after tests) | `release-please` | Creates version bump PR |
| `release.yml` | Tag (v*.*.*) | Build artifacts, create GitHub Release, Inno Setup installer (Windows) | **Common Issue**: Inno Setup install hangs with wrong flags |

## Code Standards

- Follow Rust idioms, match existing code style
- **TDD**: Write unit tests first, run them, see failure, write code
- **Linting**: Always run `cargo clippy -- -D warnings` before committing
- **Formatting**: `cargo fmt` before committing
- **Dependencies**: Check `deny.toml` for allowed licenses (Apache-2.0, MIT, GPL-3.0, BSD-2/3, etc.); no unauthorized crates
- **Commit Messages**: Conventional commits (fix:, feat:, chore:, etc.)
- **Secrets**: Never commit secrets, API keys, or credentials

---

## Key Dependencies

| Purpose | Crate | Notes |
|---------|-------|-------|
| Async runtime | tokio (rt-multi-thread) | Non-blocking I/O throughout |
| Parallelism | rayon | Parallel JPEG decoding, compression |
| HTTP client | reqwest (rustls, HTTP/2) | Fetch tiles from providers |
| Compression | zstd, texpresso | Tile cache, BC1/BC3 compression |
| Web framework | axum | REST API, WebSocket |
| UI | iced (0.14) | Desktop GUI |
| FUSE (Unix) | unifuse | Linux/macOS FUSE mounting |
| FUSE (Windows) | winfsp | Windows support (GPL-3.0) |
| Config | config-rs, serde, toml | Configuration management |
| Error handling | thiserror | Typed errors |

---

## Platform-Specific Notes

- **Windows**: WinFsp required, `winfsp-x64.dll` must be distributed (GPL-3.0 license)
- **macOS**: macFUSE required for FUSE mounting
- **Linux**: libfuse-dev required for building
- **Cross-Compilation**: Uses `cargo-dist` for releases, supports x86_64 Linux (gnu/musl), macOS (x86_64/aarch64), Windows (x86_64 msvc)

---

## Common Tasks for AI Agents

1. **Fix CI Issues**: e.g., Inno Setup hang, WinFsp install failures, clippy warnings
2. **Add New Tile Providers**: Implement `TileProvider` trait in `src/tiles/provider.rs`
3. **Update UI Components**: Modify `src/ui/screens/` (iced-based)
4. **Fix FUSE Bugs**: Update `src/fuse/` (platform-specific code)
5. **Improve Caching**: Modify `src/pipeline/cache.rs`, `budget.rs`
6. **Add Features**: Follow existing patterns, write tests first

---

## Troubleshooting

- **CI Format/Lint Errors**: Run `cargo fmt` and `cargo clippy -- -D warnings` locally first

---

## References

- [Original AutoOrtho](https://github.com/ProgrammingDinosaur/autoortho)
- [PLAN.md](PLAN.md)
- [Changelog](CHANGELOG.md)
- [deny.toml](deny.toml) (license restrictions)
