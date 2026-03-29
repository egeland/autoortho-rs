# Plan: Missing Tile Providers

**Created:** 2026-03-29

## Overview

Add support for two missing tile providers from the Python version:
- **Yandex Maps (YNDX)** - Russian satellite imagery
- **Apple Maps (APPLE)** - Apple satellite imagery with authentication

## Yandex Maps (YNDX)

### Implementation

The Yandex Maps provider is straightforward - it's a simple tile server with no authentication.

**Tile URL Format:**
```
https://sat{server_num}.maps.yandex.net/tiles?l=sat&v=3.1814.0&x={col}&y={row}&z={zoom}
```

Where `server_num` cycles through 1-4 (like Google Maps server selection).

### Tasks

1. **Add YandexMapsProvider struct** in `src/tiles/provider.rs`
   - Implement `TileProvider` trait
   - Use round-robin server selection (1-4)
   
2. **Add to ProviderFactory**
   - Register "YNDX" and "YANDEX" aliases
   
3. **Add ProviderInfo**
   - ID: "YNDX"
   - Display: "Yandex Maps"
   - Zoom: 0-17 (Yandex coverage)
   - Auth: Not required

### Testing

- Fetch tiles from Yandex and verify image data
- Test server rotation

---

## Apple Maps (APPLE)

### Implementation

Apple Maps requires dynamic authentication tokens. The token is obtained through a complex flow:
1. Fetch a token from DuckDuckGo (origin authorization)
2. Use that to fetch Apple's bootstrap API
3. Extract satellite tile URL metadata (version + accessKey)

The token expires and must be refreshed on 403/410 HTTP errors.

### Tasks

1. **Create Apple token service** in `src/tiles/apple_token.rs`
   - `AppleTokenService` struct with token refresh logic
   - Async HTTP calls to DuckDuckGo and Apple CDN
   - Token caching with lazy refresh
   - Handle 403/410 errors by rotating token

2. **Add AppleMapsProvider struct** in `src/tiles/provider.rs`
   - Implement `TileProvider` trait
   - Include token in tile URL
   - Handle auth failures gracefully

3. **Add to ProviderFactory**
   - Register "APPLE" alias
   
4. **Add ProviderInfo**
   - ID: "APPLE"
   - Display: "Apple Maps"  
   - Zoom: 0-19
   - Auth: Required

### Token Refresh Flow

```rust
struct AppleTokenService {
    client: reqwest::Client,
    token: Arc<RwLock<Option<AppleToken>>>,
    ddg_token_url: String,
    apple_bootstrap_url: String,
}

struct AppleToken {
    version: String,
    access_key: String,
}

impl AppleTokenService {
    async fn get_token(&self) -> Result<AppleToken, Error>;
    
    async fn fetch_with_retry(&self, url: &str) -> Result<Vec<u8>, ProviderError> {
        loop {
            let token = self.get_token().await?;
            let url = self.make_tile_url(col, row, zoom, &token);
            
            match self.client.get(&url).send().await {
                Ok(resp) if resp.status() == 200 => return Ok(resp.bytes().await?),
                Ok(resp) if resp.status() == 403 || resp.status() == 410 => {
                    // Token expired, reset and retry
                    self.reset_token().await;
                    continue;
                }
                Ok(resp) => return Err(ProviderError::NetworkError(resp.status())),
                Err(e) => return Err(ProviderError::NetworkError(e)),
            }
        }
    }
}
```

### Testing

- Initial token fetch
- Tile fetch with token
- Token refresh on 403/410
- Graceful degradation if token fetch fails

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/tiles/provider.rs` | Add YandexMapsProvider, AppleMapsProvider, update ProviderFactory |
| `src/tiles/mod.rs` | Add apple_token module |

## Files to Create

| File | Description |
|------|-------------|
| `src/tiles/apple_token.rs` | Apple token service |

## Acceptance Criteria

- [ ] YNDX provider fetches valid JPEG tiles
- [ ] YNDX provider appears in UI provider list
- [ ] APPLE provider fetches valid JPEG tiles
- [ ] APPLE provider handles 403/410 token expiry
- [ ] APPLE provider appears in UI provider list
- [ ] Both providers work with existing caching infrastructure

## Time Estimate

- Yandex Maps: 1-2 hours
- Apple Maps: 3-4 hours (token service complexity)
