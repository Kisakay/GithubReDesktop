app-title = GitHub ReDesktop
settings = Settings

## About
about-repository = Source code
about-issues = Report an issue
about-original = Original GitHub Desktop
about-comments = A native rewrite of GitHub Desktop for Linux, built with Rust and libcosmic.

## Tabs
tab-changes = Changes
tab-history = History

## Menus
menu-file = File
menu-view = View
menu-repository = Repository
menu-branch = Branch
menu-help = Help
menu-new-repository = New repository…
menu-add-local = Add local repository…
menu-clone = Clone repository…
menu-quit = Quit
menu-push = Push
menu-pull = Pull
menu-fetch = Fetch
menu-force-push = Force push…
menu-open-editor = Open in editor
menu-open-terminal = Open in terminal
menu-show-in-files = Show in file manager
menu-view-on-github = View on GitHub
menu-refresh = Refresh
menu-remove-repository = Remove repository…
menu-new-branch = New branch…
menu-rename-branch = Rename branch…
menu-delete-branch = Delete branch…
menu-merge = Merge into current branch…
menu-compare-on-github = Compare on GitHub
menu-stash = Stash all changes
menu-stash-pop = Restore stashed changes
menu-discard-all = Discard all changes…
menu-report-issue = Report an issue…
menu-about = About GitHub ReDesktop
no-repository-selected = No repository selected

## Toolbar
current-branch = Current branch
detached-head = Detached at { $sha }
fetch-origin = Fetch { $remote }
pull-origin = Pull { $remote }
push-origin = Push { $remote }
publish-repository = Publish repository
publish-repository-caption = Publish this repository to GitHub
publish-branch = Publish branch
publish-branch-caption = Publish this branch to the remote
last-fetched = Last fetched { $when }
never-fetched = Never fetched
please-wait = Please wait…
restore-stash = Restore stash ({ $count })

## Operations
op-fetching = Fetching…
op-pulling = Pulling…
op-pushing = Pushing…
op-publishing = Publishing…
op-cloning = Cloning…
op-working = Working…
done-fetch = Fetched
done-pull = Pulled the latest changes
done-push = Pushed to the remote
done-publish = Published to GitHub
done-commit = Changes committed
done-merge = Merge completed
done-stash = Changes stashed
done-stash-pop = Stashed changes restored
done-revert = Commit reverted
done-cherry-pick = Commit cherry-picked
done-undo = Last commit undone
done-delete-branch = Branch deleted
stash-message = GitHub ReDesktop stash on { $branch }

## Merge
merge-in-progress = A merge is in progress.
merge-conflicts = { $count ->
    [one] There is one conflicted file. Resolve it in your editor, then commit.
   *[other] There are { $count } conflicted files. Resolve them in your editor, then commit.
}
abort-merge = Abort merge
merge-title = Merge into { $branch }
merge-into = Merge into { $branch }
merge-commit = Merge commit

## Branch popup
filter-branches = Filter branches
local-branches = Branches
remote-branches = Remote branches
no-branches = No branches match
current = current
remote = remote

## Changes
changed-files = { $count ->
    [0] No changed files
    [one] 1 changed file
   *[other] { $count } changed files
}
no-local-changes = No local changes
no-local-changes-body = There are no uncommitted changes in { $repo }. Here are some friendly suggestions for what to do next.
summary-placeholder = Summary (required)
description-placeholder = Description
amend-last-commit = Amend last commit
amend-commit = Amend last commit
commit-to = Commit to { $branch }
commit-files-to = { $count ->
    [one] Commit 1 file to { $branch }
   *[other] Commit { $count } files to { $branch }
}
discard-changes = Discard changes…
ignore-file = Ignore file (add to .gitignore)
ignore-extension = Ignore all files with this extension
copy-path = Copy file path
open-file = Open with default program
loading = Loading…
loading-diff = Loading diff…
binary-file = Binary file
binary-file-body = This binary file has changed and cannot be displayed as text.
no-diff = No content changes
no-diff-body = Only the file mode or name changed.
diff-truncated = This diff is very large: only the beginning is shown.

## Suggestions
suggest-publish-body = Publish this repository to GitHub to share it and back it up.
suggest-publish-branch-body = This branch only exists on your computer. Publish it to share it.
suggest-push-title = { $count ->
    [one] You have 1 local commit waiting to be pushed
   *[other] You have { $count } local commits waiting to be pushed
}
suggest-push-body = Push your commits to share them with your collaborators.
suggest-pull-title = { $count ->
    [one] There is 1 new commit on the remote
   *[other] There are { $count } new commits on the remote
}
suggest-pull-body = Pull to bring these changes into your branch.
suggest-editor-title = Open the repository in your editor
suggest-editor-body = Edit your code with your favorite editor.
suggest-files-title = View the files of your repository
suggest-files-body = Browse the repository in your file manager.
suggest-github-title = Open the repository page on GitHub
suggest-github-body = See issues, pull requests and more in your browser.

