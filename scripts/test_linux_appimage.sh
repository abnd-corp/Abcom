#!/usr/bin/env bash
# Run only in disposable CI/container: auto-launch writes the user's autostart file.
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Run this test only in disposable CI or a disposable container' >&2; exit 1; }
image=$(realpath "${1:?Usage: test_linux_appimage.sh /path/to/Abcom.AppImage}")
work=$(mktemp -d)
pid=
cleanup() {
    if [[ -n "$pid" ]]; then
        kill -TERM -- "-$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi
    startup="$HOME/.config/autostart/Abcom.desktop"
    if [[ -f "$startup" ]] && grep -F "$image" "$startup" >/dev/null; then rm -f "$startup"; fi
    rm -rf "$work"
}
trap cleanup EXIT
export XDG_DATA_HOME="$work/data" XDG_CONFIG_HOME="$work/config" XDG_CACHE_HOME="$work/cache"
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
cd "$work"
APPIMAGE_EXTRACT_AND_RUN=1 setsid "$image" packaging-smoke > app.log 2>&1 &
pid=$!
for attempt in $(seq 1 60); do
    if ! kill -0 "$pid" 2>/dev/null; then
        cat app.log >&2
        exit 1
    fi
    if xwininfo -root -tree | grep -q '"Abcom"'; then
        sleep 2
        if ! kill -0 "$pid" 2>/dev/null; then cat app.log >&2; exit 1; fi
        grep -F "$image" "$HOME/.config/autostart/Abcom.desktop" >/dev/null
        window=$(xwininfo -root -tree | awk '/"Abcom"/ && !id { id=$1 } END { print id }')
        xprop -id "$window" _NET_WM_ICON > window-icon.txt
        grep -q 'CARDINAL)' window-icon.txt
        echo 'AppImage: native Linux window and EWMH application icon observed, no FUSE required'
        exit 0
    fi
    sleep 0.25
done
cat app.log >&2
echo 'AppImage did not create its window' >&2
exit 1
