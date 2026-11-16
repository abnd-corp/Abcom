#!/usr/bin/env python3
"""Test local Linux accessibility adapters with the application's Cargo patches."""

import argparse
import json
import os
from pathlib import Path
import shutil
import re
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ksni", action="store_true", help="also test KSNI on a private D-Bus session")
    args = parser.parse_args()
    if args.ksni and not os.environ.get("DBUS_SESSION_BUS_ADDRESS"):
        parser.error("use dbus-run-session for KSNI tests")
    patches = tomllib.loads((ROOT / "Cargo.toml").read_text())["patch"]["crates-io"]
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    env.setdefault("CARGO_BUILD_JOBS", "2")
    cases = [("atspi-common", []), ("accesskit_atspi_common", ["--all-features"]),
             ("accesskit_unix", []),
             ("accesskit_unix", ["--no-default-features", "--features", "tokio"])]
    if args.ksni:
        cases.append(("ksni", ["--no-default-features", "--features", "async-io,blocking"]))
    with tempfile.TemporaryDirectory(prefix="abcom-dbus-tests-") as directory:
        temporary = Path(directory)
        config = temporary / "patches.toml"
        config.write_text("[patch.crates-io]\n" + "".join(
            f"{name} = {{ path = {json.dumps(str(ROOT / entry['path']))} }}\n"
            for name, entry in patches.items()))
        for index, (name, features) in enumerate(cases):
            source = temporary / f"{index}-{name}"
            shutil.copytree(ROOT / patches[name]["path"], source,
                            ignore=shutil.ignore_patterns("target", "Cargo.lock"))
            if name == "ksni":
                manifest = source / "Cargo.toml"
                text = manifest.read_text()
                # GTK and portals belong to upstream examples, not these library tests.
                for dependency in ("gtk4", "ashpd", "chrono"):
                    text = re.sub(rf"\[dev-dependencies\.{dependency}\]\n[^\[]*", "", text)
                manifest.write_text(text)
            subprocess.run(["cargo", "test", "--manifest-path", str(source / "Cargo.toml"),
                            "--config", str(config), "--lib", *features],
                           env=env, check=True)


if __name__ == "__main__":
    main()
