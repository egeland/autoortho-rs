use crate::tiles::provider::PROVIDER_IDS;
use crate::ui::Message;
use crate::ui::state::{AppState, Screen};
use iced::widget::space;
use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Fill, Length};

/// Setup wizard screen — initial configuration
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Setup Wizard").size(28);

    // X-Plane folder with browse button
    let xplane_row = row![
        text("X-Plane Folder:").width(Length::Fixed(140.0)),
        text_input("~/X-Plane 12", &state.config.xplane_path)
            .on_input(Message::SetXPlanePath)
            .width(Length::Fill),
        button(text("Browse").size(13))
            .padding([6, 12])
            .on_press(Message::BrowseXPlanePath),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Cache directory with browse button
    let cache_row = row![
        text("Cache Directory:").width(Length::Fixed(140.0)),
        text_input("~/.cache/autoortho", &state.config.cache_dir)
            .on_input(Message::SetCacheDir)
            .width(Length::Fill),
        button(text("Browse").size(13))
            .padding([6, 12])
            .on_press(Message::BrowseCacheDir),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // X-Plane host
    let host_row = row![
        text("X-Plane Host:").width(Length::Fixed(140.0)),
        text_input("127.0.0.1", &state.config.xplane_host)
            .on_input(Message::SetXPlaneHost)
            .width(Length::Fixed(200.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // X-Plane port
    let port_row = row![
        text("X-Plane Port:").width(Length::Fixed(140.0)),
        text_input("49000", &state.config.xplane_port.to_string())
            .on_input(Message::SetXPlanePort)
            .width(Length::Fixed(100.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Tile provider
    let provider_row = row![
        text("Tile Provider:").width(Length::Fixed(140.0)),
        pick_list(
            PROVIDER_IDS,
            Some(state.config.tile.provider.as_str()),
            |s: &str| Message::SetTileProvider(s.to_string()),
        )
        .width(Length::Fixed(120.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Zoom range
    let zoom_row = row![
        text("Zoom Range:").width(Length::Fixed(140.0)),
        text(format!(
            "{} – {}",
            state.config.tile.min_zoom, state.config.tile.max_zoom
        ))
        .size(16),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Error message
    let error_text = if let Some(ref err) = state.error_message {
        text(err.clone())
            .size(14)
            .color(iced::Color::from_rgb(0.8, 0.1, 0.1))
    } else {
        text("").size(14)
    };

    // Buttons
    let buttons = row![
        button(text("Save & Continue").size(14))
            .padding([10, 24])
            .on_press(Message::SaveConfiguration),
        button(text("Dashboard").size(14))
            .padding([10, 24])
            .on_press(Message::GoToScreen(Screen::Dashboard)),
    ]
    .spacing(16);

    let form = column![
        title,
        space::vertical().height(24),
        xplane_row,
        cache_row,
        space::vertical().height(12),
        host_row,
        port_row,
        space::vertical().height(12),
        provider_row,
        zoom_row,
        space::vertical().height(16),
        error_text,
        space::vertical().height(8),
        buttons,
    ]
    .spacing(10)
    .padding(32)
    .max_width(650);

    container(form).center_x(Fill).center_y(Fill).into()
}
