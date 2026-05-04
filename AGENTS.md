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
  - `src/fuse/`: FUSE (Linux/macOS: unifuse; Windows: Dokan2)
  - `src/xplane/`: X-Plane integration: dataref, simbrief, UDP
  - `src/webui/`: Web UI: axum, WebSocket, REST API
  - `src/ui/`: Desktop UI (iced): setup, settings, dashboard, dev tools
  - `src/scenery/`: Scenery management: discovery, packs.ini, installer, SimHeaven
- **Config**: `src/config.rs`, `config.toml` (platform-specific paths)
- **Tests**: Unit tests in module files, integration tests in `tests/`, benches in `benches/`

## Platform-Specific Notes

- **Windows**: Dokan2 required, `Dokan.dll` must be distributed
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
   cargo clippy --all-features -- -D warnings
   cargo test --all-features
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

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **autoortho-rs** (2431 symbols, 4900 relationships, 156 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/autoortho-rs/context` | Codebase overview, check index freshness |
| `gitnexus://repo/autoortho-rs/clusters` | All functional areas |
| `gitnexus://repo/autoortho-rs/processes` | All execution flows |
| `gitnexus://repo/autoortho-rs/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
