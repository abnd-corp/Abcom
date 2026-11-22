**English** · [Français](0002-no-code-signing.fr.md)

# No code signing for released binaries

Status: accepted, 2026-11-22 (issue #91).

Signing that operating systems trust is a paid service. A macOS Developer ID and its notarization need the Apple Developer Program. A Windows Authenticode signature needs a commercial certificate, and SmartScreen reputation favours expensive EV certificates. Both add a renewal and secrets that the release CI would have to hold. Abnd uses free tools (charter R29), and Abcom is a free, local-first application distributed from its GitHub releases.

Abcom binaries are **not signed and will not be**, on any platform. The macOS bundle keeps its ad-hoc signature only because Apple silicon refuses to run unsigned code at all. An ad-hoc signature proves the bundle is intact, not who published it.

Integrity comes from GitHub instead: each release publishes `SHA256SUMS.txt` over HTTPS next to the files, the CI builds them from the tagged sources (R41), and Nix rebuilds the same tag from source. The README explains how to verify downloads and how to authorize the first launch on macOS (Privacy & Security) and on Windows (SmartScreen), without turning off either protection globally.

Consequences:

- Gatekeeper and SmartScreen warn on first launch. This is expected and documented.
- Features that require a signed macOS bundle, such as reliable notification action buttons, stay out of scope.
- Distribution goes through the project's own Homebrew tap, a WinGet manifest attached to each release, and the Nix flake. Official catalogues that require signed applications are not targeted.

Revisit only by a new ADR, for example if a free, OS-trusted signing service becomes available to open-source projects.
