# Plan: Missing Tile Providers

**Created:** 2026-03-29

## Overview

Add support for two missing tile providers from the Python version:
- **Yandex Maps (YNDX)** - Russian satellite imagery
- **Apple Maps (APPLE)** - Apple satellite imagery with authentication

## Current Status: ✅ COMPLETE

Both providers are fully implemented:

### Yandex Maps (YNDX)
- ✅ `YandexMapsProvider` struct in `src/tiles/provider.rs:537`
- ✅ Round-robin server selection (1-4)
- ✅ Registered in ProviderFactory as "YNDX" and "YANDEX" aliases
- ✅ ProviderInfo with zoom 0-17

### Apple Maps (APPLE)
- ✅ `AppleTokenService` in `src/tiles/apple_token.rs`
- ✅ `AppleMapsProvider` struct in `src/tiles/provider.rs:589`
- ✅ Token refresh on 403/410 errors
- ✅ Registered in ProviderFactory as "APPLE"
- ✅ ProviderInfo with zoom 0-19
