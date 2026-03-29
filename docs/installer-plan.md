# Plan: Native Installer Packages

**Created:** 2026-03-29

## Overview

Create native installers for AutoOrtho Rust across all three platforms:
- **Windows**: NSIS installer (.exe) and ZIP
- **macOS**: DMG with app bundle and ZIP
- **Linux**: Tarball and AppImage

## Current Status: NOT STARTED

This is a future enhancement plan. No implementation has been done yet.

## Strategy

Use **cargo-dist** (formerly known as "cxldist") for cross-platform release automation.

---

## Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add cargo-dist config |
| `.github/workflows/release.yml` | New CI workflow |

## Files to Create

| File | Description |
|------|-------------|
| `.github/workflows/ci.yml` | CI test workflow |

## Acceptance Criteria

- [ ] `cargo dist build` produces Windows .exe and .zip
- [ ] `cargo dist build` produces macOS .dmg and .tar.gz
- [ ] `cargo dist build` produces Linux .tar.gz and .AppImage
- [ ] GitHub Release created on tag push
- [ ] All artifacts have checksums
- [ ] Windows installer prompts for WinFsp
- [ ] macOS app checks for macFUSE

## Time Estimate

~10 hours
