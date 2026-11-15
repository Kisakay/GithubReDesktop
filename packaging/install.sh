#!/usr/bin/env sh
# Installs GitHub ReDesktop from the extracted tarball.
#   ./install.sh            -> installs into ~/.local
#   sudo ./install.sh /usr  -> installs system-wide
set -eu
PREFIX="${1:-$HOME/.local}"
HERE="$(cd "$(dirname "$0")" && pwd)"
APPID=io.github.kisakay.GithubReDesktop

install -Dm0755 "$HERE/bin/github-redesktop" "$PREFIX/bin/github-redesktop"
install -Dm0644 "$HERE/share/applications/$APPID.desktop" "$PREFIX/share/applications/$APPID.desktop"
install -Dm0644 "$HERE/share/metainfo/$APPID.metainfo.xml" "$PREFIX/share/metainfo/$APPID.metainfo.xml"
install -Dm0644 "$HERE/share/icons/hicolor/scalable/apps/$APPID.svg" "$PREFIX/share/icons/hicolor/scalable/apps/$APPID.svg"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q "$PREFIX/share/applications" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$PREFIX/share/icons/hicolor" || true

echo "GitHub ReDesktop installed in $PREFIX"
case ":$PATH:" in *":$PREFIX/bin:"*) ;; *) echo "Note: add $PREFIX/bin to your PATH." ;; esac
