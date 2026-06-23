use crate::ui::Message;
use crate::ui::helpers::*;
use crate::ui::state::{AppState, ServiceStatus};
use iced::widget::{button, column, container, row, rule, space, text};
use iced::{Element, Fill, Length};

/// Main dashboard screen — real-time status and controls
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Dashboard").size(28);

    // --- Start/Stop controls ---
    let controls: Element<'_, Message> = if state.any_service_running() {
        let mut btns = row![
            button(text(format!("{} Stop", ICON_STOP)).size(13))
                .padding([8, 16])
                .style(button::danger)
                .on_press(Message::StopServices),
        ]
        .spacing(8);

        if state.services.web_server.is_running() {
            btns = btns.push(
                button(text(format!("{} Web UI", ICON_GLOBE)).size(13))
                    .padding([8, 16])
                    .on_press(Message::OpenWebUI),
            );
            btns = btns.push(
                button(text(format!("{} Flight Map", ICON_MAP)).size(13))
                    .padding([8, 16])
                    .on_press(Message::OpenMapInBrowser),
            );
            btns = btns.push(
                button(text(format!("{} Map Editor", ICON_SETTINGS)).size(13))
                    .padding([8, 16])
                    .on_press(Message::OpenCustomMapEditor),
            );
        }

        btns.wrap().into()
    } else if state.scenery_dir_valid() {
        row![
            button(text(format!("{} Start AutoOrtho", ICON_PLAY)).size(16))
                .padding([12, 32])
                .style(button::success)
                .on_press(Message::StartServices),
        ]
        .into()
    } else {
        column![
            button(text(format!("{} Start AutoOrtho", ICON_PLAY)).size(16)).padding([12, 32]),
            text(format!(
                "{} X-Plane folder does not appear valid (no scenery_packs.ini found) — check Settings",
                ICON_WARNING
            ))
            .size(13)
            .color(iced::Color::from_rgb(0.9, 0.7, 0.0)),
        ]
        .spacing(8)
        .into()
    };

    // --- Service status section ---
    let web_status = status_indicator(
        "Web Server",
        state.services.web_server,
        &state.services.web_server_url,
    );
    let xp_status = status_indicator("X-Plane Tracker", state.services.xplane_tracker, &None);

    let status_section = column![
        text("Services").size(18),
        rule::horizontal(1),
        web_status,
        xp_status,
    ]
    .spacing(6);

    // --- Flight Plan section ---
    let flight_plan_section = {
        let mut section = column![text("Flight Plan").size(18), rule::horizontal(1),].spacing(6);

        if state.config.flight.simbrief_user_id.is_empty() {
            section = section.push(
                text("Set SimBrief User ID in Settings to fetch flight plans.")
                    .size(13)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            );
        } else if state.simbrief.fetching {
            section = section.push(button(text("Fetching...").size(14)).padding([8, 16]));
        } else {
            let mut btn_row = row![
                button(text(format!("{} Fetch Flight Plan", ICON_GLOBE)).size(14))
                    .padding([8, 16])
                    .style(button::success)
                    .on_press(Message::FetchSimbrief),
            ]
            .spacing(8);

            if state.simbrief.route_summary.is_some() {
                if state.prefetch.running {
                    btn_row = btn_row.push(
                        button(text(format!("{} Stop Prefetch", ICON_STOP)).size(14))
                            .padding([8, 16])
                            .style(button::danger)
                            .on_press(Message::StopPrefetch),
                    );
                } else {
                    btn_row = btn_row.push(
                        button(text(format!("{} Prefetch Route", ICON_MAP)).size(14))
                            .padding([8, 16])
                            .style(button::success)
                            .on_press(Message::PrefetchRoute),
                    );
                }
            }

            section = section.push(btn_row);

            // Prefetch progress / status
            if state.prefetch.running {
                let progress_text = if state.prefetch.total > 0 {
                    format!(
                        "{} Pre-caching tiles: {}/{}",
                        ICON_MAP, state.prefetch.completed, state.prefetch.total
                    )
                } else {
                    format!("{} Pre-caching route tiles…", ICON_MAP)
                };
                section = section.push(
                    text(progress_text)
                        .size(13)
                        .color(iced::Color::from_rgb(0.0, 0.6, 0.0)),
                );
            } else if let Some(ref status) = state.prefetch.status {
                let color = if status.to_lowercase().contains("error")
                    || status.to_lowercase().contains("fail")
                {
                    iced::Color::from_rgb(0.8, 0.1, 0.1)
                } else if status.contains("cache 90") {
                    iced::Color::from_rgb(0.9, 0.6, 0.1)
                } else {
                    iced::Color::from_rgb(0.0, 0.6, 0.0)
                };
                section = section.push(
                    text(format!("{} {}", ICON_MAP, status))
                        .size(13)
                        .color(color),
                );
            }
        }

        if let Some(ref summary) = state.simbrief.route_summary {
            let arrow = if state.simbrief.show_details {
                "\u{25BC}"
            } else {
                "\u{25B6}"
            };
            section = section.push(
                button(
                    text(format!("{} {} {}", ICON_MAP, summary, arrow))
                        .size(14)
                        .color(iced::Color::from_rgb(0.0, 0.6, 0.0)),
                )
                .padding([4, 8])
                .style(button::text)
                .on_press(Message::ToggleSimbriefDetails),
            );

            if state.simbrief.show_details && !state.simbrief.fixes.is_empty() {
                let mut fixes_col = column![].spacing(1);
                for (fix_idx, (ident, fix_type, alt)) in state.simbrief.fixes.iter().enumerate() {
                    let label = if ident == "TOC" || ident == "TOD" {
                        format!("[{}]", ident)
                    } else {
                        ident.clone()
                    };
                    let is_special = ident == "TOC" || ident == "TOD" || fix_type == "apt";
                    let color = if is_special {
                        iced::Color::from_rgb(0.8, 0.8, 0.8)
                    } else {
                        iced::Color::from_rgb(0.5, 0.5, 0.5)
                    };
                    // Show prefetch progress emoji if available
                    let emoji = if fix_idx < state.prefetch.waypoint_status.len() {
                        state.prefetch.waypoint_status[fix_idx].emoji()
                    } else {
                        ""
                    };
                    fixes_col = fixes_col.push(
                        row![
                            text(format!("{} {}", emoji, label))
                                .size(13)
                                .width(Length::Fixed(140.0))
                                .color(color),
                            text(format!("{:.0} ft", alt))
                                .size(13)
                                .width(Length::Fixed(100.0))
                                .color(color),
                        ]
                        .spacing(8),
                    );
                }
                section = section.push(fixes_col);
            }
        }

        if let Some(ref err) = state.simbrief.error {
            section = section.push(
                text(err.clone())
                    .size(13)
                    .color(iced::Color::from_rgb(0.8, 0.1, 0.1)),
            );
        }

        if let Some(ref warning) = state.simbrief.coverage_warning {
            section = section.push(
                text(warning.clone())
                    .size(13)
                    .color(iced::Color::from_rgb(0.9, 0.6, 0.1)),
            );
        }

        section
    };

    // --- Configuration summary ---
    let config_section = column![
        text("Configuration").size(18),
        rule::horizontal(1),
        config_row("Tile Provider:", state.config.tile.provider.clone()),
        config_row(
            "Zoom Range:",
            format!(
                "{} – {}",
                state.config.tile.min_zoom, state.config.tile.max_zoom
            )
        ),
        config_row("X-Plane:", state.config.xplane_path.clone()),
        config_row("Cache Dir:", state.config.cache_dir.clone()),
        config_row(
            "Night Exclusion:",
            if state.config.night.enable_night_exclusion {
                "Enabled".into()
            } else {
                "Disabled".into()
            }
        ),
        config_row(
            "Season:",
            match state.config.season_cfg.season {
                crate::config::Season::Disabled => "Disabled".into(),
                crate::config::Season::Spring => "Spring".into(),
                crate::config::Season::Summer => "Summer".into(),
                crate::config::Season::Autumn => "Autumn".into(),
                crate::config::Season::Winter => "Winter".into(),
            }
        ),
    ]
    .spacing(6);

    // --- Error message ---
    let error_section = if let Some(ref err) = state.error_message {
        column![
            text(err.clone())
                .size(14)
                .color(iced::Color::from_rgb(0.8, 0.1, 0.1)),
        ]
    } else {
        column![]
    };

    let content = column![
        title,
        space::vertical().height(16),
        controls,
        space::vertical().height(20),
        status_section,
        space::vertical().height(20),
        flight_plan_section,
        space::vertical().height(20),
        config_section,
        space::vertical().height(12),
        error_section,
    ]
    .spacing(4)
    .padding(32)
    .max_width(700);

    container(content).center_x(Fill).into()
}

fn status_indicator<'a>(
    label: &'a str,
    status: ServiceStatus,
    url: &'a Option<String>,
) -> Element<'a, Message> {
    let status_color = match status {
        ServiceStatus::Stopped => iced::Color::from_rgb(0.5, 0.5, 0.5),
        ServiceStatus::Starting => iced::Color::from_rgb(0.8, 0.6, 0.0),
        ServiceStatus::Running => iced::Color::from_rgb(0.0, 0.6, 0.0),
        ServiceStatus::Error => iced::Color::from_rgb(0.8, 0.1, 0.1),
    };

    let mut r = row![
        text(format!("{}:", label)).width(Length::Fixed(160.0)),
        text(status.label()).color(status_color),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    if let Some(u) = url {
        r = r.push(text(format!("  ({})", u)).size(13));
    }

    r.into()
}

fn config_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).width(Length::Fixed(160.0)),
        text(value).size(14),
    ]
    .spacing(8)
    .into()
}
