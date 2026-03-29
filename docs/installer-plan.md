# Plan: Native Installer Packages

**Created:** 2026-03-29
**Updated:** 2026-03-30

## Overview

Create native installers for AutoOrtho Rust across all three platforms:
- **Windows**: NSIS installer (.exe) and ZIP
- **macOS**: DMG with app bundle and ZIP
- **Linux**: Tarball and AppImage

## Current Status: PARTIALLY COMPLETE 🔄

cargo-dist is configured and release automation is in place. Native installers build on tag push but platform-specific enhancements (FUSE dependency checks) are not yet implemented.

## Strategy

Using **cargo-dist** v0.30+ for cross-platform release automation.

---

## What's Done ✅

| Item | Status |
|------|--------|
| `Cargo.toml` — cargo-dist config (targets, installer suffix) | ✅ |
| `.github/workflows/release.yml` — cargo-dist release workflow | ✅ |
| `.github/workflows/version.yml` — release-please version bumps | ✅ |
| `.github/workflows/ci.yml` — CI test workflow | ✅ |
| `.github/workflows/cross-platform.yml` — Multi-platform builds | ✅ |
| `.github/workflows/security.yml` — cargo-audit + cargo-deny | ✅ |
| Release profile (LTO thin, strip, panic abort) | ✅ |
| Three target triples configured | ✅ |

## What's NOT Done ❌

| Item | Status |
|------|--------|
| macOS DMG bundle | ❌ |
| Windows NSIS installer (vs plain .exe) | ❌ |
| Linux AppImage | ❌ |
| Windows installer prompts for WinFsp | ❌ |
| macOS app checks for macFUSE | ❌ |

## Acceptance Criteria

- [x] GitHub Release created on tag push (via cargo-dist + release-please)
- [x] Release builds for Linux x86_64, macOS arm64, Windows x86_64
- [ ] `cargo dist build` produces macOS .dmg
- [ ] `cargo dist build` produces Linux .AppImage
- [ ] All artifacts have checksums
- [ ] Windows installer prompts for WinFsp
- [ ] macOS app checks for macFUSE
