# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/) and the project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-15

First release of the Rust + libcosmic rewrite.

### Added
- Repository sidebar: add local repositories, clone (URL, `owner/repo` or from your GitHub list), create new repositories.
- Changes view: file list with status badges, per-file selection, colored diff, commit summary/description, amend.
- History view: paginated commits, refs, unpushed markers, commit details and diffs.
- Branches: picker with search, create/switch/rename/delete, remote branches, merge, abort merge, conflict banner.
- Sync: fetch/pull/push/publish button with ahead/behind counters, background fetch, force push with lease.
- GitHub integration: sign in with a token or the `gh` CLI, keyring storage, publish repository, open on GitHub.
- Commit actions: undo, revert, cherry-pick, branch from commit, copy SHA/summary.
- File actions: discard (to trash), ignore file/extension, copy path, reveal, open.
- Stash and restore, open in editor/terminal/file manager with auto-detection.
- Dark/light/system themes, English and French translations, keyboard shortcuts.
- Packaging: `.tar.gz`, `.deb`, `.rpm` for x86_64 and aarch64; Nix flake with package, overlay, NixOS and Home Manager modules.
