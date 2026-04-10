# AutoOrtho Rust - Remaining Tasks

## Deferred / Low Priority

- `filesystem.rs:690` — `slice_range()` returns `Vec<u8>`, could use `Cow<[u8]>` for zero-copy → DONE
  - **Research**: 7 call sites inside `read_dds()` - 5 can use zero-copy (disk cache+memory), 2 always allocate (night fallback, solid fallback).
  - **Implementation**: Returns `Cow<[u8]>`, converts to `.into_owned()` at each call site. All 350 tests pass, clippy clean.

## Minor Improvements

- `assembler.rs:107-133` — Image decode failures silently fall back via `.ok()`; logging would help debugging
- `fetcher.rs` — 4 constructor variants exist (new, with_cache_size, with_provider_and_cache_size, with_rate_limit); minor API bloat
- `dataref.rs:156` — `vertical_speed_fpm: 0.0` has TODO to compute from altitude delta

## Completed

- Rate limiting — implemented in `rate_limiter.rs`, used in `TileFetcher::get_chunk_data_with_provider()`