"""Check public sharing cards and the multi-resolution Windows icon container."""

from pathlib import Path
import struct
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BrandingTests(unittest.TestCase):
    def test_sharing_cards_have_the_documented_dimensions_and_size(self):
        for name in ["social-preview.png", "social-preview-fr.png"]:
            with self.subTest(name=name):
                data = (ROOT / "assets/branding" / name).read_bytes()
                self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
                self.assertEqual(struct.unpack(">II", data[16:24]), (1280, 640))
                self.assertLess(len(data), 1_000_000)

    def test_windows_icon_contains_small_and_high_dpi_images(self):
        data = (ROOT / "assets/app_icon.ico").read_bytes()
        reserved, kind, count = struct.unpack_from("<HHH", data)
        self.assertEqual((reserved, kind), (0, 1))
        sizes = set()
        for index in range(count):
            width, height, _, _, _, _, size, offset = struct.unpack_from("<BBBBHHII", data, 6 + 16 * index)
            width, height = width or 256, height or 256
            self.assertEqual(width, height)
            self.assertGreater(size, 0)
            self.assertGreaterEqual(offset, 6 + 16 * count)
            self.assertLessEqual(offset + size, len(data))
            sizes.add(width)
        self.assertTrue({16, 24, 32, 48, 64, 128, 256}.issubset(sizes))
