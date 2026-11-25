#!/usr/bin/env bash
# Disk image holding Abcom.app and an Applications link, for drag-and-drop installation.
# Usage: package_dmg.sh dist/Abcom.app abcom-macos-aarch64.dmg
set -euo pipefail
app=$1
out=$2
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
ditto "$app" "$stage/Abcom.app"
ln -s /Applications "$stage/Applications"
hdiutil create -quiet -volname Abcom -srcfolder "$stage" -fs HFS+ -format UDZO -ov "$out"
hdiutil verify -quiet "$out"
echo "$out"
