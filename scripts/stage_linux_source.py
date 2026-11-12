#!/usr/bin/env python3
"""Copy only declared, public build inputs into a new isolated work directory."""

import argparse
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    args.destination.mkdir(parents=True, exist_ok=False)
    tracked = subprocess.check_output(["git", "ls-files", "-z", "--", "src", "assets", "examples"], cwd=ROOT).decode().split("\0")
    names = {name for name in tracked if name and (name.startswith("assets/") or name.endswith(".rs"))
             and not any(part.startswith(".env") or part.endswith((".key", ".pem", ".p12")) for part in Path(name).parts)}
    names.update(["Cargo.toml", "Cargo.lock", "build.rs", "LICENSE",
                  "scripts/package_appimage.py", "scripts/package_flatpak.sh", "scripts/write_checksums.py"])
    names.update("packaging/linux/" + name for name in [
        "AppRun", "Dockerfile", "appimage-tools.json",
        "corp.abnd.abcom.desktop", "corp.abnd.abcom.metainfo.xml",
    ])
    for name in sorted(names):
        target = args.destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / name, target)
    print(f"Staged {len(names)} public build inputs; no .env, .git, user data or target directory")


if __name__ == "__main__":
    main()
