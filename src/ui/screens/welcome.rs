use crate::ui::Message;
use crate::ui::state::Screen;
use iced::widget::space;
use iced::widget::{button, column, container, text};
use iced::{Element, Fill, Length};

/// Welcome screen — introduction and setup choice
pub fn view() -> Element<'static, Message> {
    let content = column![
        space::vertical().height(Length::FillPortion(1)),
        text("AutoOrtho").size(48),
        text("Satellite Imagery for X-Plane").size(20),
        space::vertical().height(40),
        button(text("Setup Wizard").size(16))
            .padding([12, 32])
            .on_press(Message::GoToScreen(Screen::SetupWizard)),
        space::vertical().height(12),
        button(text("Dashboard").size(16))
            .padding([12, 32])
            .on_press(Message::GoToScreen(Screen::Dashboard)),
        space::vertical().height(12),
        button(text("About").size(14))
            .padding([8, 24])
            .on_press(Message::GoToScreen(Screen::About)),
        space::vertical().height(Length::FillPortion(2)),
    ]
    .align_x(iced::Alignment::Center)
    .spacing(4);

    container(content).center_x(Fill).center_y(Fill).into()
}
