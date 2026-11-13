#!/usr/bin/env python3
"""Package a Linux ELF binary, its dlopen libraries and pinned AppImage tools."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parent.parent


def elf_arch(path):
    with path.open("rb") as binary:
        header = binary.read(20)
    if len(header) != 20 or header[:6] != b"\x7fELF\x02\x01":
        raise ValueError("a 64-bit little-endian Linux ELF binary is required")
    arch = {62: "x86_64", 183: "aarch64"}.get(struct.unpack_from("<H", header, 18)[0])
    if arch is None:
        raise ValueError("unsupported ELF architecture")
    return arch


def fetch_tool(entry, target):
    if not target.exists():
        target.parent.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(entry["url"]) as response, target.open("wb") as output:
            shutil.copyfileobj(response, output)
    if hashlib.sha256(target.read_bytes()).hexdigest() != entry["sha256"]:
        raise ValueError(f"tool checksum mismatch: {target.name}; review the official asset before updating the pin")
    target.chmod(0o755)


def extract_tool(path, directory):
    directory.mkdir()
    subprocess.run([str(path), "--appimage-extract"], cwd=directory,
                   stdout=subprocess.DEVNULL, check=True)
    return directory / "squashfs-root/AppRun"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/linux")
    parser.add_argument("--tools", type=Path, default=ROOT / "dist/appimage-tools")
    args = parser.parse_args()
    arch = elf_arch(args.binary)
    if platform.system() != "Linux" or platform.machine() != arch:
        parser.error("run on Linux with the same architecture as the binary")
    version = re.search(r'(?m)^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text())[1]
    args.output.mkdir(parents=True, exist_ok=True)
    tools = json.loads((ROOT / "packaging/linux/appimage-tools.json").read_text())[arch]
    paths = {}
    for name, entry in tools.items():
        paths[name] = args.tools.resolve() / f"{name}-{arch}"
        fetch_tool(entry, paths[name])
    with tempfile.TemporaryDirectory(prefix="abcom-appimage-") as temporary:
        work = Path(temporary)
        appdir = work / "Abcom.AppDir"
        (appdir / "usr/bin").mkdir(parents=True)
        shutil.copyfile(args.binary, appdir / "usr/bin/abcom")
        (appdir / "usr/bin/abcom").chmod(0o755)
        shutil.copyfile(ROOT / "packaging/linux/AppRun", appdir / "AppRun")
        (appdir / "AppRun").chmod(0o755)
        desktop = ROOT / "packaging/linux/corp.abnd.abcom.desktop"
        icon = ROOT / "assets/app_icon.png"
        metainfo = appdir / "usr/share/metainfo"
        metainfo.mkdir(parents=True)
        shutil.copyfile(ROOT / "packaging/linux/corp.abnd.abcom.metainfo.xml", metainfo / "corp.abnd.abcom.metainfo.xml")
        licenses = appdir / "usr/share/licenses/abcom"
        licenses.mkdir(parents=True)
        shutil.copyfile(ROOT / "LICENSE", licenses / "LICENSE")
        for license_file in (ROOT / "assets/fonts").rglob("*.txt"):
            destination = licenses / "fonts" / license_file.relative_to(ROOT / "assets/fonts")
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(license_file, destination)
        notices = licenses / "build-environment"
        notices.mkdir()
        for copyright_file in Path("/usr/share/doc").glob("lib*/copyright"):
            shutil.copyfile(copyright_file, notices / (copyright_file.parent.name + ".copyright"))
        versions = subprocess.check_output(["dpkg-query", "-W", "-f=${binary:Package}\t${Version}\t${source:Package}\t${source:Version}\n", "lib*"], text=True)
        (notices / "library-versions.tsv").write_text(versions)
        deploy = extract_tool(paths["linuxdeploy"], work / "deploy")
        maker = extract_tool(paths["appimagetool"], work / "maker")
        command = [str(deploy), "--appdir", str(appdir), "--executable", str(appdir / "usr/bin/abcom"),
                   "--desktop-file", str(desktop), "--icon-file", str(icon), "--icon-filename", "corp.abnd.abcom"]
        cache = subprocess.check_output(["ldconfig", "-p"], text=True)
        # winit loads these libraries at runtime, so ldd alone cannot discover them.
        for name in ["libxkbcommon.so.0", "libxkbcommon-x11.so.0", "libwayland-client.so.0", "libwayland-cursor.so.0",
                     "libwayland-egl.so.1", "libX11.so.6", "libXcursor.so.1", "libXi.so.6",
                     "libXrandr.so.2", "libxcb.so.1", "libvulkan.so.1"]:
            matches = []
            for line in cache.splitlines():
                if not line.strip().startswith(name + " "):
                    continue
                candidate = Path(line.rsplit(" => ", 1)[1])
                try:
                    if elf_arch(candidate) == arch:
                        matches.append(str(candidate))
                except (OSError, ValueError):
                    continue
            if not matches:
                raise ValueError(f"missing build-host runtime library: {name}")
            command += ["--library", matches[0]]
        pulse_plugins = []
        for candidate in Path("/usr/lib").glob("*/alsa-lib/libasound_module_pcm_pulse.so"):
            try:
                if elf_arch(candidate) == arch:
                    pulse_plugins.append(candidate)
            except (OSError, ValueError):
                continue
        if not pulse_plugins:
            raise ValueError("missing ALSA PulseAudio plugin; install libasound2-plugins")
        command += ["--library", str(pulse_plugins[0])]
        alsa = appdir / "usr/share/alsa"
        alsa.mkdir(parents=True)
        (alsa / "abcom.conf").write_text('pcm.!default { type pulse }\nctl.!default { type pulse }\n')
        subprocess.run(command, check=True)
        os.environ["ARCH"] = arch
        os.environ["VERSION"] = version
        output = args.output.resolve() / f"Abcom-{version}-{arch}.AppImage"
        subprocess.run([str(maker), "--runtime-file", str(paths["runtime"]), str(appdir), str(output)], check=True)
        print(output)


if __name__ == "__main__":
    main()
