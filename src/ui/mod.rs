// SPDX-License-Identifier: MIT

//! Views. Each submodule renders one area of the window.

pub mod branches;
pub mod changes;
pub mod dialogs;
pub mod diff;
pub mod history;
pub mod icons;
pub mod settings;
pub mod welcome;

use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, menu};

use crate::app::{App, ContextPage, MenuAction, Message, RepoState, Tab};
use crate::fl;
use crate::git::FileStatus;
use crate::tools;

/// Width of the left pane (file list / history).
pub const SIDEBAR_WIDTH: f32 = 340.0;

/// Main window content.
pub fn view(app: &App) -> Element<'_, Message> {
    let Some(repo) = &app.repo else {
        return welcome::view(app);
    };

    let sidebar = match app.tab {
        Tab::Changes => changes::sidebar(app, repo),
        Tab::History => history::sidebar(repo),
    };
    let main = match app.tab {
        Tab::Changes => changes::main(app, repo),
        Tab::History => history::main(repo),
    };

    let tabs = widget::segmented_control::horizontal(&app.tabs)
        .on_activate(Message::TabActivated)
        .width(Length::Fill);

    let left = widget::column::with_capacity(2)
        .push(widget::container(tabs).padding([8, 10]))
        .push(sidebar)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill);

    let mut body = widget::column::with_capacity(4).push(toolbar(app, repo));
    if repo.status().merging || repo.status().has_conflicts() {
        body = body.push(merge_banner(repo));
    }
    body.push(widget::divider::horizontal::default())
        .push(
            widget::row::with_capacity(3)
                .push(left)
                .push(widget::divider::vertical::default())
                .push(main)
                .height(Length::Fill),
        )
        .into()
}

/// The menu bar shown at the start of the header.
pub fn menu_bar(app: &App) -> Element<'_, Message> {
    let has_repo = app.repo.is_some();
    let stash_count = app.repo.as_ref().map_or(0, |r| r.status().stash_count);

    let repository = if has_repo {
        vec![
            menu::Item::Button(fl!("menu-push"), None, MenuAction::Push),
            menu::Item::Button(fl!("menu-pull"), None, MenuAction::Pull),
            menu::Item::Button(fl!("menu-fetch"), None, MenuAction::Fetch),
            menu::Item::Button(fl!("menu-force-push"), None, MenuAction::ForcePush),
            menu::Item::Divider,
            menu::Item::Button(fl!("menu-open-editor"), None, MenuAction::OpenInEditor),
            menu::Item::Button(fl!("menu-open-terminal"), None, MenuAction::OpenInTerminal),
            menu::Item::Button(fl!("menu-show-in-files"), None, MenuAction::OpenInFileManager),
            menu::Item::Button(fl!("menu-view-on-github"), None, MenuAction::OpenOnGitHub),
            menu::Item::Divider,
            menu::Item::Button(fl!("menu-refresh"), None, MenuAction::Refresh),
            menu::Item::Button(fl!("menu-remove-repository"), None, MenuAction::RemoveRepository),
        ]
    } else {
        vec![menu::Item::ButtonDisabled(
            fl!("no-repository-selected"),
            None,
            MenuAction::Refresh,
        )]
    };

    let branch = if has_repo {
        vec![
            menu::Item::Button(fl!("menu-new-branch"), None, MenuAction::NewBranch),
            menu::Item::Button(fl!("menu-rename-branch"), None, MenuAction::RenameBranch),
            menu::Item::Button(fl!("menu-delete-branch"), None, MenuAction::DeleteBranch),
            menu::Item::Divider,
            menu::Item::Button(fl!("menu-merge"), None, MenuAction::Merge),
            menu::Item::Button(fl!("menu-compare-on-github"), None, MenuAction::CompareOnGitHub),
            menu::Item::Divider,
            menu::Item::Button(fl!("menu-stash"), None, MenuAction::Stash),
            if stash_count > 0 {
                menu::Item::Button(fl!("menu-stash-pop"), None, MenuAction::StashPop)
            } else {
                menu::Item::ButtonDisabled(fl!("menu-stash-pop"), None, MenuAction::StashPop)
            },
            menu::Item::Button(fl!("menu-discard-all"), None, MenuAction::DiscardAll),
        ]
    } else {
        vec![menu::Item::ButtonDisabled(
            fl!("no-repository-selected"),
            None,
            MenuAction::Refresh,
        )]
    };

    menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("menu-file")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("menu-new-repository"), None, MenuAction::NewRepository),
                    menu::Item::Button(fl!("menu-add-local"), None, MenuAction::AddLocal),
                    menu::Item::Button(fl!("menu-clone"), None, MenuAction::Clone),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-quit"), None, MenuAction::Quit),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-view")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("tab-changes"), None, MenuAction::ShowChanges),
                    menu::Item::Button(fl!("tab-history"), None, MenuAction::ShowHistory),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-repository")).apply(Element::from),
            menu::items(&app.key_binds, repository),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-branch")).apply(Element::from),
            menu::items(&app.key_binds, branch),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-help")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("menu-report-issue"), None, MenuAction::ReportIssue),
                    menu::Item::Button(fl!("menu-about"), None, MenuAction::About),
                ],
            ),
        ),
    ])
    .item_height(menu::ItemHeight::Dynamic(40))
    .item_width(menu::ItemWidth::Uniform(280))
    .spacing(4.0)
    .on_surface_action(Message::Surface)
    .into()
}

