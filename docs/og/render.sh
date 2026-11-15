#!/usr/bin/env bash
# Renders docs/og/card.html into site/og.png (1200×630 social card).
# Requires chromium (e.g. `nix-shell -p chromium --run docs/og/render.sh`).
set -euo pipefail
cd "$(dirname "$0")"
chromium --headless=new --no-sandbox --disable-gpu --hide-scrollbars \
  --force-device-scale-factor=1 --window-size=1200,630 \
  --virtual-time-budget=4000 --screenshot=../../site/og.png "file://$PWD/card.html"
