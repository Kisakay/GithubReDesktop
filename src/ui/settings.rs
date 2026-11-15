// SPDX-License-Identifier: MIT

//! Settings drawer (account, git identity, integrations, appearance).

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, settings};

use super::dim_text;
use crate::app::{App, Message};
use crate::fl;
use crate::github;

pub fn view(app: &App) -> Element<'_, Message> {
    widget::column::with_capacity(5)
        .push(account(app))
        .push(identity(app))
        .push(integrations(app))
        .push(appearance(app))
        .spacing(cosmic::theme::spacing().space_m)
        .into()
}

fn account(app: &App) -> Element<'_, Message> {
    let section = settings::section().title(fl!("account"));
    match &app.user {
        Some(user) => {
            let avatar: Element<_> = match &app.avatar {
                Some(handle) => widget::image(handle.clone())
                    .width(Length::Fixed(48.0))
                    .height(Length::Fixed(48.0))
                    .border_radius([24.0; 4])
                    .into(),
                None => super::icons::icon(super::icons::Icon::Globe, 32),
            };
            section
                .add(
                    widget::row::with_capacity(3)
                        .push(avatar)
                        .push(
                            widget::column::with_capacity(2)
                                .push(widget::text::heading(
                                    user.name.clone().unwrap_or_else(|| user.login.clone()),
                                ))
                                .push(
                                    widget::text::caption(format!("@{}", user.login))
                                        .class(cosmic::theme::Text::Custom(dim_text)),
                                )
                                .width(Length::Fill),
                        )
                        .push(widget::button::destructive(fl!("sign-out")).on_press(Message::SignOut))
                        .spacing(12)
                        .align_y(Alignment::Center)
                        .padding([8, 0]),
                )
                .into()
        }
        None => {
            let busy = app.signing_in;
            section
                .add(widget::text::caption(fl!("sign-in-body")).class(cosmic::theme::Text::Custom(dim_text)))
                .add(
                    widget::secure_input(fl!("token-placeholder"), app.token_input.as_str(), None, true)
                        .on_input(Message::TokenInput)
                        .on_submit(|_| Message::SignIn),
                )
                .add(
                    widget::row::with_capacity(3)
                        .push(widget::button::suggested(fl!("sign-in")).on_press_maybe(
                            (!busy && !app.token_input.trim().is_empty()).then_some(Message::SignIn),
                        ))
                        .push(
                            widget::button::standard(fl!("use-gh-cli"))
                                .on_press_maybe((!busy).then_some(Message::ImportGhToken)),
                        )
                        .push(
                            widget::button::link(fl!("create-token"))
                                .on_press(Message::LaunchUrl(github::NEW_TOKEN_URL.to_string())),
                        )
                        .spacing(8)
                        .align_y(Alignment::Center),
                )
                .into()
        }
    }
}

fn identity(app: &App) -> Element<'_, Message> {
    settings::section()
        .title(fl!("git-identity"))
        .add(settings::item(
            fl!("git-name"),
            widget::text_input(fl!("git-name-placeholder"), app.git_name.as_str())
                .on_input(Message::GitName)
                .width(Length::Fixed(220.0)),
        ))
        .add(settings::item(
            fl!("git-email"),
            widget::text_input("you@example.com", app.git_email.as_str())
                .on_input(Message::GitEmail)
                .width(Length::Fixed(220.0)),
        ))
        .add(
            widget::container(widget::button::standard(fl!("save")).on_press(Message::SaveGitIdentity))
                .align_right(Length::Fill),
        )
        .into()
}

fn integrations(app: &App) -> Element<'_, Message> {
    let detected = |list: &[String]| {
        if list.is_empty() {
            fl!("none-detected")
        } else {
            list.join(", ")
        }
    };
    settings::section()
        .title(fl!("integrations"))
        .add(
            widget::column::with_capacity(2)
                .push(settings::item(
                    fl!("editor"),
                    widget::text_input(fl!("auto-detect"), app.config.editor.as_str())
                        .on_input(Message::SetEditor)
                        .width(Length::Fixed(220.0)),
                ))
                .push(
                    widget::text::caption(fl!("detected", list = detected(&app.editors)))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                ),
        )
        .add(
            widget::column::with_capacity(2)
                .push(settings::item(
                    fl!("terminal"),
                    widget::text_input(fl!("auto-detect"), app.config.terminal.as_str())
                        .on_input(Message::SetTerminal)
                        .width(Length::Fixed(220.0)),
                ))
                .push(
                    widget::text::caption(fl!("detected", list = detected(&app.terminals)))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                ),
        )
        .add(settings::item(
            fl!("clone-dir"),
            widget::text_input("", app.config.clone_dir.as_str())
                .on_input(Message::SetCloneDir)
                .width(Length::Fixed(220.0)),
        ))
        .into()
}

fn appearance(app: &App) -> Element<'_, Message> {
    settings::section()
        .title(fl!("appearance-behavior"))
        .add(settings::item(
            fl!("theme"),
            widget::dropdown(
                &app.theme_names,
                Some(app.config.app_theme.index()),
                Message::SetTheme,
            ),
        ))
        .add(settings::item(
            fl!("auto-fetch"),
            widget::toggler(app.config.auto_fetch).on_toggle(Message::SetAutoFetch),
        ))
        .add(settings::item(
            fl!("confirm-discard"),
            widget::toggler(app.config.confirm_discard).on_toggle(Message::SetConfirmDiscard),
        ))
        .into()
}
