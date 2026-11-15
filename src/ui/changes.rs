// SPDX-License-Identifier: MIT

//! "Changes" tab: file list, commit box and working tree diff.

use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, menu};

use super::icons::{self, Icon};
use super::{dim_text, status_badge};
use crate::app::{App, FileAction, Message, RepoState};
use crate::fl;
use crate::git::FileStatus;
use crate::tools;

/// Left pane: changed files + commit form.
pub fn sidebar<'a>(app: &'a App, repo: &'a RepoState) -> Element<'a, Message> {
    let files = &repo.status().files;
    let included = files.len() - repo.excluded.len().min(files.len());

    let header = widget::row::with_capacity(2)
        .push(
            widget::checkbox(included == files.len() && !files.is_empty())
                .label(fl!("changed-files", count = files.len()))
                .on_toggle(Message::ToggleAll),
        )
        .padding([6, 14])
        .align_y(Alignment::Center);

    let list: Element<_> = if files.is_empty() {
        widget::container(
            widget::text::body(fl!("no-local-changes")).class(cosmic::theme::Text::Custom(dim_text)),
        )
        .center(Length::Fill)
        .into()
    } else {
        let mut col = widget::column::with_capacity(files.len())
            .spacing(1)
            .padding([0, 6]);
        for (index, file) in files.iter().enumerate() {
            col = col.push(file_row(repo, index, file));
        }
        widget::scrollable(col).height(Length::Fill).into()
    };

    widget::column::with_capacity(5)
        .push(header)
        .push(widget::divider::horizontal::light())
        .push(list)
        .push(widget::divider::horizontal::light())
        .push(commit_box(app, repo, included))
        .height(Length::Fill)
        .into()
}

fn file_row<'a>(repo: &'a RepoState, index: usize, file: &'a crate::git::FileChange) -> Element<'a, Message> {
    let (name, dir) = tools::split_path(&file.path);
    let checked = !repo.excluded.contains(&file.path);
    let path = file.path.clone();

    let mut label = widget::row::with_capacity(2)
        .push(widget::text::body(name))
        .spacing(6)
        .align_y(Alignment::Center);
    if !dir.is_empty() {
        label = label.push(
            widget::text::caption(dir)
                .class(cosmic::theme::Text::Custom(dim_text))
                .wrapping(cosmic::iced::widget::text::Wrapping::None),
        );
    }
    let label = widget::container(label).width(Length::Fill).clip(true);

    let content = widget::row::with_capacity(3)
        .push(widget::checkbox(checked).on_toggle(move |v| Message::ToggleFile(path.clone(), v)))
        .push(label)
        .push(status_badge(file.status))
        .spacing(10)
        .align_y(Alignment::Center);

    let button = widget::button::custom(content)
        .class(cosmic::theme::Button::ListItem([6.0; 4]))
        .selected(repo.selected_file.as_ref() == Some(&file.path))
        .padding([6, 8])
        .width(Length::Fill)
        .on_press(Message::SelectFile(file.path.clone()));

    let has_ext = std::path::Path::new(&file.path).extension().is_some();
    let mut items = vec![
        menu::Item::Button(fl!("discard-changes"), None, FileAction::Discard(index)),
        menu::Item::Divider,
        menu::Item::Button(fl!("ignore-file"), None, FileAction::Ignore(index)),
    ];
    if has_ext {
        items.push(menu::Item::Button(
            fl!("ignore-extension"),
            None,
            FileAction::IgnoreExtension(index),
        ));
    }
    items.extend([
        menu::Item::Divider,
        menu::Item::Button(fl!("copy-path"), None, FileAction::CopyPath(index)),
        menu::Item::Button(fl!("menu-show-in-files"), None, FileAction::Reveal(index)),
    ]);
    if file.status != FileStatus::Deleted {
        items.push(menu::Item::Button(
            fl!("open-file"),
            None,
            FileAction::OpenFile(index),
        ));
    }

    widget::context_menu(button, Some(menu::items(&HashMap::new(), items)))
        .on_surface_action(Message::Surface)
        .into()
}

fn commit_box<'a>(app: &'a App, repo: &'a RepoState, included: usize) -> Element<'a, Message> {
    let branch = repo.branch_label();
    let can_commit =
        !app.summary.trim().is_empty() && (included > 0 || app.amend) && !repo.status().has_conflicts();
    let label = if app.amend {
        fl!("amend-commit")
    } else if included == 0 {
        fl!("commit-to", branch = branch)
    } else {
        fl!("commit-files-to", count = included, branch = branch)
    };

    let mut identity = widget::row::with_capacity(2)
        .spacing(8)
        .align_y(Alignment::Center);
    if let Some(avatar) = &app.avatar {
        identity = identity.push(
            widget::image(avatar.clone())
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0))
                .border_radius([10.0; 4]),
        );
    }
    if let Some((name, email)) = &repo.snapshot.author {
        identity = identity.push(
            widget::text::caption(if email.is_empty() {
                name.clone()
            } else {
                format!("{name} <{email}>")
            })
            .class(cosmic::theme::Text::Custom(dim_text))
            .wrapping(cosmic::iced::widget::text::Wrapping::None),
        );
    } else {
        identity = identity.push(
            widget::button::link(fl!("set-identity"))
                .on_press(Message::ToggleContextPage(crate::app::ContextPage::Settings)),
        );
    }

    let commit_button = widget::button::custom(
        widget::row::with_capacity(2)
            .push(icons::icon(Icon::Commit, 16))
            .push(widget::text::body(label))
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .class(cosmic::theme::Button::Suggested)
    .padding([10, 12])
    .width(Length::Fill)
    .on_press_maybe(can_commit.then_some(Message::Commit));

    let has_commits = !repo.commits.is_empty();

    let mut col = widget::column::with_capacity(5)
        .push(identity)
        .push(
            widget::text_input(fl!("summary-placeholder"), app.summary.as_str())
                .on_input(Message::Summary)
                .on_submit(|_| Message::Commit),
        )
        .push(
            widget::text_editor::text_editor(&app.description)
                .placeholder(fl!("description-placeholder"))
                .on_action(Message::Description)
                .height(Length::Fixed(84.0))
                .padding(8),
        );
    if has_commits {
        col = col.push(
            widget::checkbox(app.amend)
                .label(fl!("amend-last-commit"))
                .on_toggle(Message::ToggleAmend),
        );
    }
    col.push(commit_button).spacing(8).padding(12).into()
}

