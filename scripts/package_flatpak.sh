#!/usr/bin/env bash
# Export a local binary bundle; no Flathub submission or remote publication.
set -euo pipefail
binary=${1:?Usage: package_flatpak.sh /path/to/linux/abcom [output-directory]}
output=${2:-dist/linux}
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
binary=$(realpath "$binary")
mkdir -p "$output"
output=$(realpath "$output")
arch=$(flatpak --default-arch)
binary_arch=$(python3 -c 'import sys,runpy,pathlib; print(runpy.run_path(sys.argv[1])["elf_arch"](pathlib.Path(sys.argv[2])))' "$root/scripts/package_appimage.py" "$binary")
[[ "$binary_arch" == "$arch" ]] || { echo "Binary architecture $binary_arch differs from Flatpak $arch" >&2; exit 1; }
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -n1)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/build/files/bin" "$work/build/files/share/applications" \
    "$work/build/files/share/icons/hicolor/512x512/apps" "$work/build/files/share/metainfo"
install -m755 "$binary" "$work/build/files/bin/abcom"
install -m644 "$root/packaging/linux/corp.abnd.abcom.desktop" "$work/build/files/share/applications/corp.abnd.abcom.desktop"
sed -i 's/StartupWMClass=abcom/StartupWMClass=corp.abnd.abcom/' "$work/build/files/share/applications/corp.abnd.abcom.desktop"
install -m644 "$root/assets/app_icon.png" "$work/build/files/share/icons/hicolor/512x512/apps/corp.abnd.abcom.png"
install -m644 "$root/packaging/linux/corp.abnd.abcom.metainfo.xml" "$work/build/files/share/metainfo/corp.abnd.abcom.metainfo.xml"
mkdir -p "$work/build/files/share/licenses/abcom"
install -m644 "$root/LICENSE" "$work/build/files/share/licenses/abcom/LICENSE"
while IFS= read -r license; do
    relative=${license#"$root/assets/fonts/"}
    install -Dm644 "$license" "$work/build/files/share/licenses/abcom/fonts/$relative"
done < <(find "$root/assets/fonts" -name '*.txt' -type f)
cat > "$work/build/metadata" <<EOF
[Application]
name=corp.abnd.abcom
runtime=org.freedesktop.Platform/$arch/25.08
sdk=org.freedesktop.Sdk/$arch/25.08
command=abcom
EOF
# Keep X11 available on Wayland too, for the existing winit CPU workaround.
flatpak build-finish --share=network --share=ipc --socket=x11 --socket=wayland \
    --socket=pulseaudio --device=dri \
    --talk-name=org.kde.StatusNotifierWatcher \
    --talk-name=org.freedesktop.Notifications "$work/build"
flatpak build-export "$work/repo" "$work/build" stable
flatpak build-bundle "$work/repo" "$output/Abcom-$version-$arch.flatpak" corp.abnd.abcom stable \
    --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo
printf '%s\n' "$output/Abcom-$version-$arch.flatpak"
