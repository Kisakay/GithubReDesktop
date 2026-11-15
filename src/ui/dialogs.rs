// SPDX-License-Identifier: MIT

//! Modal dialogs (clone, create, branch, publish, confirmations).

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;

use super::icons::{self, Icon};
use super::{dim_text, pill};
use crate::app::{App, Dialog, Field, Message, Op};
use crate::fl;
use crate::git;

pub fn view<'a>(app: &'a App, dialog: &'a Dialog) -> Element<'a, Message> {
    let cancel = widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel);

    match dialog {
        Dialog::Clone {
            url, path, filter, ..
        } => {
            let cloning = app.busy == Some(Op::Clone);
            let ready = !url.trim().is_empty() && !path.trim().is_empty() && !cloning;
            let mut body = widget::column::with_capacity(6).spacing(12);

            if app.user.is_some() {
                body = body.push(github_repo_list(app, filter, url));
            } else {
                body = body.push(
                    widget::text::caption(fl!("clone-sign-in-hint"))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                );
            }
            body = body
                .push(labeled(
                    fl!("clone-url"),
                    widget::text_input(fl!("clone-url-placeholder"), url.as_str())
                        .on_input(|v| Message::DialogText(Field::Url, v))
                        .into(),
                ))
                .push(labeled(fl!("local-path"), path_input(path)));

            let primary = widget::button::suggested(if cloning { fl!("op-cloning") } else { fl!("clone") })
                .on_press_maybe(ready.then_some(Message::DialogConfirm));
            widget::dialog()
                .title(fl!("clone-title"))
                .icon(icons::icon(Icon::Clone, 32))
                .control(body)
                .primary_action(primary)
                .secondary_action(cancel)
                .width(Length::Fixed(620.0))
                .into()
        }

        Dialog::Create {
            name,
            parent,
            readme,
            gitignore,
        } => {
            let ready = !name.trim().is_empty() && !parent.trim().is_empty();
            let full = std::path::Path::new(parent).join(name.trim());
            let body = widget::column::with_capacity(5)
                .push(labeled(
                    fl!("name"),
                    widget::text_input(fl!("repo-name-placeholder"), name.as_str())
                        .on_input(|v| Message::DialogText(Field::Name, v))
                        .on_submit(|_| Message::DialogConfirm)
                        .into(),
                ))
                .push(labeled(fl!("local-path"), path_input(parent)))
                .push(
                    widget::text::caption(fl!("will-be-created", path = full.display().to_string()))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                )
                .push(labeled(
                    fl!("gitignore"),
                    widget::text_input(fl!("gitignore-placeholder"), gitignore.as_str())
                        .on_input(|v| Message::DialogText(Field::Gitignore, v))
                        .into(),
                ))
                .push(
                    widget::checkbox(*readme)
                        .label(fl!("init-readme"))
                        .on_toggle(Message::DialogToggle),
                )
                .spacing(12);
            widget::dialog()
                .title(fl!("create-title"))
                .icon(icons::icon(Icon::NewRepo, 32))
                .control(body)
                .primary_action(
                    widget::button::suggested(fl!("create-repository"))
                        .on_press_maybe(ready.then_some(Message::DialogConfirm)),
                )
                .secondary_action(cancel)
                .width(Length::Fixed(560.0))
                .into()
        }

        Dialog::NewBranch { name, start } => {
            let sanitized = git::branch::sanitize_name(name);
            let based_on = match start {
                Some(sha) => fl!(
                    "branch-based-on-commit",
                    sha = sha.chars().take(7).collect::<String>()
                ),
                None => fl!(
                    "branch-based-on",
                    branch = app
                        .repo
                        .as_ref()
                        .map(crate::app::RepoState::branch_label)
                        .unwrap_or_default()
                ),
            };
            let mut body = widget::column::with_capacity(3)
                .push(
                    widget::text_input(fl!("branch-name-placeholder"), name.as_str())
                        .on_input(|v| Message::DialogText(Field::Name, v))
                        .on_submit(|_| Message::DialogConfirm),
                )
                .spacing(8);
            if !sanitized.is_empty() && sanitized != name.trim() {
                body = body.push(
                    widget::text::caption(fl!("will-be-created-as", name = sanitized.clone()))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                );
            }
            body = body.push(widget::text::caption(based_on).class(cosmic::theme::Text::Custom(dim_text)));
            widget::dialog()
                .title(fl!("menu-new-branch"))
                .icon(icons::icon(Icon::Branch, 32))
                .control(body)
                .primary_action(
                    widget::button::suggested(fl!("create-branch"))
                        .on_press_maybe((!sanitized.is_empty()).then_some(Message::DialogConfirm)),
                )
                .secondary_action(cancel)
                .into()
        }

        Dialog::RenameBranch { old, name } => widget::dialog()
            .title(fl!("rename-branch-title", branch = old.clone()))
            .icon(icons::icon(Icon::Pencil, 32))
            .control(
                widget::text_input(fl!("branch-name-placeholder"), name.as_str())
                    .on_input(|v| Message::DialogText(Field::Name, v))
                    .on_submit(|_| Message::DialogConfirm),
            )
            .primary_action(widget::button::suggested(fl!("rename")).on_press(Message::DialogConfirm))
            .secondary_action(cancel)
            .into(),

        Dialog::Merge { filter } => {
            let current = app
                .repo
                .as_ref()
                .map(crate::app::RepoState::branch_label)
                .unwrap_or_default();
            let f = filter.to_lowercase();
            let mut list = widget::column::with_capacity(16).spacing(2);
            if let Some(repo) = &app.repo {
                for branch in repo
                    .snapshot
                    .branches
                    .iter()
                    .filter(|b| !b.current && (f.is_empty() || b.name.to_lowercase().contains(&f)))
                {
                    list = list.push(
                        widget::button::custom(
                            widget::row::with_capacity(3)
                                .push(icons::icon(Icon::Branch, 14))
                                .push(widget::text::body(branch.name.as_str()).width(Length::Fill))
                                .push(if branch.remote {
                                    pill(fl!("remote"), false)
                                } else {
                                    widget::space::horizontal().width(Length::Fixed(0.0)).into()
                                })
                                .spacing(10)
                                .align_y(Alignment::Center),
                        )
                        .class(cosmic::theme::Button::MenuItem)
                        .padding([8, 10])
                        .width(Length::Fill)
                        .on_press(Message::MergeBranch(branch.name.clone())),
                    );
                }
            }
            widget::dialog()
                .title(fl!("merge-title", branch = current))
                .icon(icons::icon(Icon::Merge, 32))
                .control(
                    widget::column::with_capacity(2)
                        .push(
                            widget::search_input(fl!("filter-branches"), filter.as_str())
                                .on_input(|v| Message::DialogText(Field::Filter, v)),
                        )
                        .push(widget::scrollable(list).height(Length::Fixed(320.0)))
                        .spacing(10),
                )
                .secondary_action(cancel)
                .width(Length::Fixed(520.0))
                .into()
        }

        Dialog::Publish {
            name,
            description,
            private,
        } => {
            let account = app.user.as_ref().map(|u| u.login.clone()).unwrap_or_default();
            let body = widget::column::with_capacity(4)
                .push(labeled(
                    fl!("name"),
                    widget::text_input(fl!("repo-name-placeholder"), name.as_str())
                        .on_input(|v| Message::DialogText(Field::Name, v))
                        .into(),
                ))
                .push(labeled(
                    fl!("description"),
                    widget::text_input(fl!("optional"), description.as_str())
                        .on_input(|v| Message::DialogText(Field::Description, v))
                        .into(),
                ))
                .push(
                    widget::checkbox(*private)
                        .label(fl!("keep-private"))
                        .on_toggle(Message::DialogToggle),
                )
                .push(
                    widget::text::caption(fl!("publish-to", account = account))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                )
                .spacing(12);
            widget::dialog()
                .title(fl!("publish-repository"))
                .icon(icons::icon(Icon::Publish, 32))
                .control(body)
                .primary_action(
                    widget::button::suggested(fl!("publish-repository"))
                        .on_press_maybe((!name.trim().is_empty()).then_some(Message::DialogConfirm)),
                )
                .secondary_action(cancel)
                .width(Length::Fixed(520.0))
                .into()
        }

        Dialog::ConfirmDiscard(files) => {
            let mut list = widget::column::with_capacity(files.len().min(10) + 1).spacing(2);
            for file in files.iter().take(10) {
                list = list.push(widget::text::monotext(file.path.as_str()).size(12));
            }
            if files.len() > 10 {
                let more = files.len() - 10;
                list = list.push(
                    widget::text::caption(fl!("and-more", count = more))
                        .class(cosmic::theme::Text::Custom(dim_text)),
                );
            }
            widget::dialog()
                .title(fl!("discard-title", count = files.len()))
                .icon(icons::icon(Icon::Trash, 32))
                .body(fl!("discard-body"))
                .control(list)
                .primary_action(
                    widget::button::destructive(fl!("discard-changes")).on_press(Message::DialogConfirm),
                )
                .secondary_action(cancel)
                .into()
        }

        Dialog::ConfirmDeleteBranch(name) => widget::dialog()
            .title(fl!("delete-branch-title"))
            .icon(icons::icon(Icon::Trash, 32))
            .body(fl!("delete-branch-body", branch = name.clone()))
            .primary_action(widget::button::destructive(fl!("delete")).on_press(Message::DialogConfirm))
            .secondary_action(cancel)
            .into(),

        Dialog::ConfirmRemoveRepo(path) => widget::dialog()
            .title(fl!("remove-repo-title"))
            .icon(icons::icon(Icon::Repo, 32))
            .body(fl!("remove-repo-body", path = path.display().to_string()))
            .primary_action(widget::button::destructive(fl!("remove")).on_press(Message::DialogConfirm))
            .secondary_action(cancel)
            .into(),

        Dialog::ConfirmForcePush => widget::dialog()
            .title(fl!("force-push-title"))
            .icon(icons::icon(Icon::Warning, 32))
            .body(fl!("force-push-body"))
            .primary_action(
                widget::button::destructive(fl!("menu-force-push")).on_press(Message::DialogConfirm),
            )
            .secondary_action(cancel)
            .into(),
    }
}

