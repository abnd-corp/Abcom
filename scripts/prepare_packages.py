#!/usr/bin/env python3
"""Generate Homebrew/WinGet recipes from verified release artifacts."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import tomllib
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent


def generate(artifacts, output, base_url, version, tap_output=None):
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", version):
        raise ValueError("expected an Abcom release version")
    parsed = urlsplit(base_url)
    if parsed.scheme not in {"https", "http", "file"} or parsed.username or parsed.password:
        raise ValueError("use an HTTPS, local HTTP or file URL without credentials")
    if parsed.scheme in {"https", "http"} and not parsed.hostname:
        raise ValueError("artifact URLs need a host")
    if parsed.scheme == "file" and parsed.netloc:
        raise ValueError("file URLs must refer to the local machine")
    if parsed.scheme == "http" and parsed.hostname not in {"localhost", "127.0.0.1"}:
        raise ValueError("HTTP is only allowed for local tests")
    if parsed.query or parsed.fragment or any(character in base_url for character in ['"', "'", "\\", "{", "}", "\n", "\r"]):
        raise ValueError("URL characters must be percent-encoded")
    if parsed.scheme == "file" and (artifacts / "abcom-windows-x86_64-setup.exe").is_file():
        raise ValueError("WinGet needs an HTTPS or local HTTP artifact URL, not file://")
    if tap_output is not None:
        if (parsed.scheme != "https" or
                not re.fullmatch(r"https://github.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/releases/download/v" + re.escape(version), base_url)):
            raise ValueError("the remote tap requires a versioned GitHub release URL matching the package version")
        if not (artifacts / "abcom-macos-aarch64.tar.gz").is_file():
            raise ValueError("the remote tap requires the macOS release artifact")
    output.mkdir(parents=True, exist_ok=True)
    generated = []
    for filename, kind in [
        ("abcom-macos-aarch64.tar.gz", "homebrew"),
        ("abcom-windows-x86_64-setup.exe", "winget"),
    ]:
        artifact = artifacts / filename
        if not artifact.is_file():
            continue
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
        url = f"{base_url.rstrip('/')}/{filename}"
        if kind == "homebrew":
            target = output / "homebrew/Casks/abcom.rb"
            text = f'''cask "abcom" do
  version {json.dumps(version)}
  sha256 {json.dumps(digest)}

  url {json.dumps(url)}
  name "Abcom"
  desc "Encrypted peer-to-peer LAN messenger"
  homepage "https://github.com/abnd-corp/Abcom"

  depends_on arch: :arm64

  app "Abcom.app"

  uninstall launchctl: "Abcom",
            quit: "corp.abnd.abcom",
            delete: "~/Library/LaunchAgents/Abcom.plist"
end
'''
        else:
            target = output / "winget/Abnd.Abcom.yaml"
            text = f'''# yaml-language-server: $schema=https://aka.ms/winget-manifest.singleton.1.10.0.schema.json
PackageIdentifier: Abnd.Abcom
PackageVersion: {json.dumps(version)}
PackageLocale: en-US
Publisher: Abnd
PackageName: Abcom
License: AGPL-3.0-or-later
LicenseUrl: https://github.com/abnd-corp/Abcom/blob/main/LICENSE
ShortDescription: Encrypted peer-to-peer LAN messenger
PackageUrl: https://github.com/abnd-corp/Abcom
InstallerType: inno
Scope: user
UpgradeBehavior: install
ProductCode: corp.abnd.abcom_is1
Installers:
  - Architecture: x64
    InstallerUrl: {json.dumps(url)}
    InstallerSha256: {digest.upper()}
ManifestType: singleton
ManifestVersion: 1.10.0
'''
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
        if kind == "homebrew" and tap_output is not None:
            tap_output.mkdir(parents=True, exist_ok=True)
            (tap_output / "abcom.rb").write_text(text, encoding="utf-8")
        generated.append(kind)
    if not generated:
        raise ValueError("no supported artifacts found; build the macOS bundle or Windows installer first")
    return generated


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/packages")
    parser.add_argument("--tap-output", type=Path, help="also write the remote tap cask (versioned GitHub releases only)")
    parser.add_argument("--base-url", help="artifact location; defaults to the local artifact directory")
    parser.add_argument("--version", default=tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"])
    args = parser.parse_args()
    try:
        kinds = generate(args.artifacts, args.output,
                         args.base_url or args.artifacts.resolve().as_uri(), args.version, args.tap_output)
    except ValueError as error:
        parser.error(str(error))
    print("Generated: " + ", ".join(kinds))


if __name__ == "__main__":
    main()
