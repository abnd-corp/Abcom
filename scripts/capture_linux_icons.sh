#!/usr/bin/env bash
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Use a disposable CI runner or container' >&2; exit 1; }
binary=$(realpath "${1:?Usage: capture_linux_icons.sh binary output-directory [--dark] [--unread]}")
output=${2:?Output directory required}
mkdir -p "$output"
output=$(realpath "$output")
shift 2
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export HOME="$work/home" XDG_CONFIG_HOME="$work/home/.config" XDG_RUNTIME_DIR="$work/runtime"
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
xfconf-query -c xfce4-panel -p /panels --create --force-array -t int -s 1
xfconf-query -c xfce4-panel -p /panels/panel-1/position --create -t string -s 'p=6;x=640;y=20'
xfconf-query -c xfce4-panel -p /panels/panel-1/size --create -t uint -s 36
xfconf-query -c xfce4-panel -p /panels/panel-1/length --create -t uint -s 100
xfconf-query -c xfce4-panel -p /panels/panel-1/plugin-ids --create --force-array -t int -s 1 -t int -s 2
xfconf-query -c xfce4-panel -p /plugins/plugin-1 --create -t string -s tasklist
xfconf-query -c xfce4-panel -p /plugins/plugin-2 --create -t string -s systray
xfconf-query -c xfce4-panel -p /plugins/plugin-2/hide-new-items --create -t bool -s false
xfsettingsd --daemon > "$output/settings.log" 2>&1 &
xfwm4 --replace --compositor=off > "$output/wm.log" 2>&1 &
xfce4-panel --disable-wm-check > "$output/panel.log" 2>&1 &
sleep 3
"$binary" "$@" > "$output/probe.log" 2>&1 &
pid=$!
sleep 4
kill -0 "$pid"
xwininfo -root -tree > "$output/windows.txt"
grep -q '"Abcom native icons"' "$output/windows.txt"
scrot "$output/${CAPTURE_NAME:-linux}.png"
wait "$pid"