/// Right pane: the diff of the selected file, or helpful suggestions.
pub fn main<'a>(app: &'a App, repo: &'a RepoState) -> Element<'a, Message> {
    if !repo.loaded {
        return widget::container(widget::text::body(fl!("loading")))
            .center(Length::Fill)
            .into();
    }
    let Some(selected) = repo
        .selected_file
        .as_ref()
        .and_then(|s| repo.status().files.iter().find(|f| &f.path == s))
    else {
        return no_changes(app, repo);
    };

    let mut title = widget::row::with_capacity(5)
        .push(status_badge(selected.status))
        .spacing(10)
        .align_y(Alignment::Center);
    if let Some(orig) = &selected.orig_path {
        title = title.push(widget::text::body(orig.as_str()).class(cosmic::theme::Text::Custom(dim_text)));
        title = title.push(widget::text::body("→"));
    }
    title = title.push(widget::text::heading(selected.path.as_str()).width(Length::Fill));
    if let Some(diff) = &repo.diff
        && !diff.binary
    {
        title = title
            .push(super::pill(format!("+{}", diff.additions), true))
            .push(super::pill(format!("−{}", diff.deletions), false));
    }

    widget::column::with_capacity(3)
        .push(widget::container(title).padding([10, 16]))
        .push(widget::divider::horizontal::light())
        .push(super::diff::view(repo.diff.as_ref()))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// "No local changes" screen with contextual suggestions (like GitHub Desktop).
fn no_changes<'a>(app: &'a App, repo: &'a RepoState) -> Element<'a, Message> {
    let st = repo.status();
    let mut cards = widget::column::with_capacity(5).spacing(12).max_width(640);

    if repo.snapshot.remote_url.is_none() {
        cards = cards.push(suggestion(
            Icon::Publish,
            fl!("publish-repository"),
            fl!("suggest-publish-body"),
            fl!("publish-repository"),
            Message::SyncPressed,
            true,
        ));
    } else if st.upstream.is_none() && st.branch.is_some() {
        cards = cards.push(suggestion(
            Icon::Publish,
            fl!("publish-branch"),
            fl!("suggest-publish-branch-body"),
            fl!("publish-branch"),
            Message::Push,
            true,
        ));
    } else if st.ahead > 0 {
        cards = cards.push(suggestion(
            Icon::Push,
            fl!("suggest-push-title", count = st.ahead),
            fl!("suggest-push-body"),
            fl!("menu-push"),
            Message::Push,
            true,
        ));
    } else if st.behind > 0 {
        cards = cards.push(suggestion(
            Icon::Pull,
            fl!("suggest-pull-title", count = st.behind),
            fl!("suggest-pull-body"),
            fl!("menu-pull"),
            Message::Pull,
            true,
        ));
    }

    cards = cards
        .push(suggestion(
            Icon::Editor,
            fl!("suggest-editor-title"),
            fl!("suggest-editor-body"),
            fl!("menu-open-editor"),
            Message::OpenInEditor,
            false,
        ))
        .push(suggestion(
            Icon::Folder,
            fl!("suggest-files-title"),
            fl!("suggest-files-body"),
            fl!("menu-show-in-files"),
            Message::OpenInFileManager,
            false,
        ));
    if repo.web_url().is_some() {
        cards = cards.push(suggestion(
            Icon::Globe,
            fl!("suggest-github-title"),
            fl!("suggest-github-body"),
            fl!("menu-view-on-github"),
            Message::OpenOnGitHub,
            false,
        ));
    }
    let _ = app;

    let content = widget::column::with_capacity(3)
        .push(widget::text::title2(fl!("no-local-changes")))
        .push(
            widget::text::body(fl!("no-local-changes-body", repo = repo.name.clone()))
                .class(cosmic::theme::Text::Custom(dim_text)),
        )
        .push(cards)
        .spacing(16)
        .max_width(640);

    widget::scrollable(widget::container(content).padding(32).center_x(Length::Fill))
        .height(Length::Fill)
        .into()
}

fn suggestion<'a>(
    icon: Icon,
    title: String,
    body: String,
    action: String,
    msg: Message,
    primary: bool,
) -> Element<'a, Message> {
    let button = if primary {
        widget::button::suggested(action).on_press(msg)
    } else {
        widget::button::standard(action).on_press(msg)
    };
    widget::container(
        widget::row::with_capacity(3)
            .push(icons::icon(icon, 28))
            .push(
                widget::column::with_capacity(2)
                    .push(widget::text::heading(title))
                    .push(widget::text::caption(body).class(cosmic::theme::Text::Custom(dim_text)))
                    .spacing(2)
                    .width(Length::Fill),
            )
            .push(button)
            .spacing(16)
            .align_y(Alignment::Center),
    )
    .padding(16)
    .class(cosmic::theme::Container::Card)
    .width(Length::Fill)
    .into()
}
