// SPDX-License-Identifier: MIT

//! Application model, messages and update logic.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cosmic::Application;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::{Event, Subscription, event, window};
use cosmic::prelude::*;
use cosmic::widget::menu::key_bind::{KeyBind, Modifier};
use cosmic::widget::{self, about::About, menu, nav_bar, segmented_button, text_editor};

use crate::config::{AppTheme, Config};
use crate::git::{self, Branch, Commit, CommitFile, Diff, FileChange, RepoStatus};
use crate::{fl, github, tools};

pub const APP_ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/scalable/apps/io.github.kisakay.GithubReDesktop.svg");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const HISTORY_PAGE: usize = 150;
const AUTO_FETCH_EVERY: Duration = Duration::from_secs(300);

/// Left pane tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Changes,
    History,
}

/// Context drawer pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextPage {
    #[default]
    Settings,
    About,
}

/// Long running operations, used for progress & toasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Fetch,
    Pull,
    Push,
    ForcePush,
    Publish,
    Commit,
    Checkout,
    CreateBranch,
    DeleteBranch,
    RenameBranch,
    Merge,
    AbortMerge,
    Stash,
    StashPop,
    Discard,
    Ignore,
    Revert,
    CherryPick,
    Undo,
    Clone,
}

impl Op {
    /// Whether the operation talks to the network (shown in the toolbar).
    #[must_use]
    pub fn is_network(self) -> bool {
        matches!(
            self,
            Self::Fetch | Self::Pull | Self::Push | Self::ForcePush | Self::Publish | Self::Clone
        )
    }

    #[must_use]
    pub fn progress_label(self) -> String {
        match self {
            Self::Fetch => fl!("op-fetching"),
            Self::Pull => fl!("op-pulling"),
            Self::Push | Self::ForcePush => fl!("op-pushing"),
            Self::Publish => fl!("op-publishing"),
            Self::Clone => fl!("op-cloning"),
            _ => fl!("op-working"),
        }
    }

    fn success_label(self) -> Option<String> {
        Some(match self {
            Self::Fetch => fl!("done-fetch"),
            Self::Pull => fl!("done-pull"),
            Self::Push | Self::ForcePush => fl!("done-push"),
            Self::Publish => fl!("done-publish"),
            Self::Commit => fl!("done-commit"),
            Self::Merge => fl!("done-merge"),
            Self::Stash => fl!("done-stash"),
            Self::StashPop => fl!("done-stash-pop"),
            Self::Revert => fl!("done-revert"),
            Self::CherryPick => fl!("done-cherry-pick"),
            Self::Undo => fl!("done-undo"),
            Self::DeleteBranch => fl!("done-delete-branch"),
            _ => return None,
        })
    }
}

/// Modal dialogs.
#[derive(Debug, Clone)]
pub enum Dialog {
    Clone {
        url: String,
        path: String,
        filter: String,
        /// Whether `path` was edited by hand (stop auto-filling it).
        path_edited: bool,
    },
    Create {
        name: String,
        parent: String,
        readme: bool,
        gitignore: String,
    },
    NewBranch {
        name: String,
        start: Option<String>,
    },
    RenameBranch {
        old: String,
        name: String,
    },
    Merge {
        filter: String,
    },
    Publish {
        name: String,
        description: String,
        private: bool,
    },
    ConfirmDiscard(Vec<FileChange>),
    ConfirmDeleteBranch(String),
    ConfirmRemoveRepo(PathBuf),
    ConfirmForcePush,
}

/// Fields editable inside dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Url,
    Path,
    Filter,
    Name,
    Description,
    Gitignore,
}

/// Everything loaded for the selected repository.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub status: RepoStatus,
    pub branches: Vec<Branch>,
    pub remote_url: Option<String>,
    pub unpushed: Vec<String>,
    /// Effective `user.name` / `user.email` for this repository.
    pub author: Option<(String, String)>,
}

/// State of the currently opened repository.
pub struct RepoState {
    pub path: PathBuf,
    pub name: String,
    pub snapshot: Snapshot,
    pub loaded: bool,
    /// Files unchecked for the next commit (everything is included by default).
    pub excluded: HashSet<String>,
    pub selected_file: Option<String>,
    pub diff: Option<Diff>,
    pub commits: Vec<Commit>,
    pub history_complete: bool,
    pub selected_commit: Option<String>,
    pub commit_files: Vec<CommitFile>,
    pub selected_commit_file: Option<usize>,
    pub commit_diff: Option<Diff>,
    pub last_fetch: Option<Instant>,
}

impl RepoState {
    fn new(path: PathBuf) -> Self {
        let name = path.file_name().map_or_else(
            || path.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        Self {
            path,
            name,
            snapshot: Snapshot::default(),
            loaded: false,
            excluded: HashSet::new(),
            selected_file: None,
            diff: None,
            commits: Vec::new(),
            history_complete: false,
            selected_commit: None,
            commit_files: Vec::new(),
            selected_commit_file: None,
            commit_diff: None,
            last_fetch: None,
        }
    }

    #[must_use]
    pub fn status(&self) -> &RepoStatus {
        &self.snapshot.status
    }

    #[must_use]
    pub fn branch_label(&self) -> String {
        match (&self.snapshot.status.branch, &self.snapshot.status.head) {
            (Some(b), _) => b.clone(),
            (None, Some(head)) => fl!("detached-head", sha = head.clone()),
            (None, None) => "main".into(),
        }
    }

    #[must_use]
    pub fn included_files(&self) -> Vec<&FileChange> {
        self.snapshot
            .status
            .files
            .iter()
            .filter(|f| !self.excluded.contains(&f.path))
            .collect()
    }

    #[must_use]
    pub fn selected_commit(&self) -> Option<&Commit> {
        let sha = self.selected_commit.as_ref()?;
        self.commits.iter().find(|c| &c.sha == sha)
    }

    #[must_use]
    pub fn web_url(&self) -> Option<String> {
        self.snapshot.remote_url.as_deref().and_then(git::web_url)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Repository list
    RepoLoaded(PathBuf, Result<Snapshot, String>),
    AddLocal,
    AddLocalPicked(Option<PathBuf>),
    RepoAdded(Result<PathBuf, String>),
    RemoveRepo(PathBuf),
    NavAction(NavAction),
    Tick,
    Refresh,
    TabActivated(segmented_button::Entity),
    SetTab(Tab),

    // Changes
    ToggleFile(String, bool),
    ToggleAll(bool),
    SelectFile(String),
    DiffLoaded(PathBuf, String, Result<Diff, String>),
    Summary(String),
    Description(text_editor::Action),
    ToggleAmend(bool),
    Commit,
    Discard(Vec<FileChange>),
    DiscardAll,
    FileAction(FileAction),

    // History
    HistoryLoaded(PathBuf, usize, Result<Vec<Commit>, String>),
    LoadMoreHistory,
    SelectCommit(String),
    CommitFilesLoaded(PathBuf, String, Result<Vec<CommitFile>, String>),
    SelectCommitFile(usize),
    CommitDiffLoaded(PathBuf, String, usize, Result<Diff, String>),
    CommitAction(CommitAction),

    // Branches
    ToggleBranchPopover,
    CloseBranchPopover,
    BranchFilter(String),
    Checkout(Branch),
    DeleteBranch(String),
    RenameBranch(String),
    MergeBranch(String),
    AbortMerge,

    // Sync
    SyncPressed,
    Fetch,
    Pull,
    Push,
    ForcePush,
    Stash,
    StashPop,

    // Generic operation result
    OpDone(PathBuf, Op, Result<(), String>),
    Cloned(Result<PathBuf, String>),

    // Dialogs
    OpenDialog(Dialog),
    DialogText(Field, String),
    DialogToggle(bool),
    DialogBrowse,
    DialogBrowsed(Option<PathBuf>),
    DialogPickRepo(String),
    DialogConfirm,
    DialogCancel,

    // Account & settings
    TokenInput(String),
    SignIn,
    ImportGhToken,
    TokenLoaded(Option<String>),
    SignedIn(Result<(github::User, String), String>),
    SignOut,
    AvatarLoaded(Result<Vec<u8>, String>),
    GithubRepos(Result<Vec<github::Repo>, String>),
    GitIdentityLoaded(Option<String>, Option<String>),
    GitName(String),
    GitEmail(String),
    SaveGitIdentity,
    SetTheme(usize),
    SetEditor(String),
    SetTerminal(String),
    SetCloneDir(String),
    SetAutoFetch(bool),
    SetConfirmDiscard(bool),
    UpdateConfig(Config),

    // Integrations
    OpenInEditor,
    OpenInTerminal,
    OpenInFileManager,
    OpenOnGitHub,
    CompareOnGitHub,
    LaunchUrl(String),
    Copy(String),

    // Chrome
    ToggleContextPage(ContextPage),
    CloseToast(widget::ToastId),
    Menu(MenuAction),
    Surface(cosmic::surface::Action<Message>),
    Toast(String),
    Noop,
}

/// Actions from the menu bar and keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewRepository,
    AddLocal,
    Clone,
    Settings,
    About,
    Quit,
    ShowChanges,
    ShowHistory,
    Push,
    Pull,
    Fetch,
    ForcePush,
    OpenInEditor,
    OpenInTerminal,
    OpenInFileManager,
    OpenOnGitHub,
    RemoveRepository,
    NewBranch,
    RenameBranch,
    DeleteBranch,
    Merge,
    CompareOnGitHub,
    Stash,
    StashPop,
    DiscardAll,
    Refresh,
    ReportIssue,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Message {
        Message::Menu(*self)
    }
}

/// Right-click actions on the repository list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    Remove(nav_bar::Id),
    OpenFolder(nav_bar::Id),
    OpenTerminal(nav_bar::Id),
}

