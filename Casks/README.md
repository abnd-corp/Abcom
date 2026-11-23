# Abcom Homebrew tap

Casks are generated from published GitHub release artifacts, with a pinned version,
URL and SHA-256. Each release proposes `abcom.rb` through a PR to `dev`; once that
PR is promoted to `main`, the commands in the [installation guide](../packaging/README.md#homebrew-on-macos)
install that version from this repository. No Homebrew catalogue submission is required.

The application is not signed or notarized, by decision ([ADR 0002](../docs/adr/0002-no-code-signing.md)):
authorize it once in System Settings → Privacy & Security after the first launch attempt.

See the [French guide](../packaging/README.fr.md#homebrew-sur-macos).