/// Account button at the end of the header.
pub fn header_end(app: &App) -> Vec<Element<'_, Message>> {
    let account: Element<_> = match &app.user {
        Some(user) => {
            let pic: Element<_> = match &app.avatar {
                Some(handle) => widget::image(handle.clone())
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .border_radius([12.0; 4])
                    .into(),
                None => icons::icon(icons::Icon::Globe, 18),
            };
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(pic)
                    .push(widget::text::body(user.login.as_str()))
                    .spacing(8)
                    .align_y(Alignment::Center),
            )
            .padding([4, 10])
            .class(cosmic::theme::Button::HeaderBar)
            .on_press(Message::ToggleContextPage(ContextPage::Settings))
            .into()
        }
        None => widget::button::text(fl!("sign-in"))
            .class(cosmic::theme::Button::HeaderBar)
            .on_press(Message::ToggleContextPage(ContextPage::Settings))
            .into(),
    };
    let settings = widget::tooltip(
        widget::button::custom(icons::icon(icons::Icon::Settings, 16))
            .padding(8)
            .class(cosmic::theme::Button::HeaderBar)
            .on_press(Message::ToggleContextPage(ContextPage::Settings)),
        widget::text::body(fl!("settings")),
        widget::tooltip::Position::Bottom,
    );
    vec![account, settings.into()]
}

