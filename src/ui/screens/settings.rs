use crate::config::{FallbackLevel, Season};
use crate::dynamic_zoom::DynamicZoom;
use crate::tiles::provider::{PROVIDER_IDS, PROVIDER_INFO};
use crate::ui::Message;
use crate::ui::state::AppState;
use iced::widget::{
    button, column, container, pick_list, row, rule, slider, space, text, text_input, tooltip,
};
use iced::{Element, Fill, Length};

const SEASONS: &[Season] = &[
    Season::Disabled,
    Season::Spring,
    Season::Summer,
    Season::Autumn,
    Season::Winter,
];
const SEASON_LABELS: &[&str] = &["Disabled", "Spring", "Summer", "Autumn", "Winter"];

const FALLBACK_LEVELS: &[FallbackLevel] = &[
    FallbackLevel::Cache,
    FallbackLevel::Downserve,
    FallbackLevel::Network,
    FallbackLevel::Solid,
];
const FALLBACK_LABELS: &[&str] = &["Cache", "Downserve", "Network", "Solid"];

/// Settings screen - full configuration management
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Settings").size(28);

    // -- Paths section --
    let ini_warning: Element<'_, Message> = if !state.config.xplane_path.is_empty()
        && !state
            .config
            .custom_scenery_path()
            .join("scenery_packs.ini")
            .exists()
    {
        text(format!(
            "{} No scenery_packs.ini found - is this the correct X-Plane installation folder?",
            crate::ui::helpers::ICON_WARNING
        ))
        .size(13)
        .color(iced::Color::from_rgb(0.9, 0.7, 0.0))
        .into()
    } else {
        space::vertical().height(0).into()
    };

    let paths = column![
        text("Paths").size(18),
        rule::horizontal(1),
        tooltip(
            labeled_path_input(
                "X-Plane Folder:",
                &state.config.xplane_path,
                Message::SetXPlanePath,
                Message::BrowseXPlanePath
            ),
            container(text("Root X-Plane installation folder. Custom Scenery, mount point, and scenery install paths are derived from this.").size(12))
                .padding(8)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        ini_warning,
        tooltip(
            labeled_path_input(
                "Tile Cache:",
                &state.config.cache_dir,
                Message::SetCacheDir,
                Message::BrowseCacheDir
            ),
            container(text("Persistent storage for generated DDS textures. Survives restarts so tiles don't need re-downloading.").size(12))
                .padding(8)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            labeled_path_input(
                "Scenery Downloads:",
                &state.scenery_download_dir,
                Message::SetSceneryDownloadDir,
                Message::BrowseSceneryDownloadDir
            ),
            container(text("Downloaded scenery pack zip files are kept here as a cache. Re-installing won't need to re-download. Use Clean to reclaim space.").size(12))
                .padding(8)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(8);

    // -- Network section --
    let network = column![
        text("Network").size(18),
        rule::horizontal(1),
        labeled_input(
            "X-Plane Host:",
            &state.config.xplane_host,
            Message::SetXPlaneHost
        ),
        row![
            text("X-Plane Port:").width(Length::Fixed(160.0)),
            text_input("49000", &state.config.xplane_port.to_string())
                .on_input(Message::SetXPlanePort)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // -- SimBrief section --
    let simbrief = column![
        text("SimBrief").size(18),
        rule::horizontal(1),
        tooltip(
            labeled_input(
                "User ID Number:",
                &state.config.simbrief_user_id,
                Message::SetSimbriefUserId
            ),
            container(
                text("Your numeric SimBrief User ID. Find it at simbrief.com \u{2192} Account Settings \u{2192} SimBrief User ID.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Route Consideration: {} nm",
                    state.config.route_consideration_radius_nm
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    10u32..=200,
                    state.config.route_consideration_radius_nm,
                    Message::SetRouteConsiderationRadius
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Distance from route centerline to still be considered 'on route'. Used for SimBrief altitude and prefetch logic.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Deviation Threshold: {} nm",
                    state.config.route_deviation_threshold_nm
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    5u32..=100,
                    state.config.route_deviation_threshold_nm,
                    Message::SetRouteDeviationThreshold
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Max distance from route before switching from SimBrief altitude to dataref altitude.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Prefetch Radius: {} nm",
                    state.config.route_prefetch_radius_nm
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    10u32..=150,
                    state.config.route_prefetch_radius_nm,
                    Message::SetRoutePrefetchRadius
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Radius around each waypoint to prefetch tiles.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Prefetch Route Distance: {}%",
                    state.config.prefetch_route_percent
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    0u32..=100,
                    state.config.prefetch_route_percent,
                    Message::SetPrefetchRoutePercent
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Percentage of total route distance to prefetch ahead of aircraft.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text("Prefetch around airports:").width(Length::Fixed(260.0)),
                button(
                    text(if state.config.prefetch_airports {
                        "Enabled"
                    } else {
                        "Disabled"
                    })
                    .size(14)
                )
                .padding([6, 16])
                .style(if state.config.prefetch_airports {
                    button::success
                } else {
                    button::secondary
                })
                .on_press(Message::SetPrefetchAirports(!state.config.prefetch_airports)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Also prefetch tiles around origin and destination airports.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Airport Radius: {} nm",
                    state.config.airport_radius_nm
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    20u32..=150,
                    state.config.airport_radius_nm,
                    Message::SetAirportRadius
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Radius around airports to prefetch when airport prefetch is enabled.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(8);

    // Show coverage warning if SimBrief is loaded
    let simbrief = if let Some(ref warning) = state.simbrief_coverage_warning {
        column![
            simbrief,
            space::vertical().height(8),
            container(text(warning.clone()).size(13))
                .padding(8)
                .style(container::rounded_box)
                .width(Length::Fill)
        ]
    } else {
        simbrief
    };

    // -- Tiles section --
    let tiles = column![
        text("Tiles").size(18),
        rule::horizontal(1),
        row![
            text("Tile Provider:").width(Length::Fixed(160.0)),
            pick_list(
                PROVIDER_IDS,
                Some(state.config.tile_provider.as_str()),
                |s: &str| Message::SetTileProvider(s.to_string()),
            )
            .width(Length::Fixed(120.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Min Zoom: {}", state.config.min_zoom)).width(Length::Fixed(160.0)),
            slider(0..=20, state.config.min_zoom, Message::SetMinZoom).width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Max Zoom: {}", state.config.max_zoom)).width(Length::Fixed(160.0)),
            slider(0..=20, state.config.max_zoom, Message::SetMaxZoom).width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        // Dynamic Zoom section
        {
            let dz = DynamicZoom::new(state.config.zoom_rules.clone(), &state.config.tile_provider);
            let provider_max = dz.provider_max_zoom();
            let provider_name = PROVIDER_INFO
                .iter()
                .find(|p| p.id == state.config.tile_provider)
                .map(|p| p.display_name)
                .unwrap_or(&state.config.tile_provider);

            column![
                text("").size(8),
                text("Dynamic Zoom").size(16),
                rule::horizontal(1),
                row![
                    text("Enable:").width(Length::Fixed(80.0)),
                    button(text(if state.config.enable_dynamic_zoom {
                        "Enabled"
                    } else {
                        "Disabled"
                    }))
                    .on_press(Message::SetEnableDynamicZoom(
                        !state.config.enable_dynamic_zoom
                    )),
                    text(format!(
                        "  Provider: {} (max zoom: {})",
                        provider_name, provider_max
                    ))
                    .size(13)
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
                if !state.config.enable_dynamic_zoom {
                    column![text("Dynamic zoom is disabled").size(13)]
                } else {
                    column![
                        text(format!(
                            "{} zoom rule(s) configured",
                            state.config.zoom_rules.len()
                        ))
                        .size(13),
                        text("").size(4),
                        row![
                            text("Use SimBrief Altitude:").width(Length::Fixed(160.0)),
                            button(text(if state.config.use_simbrief_altitude {
                                "Enabled"
                            } else {
                                "Disabled"
                            }))
                            .on_press(
                                Message::SetUseSimBriefAltitude(
                                    !state.config.use_simbrief_altitude
                                )
                            ),
                        ]
                        .spacing(12)
                        .align_y(iced::Alignment::Center),
                    ]
                },
            ]
            .spacing(4)
        }
    ]
    .spacing(8);

    // -- SimHeaven compatibility section --
    let simheaven_section = column![
    text("SimHeaven Compatibility").size(18),
    rule::horizontal(1),
    row![
        text("Enable:").width(Length::Fixed(100.0)),
        button(text(if state.config.simheaven_compat {
            "Enabled"
        } else {
            "Disabled"
        }))
        .style(if state.config.simheaven_compat {
            button::success
        } else {
            button::secondary
        })
        .on_press(Message::SetSimHeavenCompat(!state.config.simheaven_compat)),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center),
    text("Enable if using SimHeaven X-World scenery.\nDisables AutoOrtho overlays to use SimHeaven instead.") 
        .size(12),
]
.spacing(8);

    // -- Debug section --
    let debug_section = column![
        text("Debug Mode").size(18),
        rule::horizontal(1),
        row![
            text("Enable Debug Logging:").width(Length::Fixed(100.0)),
            button(text(if state.config.debug_mode {
                "Enabled"
            } else {
                "Disabled"
            }))
            .style(if state.config.debug_mode {
                button::success
            } else {
                button::secondary
            })
            .on_press(Message::SetDebugMode(!state.config.debug_mode)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        text("Enables debug-level logging. Requires restart to take effect.").size(12),
    ]
    .spacing(8);

    // -- Cache section --
    let cache_size_mb = state.dds_cache_size_bytes / (1024 * 1024);
    let cache_max_mb = state.config.dds_cache_size_mb;
    let cache_section = column![
        text("Cache").size(18),
        rule::horizontal(1),
        row![
            text(format!(
                "DDS Cache: {} / {} MB",
                cache_size_mb, cache_max_mb
            ))
            .width(Length::Fixed(260.0)),
            button(text(format!("{} Clear", crate::ui::helpers::ICON_TRASH)).size(14))
                .padding([6, 16])
                .style(button::danger)
                .on_press(Message::ClearDdsCache),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!(
                "Max Cache Size: {} GB",
                state.config.dds_cache_size_mb / 1024
            ))
            .width(Length::Fixed(260.0)),
            slider(256u32..=16384, state.config.dds_cache_size_mb as u32, |v| {
                Message::SetDdsCacheSizeMb(v as u64)
            })
            .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text("Enable DDS Cache:").width(Length::Fixed(160.0)),
            button(
                text(if state.config.enable_dds_cache {
                    "Enabled"
                } else {
                    "Disabled"
                })
                .size(14)
            )
            .padding([6, 16])
            .style(if state.config.enable_dds_cache {
                button::success
            } else {
                button::secondary
            })
            .on_press(Message::SetEnableDdsCache(!state.config.enable_dds_cache)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        rule::horizontal(1),
        row![text(format!(
            "Memory: DDS {} MB, Chunks {} MB",
            state.config.dds_memory_cache_mb, state.config.chunk_memory_cache_mb
        ))]
        .spacing(12),
        tooltip(
            row![
                text("DDS Memory:").width(Length::Fixed(100.0)),
                slider(64u32..=2048, state.config.dds_memory_cache_mb as u32, |v| {
                    Message::SetDdsMemoryCacheMb(v as u64)
                })
                .width(Length::Fixed(150.0)),
                text(format!("{} MB", state.config.dds_memory_cache_mb))
                    .width(Length::Fixed(60.0))
                    .size(12),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(text("Max memory for cached DDS tiles (4096x4096 compressed). Higher values improve performance when revisiting areas. Takes effect on restart.").size(12))
                .padding(8)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text("Chunk Memory:").width(Length::Fixed(100.0)),
                slider(
                    128u32..=4096,
                    state.config.chunk_memory_cache_mb as u32,
                    |v| { Message::SetChunkMemoryCacheMb(v as u64) }
                )
                .width(Length::Fixed(150.0)),
                text(format!("{} MB", state.config.chunk_memory_cache_mb))
                    .width(Length::Fixed(60.0))
                    .size(12),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(text("Max memory for cached JPEG chunks (256x256 tiles). Higher values reduce network requests. Takes effect on restart.").size(12))
                .padding(8)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(8);

    // -- Advanced section --
    let night_threshold_i32 = state.config.night_threshold as i32;
    let day_threshold_i32 = state.config.day_threshold as i32;
    let advanced = column![
        text("Advanced").size(18),
        rule::horizontal(1),
        tooltip(
            row![
                text("Night Exclusion:").width(Length::Fixed(160.0)),
                button(
                    text(if state.config.enable_night_exclusion {
                        "Enabled"
                    } else {
                        "Disabled"
                    })
                    .size(14)
                )
                .padding([6, 16])
                .style(if state.config.enable_night_exclusion {
                    button::success
                } else {
                    button::secondary
                })
                .on_press(Message::SetEnableNightExclusion(
                    !state.config.enable_night_exclusion
                )),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("When enabled, AutoOrtho returns blank tiles at night so X-Plane uses its default scenery. Uses the sim's sun elevation angle.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        row![
            text(format!("Night Threshold: {}°", night_threshold_i32))
                .width(Length::Fixed(260.0)),
            slider(-20..=0i32, night_threshold_i32, Message::SetNightThreshold)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Day Threshold: {}°", day_threshold_i32)).width(Length::Fixed(260.0)),
            slider(-20..=0i32, day_threshold_i32, Message::SetDayThreshold)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // -- Seasonal section --
    let season_index = match state.config.season {
        Season::Disabled => 0,
        Season::Spring => 1,
        Season::Summer => 2,
        Season::Autumn => 3,
        Season::Winter => 4,
    };
    let spring_pct = (state.config.spring_saturation * 100.0).round() as u32;
    let summer_pct = (state.config.summer_saturation * 100.0).round() as u32;
    let autumn_pct = (state.config.autumn_saturation * 100.0).round() as u32;
    let winter_pct = (state.config.winter_saturation * 100.0).round() as u32;

    let seasonal = column![
        text("Seasonal Adjustment").size(18),
        rule::horizontal(1),
        row![
            text("Season:").width(Length::Fixed(160.0)),
            pick_list(
                SEASON_LABELS,
                Some(SEASON_LABELS[season_index]),
                |s: &str| {
                    let idx = SEASON_LABELS.iter().position(|&x| x == s).unwrap_or(0);
                    Message::SetSeason(SEASONS[idx])
                },
            )
            .width(Length::Fixed(120.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Spring: {}%", spring_pct)).width(Length::Fixed(160.0)),
            slider(0..=200u32, spring_pct, Message::SetSpringSaturation)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Summer: {}%", summer_pct)).width(Length::Fixed(160.0)),
            slider(0..=200u32, summer_pct, Message::SetSummerSaturation)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Autumn: {}%", autumn_pct)).width(Length::Fixed(160.0)),
            slider(0..=200u32, autumn_pct, Message::SetAutumnSaturation)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Winter: {}%", winter_pct)).width(Length::Fixed(160.0)),
            slider(0..=200u32, winter_pct, Message::SetWinterSaturation)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // -- Fallback section --
    let fallback_level_index = match state.config.fallback.level {
        FallbackLevel::Cache => 0,
        FallbackLevel::Downserve => 1,
        FallbackLevel::Network => 2,
        FallbackLevel::Solid => 3,
    };
    let fallback = column![
        text("Fallback").size(18),
        rule::horizontal(1),
        tooltip(
            row![
                text("Fallback Level:").width(Length::Fixed(160.0)),
                pick_list(
                    FALLBACK_LABELS,
                    Some(FALLBACK_LABELS[fallback_level_index]),
                    |s: &str| {
                        let idx = FALLBACK_LABELS.iter().position(|&x| x == s).unwrap_or(0);
                        Message::SetFallbackLevel(FALLBACK_LEVELS[idx])
                    },
                )
                .width(Length::Fixed(120.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Cache: Use lower-zoom cached tiles. Downserve: Scale from lower-res tile. Network: Download on-demand. Solid: Solid color fallback.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text(format!(
                    "Max Zoom Gap: {}",
                    state.config.fallback.max_zoom_gap
                ))
                .width(Length::Fixed(260.0)),
                slider(
                    1u32..=8,
                    state.config.fallback.max_zoom_gap,
                    Message::SetFallbackMaxZoomGap
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Maximum zoom levels to downserve when using cache fallback. Higher values allow more aggressive fallback but lower quality.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
        tooltip(
            row![
                text("Cache Fallback:").width(Length::Fixed(160.0)),
                button(
                    text(if state.config.fallback.cache_fallback {
                        "Enabled"
                    } else {
                        "Disabled"
                    })
                    .size(14)
                )
                .padding([6, 16])
                .style(if state.config.fallback.cache_fallback {
                    button::success
                } else {
                    button::secondary
                })
                .on_press(Message::SetFallbackCacheEnabled(
                    !state.config.fallback.cache_fallback
                )),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Check disk cache for lower-zoom tiles before generating. Reduces network requests for areas with partial coverage.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(8);

    // -- Rate Limiting section --
    let rate_value = state.config.rate_limit.requests_per_second.round() as u32;
    let rate_limit = column![
        text("Rate Limiting").size(18),
        rule::horizontal(1),
        tooltip(
            row![
                text(format!("Requests/sec: {}", rate_value)).width(Length::Fixed(160.0)),
                slider(1u32..=20, rate_value, |v| Message::SetRateLimit(v as f64))
                .width(Length::Fixed(200.0)),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(
                text("Limit HTTP requests to tile providers. Lower values are safer but slower. Default: 5 req/sec.")
                    .size(12),
            )
            .padding(8)
            .style(container::rounded_box),
            tooltip::Position::Bottom,
        ),
    ]
    .spacing(8);

    // -- UI section --
    // Scale slider: 50% to 150%, stored as f64 (0.5 to 1.5)
    // Slider works with integers, so we use 50..150 and divide by 100
    let scale_pct = (state.config.ui_scale * 100.0).round() as u32;
    let ui_section = column![
        text("Interface").size(18),
        rule::horizontal(1),
        row![
            text(format!("UI Scale: {}%", scale_pct)).width(Length::Fixed(160.0)),
            slider(50u32..=150, scale_pct, |v| Message::SetUIScale(
                v as f64 / 100.0
            ))
            .width(Length::Fixed(200.0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // Error and save
    let error_text = if let Some(ref err) = state.error_message {
        text(err.clone())
            .size(14)
            .color(iced::Color::from_rgb(0.8, 0.1, 0.1))
    } else {
        text("").size(14)
    };

    let save_row = row![
        button(
            text(format!(
                "{} Save Configuration",
                crate::ui::helpers::ICON_CHECK
            ))
            .size(14)
        )
        .padding([10, 24])
        .style(button::success)
        .on_press(Message::SaveConfiguration),
        button(text(format!("{} Reload", crate::ui::helpers::ICON_REFRESH)).size(14))
            .padding([10, 24])
            .on_press(Message::LoadConfiguration),
    ]
    .spacing(12);

    let content = column![
        title,
        space::vertical().height(16),
        paths,
        space::vertical().height(16),
        network,
        space::vertical().height(16),
        simbrief,
        space::vertical().height(16),
        tiles,
        space::vertical().height(16),
        cache_section,
        space::vertical().height(16),
        simheaven_section,
        space::vertical().height(16),
        debug_section,
        space::vertical().height(16),
        advanced,
        space::vertical().height(16),
        seasonal,
        space::vertical().height(16),
        fallback,
        space::vertical().height(16),
        rate_limit,
        space::vertical().height(16),
        ui_section,
        space::vertical().height(16),
        error_text,
        save_row,
    ]
    .spacing(4)
    .padding(32)
    .max_width(700);

    iced::widget::scrollable(container(content).center_x(Fill))
        .height(Fill)
        .into()
}

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(160.0)),
        text_input("", value).on_input(on_input).width(Length::Fill),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center)
    .into()
}

fn labeled_path_input<'a>(
    label: &'a str,
    value: &'a str,
    on_input: fn(String) -> Message,
    on_browse: Message,
) -> Element<'a, Message> {
    let space = crate::ui::helpers::disk_space_label(value);
    row![
        text(label).width(Length::Fixed(160.0)),
        text_input("", value).on_input(on_input).width(Length::Fill),
        button(text(format!("{} Browse", crate::ui::helpers::ICON_FOLDER)).size(13))
            .padding([6, 12])
            .on_press(on_browse),
        text(space).size(12),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
