// SPDX-License-Identifier: MIT

//! Colored unified diff viewer.

use cosmic::iced::{Alignment, Background, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;

use super::dim_text;
use crate::app::Message;
use crate::fl;
use crate::git::{Diff, DiffLineKind};

const GUTTER: f32 = 52.0;

/// Renders a diff, or a placeholder while loading / for binary files.
pub fn view(diff: Option<&Diff>) -> Element<'_, Message> {
    let Some(diff) = diff else {
        return widget::container(
            widget::text::body(fl!("loading-diff")).class(cosmic::theme::Text::Custom(dim_text)),
        )
        .center(Length::Fill)
        .into();
    };
    if diff.binary {
        return super::empty_state(
            super::icons::Icon::Warning,
            fl!("binary-file"),
            fl!("binary-file-body"),
        );
    }
    if diff.lines.is_empty() {
        return super::empty_state(super::icons::Icon::Check, fl!("no-diff"), fl!("no-diff-body"));
    }

    let mut col = widget::column::with_capacity(diff.lines.len() + 1);
    for line in &diff.lines {
        col = col.push(line_row(line.kind, line.old_no, line.new_no, &line.text));
    }
    if diff.truncated {
        col = col.push(
            widget::container(
                widget::text::body(fl!("diff-truncated")).class(cosmic::theme::Text::Custom(dim_text)),
            )
            .padding(16),
        );
    }

    widget::scrollable(col)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn line_row<'a>(
    kind: DiffLineKind,
    old: Option<u32>,
    new: Option<u32>,
    text: &'a str,
) -> Element<'a, Message> {
    let number = |n: Option<u32>| {
        widget::container(
            widget::text::monotext(n.map(|n| n.to_string()).unwrap_or_default())
                .size(12)
                .class(cosmic::theme::Text::Custom(dim_text)),
        )
        .width(Length::Fixed(GUTTER))
        .align_x(Alignment::End)
        .padding([0, 8, 0, 0])
    };
    let sign = match kind {
        DiffLineKind::Added => "+",
        DiffLineKind::Removed => "-",
        _ => " ",
    };

    let content: Element<_> = if kind == DiffLineKind::HunkHeader || kind == DiffLineKind::Marker {
        widget::row::with_capacity(2)
            .push(widget::space::horizontal().width(Length::Fixed(GUTTER * 2.0)))
            .push(
                widget::text::monotext(text)
                    .size(12)
                    .class(cosmic::theme::Text::Custom(dim_text))
                    .width(Length::Fill),
            )
            .padding([4, 0])
            .into()
    } else {
        widget::row::with_capacity(4)
            .push(number(old))
            .push(number(new))
            .push(widget::text::monotext(sign).size(13).width(Length::Fixed(16.0)))
            .push(
                widget::text::monotext(text)
                    .size(13)
                    .width(Length::Fill)
                    .wrapping(cosmic::iced::widget::text::Wrapping::Glyph),
            )
            .align_y(Alignment::Start)
            .into()
    };

    widget::container(content)
        .width(Length::Fill)
        .class(cosmic::theme::Container::custom(move |theme| {
            let c = theme.cosmic();
            let tint = |color: Color, a: f32| Color { a, ..color };
            let background = match kind {
                DiffLineKind::Added => Some(tint(c.success_color().into(), 0.16)),
                DiffLineKind::Removed => Some(tint(c.destructive_color().into(), 0.16)),
                DiffLineKind::HunkHeader => Some(tint(c.accent_color().into(), 0.10)),
                DiffLineKind::Context | DiffLineKind::Marker => None,
            };
            widget::container::Style {
                background: background.map(Background::Color),
                ..Default::default()
            }
        }))
        .into()
}
