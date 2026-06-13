//! Axum route definitions for the web UI.

use crate::scenery::paths::scenery_data_dir;
use crate::webui::{PositionUpdate, WebState};
use axum::extract::{Query, State, ws};
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{Router, get, post};
use futures_util::{SinkExt, StreamExt};
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

#[allow(unused_imports)]
use crate::{config, tiles::coords};

/// Create the Axum router with all routes.
pub fn create_router(state: Arc<WebState>) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/map", get(map_page))
        .route("/stats", get(stats_page))
        .route("/custommap", get(custommap_page))
        .route("/cache", get(cache_page))
        .route("/metrics", get(metrics_json))
        .route("/api/position", get(position_json))
        .route("/api/stats", get(metrics_json))
        .route("/ws", get(ws_handler))
        // Custom map API
        .route(
            "/api/custommap/cells",
            get(custommap_get_cells)
                .post(custommap_set_cells)
                .delete(custommap_delete_cells),
        )
        .route("/api/custommap/clear", post(custommap_clear))
        .route("/api/custommap/maptypes", get(custommap_maptypes))
        .route("/api/custommap/tiles", get(custommap_tiles))
        .route("/api/custommap/export", get(custommap_export))
        .route("/api/custommap/import", post(custommap_import))
        // Cache API
        .route("/api/cache/tiles", get(cache_tiles))
        .route("/api/cache/stats", get(cache_stats))
        .with_state(state)
}

// --- JSON API responses ---

#[derive(Serialize)]
struct PositionResponse {
    lat: f64,
    lon: f64,
    alt_agl_ft: f32,
    heading: f32,
    ground_speed_mps: f32,
    connected: bool,
}

#[derive(Serialize)]
struct MetricsResponse {
    tiles_downloaded: u64,
    bytes_downloaded: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_hit_ratio: f64,
    tiles_pending: u32,
    tiles_completed: u32,
}

// --- Route handlers ---

async fn index_page() -> Html<String> {
    Html(INDEX_HTML.to_string())
}

async fn map_page() -> Html<String> {
    Html(MAP_HTML.to_string())
}

async fn stats_page(State(state): State<Arc<WebState>>) -> Html<String> {
    let snap = state.stats.snapshot();
    let hit_ratio = state.stats.hit_ratio();

    let html = STATS_HTML
        .replace("{{tiles_downloaded}}", &snap.tiles_downloaded.to_string())
        .replace("{{bytes_downloaded}}", &format_bytes(snap.bytes_downloaded))
        .replace("{{cache_hits}}", &snap.cache_hits.to_string())
        .replace("{{cache_misses}}", &snap.cache_misses.to_string())
        .replace("{{cache_hit_ratio}}", &format!("{:.1}%", hit_ratio * 100.0))
        .replace("{{tiles_pending}}", &snap.tiles_pending.to_string())
        .replace("{{tiles_completed}}", &snap.tiles_completed.to_string());

    Html(html)
}

async fn metrics_json(State(state): State<Arc<WebState>>) -> Json<MetricsResponse> {
    let snap = state.stats.snapshot();
    Json(MetricsResponse {
        tiles_downloaded: snap.tiles_downloaded,
        bytes_downloaded: snap.bytes_downloaded,
        cache_hits: snap.cache_hits,
        cache_misses: snap.cache_misses,
        cache_hit_ratio: state.stats.hit_ratio(),
        tiles_pending: snap.tiles_pending,
        tiles_completed: snap.tiles_completed,
    })
}

async fn position_json(State(state): State<Arc<WebState>>) -> Json<PositionResponse> {
    let data = state.tracker.get_flight_data();
    let response = PositionResponse {
        lat: data.lat,
        lon: data.lon,
        alt_agl_ft: data.alt_agl_ft(),
        heading: data.heading,
        ground_speed_mps: data.ground_speed_mps,
        connected: data.connected,
    };

    // Broadcast position update to WebSocket clients
    let _ = state.position_tx.send(PositionUpdate {
        lat: data.lat,
        lon: data.lon,
        alt_agl_ft: data.alt_agl_ft(),
        heading: data.heading,
        ground_speed_mps: data.ground_speed_mps,
        connected: data.connected,
    });

    Json(response)
}