/// Top toolbar: current branch + sync + quick actions.
fn toolbar<'a>(app: &'a App, repo: &'a RepoState) -> Element<'a, Message> {
    let branch_button = toolbar_button(
        icons::Icon::Branch,
        fl!("current-branch"),
        repo.branch_label(),
        true,
        Some(Message::ToggleBranchPopover),
    );
    let branch: Element<_> = if app.branch_popover {
        widget::popover(branch_button)
            .popup(branches::popup(app, repo))
            .on_close(Message::CloseBranchPopover)
            .position(widget::popover::Position::Bottom)
            .into()
    } else {
        branch_button
    };

    let st = repo.status();
    let remote = st
        .upstream
        .as_deref()
        .and_then(|u| u.split_once('/'))
        .map_or_else(|| "origin".to_string(), |(r, _)| r.to_string());
    let (icon, title, caption, msg) = if let Some(op) = app.busy.filter(|op| op.is_network()) {
        (icons::Icon::Fetch, op.progress_label(), fl!("please-wait"), None)
    } else if repo.snapshot.remote_url.is_none() {
        (
            icons::Icon::Publish,
            fl!("publish-repository"),
            fl!("publish-repository-caption"),
            Some(Message::SyncPressed),
        )
    } else if st.upstream.is_none() {
        (
            icons::Icon::Publish,
            fl!("publish-branch"),
            fl!("publish-branch-caption"),
            Some(Message::SyncPressed),
        )
    } else if st.behind > 0 {
        (
            icons::Icon::Pull,
            fl!("pull-origin", remote = remote.clone()),
            sync_counts(st.ahead, st.behind),
            Some(Message::SyncPressed),
        )
    } else if st.ahead > 0 {
        (
            icons::Icon::Push,
            fl!("push-origin", remote = remote.clone()),
            sync_counts(st.ahead, st.behind),
            Some(Message::SyncPressed),
        )
    } else {
        let caption = match repo.last_fetch {
            Some(t) => {
                let secs = i64::try_from(t.elapsed().as_secs()).unwrap_or(0);
                fl!(
                    "last-fetched",
                    when = tools::relative_time(chrono::Utc::now().timestamp() - secs)
                )
            }
            None => fl!("never-fetched"),
        };
        (
            icons::Icon::Fetch,
            fl!("fetch-origin", remote = remote.clone()),
            caption,
            Some(Message::SyncPressed),
        )
    };
    let sync = toolbar_button(icon, caption, title, false, msg);

    let mut actions = widget::row::with_capacity(6)
        .spacing(2)
        .align_y(Alignment::Center);
    if !st.files.is_empty() {
        actions = actions.push(action_button(
            icons::Icon::Stash,
            fl!("menu-stash"),
            Message::Stash,
        ));
    }
    if st.stash_count > 0 {
        actions = actions.push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(icons::icon(icons::Icon::Undo, 16))
                    .push(widget::text::body(fl!("restore-stash", count = st.stash_count)))
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .class(cosmic::theme::Button::Standard)
            .on_press(Message::StashPop),
        );
    }
    actions = actions
        .push(action_button(
            icons::Icon::Editor,
            fl!("menu-open-editor"),
            Message::OpenInEditor,
        ))
        .push(action_button(
            icons::Icon::Terminal,
            fl!("menu-open-terminal"),
            Message::OpenInTerminal,
        ))
        .push(action_button(
            icons::Icon::Folder,
            fl!("menu-show-in-files"),
            Message::OpenInFileManager,
        ));
    if repo.web_url().is_some() {
        actions = actions.push(action_button(
            icons::Icon::Globe,
            fl!("menu-view-on-github"),
            Message::OpenOnGitHub,
        ));
    }

    widget::row::with_capacity(5)
        .push(branch)
        .push(widget::divider::vertical::light().height(Length::Fixed(36.0)))
        .push(sync)
        .push(widget::space::horizontal())
        .push(actions)
        .spacing(6)
        .padding([6, 12])
        .align_y(Alignment::Center)
        .into()
}

fn sync_counts(ahead: u32, behind: u32) -> String {
    match (ahead, behind) {
        (0, b) => format!("↓ {b}"),
        (a, 0) => format!("↑ {a}"),
        (a, b) => format!("↑ {a}  ↓ {b}"),
    }
}

/// A two-line toolbar button (caption + title) in the GitHub Desktop style.
fn toolbar_button<'a>(
    icon: icons::Icon,
    caption: String,
    title: String,
    dropdown: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let text = widget::column::with_capacity(2)
        .push(widget::text::caption(caption).class(cosmic::theme::Text::Custom(dim_text)))
        .push(widget::text::heading(title))
        .width(Length::Fill);
    let mut content = widget::row::with_capacity(3)
        .push(icons::icon(icon, 18))
        .push(text)
        .spacing(12)
        .align_y(Alignment::Center);
    if dropdown {
        content = content.push(icons::icon(icons::Icon::Down, 14));
    }
    widget::button::custom(content)
        .padding([6, 14])
        .width(Length::Fixed(260.0))
        .class(cosmic::theme::Button::Text)
        .on_press_maybe(on_press)
        .into()
}

