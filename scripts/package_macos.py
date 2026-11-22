#!/usr/bin/env python3
"""Package a built macOS executable with its application icon and metadata."""

import argparse
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("destination", type=Path, nargs="?", default=ROOT / "dist/Abcom.app")
    args = parser.parse_args()
    if not args.binary.is_file():
        parser.error(f"executable not found: {args.binary}")
    if args.destination.suffix != ".app":
        parser.error("destination must end with .app")
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    contents = args.destination / "Contents"
    (contents / "MacOS").mkdir(parents=True, exist_ok=True)
    (contents / "Resources").mkdir(parents=True, exist_ok=True)
    binary = contents / "MacOS/abcom"
    shutil.copyfile(args.binary, binary)
    binary.chmod(0o755)
    shutil.copyfile(ROOT / "assets/app_icon.icns", contents / "Resources/Abcom.icns")
    metadata = {
        "CFBundleIdentifier": "corp.abnd.abcom",
        "CFBundleName": "Abcom",
        "CFBundleDisplayName": "Abcom",
        "CFBundleExecutable": "abcom",
        "CFBundlePackageType": "APPL",
        "CFBundleIconFile": "Abcom.icns",
        "CFBundleShortVersionString": version.split("-")[0],
        "CFBundleVersion": version.split("-")[0],
        "CFBundleGetInfoString": f"Abcom {version}",
        "NSHighResolutionCapable": True,
        "NSPrincipalClass": "NSApplication",
    }
    (contents / "Info.plist").write_bytes(plistlib.dumps(metadata, sort_keys=True))
    if sys.platform == "darwin":
        for attribute in ["com.apple.FinderInfo", "com.apple.ResourceFork"]:
            subprocess.run(["/usr/bin/xattr", "-r", "-d", attribute, str(args.destination)],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
        # The executable's ad-hoc signature is not a valid seal for a new bundle.
        subprocess.run(["/usr/bin/codesign", "--force", "--sign", "-", "--timestamp=none",
                        str(args.destination)], check=True)
        subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict",
                        str(args.destination)], check=True)
    # Expose aussi la mise à jour au niveau du dossier du bundle.
    args.destination.touch()


if __name__ == "__main__":
    main()