/// WebSocket handler for live position updates
async fn ws_handler(
    State(state): State<Arc<WebState>>,
    ws: ws::WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| ws_position(socket, state))
}

async fn ws_position(socket: ws::WebSocket, state: Arc<WebState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.position_tx.subscribe();

    // Send initial position
    let data = state.tracker.get_flight_data();
    let initial = serde_json::to_string(&PositionUpdate {
        lat: data.lat,
        lon: data.lon,
        alt_agl_ft: data.alt_agl_ft(),
        heading: data.heading,
        ground_speed_mps: data.ground_speed_mps,
        connected: data.connected,
    })
    .unwrap_or_default();

    if sender
        .send(ws::Message::Text(initial.into()))
        .await
        .is_err()
    {
        return;
    }

    // Forward position updates to client
    loop {
        tokio::select! {
            update = rx.recv() => {
                match update {
                    Ok(pos) => {
                        let json = serde_json::to_string(&pos).unwrap_or_default();
                        if sender.send(ws::Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(ws::Message::Close(_))) | None => break,
                    Some(Ok(ws::Message::Ping(data))) => {
                        let _ = sender.send(ws::Message::Pong(data)).await;
                    }
                    Some(Ok(_)) | Some(Err(_)) => {}
                }
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// --- Embedded HTML templates ---

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html><head>
  <title>AutoOrtho</title>
  <meta charset="utf-8">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 600px; margin: 40px auto; padding: 0 20px; }
    h1 { color: #333; }
    a { display: block; padding: 12px 20px; margin: 8px 0; background: #0066cc; color: white; text-decoration: none; border-radius: 6px; }
    a:hover { background: #0052a3; }
    .status { color: #666; font-size: 14px; margin-top: 24px; }
  </style>
</head><body>
  <h1>AutoOrtho</h1>
  <p>Satellite Imagery for X-Plane</p>
  <a href="/map">Flight Map</a>
  <a href="/custommap">Custom Map Editor</a>
  <a href="/cache">Cache Viewer</a>
  <a href="/stats">Performance Stats</a>
  <p class="status">AutoOrtho Rust v0.1.0</p>
</body></html>"#;

const MAP_HTML: &str = r##"<!DOCTYPE html>
<html><head>
  <title>AutoOrtho - Flight Map</title>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9/dist/leaflet.css">
  <script src="https://unpkg.com/leaflet@1.9/dist/leaflet.js"></script>
  <style>
    body { margin: 0; }
    #map { width: 100vw; height: 100vh; }
    #info { position: absolute; top: 10px; right: 10px; background: rgba(255,255,255,0.9);
            padding: 10px 16px; border-radius: 8px; z-index: 1000; font-family: monospace; font-size: 13px; }
    #info .disconnected { color: #cc0000; }
    #info .connected { color: #00aa00; }
    #info .reconnecting { color: #cc6600; }
  </style>
</head><body>
  <div id="map"></div>
  <div id="info">Connecting...</div>
  <script>
    var map = L.map('map').setView([0, 0], 3);
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19, attribution: '&copy; OpenStreetMap'
    }).addTo(map);

    var marker = null;
    var firstUpdate = true;
    var ws = null;
    var reconnectAttempts = 0;
    var reconnectTimeout = null;

    function connect() {
      var protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
      ws = new WebSocket(protocol + '//' + location.host + '/ws');

      ws.onopen = function() {
        document.getElementById('info').innerHTML = '<span class="connected">Connected (WebSocket)</span>';
        reconnectAttempts = 0;
        if (reconnectTimeout) {
          clearTimeout(reconnectTimeout);
          reconnectTimeout = null;
        }
      };

      ws.onmessage = function(event) {
        try {
          var data = JSON.parse(event.data);
          var info = document.getElementById('info');
          if (!data.connected) {
            info.innerHTML = '<span class="disconnected">X-Plane not connected</span>';
            return;
          }
          if (marker === null) {
            marker = L.marker([data.lat, data.lon]).addTo(map);
          } else {
            marker.setLatLng([data.lat, data.lon]);
          }
          if (firstUpdate && data.lat !== 0) {
            map.setView([data.lat, data.lon], 8);
            firstUpdate = false;
          }
          info.innerHTML = '<span class="connected">Connected</span><br>'
            + 'Lat: ' + data.lat.toFixed(4) + '<br>'
            + 'Lon: ' + data.lon.toFixed(4) + '<br>'
            + 'Alt: ' + data.alt_agl_ft.toFixed(0) + ' ft AGL<br>'
            + 'Hdg: ' + data.heading.toFixed(0) + '&deg;<br>'
            + 'GS: ' + (data.ground_speed_mps * 1.944).toFixed(0) + ' kt';
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      ws.onclose = function() {
        document.getElementById('info').innerHTML = '<span class="reconnecting">Reconnecting...</span>';
        // Exponential backoff with max 30 seconds
        var delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
        reconnectAttempts++;
        reconnectTimeout = setTimeout(connect, delay);
      };

      ws.onerror = function() {
        document.getElementById('info').innerHTML = '<span class="disconnected">Connection error</span>';
      };
    }

    connect();
  </script>
</body></html>"##;

const STATS_HTML: &str = r#"<!DOCTYPE html>
<html><head>
  <title>AutoOrtho - Stats</title>
  <meta charset="utf-8">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }
    h1 { color: #333; }
    table { width: 100%; border-collapse: collapse; margin: 20px 0; }
    td, th { padding: 10px 16px; text-align: left; border-bottom: 1px solid #eee; }
    th { background: #f5f5f5; font-weight: 600; }
    .value { font-family: monospace; font-size: 16px; }
    a { color: #0066cc; }
    .refresh { font-size: 13px; color: #999; }
  </style>
</head><body>
  <h1>AutoOrtho Stats</h1>
  <p><a href="/">&larr; Home</a> | <a href="/map">Map</a> | <span class="refresh">Auto-refreshes every 5s</span></p>
  <table>
    <tr><th>Metric</th><th>Value</th></tr>
    <tr><td>Tiles Downloaded</td><td class="value">{{tiles_downloaded}}</td></tr>
    <tr><td>Bytes Downloaded</td><td class="value">{{bytes_downloaded}}</td></tr>
    <tr><td>Cache Hits</td><td class="value">{{cache_hits}}</td></tr>
    <tr><td>Cache Misses</td><td class="value">{{cache_misses}}</td></tr>
    <tr><td>Hit Ratio</td><td class="value">{{cache_hit_ratio}}</td></tr>
    <tr><td>Tiles Pending</td><td class="value">{{tiles_pending}}</td></tr>
    <tr><td>Tiles Completed</td><td class="value">{{tiles_completed}}</td></tr>
  </table>
  <script>setTimeout(function(){ location.reload(); }, 5000);</script>
</body></html>"#;

// --- Custom Map API handlers ---

async fn custommap_page() -> Html<String> {
    Html(CUSTOMMAP_HTML.to_string())
}

async fn custommap_get_cells(State(state): State<Arc<WebState>>) -> Json<HashMap<String, String>> {
    Json(state.custom_map.get_cells())
}

#[derive(Deserialize)]
struct SetCellsRequest {
    cells: HashMap<String, String>,
}

async fn custommap_set_cells(
    State(state): State<Arc<WebState>>,
    Json(body): Json<SetCellsRequest>,
) -> Json<HashMap<String, String>> {
    state.custom_map.set_cells(body.cells);
    Json(state.custom_map.get_cells())
}

#[derive(Deserialize)]
struct DeleteCellsRequest {
    keys: Vec<String>,
}

async fn custommap_delete_cells(
    State(state): State<Arc<WebState>>,
    Json(body): Json<DeleteCellsRequest>,
) -> Json<HashMap<String, String>> {
    state.custom_map.remove_cells(&body.keys);
    Json(state.custom_map.get_cells())
}

async fn custommap_clear(State(state): State<Arc<WebState>>) -> Json<HashMap<String, String>> {
    state.custom_map.clear();
    Json(state.custom_map.get_cells())
}

async fn custommap_maptypes() -> Json<Vec<&'static str>> {
    Json(vec![
        "BI", "NAIP", "EOX", "USGS", "Firefly", "GO2", "ARC", "YNDX", "APPLE",
    ])
}

async fn custommap_tiles(State(state): State<Arc<WebState>>) -> Json<Vec<String>> {
    // Scan installed scenery for DSF files and extract lat/lon cell keys
    let data_dir = scenery_data_dir(&state.config.read().cache_dir);
    let data_dir = data_dir.to_string_lossy().into_owned();
    let tiles = scan_dsf_tiles(&data_dir);
    Json(tiles)
}

/// Scan scenery directory for DSF files and return cell keys ("lat,lon").
fn scan_dsf_tiles(data_dir: &str) -> Vec<String> {
    use std::collections::HashSet;

    let base = std::path::Path::new(data_dir).join("scenery");
    if !base.exists() {
        return vec![];
    }

    let mut tiles = HashSet::new();

    // Walk the scenery tree looking for .dsf files
    fn walk_dir(dir: &std::path::Path, tiles: &mut HashSet<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, tiles);
            } else if path.extension().is_some_and(|e| e == "dsf") {
                // Parse lat/lon from filename like "+46+152.dsf" or "-34+151.dsf"
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Some((lat, lon)) = parse_dsf_coords(stem)
                {
                    tiles.insert(format!("{},{}", lat, lon));
                }
            }
        }
    }

    walk_dir(&base, &mut tiles);

    let mut result: Vec<String> = tiles.into_iter().collect();
    result.sort();
    result
}

/// Parse DSF filename like "+46+152" or "-34+151" into (lat, lon).
fn parse_dsf_coords(stem: &str) -> Option<(i32, i32)> {
    // Format: [+-]DD[+-]DDD
    // Find the second +/- sign (start of longitude)
    let bytes = stem.as_bytes();
    if bytes.len() < 4 {
        return None;
    }

    // First char must be + or -
    if bytes[0] != b'+' && bytes[0] != b'-' {
        return None;
    }

    // Find the second +/- (start of longitude)
    let lon_start = (1..bytes.len()).find(|&i| bytes[i] == b'+' || bytes[i] == b'-')?;

    let lat_str = &stem[..lon_start];
    let lon_str = &stem[lon_start..];

    let lat = lat_str.parse::<i32>().ok()?;
    let lon = lon_str.parse::<i32>().ok()?;

    // Validate latitude (-90 to 90) and longitude (-180 to 180) ranges
    if !(-90..=90).contains(&lat) || !(-180..=180).contains(&lon) {
        return None;
    }

    Some((lat, lon))
}

async fn custommap_export(
    State(state): State<Arc<WebState>>,
) -> (
    axum::http::StatusCode,
    [(axum::http::header::HeaderName, &'static str); 2],
    String,
) {
    let json = state.custom_map.export_json();
    (
        axum::http::StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "application/json"),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "attachment; filename=\"custom_map.json\"",
            ),
        ],
        json,
    )
}

#[derive(Deserialize)]
struct ImportQuery {
    #[serde(default)]
    merge: Option<String>,
}

async fn custommap_import(
    State(state): State<Arc<WebState>>,
    Query(query): Query<ImportQuery>,
    body: String,
) -> Json<HashMap<String, String>> {
    let merge = query.merge.as_deref() == Some("true");
    if let Err(e) = state.custom_map.import_json(&body, merge) {
        warn!("Failed to import custom map JSON: {}", e);
        return Json(HashMap::new());
    }
    Json(state.custom_map.get_cells())
}

// --- Cache Viewer API handlers ---

#[derive(Serialize)]
struct CachedTile {
    col: u32,
    row: u32,
    zoom: u32,
    provider: String,
    size_bytes: u64,
    built: f64,
    lat_n: f64,
    lon_w: f64,
    lat_s: f64,
    lon_e: f64,
}

async fn cache_tiles(State(state): State<Arc<WebState>>) -> Json<Vec<CachedTile>> {
    let config = state.config.read();
    let cache_dir = std::path::PathBuf::from(&config.cache_dir).join("dds");
    if !cache_dir.exists() {
        return Json(vec![]);
    }

    let mut tiles = Vec::new();

    let entries = match std::fs::read_dir(&cache_dir) {
        Ok(e) => e,
        Err(_) => return Json(vec![]),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !name.ends_with(".ddm") {
            continue;
        }

        let Some(stem) = name.strip_suffix(".ddm") else {
            continue;
        };

        // Read metadata — use tile_row/tile_col from the .ddm file
        // rather than parsing the filename, because the key format
        // varies between callers (filesystem.rs vs DdsCache::tile_key).
        let meta_str = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let meta: serde_json::Value = match serde_json::from_str(&meta_str) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let row = match meta.get("tile_row").and_then(|v| v.as_u64()) {
            Some(r) => r as u32,
            None => continue,
        };
        let col = match meta.get("tile_col").and_then(|v| v.as_u64()) {
            Some(c) => c as u32,
            None => continue,
        };
        let provider = meta
            .get("map")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let zoom = match meta.get("zl").and_then(|v| v.as_u64()) {
            Some(z) if z <= 21 => z as u32,
            _ => continue,
        };
        let built = meta.get("built").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Get DDS file size
        let dds_path = cache_dir.join(format!("{}.dds.zst", stem));
        let size_bytes = dds_path.metadata().map(|m| m.len()).unwrap_or(0);

        // Calculate bounds
        let (lat_n, lon_w, lat_s, lon_e) =
            match crate::tiles::coords::TileCoords::tile_bounds(col, row, zoom) {
                Ok(b) => b,
                Err(_) => continue,
            };

        tiles.push(CachedTile {
            col,
            row,
            zoom,
            provider,
            size_bytes,
            built,
            lat_n,
            lon_w,
            lat_s,
            lon_e,
        });
    }

    Json(tiles)
}

#[derive(Serialize)]
struct CacheStats {
    entry_count: usize,
    size_bytes: u64,
}

async fn cache_stats(State(state): State<Arc<WebState>>) -> Json<CacheStats> {
    let config = state.config.read();
    let cache_dir = std::path::PathBuf::from(&config.cache_dir).join("dds");
    if !cache_dir.exists() {
        return Json(CacheStats {
            entry_count: 0,
            size_bytes: 0,
        });
    }

    let entries = match std::fs::read_dir(&cache_dir) {
        Ok(e) => e,
        Err(_) => {
            return Json(CacheStats {
                entry_count: 0,
                size_bytes: 0,
            });
        }
    };

    let mut count = 0usize;
    let mut size = 0u64;

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name.ends_with(".ddm") {
            count += 1;
        } else if name.ends_with(".dds.zst") {
            size += path.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Json(CacheStats {
        entry_count: count,
        size_bytes: size,
    })
}

async fn cache_page() -> Html<String> {
    Html(CACHE_VIEWER_HTML.to_string())
}

// The Custom Map Editor HTML is embedded from the Python AutoOrtho project.
// It's a self-contained Leaflet.js app that talks to our REST API.
const CUSTOMMAP_HTML: &str = include_str!("../../assets/custommap_editor.html");

const CACHE_VIEWER_HTML: &str = include_str!("../../assets/cache_viewer.html");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::StatsStore;
    use crate::webui::custommap::CustomMapStore;
    use crate::xplane::dataref::DatarefTracker;

    fn make_state() -> Arc<WebState> {
        let tmp = std::env::temp_dir().join("autoortho_test_custommap.json");
        Arc::new(WebState::new(
            Arc::new(StatsStore::new()),
            Arc::new(DatarefTracker::new()),
            CustomMapStore::load(tmp),
            Arc::new(parking_lot::RwLock::new(
                crate::config::AutoOrthoConfig::default(),
            )),
        ))
    }

    #[tokio::test]
    async fn test_index_returns_html() {
        let response = index_page().await;
        assert!(response.0.contains("AutoOrtho"));
    }

    #[tokio::test]
    async fn test_map_returns_leaflet() {
        let response = map_page().await;
        assert!(response.0.contains("leaflet"));
    }

    #[tokio::test]
    async fn test_metrics_json_default() {
        let state = make_state();
        let Json(metrics) = metrics_json(State(state)).await;
        assert_eq!(metrics.tiles_downloaded, 0);
        assert_eq!(metrics.cache_hit_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_position_json_default() {
        let state = make_state();
        let Json(pos) = position_json(State(state)).await;
        assert!(!pos.connected);
        assert_eq!(pos.lat, 0.0);
    }

    #[tokio::test]
    async fn test_metrics_after_activity() {
        let state = make_state();
        state.stats.record_download(1024);
        state.stats.record_cache_hit();
        state.stats.record_cache_miss();

        let Json(metrics) = metrics_json(State(state)).await;
        assert_eq!(metrics.tiles_downloaded, 1);
        assert_eq!(metrics.bytes_downloaded, 1024);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[tokio::test]
    async fn test_cache_tiles_returns_bounds() {
        use crate::config::AutoOrthoConfig;
        use crate::pipeline::cache::DdsCacheMetadata;
        use crate::stats::StatsStore;
        use crate::webui::custommap::CustomMapStore;
        use crate::webui::routes::WebState;
        use crate::xplane::dataref::DatarefTracker;
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        // Create a mock .ddm metadata file with key format: col_row_provider_zzoom
        // e.g., 100_200_BI_z16.ddm
        let meta = DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 16,
            max_zl: 16,
            fmt: "BC1".to_string(),
            map: "BI".to_string(),
            built: 1700000000.0,
            tile_row: 200,
            tile_col: 100,
            populated_mipmaps: vec![0, 1, 2, 3, 4],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        let meta_json = serde_json::to_string_pretty(&meta).unwrap();
        std::fs::write(cache_dir.join("100_200_BI_z16.ddm"), meta_json).unwrap();

        // Also create the .dds.zst file so it exists
        std::fs::write(cache_dir.join("100_200_BI_z16.dds.zst"), b"fake dds").unwrap();

        let config = AutoOrthoConfig {
            cache_dir: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let state = Arc::new(WebState::new(
            Arc::new(StatsStore::new()),
            Arc::new(DatarefTracker::new()),
            CustomMapStore::load(tmp.path().join("custom_map.json")),
            Arc::new(RwLock::new(config)),
        ));

        let response = cache_tiles(State(state)).await;
        let tiles = response.0;

        assert_eq!(tiles.len(), 1, "Should return 1 tile");
        let tile = &tiles[0];
        assert_eq!(tile.col, 100);
        assert_eq!(tile.row, 200);
        assert_eq!(tile.zoom, 16);
        assert_eq!(tile.provider, "BI");

        // Verify bounds are calculated correctly
        // At zoom 16, tile 100, 200 should have valid lat/lng bounds
        assert!(tile.lat_n > tile.lat_s, "North should be > South");
        assert!(tile.lon_e > tile.lon_w, "East should be > West");

        // Verify bounds are reasonable for zoom 16
        let lat_diff = tile.lat_n - tile.lat_s;
        let lon_diff = tile.lon_e - tile.lon_w;
        assert!(
            lat_diff > 0.0 && lat_diff < 0.1,
            "Latitude diff should be small at zoom 16"
        );
        assert!(
            lon_diff > 0.0 && lon_diff < 0.1,
            "Longitude diff should be small at zoom 16"
        );
    }

    #[tokio::test]
    async fn test_cache_tiles_reads_metadata_not_filename() {
        // Regression test: cache viewer must use .ddm metadata (tile_row/tile_col)
        // instead of parsing the filename, because filesystem.rs and DdsCache::tile_key
        // use different key formats.
        use crate::config::AutoOrthoConfig;
        use crate::pipeline::cache::DdsCacheMetadata;
        use crate::stats::StatsStore;
        use crate::webui::custommap::CustomMapStore;
        use crate::webui::routes::WebState;
        use crate::xplane::dataref::DatarefTracker;
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        // Simulate a cache file written by filesystem.rs: row_col_maptype_zoom
        // (key format differs from DdsCache::tile_key which uses col_row_maptype_zzoom)
        // Filename: 200_100_BI_16 (row=200, col=100)
        let meta = DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 16,
            max_zl: 16,
            fmt: "BC1".to_string(),
            map: "BI".to_string(),
            built: 1700000000.0,
            tile_row: 200,
            tile_col: 100,
            populated_mipmaps: vec![0, 1, 2, 3, 4],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        let meta_json = serde_json::to_string_pretty(&meta).unwrap();
        std::fs::write(cache_dir.join("200_100_BI_16.ddm"), meta_json).unwrap();
        std::fs::write(cache_dir.join("200_100_BI_16.dds.zst"), b"fake dds").unwrap();

        let config = AutoOrthoConfig {
            cache_dir: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let state = Arc::new(WebState::new(
            Arc::new(StatsStore::new()),
            Arc::new(DatarefTracker::new()),
            CustomMapStore::load(tmp.path().join("custom_map.json")),
            Arc::new(RwLock::new(config)),
        ));

        let tiles = cache_tiles(State(state)).await.0;
        assert_eq!(tiles.len(), 1, "Should return 1 tile");
        let tile = &tiles[0];
        // Must read from metadata, not filename
        assert_eq!(tile.col, 100, "col from metadata tile_col");
        assert_eq!(tile.row, 200, "row from metadata tile_row");
        assert_eq!(tile.zoom, 16);
        assert_eq!(tile.provider, "BI");

        // Verify bounds match the metadata coordinates, not a swapped parse.
        // Original bug: filename "200_100_BI_16" was parsed as col=200,row=100
        // (swapped), placing tiles at ~85°N instead of correct position.
        // With metadata, col=100 row=200 zoom=16 -> lat_n ≈ 84.95°
        assert!(
            tile.lat_n > 84.0 && tile.lat_n < 86.0,
            "lat_n should match col=100 row=200 at z16, got {}",
            tile.lat_n
        );
    }

    #[tokio::test]
    async fn test_cache_tiles_both_key_formats() {
        // Regression: both filesystem.rs (row_col_maptype_zoom) and
        // DdsCache::tile_key (col_row_maptype_zzoom) key formats must work.
        use crate::config::AutoOrthoConfig;
        use crate::pipeline::cache::DdsCacheMetadata;
        use crate::stats::StatsStore;
        use crate::webui::custommap::CustomMapStore;
        use crate::webui::routes::WebState;
        use crate::xplane::dataref::DatarefTracker;
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        // File 1: filesystem.rs format — row_col_maptype_zoom
        let meta1 = DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 16,
            max_zl: 16,
            fmt: "BC1".to_string(),
            map: "BI".to_string(),
            built: 1700000000.0,
            tile_row: 300,
            tile_col: 150,
            populated_mipmaps: vec![0, 1, 2],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        std::fs::write(
            cache_dir.join("300_150_BI_16.ddm"),
            serde_json::to_string_pretty(&meta1).unwrap(),
        )
        .unwrap();
        std::fs::write(cache_dir.join("300_150_BI_16.dds.zst"), b"dds1").unwrap();

        // File 2: DdsCache::tile_key format — col_row_maptype_zzoom
        let meta2 = DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 17,
            max_zl: 17,
            fmt: "BC3".to_string(),
            map: "GO2".to_string(),
            built: 1700000001.0,
            tile_row: 400,
            tile_col: 200,
            populated_mipmaps: vec![0, 1, 2, 3],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        std::fs::write(
            cache_dir.join("200_400_GO2_z17.ddm"),
            serde_json::to_string_pretty(&meta2).unwrap(),
        )
        .unwrap();
        std::fs::write(cache_dir.join("200_400_GO2_z17.dds.zst"), b"dds2").unwrap();

        let config = AutoOrthoConfig {
            cache_dir: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let state = Arc::new(WebState::new(
            Arc::new(StatsStore::new()),
            Arc::new(DatarefTracker::new()),
            CustomMapStore::load(tmp.path().join("custom_map.json")),
            Arc::new(RwLock::new(config)),
        ));

        let tiles = cache_tiles(State(state)).await.0;
        assert_eq!(tiles.len(), 2, "Should return both tiles");

        let bi = tiles.iter().find(|t| t.provider == "BI").unwrap();
        assert_eq!(bi.col, 150);
        assert_eq!(bi.row, 300);
        assert_eq!(bi.zoom, 16);

        let go2 = tiles.iter().find(|t| t.provider == "GO2").unwrap();
        assert_eq!(go2.col, 200);
        assert_eq!(go2.row, 400);
        assert_eq!(go2.zoom, 17);
    }

    #[tokio::test]
    async fn test_cache_tiles_ignores_bad_filename_uses_metadata() {
        // Regression: even a completely unparseable filename must be handled
        // correctly as long as the .ddm metadata is valid.
        use crate::config::AutoOrthoConfig;
        use crate::pipeline::cache::DdsCacheMetadata;
        use crate::stats::StatsStore;
        use crate::webui::custommap::CustomMapStore;
        use crate::webui::routes::WebState;
        use crate::xplane::dataref::DatarefTracker;
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let meta = DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: 13,
            zl: 14,
            max_zl: 14,
            fmt: "BC1".to_string(),
            map: "ARC".to_string(),
            built: 1700000002.0,
            tile_row: 500,
            tile_col: 250,
            populated_mipmaps: vec![0, 1],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        std::fs::write(
            cache_dir.join("garbage_filename.ddm"),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .unwrap();
        std::fs::write(cache_dir.join("garbage_filename.dds.zst"), b"dds").unwrap();

        let config = AutoOrthoConfig {
            cache_dir: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let state = Arc::new(WebState::new(
            Arc::new(StatsStore::new()),
            Arc::new(DatarefTracker::new()),
            CustomMapStore::load(tmp.path().join("custom_map.json")),
            Arc::new(RwLock::new(config)),
        ));

        let tiles = cache_tiles(State(state)).await.0;
        assert_eq!(tiles.len(), 1, "Should return tile from metadata");
        let tile = &tiles[0];
        assert_eq!(tile.col, 250);
        assert_eq!(tile.row, 500);
        assert_eq!(tile.zoom, 14);
        assert_eq!(tile.provider, "ARC");
    }

    #[test]
    fn test_scan_dsf_tiles() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("z_autoortho");

        // Create a fake DSF file in the expected location
        let dsf_dir = data_dir
            .join("scenery")
            .join("z_ao_na")
            .join("Earth nav data");
        std::fs::create_dir_all(&dsf_dir).unwrap();
        std::fs::write(dsf_dir.join("+46+152.dsf"), b"fake dsf").unwrap();

        // It should find the tile and return its coords
        let tiles = scan_dsf_tiles(data_dir.to_str().unwrap());
        assert_eq!(tiles, vec!["46,152"]);
    }

    #[test]
    fn test_router_creation() {
        let state = make_state();
        let _router = create_router(state);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_cache_viewer_sri_hashes() {
        // Regression test for issue #390: SRI hashes must match actual Leaflet 1.9.4 files
        let html = CACHE_VIEWER_HTML;
        assert!(
            html.contains("integrity=\"sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=\""),
            "leaflet.css SRI hash incorrect"
        );
        assert!(
            html.contains("integrity=\"sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=\""),
            "leaflet.js SRI hash incorrect"
        );
    }
}
