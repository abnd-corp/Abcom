#!/usr/bin/env bash
# Use a disposable user installation; never alter the user's Flatpak remotes/data.
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Run this installation test only in disposable CI or a disposable container' >&2; exit 1; }
bundle=$(realpath "${1:?Usage: test_linux_flatpak.sh /path/to/Abcom.flatpak}")
work=$(mktemp -d)
pid=
cleanup() {
    if [[ -n "$pid" ]]; then
        kill -TERM -- "-$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi
    rm -rf "$work"
}
trap cleanup EXIT
export FLATPAK_USER_DIR="${ABCOM_FLATPAK_TEST_INSTALLATION:-$work/installation}"
export XDG_DATA_HOME="$work/data" XDG_CONFIG_HOME="$work/config" XDG_CACHE_HOME="$work/cache"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-$work/runtime}"
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y --no-related flathub org.freedesktop.Platform//25.08 org.freedesktop.Platform.GL.default//25.08
flatpak install --user -y --no-deps --no-related "$bundle"
flatpak info --user --show-permissions corp.abnd.abcom
cd "$work"
setsid flatpak run --user corp.abnd.abcom packaging-smoke > app.log 2>&1 &
pid=$!
for attempt in $(seq 1 80); do
    if ! kill -0 "$pid" 2>/dev/null; then
        cat app.log >&2
        exit 1
    fi
    if xwininfo -root -tree | grep -q '"Abcom"'; then
        sleep 2
        if ! kill -0 "$pid" 2>/dev/null; then cat app.log >&2; exit 1; fi
        window=$(xwininfo -root -tree | awk '/"Abcom"/{print $1; exit}')
        xprop -id "$window" WM_CLASS | grep -F 'corp.abnd.abcom'
        flatpak run --user --command=sh corp.abnd.abcom -c 'test ! -e "$XDG_CONFIG_HOME/autostart/Abcom.desktop"'
        flatpak kill corp.abnd.abcom
        wait "$pid" 2>/dev/null || true
        pid=
        flatpak uninstall --user -y corp.abnd.abcom
        echo 'Flatpak: installed, native Linux window observed, uninstalled'
        exit 0
    fi
    sleep 0.25
done
cat app.log >&2
echo 'Flatpak did not create its window' >&2
exit 1
