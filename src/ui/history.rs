// SPDX-License-Identifier: MIT

//! "History" tab: commit list and commit details.

use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, menu};

use super::icons::{self, Icon};
use super::{dim_text, pill, status_badge};
use crate::app::{CommitAction, Message, RepoState};
use crate::fl;
use crate::tools;

pub fn sidebar(repo: &RepoState) -> Element<'_, Message> {
    if repo.commits.is_empty() {
        return widget::container(
            widget::text::body(fl!("no-commits")).class(cosmic::theme::Text::Custom(dim_text)),
        )
        .center(Length::Fill)
        .into();
    }

    let head_unpushed = repo
        .commits
        .first()
        .is_some_and(|c| repo.snapshot.unpushed.contains(&c.sha) || repo.status().upstream.is_none());

    let mut col = widget::column::with_capacity(repo.commits.len() + 1)
        .spacing(1)
        .padding([0, 6]);
    for (index, commit) in repo.commits.iter().enumerate() {
        let unpushed = repo.snapshot.unpushed.contains(&commit.sha);

        let mut meta = widget::row::with_capacity(4)
            .push(
                widget::text::caption(format!(
                    "{} · {}",
                    commit.author,
                    tools::relative_time(commit.time)
                ))
                .class(cosmic::theme::Text::Custom(dim_text))
                .width(Length::Fill)
                .wrapping(cosmic::iced::widget::text::Wrapping::None),
            )
            .spacing(6)
            .align_y(Alignment::Center);
        for r in commit.refs.iter().take(2) {
            meta = meta.push(pill(
                r.trim_start_matches("tag: ").to_string(),
                !r.starts_with("tag: "),
            ));
        }
        if unpushed {
            meta = meta.push(icons::icon(Icon::Push, 12));
        }

        let content = widget::column::with_capacity(2)
            .push(
                widget::text::body(commit.summary.as_str())
                    .wrapping(cosmic::iced::widget::text::Wrapping::None),
            )
            .push(meta)
            .spacing(2)
            .width(Length::Fill);

        let button = widget::button::custom(widget::container(content).clip(true))
            .class(cosmic::theme::Button::ListItem([6.0; 4]))
            .selected(repo.selected_commit.as_ref() == Some(&commit.sha))
            .padding([8, 10])
            .width(Length::Fill)
            .on_press(Message::SelectCommit(commit.sha.clone()));

        let mut items = Vec::new();
        if index == 0 && head_unpushed {
            items.push(menu::Item::Button(
                fl!("undo-commit"),
                None,
                CommitAction::Undo(index),
            ));
        }
        items.extend([
            menu::Item::Button(fl!("revert-commit"), None, CommitAction::Revert(index)),
            menu::Item::Button(fl!("cherry-pick"), None, CommitAction::CherryPick(index)),
            menu::Item::Button(fl!("branch-from-commit"), None, CommitAction::BranchFrom(index)),
            menu::Item::Divider,
            menu::Item::Button(fl!("copy-sha"), None, CommitAction::CopySha(index)),
            menu::Item::Button(fl!("copy-summary"), None, CommitAction::CopySummary(index)),
        ]);
        if repo.web_url().is_some() && !unpushed {
            items.push(menu::Item::Button(
                fl!("menu-view-on-github"),
                None,
                CommitAction::ViewOnGitHub(index),
            ));
        }

        col = col.push(
            widget::context_menu(button, Some(menu::items(&HashMap::new(), items)))
                .on_surface_action(Message::Surface),
        );
    }
    if !repo.history_complete {
        col = col.push(
            widget::container(widget::button::text(fl!("load-more")).on_press(Message::LoadMoreHistory))
                .center_x(Length::Fill)
                .padding(8),
        );
    }
    widget::scrollable(col).height(Length::Fill).into()
}

pub fn main(repo: &RepoState) -> Element<'_, Message> {
    let Some(commit) = repo.selected_commit() else {
        return super::empty_state(Icon::Commit, fl!("select-commit"), fl!("select-commit-body"));
    };

    let mut header = widget::column::with_capacity(4)
        .push(widget::text::title4(commit.summary.as_str()))
        .spacing(6);
    if !commit.body.is_empty() {
        header = header
            .push(widget::text::body(commit.body.as_str()).class(cosmic::theme::Text::Custom(dim_text)));
    }
    let mut meta = widget::row::with_capacity(6)
        .push(widget::text::caption_heading(commit.author.as_str()))
        .push(
            widget::text::caption(tools::format_date(commit.time))
                .class(cosmic::theme::Text::Custom(dim_text)),
        )
        .push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(icons::icon(Icon::Commit, 12))
                    .push(widget::text::monotext(commit.short_sha.as_str()).size(12))
                    .spacing(4)
                    .align_y(Alignment::Center),
            )
            .class(cosmic::theme::Button::Text)
            .padding([2, 6])
            .on_press(Message::Copy(commit.sha.clone())),
        )
        .spacing(12)
        .align_y(Alignment::Center);
    if commit.parents > 1 {
        meta = meta.push(pill(fl!("merge-commit"), false));
    }
    meta = meta.push(
        widget::text::caption(fl!("files-changed", count = repo.commit_files.len()))
            .class(cosmic::theme::Text::Custom(dim_text)),
    );
    header = header.push(meta);

    let mut files = widget::column::with_capacity(repo.commit_files.len())
        .spacing(1)
        .padding(6);
    for (index, file) in repo.commit_files.iter().enumerate() {
        let (name, dir) = tools::split_path(&file.path);
        let label = widget::column::with_capacity(2)
            .push(widget::text::body(name).wrapping(cosmic::iced::widget::text::Wrapping::None))
            .push(
                widget::text::caption(dir)
                    .class(cosmic::theme::Text::Custom(dim_text))
                    .wrapping(cosmic::iced::widget::text::Wrapping::None),
            )
            .width(Length::Fill);
        files = files.push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(widget::container(label).width(Length::Fill).clip(true))
                    .push(status_badge(file.status))
                    .spacing(8)
                    .align_y(Alignment::Center),
            )
            .class(cosmic::theme::Button::ListItem([6.0; 4]))
            .selected(repo.selected_commit_file == Some(index))
            .padding([6, 8])
            .width(Length::Fill)
            .on_press(Message::SelectCommitFile(index)),
        );
    }

    widget::column::with_capacity(3)
        .push(widget::container(header).padding([14, 16]))
        .push(widget::divider::horizontal::light())
        .push(
            widget::row::with_capacity(3)
                .push(
                    widget::scrollable(files)
                        .width(Length::Fixed(260.0))
                        .height(Length::Fill),
                )
                .push(widget::divider::vertical::light())
                .push(super::diff::view(repo.commit_diff.as_ref()))
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
