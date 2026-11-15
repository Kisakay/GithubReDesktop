// SPDX-License-Identifier: MIT

//! Welcome screen shown when no repository is open.

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;

use super::dim_text;
use super::icons::{self, Icon};
use crate::app::{APP_ICON, App, ContextPage, MenuAction, Message};
use crate::fl;

pub fn view(app: &App) -> Element<'_, Message> {
    let card = |icon: Icon, title: String, body: String, msg: Message| {
        widget::button::custom(
            widget::row::with_capacity(3)
                .push(icons::icon(icon, 28))
                .push(
                    widget::column::with_capacity(2)
                        .push(widget::text::heading(title))
                        .push(widget::text::caption(body).class(cosmic::theme::Text::Custom(dim_text)))
                        .spacing(2)
                        .width(Length::Fill),
                )
                .spacing(16)
                .align_y(Alignment::Center),
        )
        .class(cosmic::theme::Button::ListItem([12.0; 4]))
        .padding([16, 20])
        .width(Length::Fill)
        .on_press(msg)
    };

    let mut col = widget::column::with_capacity(8)
        .push(widget::icon(widget::icon::from_svg_bytes(APP_ICON)).size(96))
        .push(widget::text::title1(fl!("welcome-title")))
        .push(
            widget::text::body(fl!("welcome-body"))
                .class(cosmic::theme::Text::Custom(dim_text))
                .align_x(Alignment::Center),
        )
        .push(widget::space::vertical().height(Length::Fixed(12.0)))
        .push(card(
            Icon::Clone,
            fl!("welcome-clone"),
            fl!("welcome-clone-body"),
            Message::Menu(MenuAction::Clone),
        ))
        .push(card(
            Icon::NewRepo,
            fl!("welcome-create"),
            fl!("welcome-create-body"),
            Message::Menu(MenuAction::NewRepository),
        ))
        .push(card(
            Icon::AddFolder,
            fl!("welcome-add"),
            fl!("welcome-add-body"),
            Message::AddLocal,
        ))
        .spacing(12)
        .align_x(Alignment::Center)
        .max_width(560);

    if app.user.is_none() {
        col = col.push(
            widget::button::link(fl!("welcome-sign-in"))
                .on_press(Message::ToggleContextPage(ContextPage::Settings)),
        );
    }

    widget::scrollable(widget::container(col).padding(32).center_x(Length::Fill))
        .height(Length::Fill)
        .into()
}