impl menu::action::MenuAction for NavAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::NavAction(*self))
    }
}

/// Right-click actions on a changed file (index into the status list).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    Discard(usize),
    Ignore(usize),
    IgnoreExtension(usize),
    CopyPath(usize),
    Reveal(usize),
    OpenFile(usize),
}

impl menu::action::MenuAction for FileAction {
    type Message = Message;

    fn message(&self) -> Message {
        Message::FileAction(*self)
    }
}

/// Right-click actions on a commit (index into the history list).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitAction {
    Revert(usize),
    CherryPick(usize),
    BranchFrom(usize),
    CopySha(usize),
    CopySummary(usize),
    Undo(usize),
    ViewOnGitHub(usize),
}

impl menu::action::MenuAction for CommitAction {
    type Message = Message;

    fn message(&self) -> Message {
        Message::CommitAction(*self)
    }
}

pub struct App {
    pub core: cosmic::Core,
    pub config: Config,
    config_handler: Option<cosmic_config::Config>,
    pub nav: nav_bar::Model,
    pub tabs: segmented_button::SingleSelectModel,
    pub key_binds: HashMap<KeyBind, MenuAction>,
    pub toasts: widget::Toasts<Message>,
    pub about: About,
    pub context_page: ContextPage,
    pub dialog: Option<Dialog>,
    pub repo: Option<RepoState>,
    pub tab: Tab,
    pub busy: Option<Op>,
    pub branch_popover: bool,
    pub branch_filter: String,
    pub summary: String,
    pub description: text_editor::Content,
    pub amend: bool,
    // Account
    pub token: Option<String>,
    pub token_input: String,
    pub signing_in: bool,
    pub user: Option<github::User>,
    pub avatar: Option<widget::image::Handle>,
    pub github_repos: Vec<github::Repo>,
    pub github_repos_loading: bool,
    // Git identity
    pub git_name: String,
    pub git_email: String,
    pub editors: Vec<String>,
    pub terminals: Vec<String>,
    pub theme_names: Vec<String>,
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.kisakay.GithubReDesktop";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<cosmic::Action<Message>>) {
        let (config_handler, config) = Config::load(Self::APP_ID);

        let mut tabs = segmented_button::SingleSelectModel::default();
        tabs.insert()
            .text(fl!("tab-changes"))
            .data(Tab::Changes)
            .activate();
        tabs.insert().text(fl!("tab-history")).data(Tab::History);

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .author("Kisakay")
            .license("MIT")
            .links([
                (fl!("about-repository"), REPOSITORY.to_string()),
                (fl!("about-issues"), format!("{REPOSITORY}/issues")),
                (
                    fl!("about-original"),
                    "https://github.com/desktop/desktop".to_string(),
                ),
            ])
            .comments(fl!("about-comments"));

        let mut app = App {
            core,
            config,
            config_handler,
            nav: nav_bar::Model::default(),
            tabs,
            key_binds: key_binds(),
            toasts: widget::Toasts::new(Message::CloseToast),
            about,
            context_page: ContextPage::default(),
            dialog: None,
            repo: None,
            tab: Tab::Changes,
            busy: None,
            branch_popover: false,
            branch_filter: String::new(),
            summary: String::new(),
            description: text_editor::Content::new(),
            amend: false,
            token: None,
            token_input: String::new(),
            signing_in: false,
            user: None,
            avatar: None,
            github_repos: Vec::new(),
            github_repos_loading: false,
            git_name: String::new(),
            git_email: String::new(),
            editors: tools::available_editors(),
            terminals: tools::available_terminals(),
            theme_names: vec![fl!("theme-system"), fl!("theme-dark"), fl!("theme-light")],
        };

        app.rebuild_nav();

        let mut tasks = vec![
            cosmic::command::set_theme(app.config.app_theme.theme()),
            cosmic::task::future(async { Message::TokenLoaded(github::load_token().await) }),
            cosmic::task::future(async {
                let name = git::config_get(None, "user.name").await;
                let email = git::config_get(None, "user.email").await;
                Message::GitIdentityLoaded(name, email)
            }),
        ];

