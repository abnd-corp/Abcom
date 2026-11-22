#!/usr/bin/env python3
"""Build application PNG/ICO and the macOS ICNS from the exported artwork."""

from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
PNG = ROOT / "assets/branding/png"


def main():
    shutil.copyfile(PNG / "abcom-app-light-512.png", ROOT / "assets/app_icon.png")
    sizes = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 128, 256]
    images = [(PNG / f"abcom-app-light-{size}.png").read_bytes() for size in sizes]
    offset = 6 + 16 * len(images)
    directory = bytearray(struct.pack("<HHH", 0, 1, len(images)))
    for size, data in zip(sizes, images):
        assert struct.unpack(">II", data[16:24]) == (size, size)
        dimension = size if size < 256 else 0
        directory.extend(struct.pack("<BBBBHHII", dimension, dimension, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    (ROOT / "assets/app_icon.ico").write_bytes(bytes(directory) + b"".join(images))
    if sys.platform == "darwin":
        with tempfile.TemporaryDirectory(prefix="abcom-icons-") as temporary:
            iconset = Path(temporary) / "Abcom.iconset"
            iconset.mkdir()
            for size in [16, 32, 128, 256, 512]:
                for scale in [1, 2]:
                    suffix = "@2x" if scale == 2 else ""
                    shutil.copyfile(
                        PNG / f"abcom-dock-{size * scale}.png",
                        iconset / f"icon_{size}x{size}{suffix}.png",
                    )
            subprocess.run(["iconutil", "-c", "icns", "-o", str(ROOT / "assets/app_icon.icns"), str(iconset)], check=True)


if __name__ == "__main__":
    main()
