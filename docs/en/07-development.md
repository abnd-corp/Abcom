# 07 — Development

**English** · [Français](../07-developpement.md) · [Documentation](../README.md)

## Setup

Install Rust satisfying `rust-version` in `Cargo.toml` (**1.95** minimum), plus [Linux packages](06-installation.md#linux-build-dependencies) when applicable.

```sh
git clone https://github.com/abnd-corp/Abcom.git
cd Abcom
git config core.hooksPath .githooks
cargo build --locked
```

Optional service configuration is covered by [installation](06-installation.md). Never commit credentials.

Python 3 is required for documentation checks, with no additional Python packages. It is not an Abcom runtime dependency.

## Checks before contributing

```sh
make check
# Documentation only / Documentation seule
make check-docs
```

`make check` runs:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
```

Also check the minimal configuration:

```sh
cargo check --all-targets --no-default-features --locked
cargo test --no-default-features --locked
```

Network tests use real local sockets. A sandbox that prohibits them produces permission failures; distinguish that restriction from an application regression.

The 11 November 2026 local validation on macOS passed **642 tests** with all features and **640** with default features disabled: 634 or 632 library tests, 2 binary tests, 1 P2P integration test and 5 external interaction tests. The KSNI test on a private D-Bus session is reserved for Linux CI; its execution and rendering on an actual Linux desktop remain to be confirmed. The [audit log](08-history-and-audits.md) records the environment and checks not performed. This dated result does not replace rerunning the checks.

## Test organization

Unit tests live in `src/tests/`, connected from their corresponding module:

```rust
#[cfg(test)]
#[path = "../tests/test_app_groups.rs"]
mod tests;
```

New modules get their own test file. A few historical modules retain inline tests. Scenarios in `tests/` use the public API. `bash scripts/integration_test.sh` runs only the authenticated TCP P2P exchange. Process-level discovery and native integrations remain in the [manual checklist](10-testing.md).

## Contribution and workflow

[CONTRIBUTING.md](../../CONTRIBUTING.md) contains the repository's code and commit rules in French. The [Abnd charter](https://github.com/abnd-corp/gouvernance/blob/main/CHARTE.md), also in French, governs organization workflow. These authoritative files are not duplicated or rewritten in the guides.

- Work on a `feature/…` branch; associate a PR with an issue and milestone as required by the charter.
- Propose changes through a PR and follow the organization's promotion flow.
- Commit messages use `type(scope): description` in French, imperative, lowercase after `:` and no final period. Keep one intention per commit.
- Agents never push without explicit approval and never add a `Co-Authored-By` trailer. Merging requires an explicit human instruction and the required checks and reviews.

For documentation changes, update both languages and run the link checker. The [documentation index](../README.md#maintaining-these-guides) explains the translation contract.

## Continuous integration

| Workflow | Purpose |
|---|---|
| [ci-dev.yml](../../.github/workflows/ci-dev.yml) | Formatting, Clippy, release build, Linux tests, MSRV, minimal build, macOS/Windows checks, coverage and dependency audit |
| [ci-main.yml](../../.github/workflows/ci-main.yml) | Delivery checks and P2P scenario |
| [release.yml](../../.github/workflows/release.yml) | Linux x86-64, macOS arm64, Windows x86-64 archives and `SHA256SUMS.txt` on `v*` tags or manual dispatch |
| [dependencies.yml](../../.github/workflows/dependencies.yml) | Dependency monitoring |

The MSRV comes from `rust-version`, not from whichever toolchain is currently stable. Compilation checks do not certify tray behavior, notifications, native pickers or accessibility on all three platforms.

## Dependencies and security

```sh
cargo audit
cargo deny --locked check licenses sources bans
cargo deny --locked check advisories
```

`Cargo.lock` is versioned. Justify new dependencies, enable only the necessary features, and keep unrelated dependency upgrades separate. The [11 November audit](08-history-and-audits.md) records unresolved RustSec advisories; passing application tests does not make that audit pass.

The renderer uses eframe/egui **0.37** with **wgpu**. Do not restore eframe's default features and accidentally add its Glow renderer. On macOS the current graph should not link Glow; on other systems wgpu can include its own GLES fallback dependency.

Release builds use thin LTO, one codegen unit, stripped symbols and `panic = "abort"`. Publish timing and memory measurements with their date and release-build environment.

## Useful scripts

| Script | Purpose |
|---|---|
| [run-multi.sh](../../scripts/run-multi.sh) | Multiple local instances |
| [seed-demo.py](../../scripts/seed-demo.py) | Demo data |
| [integration_test.sh](../../scripts/integration_test.sh) | Headless P2P test |
| [check_docs.py](../../scripts/check_docs.py) | EN/FR pairs, links, anchors and test IDs |
| [build-and-distribute.sh](../../scripts/build-and-distribute.sh) | Local distribution archive |
| [install-windows.ps1](../../scripts/install-windows.ps1) | Windows installation |
| [abcom-install.sh](../../scripts/abcom-install.sh) | Linux binary installation |
| [uninstall.sh](../../scripts/uninstall.sh) | Linux removal |

## Bundled assets

Application code is licensed under [AGPL-3.0-only](../../LICENSE). Fonts and emojis retain their own licenses, recorded in their assets and Settings → Credits. Noto Sans, Noto Sans Symbols 2, Inter and the suggested fonts use OFL-1.1. GNU Unifont offers OFL-1.1 or GPLv2+ with the font embedding exception. Read the applicable notices in [assets/fonts](../../assets/fonts/).

Fallback glyphs cover non-Latin scripts; conversation emojis use embedded PNG artwork. Check coverage tests before changing that chain.
