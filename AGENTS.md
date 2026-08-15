# Project-Specific Details

### Logging

- **Stack**: We use the `tracing` ecosystem (`tracing`, `tracing-subscriber`, `tracing-appender`, `tracing-log`).
- `env_logger` has been removed.
- **File Logging**: Logs go to a file (not stdout/stderr) via `tracing-appender`.
  - Configured in `src/main.rs` via `init_logger()`.
  - Log rotation is set in `config.toml` via `log_rotation` ("daily", "hourly", "never").
- **Windows Behavior**: Release builds use `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`.
  - No console window appears in release GUI mode.
  - Debug builds retain the console for development.

### Lessons Learned

- **Release Completion**: A release isn't complete when the GitHub Release tag is created. It's only done when the `release.yml` workflow (which compiles binaries for Linux/macOS/Windows and uploads them) completes successfully.
- **Windows Subsystem**: `cfg_attr` is compile-time only. Runtime console toggle needs `hide_console` crate or WinAPI `FreeConsole`.
- **Dokan + widestring version mismatch (May 2026)**: The crates.io `dokan` 0.3.x depends on `widestring 0.4.x` (generic `UCStr<C>` type), while a direct dep on `widestring 1.x` (concrete `U16CStr` type) caused two incompatible versions to coexist. Fix: use the GitHub dokan (`dokan230`) which uses `widestring 1.2`, add `widestring = "1.2"` as a direct dep (enabled by `fuse` feature), and import `U16CStr`/`U16CString` directly from `widestring` (not from `dokan`, which doesn't pub-re-export them).

## Project Identity

- **Name**: autoortho-rs
- **Description**: Pure Rust AutoOrtho reimplementation: X-Plane satellite scenery, high-performance tile imagery, cross-platform.
- **Repository**: <https://github.com/egeland/autoortho-rs>
- **License**: Apache-2.0 OR GPL-3.0-or-later
- **Author**: Frode Egeland
- **Rust Edition**: 2024 (stable ≥1.96; no toolchain pinning, check upgrades)

## Codebase Structure

- **Entry Points**:
  - `src/main.rs`: Binary entry point (CLI/GUI)
  - `src/lib.rs`: Library crate (`autoortho_lib`)

See the DOX section below for more about the structure.

## Platform-Specific Notes

- **Windows**: Dokan2 required, `Dokan.dll` must be distributed. Use the GitHub dokan (dokan230) in `Cargo.toml`, not crates.io version (dokan206) — the crates.io version depends on incompatible `widestring 0.4.x`.
- **macOS**: macFUSE required for FUSE mounting
- **Linux**: libfuse-dev required for building
- **Cross-Compilation**: Cross-compile via `cargo-dist`: x86_64 Linux (gnu/musl), macOS (x86_64/aarch64), Windows (x86_64 msvc)

NOTE: local development happens on a MacOS machine, and some features for other platforms are gated, and won't run on Mac. Do not assume that a clean `clippy build` on Mac means Windows will also build. This needs CI testing to confirm.

## Common Commands

| Task | Command |
| ------ | -------- |
| Debug build | `cargo build` |
| Release build | `cargo build --release` |
| Run (GUI) | `./target/release/autoortho --gui` |
| Run (CLI) | `./target/release/autoortho --xplane /path/to/X-Plane` |
| Format (run after each edit of .rs file) | `cargo fmt` |
| Lint | `cargo clippy -- -D warnings` |
| Test | `cargo test` |
| Library tests | `cargo test --lib` |
| Integration tests | `cargo test --test integration_test` |
| Test coverage - baseline | `cargo llvm-cov --all-features --lcov --output-path /tmp/lcov_baseline.info` Run first, before doing any work, set baseline for test coverage. |
| Test coverage - delta | `cargo llvm-cov --all-features --lcov --output-path /tmp/lcov_delta.info`  |

## Workflow Specifics

- NEVER change the `origin` git remote from `ssh` to `https`.

1. **Pre-Commit Checks**:

   ```bash
   cargo fmt
   cargo clippy --all-features -- -D warnings
   cargo test --all-features
   cargo llvm-cov --all-features --lcov --output-path lcov.info
   ```

Run `cargo llvm-cov` before each commit, compare with baseline for test coverage. Coverage must at worst stay unchanged, but aim to increase coverage with each commit.

1. **Commits**: Use Conventional commit format prefixes: (`fix:`, `feat:`, `chore:`)

## CI/CD Details

- **ci.yml**: Cross-platform tests: Linux/macOS/Windows
- **release.yml**: Release builds; Inno Setup deprecated
- **release-plz.yml**: Version bumps + release PRs via release-plz
- **security.yml**: PR + main push: cargo audit, cargo deny

## References

- Original ("python version") AutoOrtho: <https://github.com/ProgrammingDinosaur/autoortho>
- CHANGELOG.md
- deny.toml: license restrictions

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **autoortho-rs** (2864 symbols, 6028 relationships, 234 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

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

# DOX framework

- DOX is highly performant AGENTS.md hierarchy installed here
- Agent must follow DOX instructions across any edits

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it

## Read Before Editing

1. Read the root AGENTS.md
2. Identify every file or folder you expect to touch
3. Walk from the repository root to each target path
4. Read every AGENTS.md found along each route
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Update After Editing

Every meaningful change requires a DOX pass before the task is done.

Update the closest owning AGENTS.md when a change affects:

- purpose, scope, ownership, or responsibilities
- durable structure, contracts, workflows, or operating rules
- required inputs, outputs, permissions, constraints, side effects, or artifacts
- user preferences about behavior, communication, process, organization, or quality
- AGENTS.md creation, deletion, move, rename, or index contents

Update parent docs when parent-level structure, ownership, workflow, or child index changes. Update child docs when parent changes alter local rules. Remove stale or contradictory text immediately. Small edits that do not change behavior or contracts may leave docs unchanged, but the DOX pass still must happen.

## Hierarchy

- Root AGENTS.md is the DOX rail: project-wide instructions, global preferences, durable workflow rules, and the top-level Child DOX Index
- Child AGENTS.md files own domain-specific instructions and their own Child DOX Index
- Each parent explains what its direct children cover and what stays owned by the parent
- The closer a doc is to the work, the more specific and practical it must be

## Child Doc Shape

- Create a child AGENTS.md when a folder becomes a durable boundary with its own purpose, rules, responsibilities, workflow, materials, or quality standards
- Work Guidance must reflect the current standards of the project or user instructions; if there are no specific standards or instructions yet, leave it empty
- Verification must reflect an existing check; if no verification framework exists yet, leave it empty and update it when one exists

Default section order:

- Purpose
- Ownership
- Local Contracts
- Work Guidance
- Verification
- Child DOX Index

## Style

- Keep docs concise, current, and operational
- Document stable contracts, not diary entries
- Put broad rules in parent docs and concrete details in child docs
- Prefer direct bullets with explicit names
- Do not duplicate rules across many files unless each scope needs a local version
- Delete stale notes instead of explaining history
- Trim obvious statements, repeated rules, misplaced detail, and warnings for risks that no longer exist

## Closeout

1. Re-check changed paths against the DOX chain
2. Update nearest owning docs and any affected parents or children
3. Refresh every affected Child DOX Index
4. Remove stale or contradictory text
5. Run existing verification when relevant
6. Report any docs intentionally left unchanged and why

## User Preferences

When the user requests a durable behavior change, record it here or in the relevant child AGENTS.md

## Child DOX Index

| Path | Scope |
|------|-------|
| `src/pipeline/AGENTS.md` | Image processing: JPEG decode, DDS gen, cache, budget |
| `src/tiles/AGENTS.md` | Tile engine: coords, chunks, assembly, prefetch, providers, fetcher, fallback |
| `src/fuse/AGENTS.md` | FUSE/Dokan virtual filesystem, DDS path parsing |
| `src/services/AGENTS.md` | Service traits for DI: TileService, FakeTileService |
| `src/xplane/AGENTS.md` | X-Plane integration: dataref, simbrief, UDP |
| `src/webui/AGENTS.md` | Web UI: axum server, WebSocket, REST API, custom map |
| `src/ui/AGENTS.md` | Desktop UI (iced MVU): app lifecycle, state, handlers |
| `src/ui/screens/AGENTS.md` | Screen implementations (welcome, setup, settings, dashboard, etc.) |
| `src/config/AGENTS.md` | Configuration: sub-configs, defaults, validation, env overrides |
| `src/scenery/AGENTS.md` | Scenery pack discovery, download, installation, SimHeaven |
