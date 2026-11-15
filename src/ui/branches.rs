// SPDX-License-Identifier: MIT

//! Branch picker popup.

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;

use super::icons::{self, Icon};
use super::{dim_text, pill};
use crate::app::{App, Dialog, Message, RepoState};
use crate::fl;
use crate::git::Branch;
use crate::tools;

pub fn popup<'a>(app: &'a App, repo: &'a RepoState) -> Element<'a, Message> {
    let filter = app.branch_filter.to_lowercase();
    let matches = |b: &&Branch| filter.is_empty() || b.name.to_lowercase().contains(&filter);

    let local: Vec<&Branch> = repo
        .snapshot
        .branches
        .iter()
        .filter(|b| !b.remote)
        .filter(matches)
        .collect();
    // Hide remote branches that already have a local counterpart.
    let remote: Vec<&Branch> = repo
        .snapshot
        .branches
        .iter()
        .filter(|b| b.remote)
        .filter(|b| {
            let short = b.name.split_once('/').map_or(b.name.as_str(), |(_, n)| n);
            !repo
                .snapshot
                .branches
                .iter()
                .any(|l| !l.remote && l.name == short)
        })
        .filter(matches)
        .collect();

    let mut list = widget::column::with_capacity(local.len() + remote.len() + 2).spacing(2);
    list = list.push(section(fl!("local-branches")));
    if local.is_empty() {
        list =
            list.push(widget::text::caption(fl!("no-branches")).class(cosmic::theme::Text::Custom(dim_text)));
    }
    for branch in local {
        list = list.push(row(branch));
    }
    if !remote.is_empty() {
        list = list.push(section(fl!("remote-branches")));
        for branch in remote {
            list = list.push(row(branch));
        }
    }

    let actions = widget::row::with_capacity(2)
        .push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(icons::icon(Icon::Plus, 14))
                    .push(widget::text::body(fl!("menu-new-branch")))
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .class(cosmic::theme::Button::Suggested)
            .padding([6, 12])
            .on_press(Message::OpenDialog(Dialog::NewBranch {
                name: app.branch_filter.clone(),
                start: None,
            })),
        )
        .push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(icons::icon(Icon::Merge, 14))
                    .push(widget::text::body(fl!(
                        "merge-into",
                        branch = repo.branch_label()
                    )))
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .class(cosmic::theme::Button::Standard)
            .padding([6, 12])
            .on_press(Message::OpenDialog(Dialog::Merge {
                filter: String::new(),
            })),
        )
        .spacing(8);

    widget::container(
        widget::column::with_capacity(3)
            .push(
                widget::search_input(fl!("filter-branches"), app.branch_filter.as_str())
                    .on_input(Message::BranchFilter)
                    .on_clear(Message::BranchFilter(String::new())),
            )
            .push(actions)
            .push(widget::scrollable(list).height(Length::Fixed(360.0)))
            .spacing(10),
    )
    .padding(12)
    .width(Length::Fixed(420.0))
    .class(cosmic::theme::Container::Dropdown)
    .into()
}

fn section<'a>(label: String) -> Element<'a, Message> {
    widget::container(widget::text::caption_heading(label).class(cosmic::theme::Text::Custom(dim_text)))
        .padding([8, 8, 4, 8])
        .into()
}

fn row(branch: &Branch) -> Element<'_, Message> {
    let lead = if branch.current { Icon::Check } else { Icon::Branch };
    let mut content = widget::row::with_capacity(4)
        .push(icons::icon(lead, 14))
        .push(
            widget::column::with_capacity(2)
                .push(widget::text::body(branch.name.as_str()))
                .push(
                    widget::text::caption(tools::relative_time(branch.time))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                )
                .width(Length::Fill),
        )
        .spacing(10)
        .align_y(Alignment::Center);
    if branch.current {
        content = content.push(pill(fl!("current"), true));
    }

    let main = widget::button::custom(content)
        .class(cosmic::theme::Button::MenuItem)
        .padding([6, 8])
        .width(Length::Fill)
        .on_press(Message::Checkout(branch.clone()));

    let mut row = widget::row::with_capacity(3)
        .push(main)
        .spacing(2)
        .align_y(Alignment::Center);
    if !branch.remote {
        row = row.push(
            widget::button::custom(icons::icon(Icon::Pencil, 14))
                .class(cosmic::theme::Button::Icon)
                .padding(6)
                .on_press(Message::RenameBranch(branch.name.clone())),
        );
        if !branch.current {
            row = row.push(
                widget::button::custom(icons::icon(Icon::Trash, 14))
                    .class(cosmic::theme::Button::Icon)
                    .padding(6)
                    .on_press(Message::DeleteBranch(branch.name.clone())),
            );
        }
    }
    row.into()
}
