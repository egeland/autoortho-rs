# ADR: Retain Services Layer Architecture

**Date:** 2026-06-23
**Status:** Accepted
**Supersedes:** Architecture Review §1 "Shallow Service Wrappers" recommendation

---

## Context

The 2026-06-23 architecture review identified the `services/` module as "shallow wrappers doing pure delegation" and recommended collapsing it. Upon analysis, we are retaining the current architecture. This document explains why.

## What the Review Got Right

The review accurately describes the structure:

```
StatsService trait → StatsServiceImpl wraps StatsStore → delegates 1:1
FallbackService trait → FallbackServiceImpl wraps FallbackSystem → delegates 1:1
CacheService trait → CacheServiceImpl wraps DdsCache → delegates 1:1
```

Each wrapper is thin. The delegation is mechanical. The duplication of `StatsSnapshot` was real (now fixed).

## Why We Keep It

### 1. The Traits Are Load-Bearing

`DdsFileSystem` holds `Arc<dyn StatsService>` and `Arc<dyn FallbackService>`. These are not decorative—they enable:

- **Unit tests** that inject `FakeStatsService` / `FakeFallbackService` without touching disk, network, or real caches
- **Integration tests** that verify stats recording and fallback behavior in isolation
- **Future flexibility** to swap implementations (e.g., a Prometheus-backed stats service)

Removing the traits would force `DdsFileSystem` to depend on concrete types, coupling the hot path (every X-Plane tile request) to real I/O.

### 2. The Deletion Test Fails

The review suggests: "Delete the service layer—complexity reappears only as 'tests need to mock this module.'"

That *is* the problem. The mockability IS the complexity being managed. The service layer exists to make `DdsFileSystem` testable without spinning up FUSE, tile providers, or disk caches. Without it, test setup becomes:

```rust
// WITHOUT services layer
let stats = Arc::new(StatsStore::new());
let cache = Arc::new(Mutex::new(DdsCache::new(temp_dir, 256)));
let fallback_system = FallbackSystem::new(temp_dir, FallbackConfig::default());
let fallback = Arc::new(Mutex::new(fallback_system));
// ... wire everything together manually
```

```rust
// WITH services layer
let fake_stats = Arc::new(FakeStatsService::new());
let fake_fallback = Arc::new(FakeFallbackService::new(true));
// ... clean, focused, one line each
```

### 3. The Cost Is Low

The services layer is ~500 lines across 4 files. The traits are stable. The implementations are trivial. There is no maintenance burden—the code rarely changes because the interface is well-defined.

### 4. The Real Win Was Already Claimed

We already fixed the one genuine issue: `StatsSnapshot` duplication. The `stats_service::StatsSnapshot` type was removed, and both layers now use `crate::stats::StatsSnapshot` directly. This eliminated the conversion layer and the type confusion.

## What We Did Instead

Rather than collapsing the services layer, we targeted higher-value improvements:

| Change | Lines Saved | Risk | Value |
|--------|-------------|------|-------|
| Extract PassThroughFs from DdsFileSystem | ~50 | Low | Better separation of concerns |
| Group SimBriefState in AppState | ~30 | Low | Reduced field count, locality |
| Remove duplicate StatsSnapshot | ~15 | None | Single source of truth |
| Delete dead TrackerAdapter | 45 | None | Less confusion |
| Extract codec/averagers from xplane/mod.rs | ~150 | Low | Focused modules |

Total: ~290 lines reduced or reorganized, zero behavioral changes, all tests passing.

## When to Revisit

Re-evaluate if:

1. **A third consumer** of `StatsStore` appears that doesn't need mocking—the trait may be unnecessary for that path
2. **The trait methods grow** beyond simple delegation (e.g., batching, caching at the trait level)
3. **Compile times** become a measurable problem from the trait object indirection

## Decision

Retain `services/` as-is. The traits provide real testability value. The cost is trivial. The architecture review's recommendation was sound in principle but the testing benefit in practice.

---

*This ADR is a living document. Update when the services layer's cost-benefit ratio shifts.*
