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

        if state.web_server.is_running() {
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
                "{} Scenery Install directory does not contain scenery_packs.ini — check Settings",
                ICON_WARNING
            ))
            .size(13)
            .color(iced::Color::from_rgb(0.9, 0.7, 0.0)),
        ]
        .spacing(8)
        .into()
    };

    // --- Service status section ---
    let web_status = status_indicator("Web Server", state.web_server, &state.web_server_url);
    let xp_status = status_indicator("X-Plane Tracker", state.xplane_tracker, &None);

    let status_section = column![
        text("Services").size(18),
        rule::horizontal(1),
        web_status,
        xp_status,
    ]
    .spacing(6);

    // --- Configuration summary ---
    let config_section = column![
        text("Configuration").size(18),
        rule::horizontal(1),
        config_row("Tile Provider:", state.config.tile_provider.clone()),
        config_row(
            "Zoom Range:",
            format!("{} – {}", state.config.min_zoom, state.config.max_zoom)
        ),
        config_row("X-Plane:", state.config.xplane_path.clone()),
        config_row("Cache Dir:", state.config.cache_dir.clone()),
        config_row(
            "Night Exclusion:",
            if state.config.enable_night_exclusion {
                "Enabled".into()
            } else {
                "Disabled".into()
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
