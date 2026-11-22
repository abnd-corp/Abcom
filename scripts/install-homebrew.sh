#!/usr/bin/env bash
# Install a generated cask through a local tap, without a public catalogue.
set -euo pipefail

recipe=${1:?Usage: bash install-homebrew.sh /path/to/Casks/abcom.rb [--local-build]}
command -v brew >/dev/null || { echo 'Homebrew is required: https://brew.sh' >&2; exit 1; }
test -f "$recipe" || { echo "Missing cask: $recipe" >&2; exit 1; }
local_build=false
if [[ ${2:-} == --local-build ]]; then
    if ! /usr/bin/grep -q '^  url "file:///' "$recipe"; then
        echo '--local-build requires a local file URL, never a downloaded release.' >&2
        exit 1
    fi
    echo 'Installing a locally built archive without download quarantine; system security settings remain unchanged.'
    local_build=true
elif [[ $# -gt 1 ]]; then
    echo "Unknown option: $2" >&2
    exit 1
fi
tap=abnd-corp/abcom-local
tap_dir=$(brew --repository "$tap")
if ! test -d "$tap_dir"; then
    brew tap-new --no-git "$tap"
fi
mkdir -p "$tap_dir/Casks"
cp "$recipe" "$tap_dir/Casks/abcom.rb"
if brew list --cask --versions abcom >/dev/null 2>&1; then
    brew reinstall --cask "$tap/abcom"
else
    brew install --cask "$tap/abcom"
fi
if "$local_build"; then
    caskroom=$(brew --caskroom abcom)
    found=false
    for link in "$caskroom"/*/Abcom.app; do
        if [[ -L "$link" ]]; then
            app=$(readlink "$link")
            [[ "$app" == /* ]] || { echo 'Expected an absolute Homebrew app link' >&2; exit 1; }
            [[ $(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$app/Contents/Info.plist") == corp.abnd.abcom ]]
            /usr/bin/codesign --verify --deep --strict "$app"
            /usr/bin/xattr -r -d com.apple.quarantine "$app"
            found=true
        fi
    done
    "$found" || { echo 'Installed application link not found' >&2; exit 1; }
fi
