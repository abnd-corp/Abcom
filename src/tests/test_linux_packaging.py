import hashlib
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("package_appimage", ROOT / "scripts/package_appimage.py")
PACKAGES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGES)


class LinuxRecipeTests(unittest.TestCase):
    def test_checksum_manifest_never_hashes_itself(self):
        spec = importlib.util.spec_from_file_location("write_checksums", ROOT / "scripts/write_checksums.py")
        checksums = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(checksums)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "artifact.flatpak").write_bytes(b"artifact")
            (root / "SHA256SUMS.txt").write_text("stale checksums")
            checksums.write_checksums(root)
            expected = hashlib.sha256(b"artifact").hexdigest() + "  artifact.flatpak\n"
            self.assertEqual((root / "SHA256SUMS.txt").read_text(), expected)

    def test_architectures_are_read_from_elf_not_the_filename(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "anything"
            for machine, expected in [(62, "x86_64"), (183, "aarch64")]:
                header = bytearray(20)
                header[:6] = b"\x7fELF\x02\x01"
                struct.pack_into("<H", header, 18, machine)
                path.write_bytes(header)
                self.assertEqual(PACKAGES.elf_arch(path), expected)
            path.write_bytes(b"Mach-O or truncated ELF")
            with self.assertRaises(ValueError):
                PACKAGES.elf_arch(path)

    def test_cached_tools_with_wrong_checksums_are_never_executed(self):
        with tempfile.TemporaryDirectory() as directory:
            tool = Path(directory) / "tool"
            tool.write_bytes(b"modified tool")
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                PACKAGES.fetch_tool({"url": "https://example.invalid", "sha256": hashlib.sha256(b"expected").hexdigest()}, tool)


if __name__ == "__main__":
    unittest.main()
