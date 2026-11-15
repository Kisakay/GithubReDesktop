// SPDX-License-Identifier: MIT

//! Embedded symbolic icons (tinted by the current theme).

use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Branch,
    Commit,
    Push,
    Pull,
    Fetch,
    Publish,
    Repo,
    Stash,
    Merge,
    Editor,
    Terminal,
    Folder,
    Globe,
    Plus,
    Trash,
    Pencil,
    Check,
    Down,
    Clone,
    AddFolder,
    NewRepo,
    Settings,
    Lock,
    Undo,
    Warning,
}

impl Icon {
    fn bytes(self) -> &'static [u8] {
        macro_rules! svg {
            ($name:literal) => {
                include_bytes!(concat!("../../resources/icons/ui/", $name, ".svg"))
            };
        }
        match self {
            Self::Branch => svg!("branch"),
            Self::Commit => svg!("commit"),
            Self::Push => svg!("push"),
            Self::Pull => svg!("pull"),
            Self::Fetch => svg!("fetch"),
            Self::Publish => svg!("publish"),
            Self::Repo => svg!("repo"),
            Self::Stash => svg!("stash"),
            Self::Merge => svg!("merge"),
            Self::Editor => svg!("editor"),
            Self::Terminal => svg!("terminal"),
            Self::Folder => svg!("folder"),
            Self::Globe => svg!("globe"),
            Self::Plus => svg!("plus"),
            Self::Trash => svg!("trash"),
            Self::Pencil => svg!("pencil"),
            Self::Check => svg!("check"),
            Self::Down => svg!("down"),
            Self::Clone => svg!("clone"),
            Self::AddFolder => svg!("add-folder"),
            Self::NewRepo => svg!("new-repo"),
            Self::Settings => svg!("settings"),
            Self::Lock => svg!("lock"),
            Self::Undo => svg!("undo"),
            Self::Warning => svg!("warning"),
        }
    }

    pub fn handle(self) -> widget::icon::Handle {
        let mut handle = widget::icon::from_svg_bytes(self.bytes());
        handle.symbolic = true;
        handle
    }
}

/// A themed icon widget.
pub fn icon<'a>(which: Icon, size: u16) -> Element<'a, Message> {
    widget::icon(which.handle()).size(size).into()
}
