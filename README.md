**English** · [Français](README.fr.md)

# Abcom — encrypted LAN messenger

[![CI](https://github.com/abnd-corp/Abcom/actions/workflows/ci-dev.yml/badge.svg?branch=dev)](https://github.com/abnd-corp/Abcom/actions/workflows/ci-dev.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

Abcom connects computers on the same local network for private chats, group conversations and file sharing. It runs without a server or an account, with encrypted peer-to-peer connections and history stored on your computer.

Built in Rust with a native egui interface for macOS, Windows and Linux.

- Automatic peer discovery; direct messages and groups.
- Markdown, emojis, reactions, replies and file transfers up to 2 GiB.
- System notifications, tray mode and configurable themes, text and fonts.
- Local messaging works offline. GIFs and the opt-in online font catalogue use Internet services.

## Beta installation

[GitHub releases](https://github.com/abnd-corp/Abcom/releases) · [Homebrew, Nix, WinGet, AppImage and Flatpak](packaging/README.md)

No catalogue registration required. Homebrew will use this repository as a tap after the first release and promotion of its cask to `main`; Nix can already build the published branch. See the guide for each format's commands and limitations.

## Quick start

Install [Rust](https://rustup.rs/), then:

```sh
git clone https://github.com/abnd-corp/Abcom.git
cd Abcom
cargo run --release -- alice
```

Run Abcom on another computer on the same LAN with a different name. Linux builds need [system packages](docs/en/06-installation.md#linux-build-dependencies).

To try two local instances on macOS or Linux: `make run2`.

## Documentation

[Installation](docs/en/06-installation.md) · [User guide](docs/en/05-features.md) · [Troubleshooting and FAQ](docs/en/11-troubleshooting.md) · [All documentation](docs/README.md)

**Beta software.** First-contact identity verification and unencrypted local storage are described in the [security guide](docs/en/03-network-and-security.md). See [known limitations](docs/en/09-limitations.md) and the [changelog](CHANGELOG.md) before deployment.

## Contributing and license

Start with the [development guide](docs/en/07-development.md) and [contribution rules](CONTRIBUTING.md). Report reproducible problems in [GitHub Issues](https://github.com/abnd-corp/Abcom/issues).

Licensed under [GNU AGPL v3](LICENSE). Bundled assets retain their own licenses; see [credits](docs/en/07-development.md#bundled-assets).
