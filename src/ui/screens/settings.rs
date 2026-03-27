use crate::ui::Message;
use crate::ui::state::AppState;
use iced::widget::{
    button, column, container, pick_list, row, rule, slider, space, text, text_input,
};
use iced::{Element, Fill, Length};

const PROVIDERS: &[&str] = &["GO2", "BI", "ARC", "NAIP", "USGS", "EOX", "FIREFLY"];

/// Settings screen — full configuration management
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Settings").size(28);

    // -- Paths section --
    let paths = column![
        text("Paths").size(18),
        rule::horizontal(1),
        labeled_path_input(
            "Mount Directory:",
            &state.config.mount_dir,
            Message::SetMountDir,
            Message::BrowseMountDir
        ),
        labeled_path_input(
            "Cache Directory:",
            &state.config.cache_dir,
            Message::SetCacheDir,
            Message::BrowseCacheDir
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

    // -- Tiles section --
    let tiles = column![
        text("Tiles").size(18),
        rule::horizontal(1),
        row![
            text("Tile Provider:").width(Length::Fixed(160.0)),
            pick_list(
                PROVIDERS,
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
    ]
    .spacing(8);

    // -- Advanced section --
    let advanced = column![
        text("Advanced").size(18),
        rule::horizontal(1),
        row![
            text("Night Exclusion:").width(Length::Fixed(160.0)),
            text(if state.config.enable_night_exclusion {
                "Enabled"
            } else {
                "Disabled"
            }),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
        row![
            text("Night Threshold:").width(Length::Fixed(160.0)),
            text(format!("{}°", state.config.night_threshold)),
        ]
        .spacing(12),
        row![
            text("Day Threshold:").width(Length::Fixed(160.0)),
            text(format!("{}°", state.config.day_threshold)),
        ]
        .spacing(12),
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
        tiles,
        space::vertical().height(16),
        advanced,
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