fn action_button<'a>(icon: icons::Icon, tooltip: String, msg: Message) -> Element<'a, Message> {
    widget::tooltip(
        widget::button::custom(icons::icon(icon, 16))
            .padding(8)
            .class(cosmic::theme::Button::Icon)
            .on_press(msg),
        widget::text::body(tooltip),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

fn merge_banner(repo: &RepoState) -> Element<'_, Message> {
    let conflicts = repo
        .status()
        .files
        .iter()
        .filter(|f| f.status == FileStatus::Conflicted)
        .count();
    let text = if conflicts > 0 {
        fl!("merge-conflicts", count = conflicts)
    } else {
        fl!("merge-in-progress")
    };
    widget::container(
        widget::row::with_capacity(3)
            .push(icons::icon(icons::Icon::Warning, 16))
            .push(widget::text::body(text).width(Length::Fill))
            .push(widget::button::destructive(fl!("abort-merge")).on_press(Message::AbortMerge))
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([8, 16])
    .class(cosmic::theme::Container::custom(|theme| {
        let mut bg: Color = theme.cosmic().warning_color().into();
        bg.a = 0.18;
        widget::container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        }
    }))
    .width(Length::Fill)
    .into()
}

/// Dimmed secondary text color.
pub fn dim_text(theme: &cosmic::Theme) -> cosmic::iced::widget::text::Style {
    let mut c: Color = theme.cosmic().on_bg_color().into();
    c.a = 0.65;
    cosmic::iced::widget::text::Style {
        color: Some(c),
        ..Default::default()
    }
}

/// A colored, rounded status badge (M, A, D, …).
pub fn status_badge<'a>(status: FileStatus) -> Element<'a, Message> {
    widget::container(widget::text::caption_heading(status.letter()))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(18.0))
        .center_x(Length::Fixed(20.0))
        .center_y(Length::Fixed(18.0))
        .class(cosmic::theme::Container::custom(move |theme| {
            let c = theme.cosmic();
            let color: Color = match status {
                FileStatus::Modified | FileStatus::TypeChanged => c.warning_color(),
                FileStatus::Added | FileStatus::Untracked | FileStatus::Copied => c.success_color(),
                FileStatus::Deleted | FileStatus::Conflicted => c.destructive_color(),
                FileStatus::Renamed => c.accent_color(),
            }
            .into();
            let mut bg = color;
            bg.a = 0.16;
            widget::container::Style {
                text_color: Some(color),
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color,
                },
                ..Default::default()
            }
        }))
        .into()
}

/// Small pill used for refs and counters.
pub fn pill<'a>(label: String, accent: bool) -> Element<'a, Message> {
    widget::container(widget::text::caption(label))
        .padding([1, 6])
        .class(cosmic::theme::Container::custom(move |theme| {
            let c = theme.cosmic();
            let color: Color = if accent {
                c.accent_color().into()
            } else {
                c.on_bg_color().into()
            };
            let mut bg = color;
            bg.a = 0.14;
            widget::container::Style {
                text_color: Some(color),
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

/// Centered placeholder with an icon, a title and a description.
pub fn empty_state<'a>(icon: icons::Icon, title: String, body: String) -> Element<'a, Message> {
    widget::container(
        widget::column::with_capacity(3)
            .push(icons::icon(icon, 48))
            .push(widget::text::title3(title))
            .push(
                widget::text::body(body)
                    .class(cosmic::theme::Text::Custom(dim_text))
                    .align_x(Alignment::Center),
            )
            .spacing(12)
            .align_x(Alignment::Center)
            .max_width(420),
    )
    .center(Length::Fill)
    .into()
}
