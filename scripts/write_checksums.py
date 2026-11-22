#!/usr/bin/env python3
"""Write SHA256SUMS.txt for artifacts, excluding the checksum file itself."""

import argparse
import hashlib
from pathlib import Path


def write_checksums(directory):
    lines = []
    for path in sorted(directory.iterdir()):
        if path.name == "SHA256SUMS.txt" or not path.is_file() or path.is_symlink():
            continue
        if any(character in path.name for character in ["\n", "\r", "\\"]):
            raise ValueError("artifact filenames must not contain line breaks or backslashes")
        digest = hashlib.sha256()
        with path.open("rb") as artifact:
            for block in iter(lambda: artifact.read(1024 * 1024), b""):
                digest.update(block)
        lines.append(f"{digest.hexdigest()}  {path.name}\n")
    if not lines:
        raise ValueError("no artifacts found")
    (directory / "SHA256SUMS.txt").write_text("".join(lines), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    write_checksums(args.directory)


if __name__ == "__main__":
    main()