        let last = app
            .config
            .last_repository
            .clone()
            .filter(|p| app.config.repositories.contains(p))
            .or_else(|| app.config.repositories.first().cloned());
        if let Some(last) = last {
            tasks.push(app.open_repo(PathBuf::from(last)));
        }
        tasks.push(app.update_title());

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![crate::ui::menu_bar(self)]
    }

    fn header_end(&self) -> Vec<Element<'_, Message>> {
        crate::ui::header_end(self)
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        // No sidebar until there is at least one repository to show.
        (!self.config.repositories.is_empty()).then_some(&self.nav)
    }

    fn nav_context_menu(&self) -> Option<Vec<menu::Tree<cosmic::Action<Message>>>> {
        Some(menu::nav_context(
            &HashMap::new(),
            self.nav
                .iter()
                .map(|id| {
                    vec![
                        menu::Item::Button(fl!("menu-show-in-files"), None, NavAction::OpenFolder(id)),
                        menu::Item::Button(fl!("menu-open-terminal"), None, NavAction::OpenTerminal(id)),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-remove-repository"), None, NavAction::Remove(id)),
                    ]
                })
                .collect(),
        ))
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Message>> {
        let Some(path) = self.nav.data::<PathBuf>(id).cloned() else {
            return Task::none();
        };
        self.nav.activate(id);
        if self.repo.as_ref().is_some_and(|r| r.path == path) {
            return Task::none();
        }
        let open = self.open_repo(path);
        Task::batch([open, self.update_title()])
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        if !self.core.window.show_context {
            return None;
        }
        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::Settings => context_drawer::context_drawer(
                crate::ui::settings::view(self),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        self.dialog.as_ref().map(|d| crate::ui::dialogs::view(self, d))
    }

    fn on_escape(&mut self) -> Task<cosmic::Action<Message>> {
        if self.dialog.take().is_some() {
            return Task::none();
        }
        if self.branch_popover {
            self.branch_popover = false;
            return Task::none();
        }
        if self.core.window.show_context {
            self.core.window.show_context = false;
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        widget::toaster(&self.toasts, crate::ui::view(self))
    }

    fn subscription(&self) -> Subscription<Message> {
        let keys = event::listen_with(|event, status, _id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                physical_key,
                ..
            }) => {
                if status == event::Status::Captured && !modifiers.control() {
                    return None;
                }
                key_binds()
                    .into_iter()
                    .find(|(bind, _)| bind.matches(modifiers, &key, Some(&physical_key)))
                    .map(|(_, action)| Message::Menu(action))
            }
            Event::Window(window::Event::Focused) => Some(Message::Refresh),
            _ => None,
        });

        Subscription::batch([
            keys,
            cosmic::iced::time::every(Duration::from_secs(5)).map(|_| Message::Tick),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::RepoLoaded(path, result) => {
                let Some(repo) = self.repo.as_mut().filter(|r| r.path == path) else {
                    return Task::none();
                };
                match result {
                    Ok(snapshot) => {
                        let head_changed = repo.snapshot.status.head != snapshot.status.head
                            || repo.snapshot.status.branch != snapshot.status.branch;
                        // Drop exclusions for files that are gone.
                        repo.excluded
                            .retain(|p| snapshot.status.files.iter().any(|f| &f.path == p));
                        let first_load = !repo.loaded;
                        repo.snapshot = snapshot;
                        repo.loaded = true;

                        let mut tasks = Vec::new();
                        // Keep the selected file valid.
                        let selected_still_exists = repo
                            .selected_file
                            .as_ref()
                            .is_some_and(|s| repo.snapshot.status.files.iter().any(|f| &f.path == s));
                        if !selected_still_exists {
                            repo.selected_file = repo.snapshot.status.files.first().map(|f| f.path.clone());
                            repo.diff = None;
                        }
                        if let Some(sel) = repo.selected_file.clone() {
                            tasks.push(self.load_diff(sel));
                        }
                        if first_load || head_changed {
                            tasks.push(self.load_history(0));
                        }
                        return Task::batch(tasks);
                    }
                    Err(err) => {
                        repo.loaded = true;
                        return self.toast(err);
                    }
                }
            }

            Message::Tick => {
                if self.busy.is_none() && self.dialog.is_none() {
                    let auto_fetch = self.config.auto_fetch
                        && self.token.is_some()
                        && self.repo.as_ref().is_some_and(|r| {
                            r.snapshot.remote_url.is_some()
                                && r.last_fetch.is_none_or(|t| t.elapsed() >= AUTO_FETCH_EVERY)
                        });
                    if auto_fetch {
                        return self.update(Message::Fetch);
                    }
                    return self.refresh();
                }
            }

            Message::Refresh => return self.refresh(),

            Message::AddLocal => {
                return cosmic::task::future(async {
                    let picked = rfd::AsyncFileDialog::new()
                        .set_title(fl!("add-local-title"))
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf());
                    Message::AddLocalPicked(picked)
                });
            }

            Message::AddLocalPicked(Some(path)) => {
                return cosmic::task::future(async move {
                    Message::RepoAdded(
                        git::toplevel(&path)
                            .await
                            .map_err(|_| fl!("not-a-repository", path = path.display().to_string())),
                    )
                });
            }
            Message::AddLocalPicked(None) => {}

            Message::RepoAdded(Ok(path)) | Message::Cloned(Ok(path)) => {
                self.dialog = None;
                self.busy = None;
                let key = path.to_string_lossy().into_owned();
                if !self.config.repositories.contains(&key) {
                    self.config.repositories.push(key);
                    self.config
                        .repositories
                        .sort_by_key(|p| repo_display_name(p).to_lowercase());
                    self.save_config();
                }
                self.rebuild_nav();
                let open = self.open_repo(path);
                return Task::batch([open, self.update_title()]);
            }
            Message::RepoAdded(Err(err)) | Message::Cloned(Err(err)) => {
                self.busy = None;
                return self.toast(err);
            }

            Message::RemoveRepo(path) => {
                self.dialog = None;
                let key = path.to_string_lossy().into_owned();
                self.config.repositories.retain(|p| p != &key);
                if self.repo.as_ref().is_some_and(|r| r.path == path) {
                    self.repo = None;
                    self.config.last_repository = None;
                }
                self.save_config();
                self.rebuild_nav();
                if self.repo.is_none()
                    && let Some(next) = self.config.repositories.first().cloned()
                {
                    let open = self.open_repo(PathBuf::from(next));
                    return Task::batch([open, self.update_title()]);
                }
                return self.update_title();
            }

            Message::NavAction(action) => {
                let id = match action {
                    NavAction::Remove(id) | NavAction::OpenFolder(id) | NavAction::OpenTerminal(id) => id,
                };
                let Some(path) = self.nav.data::<PathBuf>(id).cloned() else {
                    return Task::none();
                };
                match action {
                    NavAction::Remove(_) => self.dialog = Some(Dialog::ConfirmRemoveRepo(path)),
                    NavAction::OpenFolder(_) => return self.report(tools::open_path(&path)),
                    NavAction::OpenTerminal(_) => {
                        return self.report(tools::open_in_terminal(&self.config.terminal, &path));
                    }
                }
            }

            Message::TabActivated(entity) => {
                self.tabs.activate(entity);
                if let Some(tab) = self.tabs.data::<Tab>(entity).copied() {
                    return self.update(Message::SetTab(tab));
                }
            }

            Message::SetTab(tab) => {
                self.tab = tab;
                let entity = self.tabs.iter().find(|e| self.tabs.data::<Tab>(*e) == Some(&tab));
                if let Some(entity) = entity {
                    self.tabs.activate(entity);
                }
                if tab == Tab::History
                    && let Some(repo) = &self.repo
                    && repo.selected_commit.is_none()
                    && let Some(first) = repo.commits.first()
                {
                    return self.update(Message::SelectCommit(first.sha.clone()));
                }
            }

            // ----- Changes -----
            Message::ToggleFile(path, included) => {
                if let Some(repo) = &mut self.repo {
                    if included {
                        repo.excluded.remove(&path);
                    } else {
                        repo.excluded.insert(path);
                    }
                }
            }

            Message::ToggleAll(included) => {
                if let Some(repo) = &mut self.repo {
                    if included {
                        repo.excluded.clear();
                    } else {
                        repo.excluded = repo
                            .snapshot
                            .status
                            .files
                            .iter()
                            .map(|f| f.path.clone())
                            .collect();
                    }
                }
            }

            Message::SelectFile(path) => {
                if let Some(repo) = &mut self.repo {
                    if repo.selected_file.as_ref() != Some(&path) {
                        repo.diff = None;
                    }
                    repo.selected_file = Some(path.clone());
                    return self.load_diff(path);
                }
            }

            Message::DiffLoaded(repo_path, file, result) => {
                if let Some(repo) = self.repo.as_mut().filter(|r| r.path == repo_path)
                    && repo.selected_file.as_ref() == Some(&file)
                {
                    match result {
                        Ok(diff) => repo.diff = Some(diff),
                        Err(err) => {
                            repo.diff = None;
                            return self.toast(err);
                        }
                    }
                }
            }

            Message::Summary(s) => self.summary = s,
            Message::Description(action) => self.description.perform(action),
            Message::ToggleAmend(amend) => {
                self.amend = amend;
                if amend
                    && self.summary.is_empty()
                    && let Some(head) = self.repo.as_ref().and_then(|r| r.commits.first())
                {
                    self.summary = head.summary.clone();
                    self.description = text_editor::Content::with_text(&head.body);
                }
            }

            Message::Commit => {
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let summary = self.summary.trim().to_string();
                if summary.is_empty() {
                    return Task::none();
                }
                let mut paths = Vec::new();
                for f in repo.included_files() {
                    paths.push(f.path.clone());
                    if let Some(orig) = &f.orig_path {
                        paths.push(orig.clone());
                    }
                }
                if paths.is_empty() && !self.amend {
                    return Task::none();
                }
                let description = self.description.text();
                let amend = self.amend;
                let path = repo.path.clone();
                return self.run_op(Op::Commit, async move {
                    git::commit(&path, &paths, &summary, &description, amend).await
                });
            }

            Message::Discard(files) => {
                if self.config.confirm_discard {
                    self.dialog = Some(Dialog::ConfirmDiscard(files));
                } else {
                    return self.discard(files);
                }
            }

            Message::DiscardAll => {
                if let Some(repo) = &self.repo {
                    let files = repo.snapshot.status.files.clone();
                    if !files.is_empty() {
                        return self.update(Message::Discard(files));
                    }
                }
            }

            Message::FileAction(action) => return self.file_action(action),

            // ----- History -----
            Message::HistoryLoaded(path, skip, result) => {
                let Some(repo) = self.repo.as_mut().filter(|r| r.path == path) else {
                    return Task::none();
                };
                match result {
                    Ok(commits) => {
                        repo.history_complete = commits.len() < HISTORY_PAGE;
                        if skip == 0 {
                            repo.commits = commits;
                        } else {
                            repo.commits.extend(commits);
                        }
                        let selected_valid = repo
                            .selected_commit
                            .as_ref()
                            .is_some_and(|s| repo.commits.iter().any(|c| &c.sha == s));
                        if !selected_valid {
                            repo.selected_commit = None;
                            repo.commit_files.clear();
                            repo.commit_diff = None;
                            if self.tab == Tab::History
                                && let Some(first) = repo.commits.first()
                            {
                                let sha = first.sha.clone();
                                return self.update(Message::SelectCommit(sha));
                            }
                        }
                    }
                    Err(err) => return self.toast(err),
                }
            }

            Message::LoadMoreHistory => {
                if let Some(repo) = &self.repo {
                    let skip = repo.commits.len();
                    return self.load_history(skip);
                }
            }

            Message::SelectCommit(sha) => {
                let Some(repo) = &mut self.repo else {
                    return Task::none();
                };
                repo.selected_commit = Some(sha.clone());
                repo.commit_files.clear();
                repo.selected_commit_file = None;
                repo.commit_diff = None;
                let path = repo.path.clone();
                return cosmic::task::future(async move {
                    let files = git::log::commit_files(&path, &sha).await.map_err(|e| e.0);
                    Message::CommitFilesLoaded(path, sha, files)
                });
            }

            Message::CommitFilesLoaded(path, sha, result) => {
                let Some(repo) = self.repo.as_mut().filter(|r| r.path == path) else {
                    return Task::none();
                };
                if repo.selected_commit.as_ref() != Some(&sha) {
                    return Task::none();
                }
                match result {
                    Ok(files) => {
                        repo.commit_files = files;
                        if !repo.commit_files.is_empty() {
                            return self.update(Message::SelectCommitFile(0));
                        }
                    }
                    Err(err) => return self.toast(err),
                }
            }

            Message::SelectCommitFile(index) => {
                let Some(repo) = &mut self.repo else {
                    return Task::none();
                };
                let (Some(sha), Some(file)) = (
                    repo.selected_commit.clone(),
                    repo.commit_files.get(index).cloned(),
                ) else {
                    return Task::none();
                };
                repo.selected_commit_file = Some(index);
                repo.commit_diff = None;
                let path = repo.path.clone();
                return cosmic::task::future(async move {
                    let diff = git::diff::commit_diff(&path, &sha, &file.path, file.orig_path.as_deref())
                        .await
                        .map_err(|e| e.0);
                    Message::CommitDiffLoaded(path, sha, index, diff)
                });
            }

            Message::CommitDiffLoaded(path, sha, index, result) => {
                if let Some(repo) = self.repo.as_mut().filter(|r| r.path == path)
                    && repo.selected_commit.as_ref() == Some(&sha)
                    && repo.selected_commit_file == Some(index)
                {
                    match result {
                        Ok(diff) => repo.commit_diff = Some(diff),
                        Err(err) => return self.toast(err),
                    }
                }
            }

            Message::CommitAction(action) => return self.commit_action(action),

            // ----- Branches -----
            Message::ToggleBranchPopover => {
                self.branch_popover = !self.branch_popover;
                self.branch_filter.clear();
                if self.branch_popover {
                    return self.refresh();
                }
            }
            Message::CloseBranchPopover => self.branch_popover = false,
            Message::BranchFilter(filter) => self.branch_filter = filter,

            Message::Checkout(branch) => {
                self.branch_popover = false;
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                if branch.current {
                    return Task::none();
                }
                let path = repo.path.clone();
                return self.run_op(Op::Checkout, async move {
                    git::branch::checkout(&path, &branch).await
                });
            }

            Message::DeleteBranch(name) => {
                self.branch_popover = false;
                self.dialog = Some(Dialog::ConfirmDeleteBranch(name));
            }

            Message::RenameBranch(old) => {
                self.branch_popover = false;
                self.dialog = Some(Dialog::RenameBranch {
                    name: old.clone(),
                    old,
                });
            }

            Message::MergeBranch(name) => {
                self.dialog = None;
                self.branch_popover = false;
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::Merge, async move { git::branch::merge(&path, &name).await });
            }

            Message::AbortMerge => {
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::AbortMerge, async move { git::merge_abort(&path).await });
            }

            // ----- Sync -----
            Message::SyncPressed => {
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let st = repo.status();
                if repo.snapshot.remote_url.is_none() {
                    return self.open_publish_dialog();
                }
                return self.update(if st.upstream.is_none() || st.ahead > 0 && st.behind == 0 {
                    Message::Push
                } else if st.behind > 0 {
                    Message::Pull
                } else {
                    Message::Fetch
                });
            }

            Message::Fetch => {
                let Some(repo) = &mut self.repo else {
                    return Task::none();
                };
                if repo.snapshot.remote_url.is_none() {
                    return Task::none();
                }
                repo.last_fetch = Some(Instant::now());
                let path = repo.path.clone();
                let token = self.token.clone();
                return self.run_op(
                    Op::Fetch,
                    async move { git::fetch(&path, token.as_deref()).await },
                );
            }

            Message::Pull => {
                let Some(repo) = &mut self.repo else {
                    return Task::none();
                };
                repo.last_fetch = Some(Instant::now());
                let path = repo.path.clone();
                let token = self.token.clone();
                return self.run_op(Op::Pull, async move { git::pull(&path, token.as_deref()).await });
            }

            Message::Push | Message::ForcePush => {
                let force = matches!(message, Message::ForcePush);
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                if repo.snapshot.remote_url.is_none() {
                    return self.open_publish_dialog();
                }
                let path = repo.path.clone();
                let has_upstream = repo.status().upstream.is_some();
                let token = self.token.clone();
                self.dialog = None;
                let op = if force { Op::ForcePush } else { Op::Push };
                return self.run_op(op, async move {
                    git::push(&path, has_upstream, force, token.as_deref()).await
                });
            }

            Message::Stash => {
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                let message = fl!("stash-message", branch = repo.branch_label());
                return self.run_op(Op::Stash, async move { git::stash(&path, &message).await });
            }

            Message::StashPop => {
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::StashPop, async move { git::stash_pop(&path).await });
            }

            Message::OpDone(path, op, result) => {
                if self.busy == Some(op) {
                    self.busy = None;
                }
                let mut tasks = Vec::new();
                match result {
                    Ok(()) => {
                        if op == Op::Commit {
                            self.summary.clear();
                            self.description = text_editor::Content::new();
                            self.amend = false;
                            if let Some(repo) = &mut self.repo {
                                repo.excluded.clear();
                            }
                        }
                        if let Some(label) = op.success_label()
                            && op != Op::Fetch
                        {
                            tasks.push(self.toast(label));
                        }
                    }
                    Err(err) => tasks.push(self.toast(err)),
                }
                if self.repo.as_ref().is_some_and(|r| r.path == path) {
                    tasks.push(self.refresh());
                    if matches!(
                        op,
                        Op::Commit | Op::Pull | Op::Revert | Op::CherryPick | Op::Undo | Op::Merge
                    ) {
                        tasks.push(self.load_history(0));
                    }
                }
                return Task::batch(tasks);
            }

            // ----- Dialogs -----
            Message::OpenDialog(dialog) => {
                self.branch_popover = false;
                let needs_repos = matches!(dialog, Dialog::Clone { .. })
                    && self.github_repos.is_empty()
                    && !self.github_repos_loading;
                self.dialog = Some(dialog);
                if needs_repos && let Some(token) = self.token.clone() {
                    self.github_repos_loading = true;
                    return cosmic::task::future(
                        async move { Message::GithubRepos(github::repos(&token).await) },
                    );
                }
            }

            Message::DialogText(field, value) => self.dialog_text(field, value),

            Message::DialogToggle(value) => match &mut self.dialog {
                Some(Dialog::Create { readme, .. }) => *readme = value,
                Some(Dialog::Publish { private, .. }) => *private = value,
                _ => {}
            },

            Message::DialogBrowse => {
                return cosmic::task::future(async {
                    let picked = rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf());
                    Message::DialogBrowsed(picked)
                });
            }

            Message::DialogBrowsed(Some(dir)) => match &mut self.dialog {
                Some(Dialog::Clone {
                    url,
                    path,
                    path_edited,
                    ..
                }) => {
                    let name = git::repo_name_from_url(url);
                    *path = if url.trim().is_empty() {
                        dir.to_string_lossy().into_owned()
                    } else {
                        dir.join(name).to_string_lossy().into_owned()
                    };
                    *path_edited = true;
                }
                Some(Dialog::Create { parent, .. }) => *parent = dir.to_string_lossy().into_owned(),
                _ => {}
            },
            Message::DialogBrowsed(None) => {}

            Message::DialogPickRepo(clone_url) => {
                self.dialog_text(Field::Url, clone_url);
            }

            Message::DialogCancel => self.dialog = None,

            Message::DialogConfirm => return self.confirm_dialog(),

            // ----- Account -----
            Message::TokenInput(t) => self.token_input = t,

            Message::SignIn => {
                let token = self.token_input.trim().to_string();
                if token.is_empty() {
                    return Task::none();
                }
                return self.sign_in(token);
            }

            Message::ImportGhToken => {
                self.signing_in = true;
                return cosmic::task::future(async {
                    match github::gh_cli_token().await {
                        Some(token) => {
                            let result = github::user(&token).await.map(|u| (u, token));
                            Message::SignedIn(result)
                        }
                        None => Message::SignedIn(Err(fl!("gh-not-found"))),
                    }
                });
            }

            Message::TokenLoaded(Some(token)) => {
                self.token = Some(token.clone());
                return cosmic::task::future(async move {
                    Message::SignedIn(github::user(&token).await.map(|u| (u, token)))
                });
            }
            Message::TokenLoaded(None) => {}

            Message::SignedIn(Ok((user, token))) => {
                let first_sign_in = self.user.is_none() && self.signing_in;
                self.signing_in = false;
                self.token_input.clear();
                self.token = Some(token.clone());
                let avatar_url = user.avatar_url.clone();
                let login = user.login.clone();
                self.user = Some(user);
                let mut tasks = vec![
                    cosmic::task::future(
                        async move { Message::AvatarLoaded(github::avatar(&avatar_url).await) },
                    ),
                    cosmic::task::future(async move {
                        let _ = github::store_token(token).await;
                        Message::Noop
                    }),
                ];
                if first_sign_in {
                    tasks.push(self.toast(fl!("signed-in-as", login = login)));
                }
                return Task::batch(tasks);
            }
            Message::SignedIn(Err(err)) => {
                self.signing_in = false;
                return self.toast(err);
            }

            Message::SignOut => {
                self.token = None;
                self.user = None;
                self.avatar = None;
                self.github_repos.clear();
                return cosmic::task::future(async {
                    github::delete_token().await;
                    Message::Noop
                });
            }

            Message::AvatarLoaded(Ok(bytes)) => {
                self.avatar = Some(widget::image::Handle::from_bytes(bytes));
            }
            Message::AvatarLoaded(Err(_)) => {}

            Message::GithubRepos(result) => {
                self.github_repos_loading = false;
                match result {
                    Ok(repos) => self.github_repos = repos,
                    Err(err) => return self.toast(err),
                }
            }

            Message::GitIdentityLoaded(name, email) => {
                self.git_name = name.unwrap_or_default();
                self.git_email = email.unwrap_or_default();
            }
            Message::GitName(n) => self.git_name = n,
            Message::GitEmail(e) => self.git_email = e,
            Message::SaveGitIdentity => {
                let name = self.git_name.trim().to_string();
                let email = self.git_email.trim().to_string();
                return cosmic::task::future(async move {
                    let mut result = Ok(());
                    if !name.is_empty() {
                        result = git::config_set_global("user.name", &name).await;
                    }
                    if result.is_ok() && !email.is_empty() {
                        result = git::config_set_global("user.email", &email).await;
                    }
                    match result {
                        Ok(()) => Message::Toast(fl!("identity-saved")),
                        Err(e) => Message::Toast(e.0),
                    }
                });
            }

            Message::SetTheme(index) => {
                self.config.app_theme = AppTheme::ALL.get(index).copied().unwrap_or_default();
                self.save_config();
                return cosmic::command::set_theme(self.config.app_theme.theme());
            }
            Message::SetEditor(e) => {
                self.config.editor = e;
                self.save_config();
            }
            Message::SetTerminal(t) => {
                self.config.terminal = t;
                self.save_config();
            }
            Message::SetCloneDir(d) => {
                self.config.clone_dir = d;
                self.save_config();
            }
            Message::SetAutoFetch(v) => {
                self.config.auto_fetch = v;
                self.save_config();
            }
            Message::SetConfirmDiscard(v) => {
                self.config.confirm_discard = v;
                self.save_config();
            }
            Message::UpdateConfig(config) => {
                if config != self.config {
                    let theme_changed = config.app_theme != self.config.app_theme;
                    self.config = config;
                    self.rebuild_nav();
                    if theme_changed {
                        return cosmic::command::set_theme(self.config.app_theme.theme());
                    }
                }
            }

            // ----- Integrations -----
            Message::OpenInEditor => {
                if let Some(repo) = &self.repo {
                    return self.report(tools::open_in_editor(&self.config.editor, &repo.path));
                }
            }
            Message::OpenInTerminal => {
                if let Some(repo) = &self.repo {
                    return self.report(tools::open_in_terminal(&self.config.terminal, &repo.path));
                }
            }
            Message::OpenInFileManager => {
                if let Some(repo) = &self.repo {
                    return self.report(tools::open_path(&repo.path));
                }
            }
            Message::OpenOnGitHub => {
                if let Some(url) = self.repo.as_ref().and_then(RepoState::web_url) {
                    return self.report(tools::open_url(&url));
                }
            }
            Message::CompareOnGitHub => {
                if let Some(repo) = &self.repo
                    && let Some(url) = repo.web_url()
                {
                    let branch = repo.branch_label();
                    return self.report(tools::open_url(&format!("{url}/compare/{branch}?expand=1")));
                }
            }
            Message::LaunchUrl(url) => return self.report(tools::open_url(&url)),
            Message::Copy(text) => {
                if !text.is_empty() {
                    return cosmic::iced::clipboard::write(text);
                }
            }

            // ----- Chrome -----
            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }
            Message::CloseToast(id) => self.toasts.remove(id),
            Message::Menu(action) => return self.menu_action(action),
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Surface(action));
            }
            Message::Toast(text) => return self.toast(text),
            Message::Noop => {}
        }
        Task::none()
    }
}

