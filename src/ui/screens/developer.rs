use crate::tiles::provider;
use crate::ui::Message;
use crate::ui::state::AppState;
use iced::widget::{
    button, column, container, image as iced_image, pick_list, row, rule, scrollable, slider,
    space, text, text_input,
};
use iced::{Element, Fill, Length};

const PROVIDERS: &[&str] = &["ARC", "BI", "GO2"];

/// Developer tools screen — testing and diagnostics
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Developer Tools").size(28);

    // Provider info for zoom limits
    let info = provider::provider_info(&state.config.tile_provider);
    let min_z = info.map_or(0, |p| p.min_zoom);
    let max_z = info.map_or(19, |p| p.max_zoom);

    // --- Test Tile Fetch ---
    let test_section = column![
        text("Test Tile Fetch").size(18),
        rule::horizontal(1),
        text("Fetch satellite tiles and assemble into a DDS texture.").size(13),
        space::vertical().height(8),
        provider_row(state),
        row![
            text("Latitude:").width(Length::Fixed(80.0)),
            text_input("-33.86", &state.test_tile_lat)
                .on_input(Message::SetTestLat)
                .width(Length::Fixed(120.0)),
            text("Longitude:").width(Length::Fixed(80.0)),
            text_input("151.21", &state.test_tile_lon)
                .on_input(Message::SetTestLon)
                .width(Length::Fixed(120.0)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        row![
            text(format!("Zoom: {}", state.test_tile_zoom)).width(Length::Fixed(80.0)),
            slider(min_z..=max_z, state.test_tile_zoom, Message::SetTestZoom)
                .width(Length::Fixed(300.0)),
            text(format!("{}", min_z)).size(12),
            text("—").size(12),
            text(format!("{}", max_z)).size(12),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        space::vertical().height(4),
    ]
    .spacing(6);

    let fetch_button = if state.test_tile_running {
        button(text("Fetching...").size(14)).padding([10, 24])
    } else {
        button(text("Fetch Test Tile").size(14))
            .padding([10, 24])
            .on_press(Message::FetchTestTile)
    };

    let action_row = row![fetch_button].spacing(12);

    let status_text: Element<'_, Message> = if let Some(ref status) = state.test_tile_status {
        let color = if status.starts_with("Error") || status.contains("failed") {
            iced::Color::from_rgb(0.8, 0.1, 0.1)
        } else if status.starts_with("Fetching") {
            iced::Color::from_rgb(0.4, 0.4, 0.4)
        } else {
            iced::Color::from_rgb(0.0, 0.5, 0.0)
        };
        text(status.clone()).size(13).color(color).into()
    } else {
        text("No test run yet.").size(13).into()
    };

    // --- Image preview (constrained to fit) ---
    let preview: Element<'_, Message> = if let Some((w, h, ref rgba)) = state.test_tile_image {
        let handle = iced_image::Handle::from_rgba(w, h, rgba.clone());
        let display_size = 400.0f32.min(w as f32);
        column![
            space::vertical().height(8),
            text(format!("Preview ({}×{})", w, h)).size(14),
            iced_image(handle)
                .width(Length::Fixed(display_size))
                .height(Length::Fixed(display_size)),
        ]
        .spacing(4)
        .into()
    } else {
        space::vertical().height(0).into()
    };

    // --- Presets ---
    let presets = column![
        space::vertical().height(12),
        text("Quick Presets").size(16),
        row![
            preset_button("Sydney", "-33.86", "151.21"),
            preset_button("London", "51.50", "-0.12"),
            preset_button("New York", "40.71", "-74.01"),
            preset_button("Tokyo", "35.68", "139.69"),
        ]
        .spacing(8),
        row![
            preset_button("Los Angeles", "33.94", "-118.41"),
            preset_button("Paris", "48.86", "2.35"),
            preset_button("Dubai", "25.20", "55.27"),
            preset_button("Singapore", "1.35", "103.82"),
        ]
        .spacing(8),
    ]
    .spacing(6);

    // --- Notes ---
    let notes = column![
        space::vertical().height(12),
        text("GO2 (Google) requires browser-style User-Agent header and may block automated requests.").size(12),
        text("ARC (ArcGIS) and BI (Bing) are recommended for reliable tile fetching.").size(12),
        space::vertical().height(20),
    ];

    let content = column![
        title,
        space::vertical().height(12),
        test_section,
        action_row,
        space::vertical().height(4),
        status_text,
        preview,
        presets,
        notes,
    ]
    .spacing(4)
    .padding(32)
    .max_width(780);

    scrollable(container(content).center_x(Fill))
        .height(Fill)
        .into()
}

fn provider_row(state: &AppState) -> Element<'_, Message> {
    let info = provider::provider_info(&state.config.tile_provider);
    let auth_warning: Element<'_, Message> = if info.is_some_and(|p| p.requires_auth) {
        text("Requires auth")
            .size(12)
            .color(iced::Color::from_rgb(0.8, 0.5, 0.0))
            .into()
    } else {
        text("").size(12).into()
    };

    row![
        text("Provider:").width(Length::Fixed(80.0)),
        pick_list(
            PROVIDERS,
            Some(state.config.tile_provider.as_str()),
            |s: &str| Message::SetTileProvider(s.to_string()),
        )
        .width(Length::Fixed(100.0)),
        auth_warning,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Preset buttons no longer encode zoom — the slider controls that independently.
fn preset_button(name: &'static str, lat: &str, lon: &str) -> Element<'static, Message> {
    let payload = format!("{}|{}|keep", lat, lon);
    button(text(name).size(13))
        .padding([6, 12])
        .on_press(Message::SetTestLat(payload))
        .into()
}
