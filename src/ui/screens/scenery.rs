use crate::ui::Message;
use crate::ui::helpers;
use crate::ui::state::AppState;
use iced::widget::{button, column, container, progress_bar, row, rule, scrollable, space, text};
use iced::{Element, Fill, Length};

/// Scenery pack management screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = text("Scenery Packs").size(28);

    // --- Refresh + status ---
    let refresh_row = {
        let btn = if state.scenery_refreshing {
            button(text(format!("{} Refreshing...", helpers::ICON_REFRESH)).size(13))
                .padding([8, 14])
        } else {
            button(text(format!("{} Refresh", helpers::ICON_REFRESH)).size(13))
                .padding([8, 14])
                .on_press(Message::RefreshAvailableRegions)
        };

        let status: Element<'_, Message> = match &state.scenery_status {
            Some(s) => {
                let color = if s.starts_with("Error") {
                    iced::Color::from_rgb(0.8, 0.1, 0.1)
                } else if s.contains("ancel") {
                    iced::Color::from_rgb(0.7, 0.5, 0.0)
                } else {
                    iced::Color::from_rgb(0.5, 0.5, 0.5)
                };
                text(s.clone()).size(13).color(color).into()
            }
            None => text("").into(),
        };

        row![btn, status]
            .spacing(12)
            .align_y(iced::Alignment::Center)
    };

    // --- Region list ---
    let region_list = if state.available_regions.is_empty() && !state.scenery_refreshing {
        column![text("Click Refresh to check for available scenery packs.").size(14)]
    } else {
        let mut col = column![].spacing(2);

        // Table header
        col = col.push(
            row![
                text("Region").size(12).width(Length::FillPortion(4)),
                text("Size").size(12).width(Length::FillPortion(2)),
                text("").size(12).width(Length::FillPortion(4)),
            ]
            .spacing(8)
            .padding([4, 0]),
        );
        col = col.push(rule::horizontal(1));

        for region in &state.available_regions {
            col = col.push(region_row(state, region));
        }

        col
    };

    let content = column![
        title,
        space::vertical().height(10),
        refresh_row,
        space::vertical().height(10),
        region_list,
        space::vertical().height(20),
    ]
    .spacing(2)
    .padding(24)
    .max_width(850);

    scrollable(container(content).center_x(Fill))
        .height(Fill)
        .into()
}

fn region_row<'a>(
    state: &'a AppState,
    region: &'a crate::ui::state::SceneryRegionInfo,
) -> Element<'a, Message> {
    let size_gb = region.total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let size_str = if size_gb >= 1.0 {
        format!("{:.1} GB", size_gb)
    } else {
        format!("{:.0} MB", region.total_size_bytes as f64 / 1_048_576.0)
    };

    let installed_ver = state
        .installed_packs
        .iter()
        .find(|p| p.id == region.id)
        .map(|p| p.version.clone());
    let installed = installed_ver.is_some();
    let update_available = installed_ver.as_ref().is_some_and(|v| v != &region.version);
    let downloading = state.downloading_regions.get(&region.id);

    // Name column
    let name_col: Element<'a, Message> = column![
        text(&region.name).size(15),
        text(format!(
            "v{} · {} files",
            region.version, region.package_count
        ))
        .size(12),
    ]
    .spacing(2)
    .width(Length::FillPortion(4))
    .into();

    // Size column
    let size_col: Element<'a, Message> =
        text(size_str).size(14).width(Length::FillPortion(2)).into();

    // Actions column — primary + secondary buttons in one row
    let action_col: Element<'a, Message> = if let Some(dl) = downloading {
        let pct = dl.progress_percent();
        let info = format!(
            "{:.0}/{:.0} MB · {}/{} files",
            dl.downloaded_mb(),
            dl.total_mb(),
            dl.files_completed(),
            dl.files_total,
        );
        column![
            row![
                progress_bar(0.0..=100.0, pct),
                text(format!("{:.0}%", pct))
                    .size(12)
                    .width(Length::Fixed(36.0)),
                button(text(format!("{} Cancel", helpers::ICON_TIMES)).size(12))
                    .padding([4, 10])
                    .on_press(Message::CancelDownload(region.id.clone())),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
            text(info).size(11),
        ]
        .spacing(2)
        .width(Length::FillPortion(4))
        .into()
    } else if installed && !update_available {
        row![
            button(
                text(format!(
                    "{} Installed v{}",
                    helpers::ICON_CHECK,
                    installed_ver.as_deref().unwrap_or("?")
                ))
                .size(13)
            )
            .padding([5, 14])
            .style(button::success),
            button(text(format!("{} Uninstall", helpers::ICON_TRASH)).size(11))
                .padding([4, 10])
                .style(button::danger)
                .on_press(Message::UninstallRegion(region.id.clone())),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::FillPortion(4))
        .into()
    } else if update_available {
        row![
            button(
                text(format!(
                    "{} Update → v{}",
                    helpers::ICON_DOWNLOAD,
                    region.version
                ))
                .size(12)
            )
            .padding([5, 12])
            .on_press(Message::DownloadRegion(region.id.clone())),
            text(format!("have v{}", installed_ver.as_deref().unwrap_or("?"))).size(11),
            button(text(format!("{} Uninstall", helpers::ICON_TRASH)).size(11))
                .padding([4, 10])
                .style(button::danger)
                .on_press(Message::UninstallRegion(region.id.clone())),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::FillPortion(4))
        .into()
    } else {
        let dl_label = if region.has_partial_download {
            format!("{} Resume", helpers::ICON_PLAY)
        } else {
            format!("{} Download", helpers::ICON_CLOUD_DL)
        };
        let mut actions = row![
            button(text(dl_label).size(13))
                .padding([5, 14])
                .on_press(Message::DownloadRegion(region.id.clone())),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        if region.has_partial_download {
            actions = actions.push(
                button(text(format!("{} Clean", helpers::ICON_TRASH)).size(11))
                    .padding([4, 10])
                    .style(button::danger)
                    .on_press(Message::CleanRegionDownloads(region.id.clone())),
            );
        }

        actions.width(Length::FillPortion(4)).into()
    };

    container(
        row![name_col, size_col, action_col]
            .spacing(8)
            .align_y(iced::Alignment::Center),
    )
    .padding([8, 0])
    .into()
}