impl App {
    /// Updates the window title.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut title = fl!("app-title");
        if let Some(repo) = &self.repo {
            title = format!("{} — {}", repo.name, title);
        }
        self.set_header_title(title.clone());
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(title, id)
        } else {
            Task::none()
        }
    }

    fn save_config(&self) {
        if let Some(handler) = &self.config_handler
            && let Err(err) = self.config.write_entry(handler)
        {
            eprintln!("failed to save config: {err}");
        }
    }

    /// Rebuilds the sidebar from the configured repositories.
    fn rebuild_nav(&mut self) {
        let active = self.repo.as_ref().map(|r| r.path.clone());
        self.nav.clear();
        for repo in &self.config.repositories {
            let path = PathBuf::from(repo);
            let exists = path.exists();
            let is_active = active.as_ref() == Some(&path);
            let mut item = self
                .nav
                .insert()
                .text(repo_display_name(repo))
                .icon(widget::icon::from_name(if exists {
                    "folder-symbolic"
                } else {
                    "dialog-warning-symbolic"
                }))
                .data::<PathBuf>(path);
            if is_active {
                item = item.activate();
            }
            let _ = item;
        }
    }

    /// Switches to the repository at `path`.
    pub fn open_repo(&mut self, path: PathBuf) -> Task<cosmic::Action<Message>> {
        self.branch_popover = false;
        self.summary.clear();
        self.description = text_editor::Content::new();
        self.amend = false;
        self.repo = Some(RepoState::new(path.clone()));
        let key = path.to_string_lossy().into_owned();
        if self.config.last_repository.as_ref() != Some(&key) {
            self.config.last_repository = Some(key);
            self.save_config();
        }
        let id = self
            .nav
            .iter()
            .find(|id| self.nav.data::<PathBuf>(*id) == Some(&path));
        if let Some(id) = id {
            self.nav.activate(id);
        }
        self.refresh()
    }

    /// Reloads the status of the current repository.
    pub fn refresh(&self) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let path = repo.path.clone();
        cosmic::task::future(async move {
            let result = load_snapshot(&path).await;
            Message::RepoLoaded(path, result)
        })
    }

    fn load_diff(&self, file_path: String) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let Some(file) = repo
            .snapshot
            .status
            .files
            .iter()
            .find(|f| f.path == file_path)
            .cloned()
        else {
            return Task::none();
        };
        let path = repo.path.clone();
        cosmic::task::future(async move {
            let diff = git::diff::working_diff(&path, &file).await.map_err(|e| e.0);
            Message::DiffLoaded(path, file.path, diff)
        })
    }

    fn load_history(&self, skip: usize) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let path = repo.path.clone();
        cosmic::task::future(async move {
            let commits = git::log::log(&path, skip, HISTORY_PAGE).await.map_err(|e| e.0);
            Message::HistoryLoaded(path, skip, commits)
        })
    }

    /// Runs a git operation on the current repository and reports its result.
    fn run_op<F>(&mut self, op: Op, fut: F) -> Task<cosmic::Action<Message>>
    where
        F: std::future::Future<Output = git::Result<()>> + Send + 'static,
    {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        if op.is_network() {
            if self.busy.is_some() {
                return Task::none();
            }
            self.busy = Some(op);
        }
        let path = repo.path.clone();
        cosmic::task::future(async move {
            let result = fut.await.map_err(|e| e.0);
            Message::OpDone(path, op, result)
        })
    }

    pub fn toast(&mut self, message: impl Into<String>) -> Task<cosmic::Action<Message>> {
        self.toasts
            .push(widget::Toast::new(message.into()))
            .map(cosmic::Action::App)
    }

    fn report(&mut self, result: Result<(), String>) -> Task<cosmic::Action<Message>> {
        match result {
            Ok(()) => Task::none(),
            Err(err) => self.toast(err),
        }
    }

    fn discard(&mut self, files: Vec<FileChange>) -> Task<cosmic::Action<Message>> {
        self.dialog = None;
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let path = repo.path.clone();
        self.run_op(Op::Discard, async move { git::discard(&path, &files).await })
    }

    fn sign_in(&mut self, token: String) -> Task<cosmic::Action<Message>> {
        self.signing_in = true;
        cosmic::task::future(async move { Message::SignedIn(github::user(&token).await.map(|u| (u, token))) })
    }

    fn open_publish_dialog(&mut self) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        if self.token.is_none() {
            self.context_page = ContextPage::Settings;
            self.core.window.show_context = true;
            return self.toast(fl!("sign-in-to-publish"));
        }
        self.dialog = Some(Dialog::Publish {
            name: repo.name.clone(),
            description: String::new(),
            private: true,
        });
        Task::none()
    }

    fn dialog_text(&mut self, field: Field, value: String) {
        let Some(dialog) = &mut self.dialog else { return };
        match (dialog, field) {
            (
                Dialog::Clone {
                    url,
                    path,
                    path_edited,
                    ..
                },
                Field::Url,
            ) => {
                *url = value;
                if !*path_edited {
                    let name = git::repo_name_from_url(&git::normalize_clone_url(url));
                    *path = if url.trim().is_empty() {
                        String::new()
                    } else {
                        Path::new(&self.config.clone_dir)
                            .join(name)
                            .to_string_lossy()
                            .into_owned()
                    };
                }
            }
            (
                Dialog::Clone {
                    path, path_edited, ..
                },
                Field::Path,
            ) => {
                *path = value;
                *path_edited = true;
            }
            (Dialog::Clone { filter, .. } | Dialog::Merge { filter }, Field::Filter) => *filter = value,
            (Dialog::Create { name, .. }, Field::Name) => *name = value,
            (Dialog::Create { parent, .. }, Field::Path) => *parent = value,
            (Dialog::Create { gitignore, .. }, Field::Gitignore) => *gitignore = value,
            (Dialog::NewBranch { name, .. } | Dialog::RenameBranch { name, .. }, Field::Name) => {
                *name = value
            }
            (Dialog::Publish { name, .. }, Field::Name) => *name = value,
            (Dialog::Publish { description, .. }, Field::Description) => *description = value,
            _ => {}
        }
    }

    #[allow(clippy::too_many_lines)]
    fn confirm_dialog(&mut self) -> Task<cosmic::Action<Message>> {
        let Some(dialog) = self.dialog.clone() else {
            return Task::none();
        };
        match dialog {
            Dialog::Clone { url, path, .. } => {
                let url = git::normalize_clone_url(&url);
                if url.is_empty() || path.trim().is_empty() || self.busy.is_some() {
                    return Task::none();
                }
                let dest = expand_tilde(path.trim());
                let token = self.token.clone();
                self.busy = Some(Op::Clone);
                return cosmic::task::future(async move {
                    if dest.exists() && std::fs::read_dir(&dest).is_ok_and(|mut d| d.next().is_some()) {
                        return Message::Cloned(Err(fl!(
                            "destination-not-empty",
                            path = dest.display().to_string()
                        )));
                    }
                    let result = git::clone(&url, &dest, token.as_deref())
                        .await
                        .map(|()| dest)
                        .map_err(|e| e.0);
                    Message::Cloned(result)
                });
            }
            Dialog::Create {
                name,
                parent,
                readme,
                gitignore,
            } => {
                let name = name.trim().to_string();
                if name.is_empty() {
                    return Task::none();
                }
                let dest = expand_tilde(parent.trim()).join(&name);
                return cosmic::task::future(async move {
                    if dest.join(".git").exists() {
                        return Message::RepoAdded(Err(fl!("already-a-repository")));
                    }
                    let result = git::init(&dest, &name, readme, Some(&gitignore))
                        .await
                        .map(|()| dest)
                        .map_err(|e| e.0);
                    Message::RepoAdded(result)
                });
            }
            Dialog::NewBranch { name, start } => {
                let name = git::branch::sanitize_name(&name);
                if name.is_empty() {
                    return Task::none();
                }
                self.dialog = None;
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::CreateBranch, async move {
                    git::branch::create(&path, &name, start.as_deref()).await
                });
            }
            Dialog::RenameBranch { old, name } => {
                let name = git::branch::sanitize_name(&name);
                if name.is_empty() || name == old {
                    self.dialog = None;
                    return Task::none();
                }
                self.dialog = None;
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::RenameBranch, async move {
                    git::branch::rename(&path, &old, &name).await
                });
            }
            Dialog::Merge { .. } => {}
            Dialog::Publish {
                name,
                description,
                private,
            } => {
                let (Some(token), Some(repo)) = (self.token.clone(), &self.repo) else {
                    return Task::none();
                };
                let name = name.trim().to_string();
                if name.is_empty() {
                    return Task::none();
                }
                self.dialog = None;
                let path = repo.path.clone();
                return self.run_op(Op::Publish, async move {
                    let created = github::create_repo(&token, &name, &description, private)
                        .await
                        .map_err(git::GitError)?;
                    git::set_origin(&path, &created.clone_url).await?;
                    if git::has_head(&path).await {
                        git::push(&path, false, false, Some(&token)).await?;
                    }
                    Ok(())
                });
            }
            Dialog::ConfirmDiscard(files) => return self.discard(files),
            Dialog::ConfirmDeleteBranch(name) => {
                self.dialog = None;
                let Some(repo) = &self.repo else {
                    return Task::none();
                };
                let path = repo.path.clone();
                return self.run_op(Op::DeleteBranch, async move {
                    git::branch::delete(&path, &name).await
                });
            }
            Dialog::ConfirmRemoveRepo(path) => return self.update(Message::RemoveRepo(path)),
            Dialog::ConfirmForcePush => return self.update(Message::ForcePush),
        }
        Task::none()
    }

    fn file_action(&mut self, action: FileAction) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let index = match action {
            FileAction::Discard(i)
            | FileAction::Ignore(i)
            | FileAction::IgnoreExtension(i)
            | FileAction::CopyPath(i)
            | FileAction::Reveal(i)
            | FileAction::OpenFile(i) => i,
        };
        let Some(file) = repo.snapshot.status.files.get(index).cloned() else {
            return Task::none();
        };
        let full = repo.path.join(&file.path);
        let path = repo.path.clone();
        match action {
            FileAction::Discard(_) => self.update(Message::Discard(vec![file])),
            FileAction::Ignore(_) => {
                let pattern = format!("/{}", file.path);
                self.run_op(Op::Ignore, async move { git::ignore(&path, &pattern).await })
            }
            FileAction::IgnoreExtension(_) => {
                let Some(ext) = Path::new(&file.path).extension() else {
                    return Task::none();
                };
                let pattern = format!("*.{}", ext.to_string_lossy());
                self.run_op(Op::Ignore, async move { git::ignore(&path, &pattern).await })
            }
            FileAction::CopyPath(_) => cosmic::iced::clipboard::write(full.to_string_lossy().into_owned()),
            FileAction::Reveal(_) => {
                let dir = full.parent().map_or(path, Path::to_path_buf);
                self.report(tools::open_path(&dir))
            }
            FileAction::OpenFile(_) => self.report(tools::open_path(&full)),
        }
    }

    fn commit_action(&mut self, action: CommitAction) -> Task<cosmic::Action<Message>> {
        let Some(repo) = &self.repo else {
            return Task::none();
        };
        let index = match action {
            CommitAction::Revert(i)
            | CommitAction::CherryPick(i)
            | CommitAction::BranchFrom(i)
            | CommitAction::CopySha(i)
            | CommitAction::CopySummary(i)
            | CommitAction::Undo(i)
            | CommitAction::ViewOnGitHub(i) => i,
        };
        let Some(commit) = repo.commits.get(index).cloned() else {
            return Task::none();
        };
        let path = repo.path.clone();
        match action {
            CommitAction::Revert(_) => {
                self.run_op(Op::Revert, async move { git::revert(&path, &commit.sha).await })
            }
            CommitAction::CherryPick(_) => self.run_op(Op::CherryPick, async move {
                git::cherry_pick(&path, &commit.sha).await
            }),
            CommitAction::BranchFrom(_) => {
                self.dialog = Some(Dialog::NewBranch {
                    name: String::new(),
                    start: Some(commit.sha),
                });
                Task::none()
            }
            CommitAction::CopySha(_) => cosmic::iced::clipboard::write(commit.sha),
            CommitAction::CopySummary(_) => cosmic::iced::clipboard::write(commit.summary),
            CommitAction::Undo(_) => {
                // Restore the commit message so it can be edited & recommitted.
                self.summary = commit.summary.clone();
                self.description = text_editor::Content::with_text(&commit.body);
                self.run_op(Op::Undo, async move { git::undo_last_commit(&path).await })
            }
            CommitAction::ViewOnGitHub(_) => match repo.web_url() {
                Some(url) => self.report(tools::open_url(&format!("{url}/commit/{}", commit.sha))),
                None => Task::none(),
            },
        }
    }

    fn menu_action(&mut self, action: MenuAction) -> Task<cosmic::Action<Message>> {
        let has_repo = self.repo.is_some();
        match action {
            MenuAction::NewRepository => self.update(Message::OpenDialog(Dialog::Create {
                name: String::new(),
                parent: self.config.clone_dir.clone(),
                readme: true,
                gitignore: String::new(),
            })),
            MenuAction::AddLocal => self.update(Message::AddLocal),
            MenuAction::Clone => self.update(Message::OpenDialog(Dialog::Clone {
                url: String::new(),
                path: String::new(),
                filter: String::new(),
                path_edited: false,
            })),
            MenuAction::Settings => self.update(Message::ToggleContextPage(ContextPage::Settings)),
            MenuAction::About => self.update(Message::ToggleContextPage(ContextPage::About)),
            MenuAction::Quit => match self.core.main_window_id() {
                Some(id) => window::close(id),
                None => Task::none(),
            },
            MenuAction::ShowChanges => self.update(Message::SetTab(Tab::Changes)),
            MenuAction::ShowHistory => self.update(Message::SetTab(Tab::History)),
            MenuAction::Refresh => self.refresh(),
            MenuAction::ReportIssue => self.update(Message::LaunchUrl(format!("{REPOSITORY}/issues/new"))),
            _ if !has_repo => Task::none(),
            MenuAction::Push => self.update(Message::Push),
            MenuAction::Pull => self.update(Message::Pull),
            MenuAction::Fetch => self.update(Message::Fetch),
            MenuAction::ForcePush => {
                self.dialog = Some(Dialog::ConfirmForcePush);
                Task::none()
            }
            MenuAction::OpenInEditor => self.update(Message::OpenInEditor),
            MenuAction::OpenInTerminal => self.update(Message::OpenInTerminal),
            MenuAction::OpenInFileManager => self.update(Message::OpenInFileManager),
            MenuAction::OpenOnGitHub => self.update(Message::OpenOnGitHub),
            MenuAction::CompareOnGitHub => self.update(Message::CompareOnGitHub),
            MenuAction::RemoveRepository => {
                if let Some(repo) = &self.repo {
                    self.dialog = Some(Dialog::ConfirmRemoveRepo(repo.path.clone()));
                }
                Task::none()
            }
            MenuAction::NewBranch => self.update(Message::OpenDialog(Dialog::NewBranch {
                name: String::new(),
                start: None,
            })),
            MenuAction::RenameBranch => match self.repo.as_ref().and_then(|r| r.status().branch.clone()) {
                Some(b) => self.update(Message::RenameBranch(b)),
                None => Task::none(),
            },
            MenuAction::DeleteBranch => match self.repo.as_ref().and_then(|r| r.status().branch.clone()) {
                Some(b) => self.update(Message::DeleteBranch(b)),
                None => Task::none(),
            },
            MenuAction::Merge => self.update(Message::OpenDialog(Dialog::Merge {
                filter: String::new(),
            })),
            MenuAction::Stash => self.update(Message::Stash),
            MenuAction::StashPop => self.update(Message::StashPop),
            MenuAction::DiscardAll => self.update(Message::DiscardAll),
        }
    }
}

