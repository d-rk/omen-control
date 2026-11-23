#!/bin/sh
# Install the omen-control desktop entry and icon for the current user, so the desktop shows the app's own icon and
# lists it in the application menu.
#
# Usage: scripts/install-desktop-entry.sh [BINARY]    (default: target/release/omen-control)
#        scripts/install-desktop-entry.sh --uninstall
set -eu

repo=$(cd "$(dirname "$0")/.." && pwd)
data="${XDG_DATA_HOME:-$HOME/.local/share}"
apps="$data/applications"
icons="$data/icons/hicolor/scalable/apps"

refresh() {
    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$apps" || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$data/icons/hicolor" 2>/dev/null || true
    command -v kbuildsycoca6 >/dev/null 2>&1 && kbuildsycoca6 >/dev/null 2>&1 || true
}

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$apps/omen-control.desktop" "$icons/omen-control.svg"
    refresh
    echo "Removed the omen-control desktop entry and icon."
    exit 0
fi

bin=${1:-"$repo/target/release/omen-control"}
if [ ! -x "$bin" ]; then
    echo "No executable at $bin. Build it first (cargo build --release) or pass its path." >&2
    exit 1
fi
bin=$(cd "$(dirname "$bin")" && pwd)/$(basename "$bin")

mkdir -p "$apps" "$icons"
cp "$repo/data/omen-control.svg" "$icons/omen-control.svg"
sed "s|^Exec=.*|Exec=$bin|" "$repo/data/omen-control.desktop" > "$apps/omen-control.desktop"
refresh
echo "Installed $apps/omen-control.desktop (runs $bin) and the icon."
