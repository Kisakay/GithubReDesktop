// SPDX-License-Identifier: MIT

//! GitHub ReDesktop — a native GitHub Desktop for Linux, built with libcosmic.

mod app;
mod config;
mod git;
mod github;
mod i18n;
mod tools;
mod ui;

fn main() -> cosmic::iced::Result {
    // When git calls us back as GIT_ASKPASS, answer and exit immediately.
    if git::handle_askpass() {
        return Ok(());
    }

    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("github-redesktop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1240.0, 800.0))
        .size_limits(cosmic::iced::Limits::NONE.min_width(900.0).min_height(560.0));

    cosmic::app::run::<app::App>(settings, ())
}