## History
no-commits = No commits yet
load-more = Load more commits
select-commit = No commit selected
select-commit-body = Select a commit in the list to see its changes.
files-changed = { $count ->
    [one] 1 changed file
   *[other] { $count } changed files
}
undo-commit = Undo commit
revert-commit = Revert changes in commit
cherry-pick = Cherry-pick onto current branch
branch-from-commit = Create branch from commit…
copy-sha = Copy SHA
copy-summary = Copy summary

## Dialogs
cancel = Cancel
choose = Choose…
name = Name
description = Description
optional = Optional
local-path = Local path
clone = Clone
clone-title = Clone a repository
clone-url = Repository URL or GitHub username/repository
clone-url-placeholder = https://github.com/owner/repo.git or owner/repo
clone-sign-in-hint = Sign in to GitHub in the settings to pick one of your repositories from a list.
filter-repositories = Filter your repositories
fork = fork
create-title = Create a new repository
create-repository = Create repository
repo-name-placeholder = my-awesome-project
will-be-created = The repository will be created in { $path }
gitignore = .gitignore
gitignore-placeholder = e.g. target/  node_modules/  *.log
init-readme = Initialize this repository with a README
branch-name-placeholder = my-new-feature
branch-based-on = Your new branch will be based on { $branch }.
branch-based-on-commit = Your new branch will be based on commit { $sha }.
will-be-created-as = Will be created as { $name }
create-branch = Create branch
rename-branch-title = Rename { $branch }
rename = Rename
keep-private = Keep this code private
publish-to = The repository will be created on the { $account } account.
discard-title = { $count ->
    [one] Discard changes to 1 file?
   *[other] Discard changes to { $count } files?
}
discard-body = Changes will be lost. Untracked files are moved to the trash when possible.
and-more = …and { $count } more
delete-branch-title = Delete branch
delete-branch-body = Delete the local branch “{ $branch }”? This cannot be undone.
delete = Delete
remove-repo-title = Remove repository
remove-repo-body = Remove { $path } from the list? The files on disk will not be deleted.
remove = Remove
force-push-title = Force push?
force-push-body = Force pushing rewrites history on the remote. Anyone working on this branch will need to reset their local copy.

## Account & settings
account = GitHub account
sign-in = Sign in
sign-out = Sign out
signed-in-as = Signed in as { $login }
sign-in-body = Paste a personal access token (scopes: repo, read:org, workflow) or reuse your GitHub CLI login. The token is stored in your system keyring.
token-placeholder = ghp_…
use-gh-cli = Use GitHub CLI login
create-token = Create a token
gh-not-found = Could not get a token from the GitHub CLI. Run “gh auth login” first.
sign-in-to-publish = Sign in to GitHub to publish this repository.
git-identity = Git identity
git-name = Name
git-name-placeholder = Your Name
git-email = Email
save = Save
set-identity = Configure your Git name and email…
identity-saved = Git identity saved
integrations = Integrations
editor = External editor
terminal = Terminal
auto-detect = Auto-detect
detected = Detected: { $list }
none-detected = none
clone-dir = Default clone folder
appearance-behavior = Appearance & behavior
theme = Theme
theme-system = Match desktop
theme-dark = Dark
theme-light = Light
auto-fetch = Fetch in the background every 5 minutes
confirm-discard = Confirm before discarding changes

## Welcome
welcome-title = Let's get started!
welcome-body = Add a repository to GitHub ReDesktop to start collaborating.
welcome-clone = Clone a repository from the Internet…
welcome-clone-body = From GitHub.com or any Git URL
welcome-create = Create a new repository on your computer…
welcome-create-body = Start a fresh project with Git
welcome-add = Add an existing repository from your computer…
welcome-add-body = Pick a folder that already contains a Git repository
welcome-sign-in = Sign in to GitHub to see your repositories

## Errors
add-local-title = Choose a local repository
not-a-repository = { $path } is not a Git repository.
already-a-repository = This folder is already a Git repository.
destination-not-empty = The destination { $path } already exists and is not empty.
repository-missing = The repository at { $path } could not be found. It may have been moved or deleted.
no-editor-found = No editor found. Configure one in the settings.
no-terminal-found = No terminal found. Configure one in the settings.

## Time
time-just-now = just now
time-minutes = { $count ->
    [one] 1 minute ago
   *[other] { $count } minutes ago
}
time-hours = { $count ->
    [one] 1 hour ago
   *[other] { $count } hours ago
}
time-days = { $count ->
    [one] 1 day ago
   *[other] { $count } days ago
}
time-months = { $count ->
    [one] 1 month ago
   *[other] { $count } months ago
}
time-years = { $count ->
    [one] 1 year ago
   *[other] { $count } years ago
}
