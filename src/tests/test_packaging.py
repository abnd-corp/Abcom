"""Recipe checks without Homebrew, WinGet, network access or real installers."""

import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("prepare_packages", ROOT / "scripts/prepare_packages.py")
PACKAGES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGES)


class RecipeTests(unittest.TestCase):
    def test_both_recipes_pin_the_actual_artifact_bytes_and_keep_user_data(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifacts, output = root / "artifacts", root / "output"
            artifacts.mkdir()
            for name in ["abcom-macos-aarch64.tar.gz", "abcom-windows-x86_64-setup.exe"]:
                (artifacts / name).write_bytes(name.encode())
            self.assertEqual(PACKAGES.generate(artifacts, output, "https://example.org/v1", "1.0.0-beta.1"), ["homebrew", "winget"])
            cask = (output / "homebrew/Casks/abcom.rb").read_text()
            manifest = (output / "winget/Abnd.Abcom.yaml").read_text()
            self.assertIn(hashlib.sha256((artifacts / "abcom-macos-aarch64.tar.gz").read_bytes()).hexdigest(), cask)
            self.assertIn(hashlib.sha256((artifacts / "abcom-windows-x86_64-setup.exe").read_bytes()).hexdigest().upper(), manifest)
            self.assertIn('app "Abcom.app"', cask)
            self.assertIn("InstallerType: inno", manifest)
            self.assertNotIn("Application Support", cask)
            self.assertNotIn("zap", cask)

    def test_missing_artifacts_do_not_produce_a_fictitious_package(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ValueError, "no supported artifacts"):
                PACKAGES.generate(root, root / "out", "https://example.org", "1.0.0")

    def test_untrusted_urls_and_versions_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for url, version in [
                ("https://example.org/#{exit}", "1.0.0"),
                ("https://user:password@example.org", "1.0.0"),
                ("http://example.org", "1.0.0"),
                ("https://example.org", '1.0.0"; exit'),
            ]:
                with self.assertRaises(ValueError):
                    PACKAGES.generate(root, root / "out", url, version)

    def test_a_windows_installer_needs_a_downloadable_url(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "abcom-windows-x86_64-setup.exe").write_bytes(b"fixture")
            with self.assertRaisesRegex(ValueError, "WinGet needs"):
                PACKAGES.generate(root, root / "out", root.as_uri(), "1.0.0")


if __name__ == "__main__":
    unittest.main()