/// Loads everything displayed for a repository.
async fn load_snapshot(path: &Path) -> Result<Snapshot, String> {
    if !path.exists() {
        return Err(fl!("repository-missing", path = path.display().to_string()));
    }
    let status = git::status::status(path).await.map_err(|e| e.0)?;
    let branches = git::branch::list(path).await.unwrap_or_default();
    let remote_url = git::remote_url(path).await;
    let author = match (
        git::config_get(Some(path), "user.name").await,
        git::config_get(Some(path), "user.email").await,
    ) {
        (Some(name), Some(email)) => Some((name, email)),
        (Some(name), None) => Some((name, String::new())),
        _ => None,
    };
    let unpushed = if status.upstream.is_some() {
        git::log::unpushed_shas(path).await
    } else {
        Vec::new()
    };
    Ok(Snapshot {
        status,
        branches,
        remote_url,
        unpushed,
        author,
    })
}

fn repo_display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_string(), |n| n.to_string_lossy().into_owned())
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(path)
}

/// Keyboard shortcuts (mirroring GitHub Desktop where it makes sense).
pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut binds = HashMap::new();
    macro_rules! bind {
        ([$($m:ident),*], $key:expr, $action:ident) => {
            binds.insert(
                KeyBind { modifiers: vec![$(Modifier::$m),*], key: Key::Character($key.into()) },
                MenuAction::$action,
            );
        };
    }
    bind!([Ctrl], "n", NewRepository);
    bind!([Ctrl], "o", AddLocal);
    bind!([Ctrl, Shift], "o", Clone);
    bind!([Ctrl], ",", Settings);
    bind!([Ctrl], "q", Quit);
    bind!([Ctrl], "1", ShowChanges);
    bind!([Ctrl], "2", ShowHistory);
    bind!([Ctrl], "p", Push);
    bind!([Ctrl, Shift], "p", Pull);
    bind!([Ctrl, Shift], "t", Fetch);
    bind!([Ctrl, Shift], "a", OpenInEditor);
    bind!([Ctrl], "`", OpenInTerminal);
    bind!([Ctrl, Shift], "f", OpenInFileManager);
    bind!([Ctrl, Shift], "g", OpenOnGitHub);
    bind!([Ctrl, Shift], "n", NewBranch);
    bind!([Ctrl, Shift], "r", RenameBranch);
    bind!([Ctrl, Shift], "m", Merge);
    bind!([Ctrl, Shift], "s", Stash);
    bind!([Ctrl, Shift], "c", CompareOnGitHub);
    bind!([Ctrl], "r", Refresh);
    binds
}
