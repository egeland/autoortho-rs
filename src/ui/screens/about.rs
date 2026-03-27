use crate::ui::Message;
use iced::widget::space;
use iced::widget::{column, container, text};
use iced::{Element, Fill, Length};

/// About screen — application information
pub fn view() -> Element<'static, Message> {
    let content = column![
        space::vertical().height(Length::FillPortion(1)),
        text("AutoOrtho").size(36),
        text("v0.1.0").size(16),
        space::vertical().height(20),
        text("Satellite Imagery for X-Plane"),
        space::vertical().height(12),
        text("A Rust rewrite of the original Python/C implementation.").size(14),
        text("High-performance FUSE virtual filesystem serving").size(14),
        text("on-demand DDS textures from satellite imagery providers.").size(14),
        space::vertical().height(20),
        text("© 2026 AutoOrtho Contributors").size(13),
        space::vertical().height(Length::FillPortion(2)),
    ]
    .align_x(iced::Alignment::Center)
    .spacing(4);

    container(content).center_x(Fill).center_y(Fill).into()
}