fn labeled<'a>(label: String, input: Element<'a, Message>) -> Element<'a, Message> {
    widget::column::with_capacity(2)
        .push(widget::text::caption_heading(label))
        .push(input)
        .spacing(4)
        .into()
}

fn path_input(path: &str) -> Element<'_, Message> {
    widget::row::with_capacity(2)
        .push(
            widget::text_input("", path)
                .on_input(|v| Message::DialogText(Field::Path, v))
                .width(Length::Fill),
        )
        .push(widget::button::standard(fl!("choose")).on_press(Message::DialogBrowse))
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

/// Filterable list of the signed-in user's GitHub repositories.
fn github_repo_list<'a>(app: &'a App, filter: &'a str, selected_url: &'a str) -> Element<'a, Message> {
    let f = filter.to_lowercase();
    let mut list = widget::column::with_capacity(32).spacing(2);
    if app.github_repos_loading {
        list = list.push(widget::text::body(fl!("loading")).class(cosmic::theme::Text::Custom(dim_text)));
    }
    for repo in app
        .github_repos
        .iter()
        .filter(|r| f.is_empty() || r.full_name.to_lowercase().contains(&f))
        .take(200)
    {
        let mut row = widget::row::with_capacity(4)
            .push(icons::icon(
                if repo.private { Icon::Lock } else { Icon::Repo },
                14,
            ))
            .push(
                widget::column::with_capacity(2)
                    .push(widget::text::body(repo.full_name.as_str()))
                    .push_maybe(repo.description.as_deref().filter(|d| !d.is_empty()).map(|d| {
                        widget::text::caption(d)
                            .class(cosmic::theme::Text::Custom(dim_text))
                            .wrapping(cosmic::iced::widget::text::Wrapping::None)
                    }))
                    .width(Length::Fill),
            )
            .spacing(10)
            .align_y(Alignment::Center);
        if repo.fork {
            row = row.push(pill(fl!("fork"), false));
        }
        list = list.push(
            widget::button::custom(widget::container(row).clip(true))
                .class(cosmic::theme::Button::ListItem([6.0; 4]))
                .selected(repo.clone_url == selected_url)
                .padding([6, 10])
                .width(Length::Fill)
                .on_press(Message::DialogPickRepo(repo.clone_url.clone())),
        );
    }

    widget::column::with_capacity(2)
        .push(
            widget::search_input(fl!("filter-repositories"), filter)
                .on_input(|v| Message::DialogText(Field::Filter, v))
                .on_clear(Message::DialogText(Field::Filter, String::new())),
        )
        .push(
            widget::container(widget::scrollable(list).height(Length::Fixed(240.0)))
                .class(cosmic::theme::Container::Card)
                .padding(4),
        )
        .spacing(8)
        .into()
}
