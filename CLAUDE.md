# Global Claude Agent Instructions

## Coding Conventions

- Prefer Rust
- Format: `cargo fmt` pre-commit
- Lint: `cargo clippy` pre-commit
- TDD: 1 test → run → implement → pass → repeat
- No batching: 1 test/code change at a time
- Deep functions, clear interfaces, unit-testable
- Tests pass before commit

## Safety Rules

- Confirm destructive ops: `rm`, overwrites

## Response Preferences

- Code/config: accurate. Non-code: terse.
- Ultra-terse: fragments > sentences.
- Honest, direct.
- Challenge user if wrong.
- No chatter. Accuracy first.

## Tool Preferences

- Prefer `fd` > `find`, `rg` > `grep -r`

## Git Preferences

- No clone unless requested; use local repos.
- GitHub: SSH, YubiKey SSH keys require user unlock.
- Main branch start: branch/worktree via `wt switch --create` (not `git worktree`).
- Non-git GitHub ops: `gh` CLI.
- Post-merge: `gh poi` clean branches.

## Planning

- User plans: interview per aspect, walk design tree, resolve deps sequentially.
- One question at a time, give recommended answer + evidence.
- Check codebase first for answers.
- Research options to ≥95% certainty, cite evidence.
- In git repos: maintain PLAN.md; otherwise ask if needed.

## Project-Specific Details

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

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **autoortho-rs** (2542 symbols, 5404 relationships, 223 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

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
