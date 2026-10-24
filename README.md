# Abcom

[![CI — dev](https://github.com/abnd-corp/abcom/actions/workflows/ci-dev.yml/badge.svg?branch=dev)](https://github.com/abnd-corp/abcom/actions/workflows/ci-dev.yml)
[![CI — main](https://github.com/abnd-corp/abcom/actions/workflows/ci-main.yml/badge.svg?branch=main)](https://github.com/abnd-corp/abcom/actions/workflows/ci-main.yml)
[![Licence AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue.svg)](LICENSE)

English · [Français](README.fr.md)

Instant messenger for local networks, written in Rust. Machines on the same LAN discover
each other automatically and exchange messages, files and media peer-to-peer: no server,
no account, no Internet connection. All traffic is end-to-end encrypted (Noise XX) and
the history is stored locally in SQLite.

Native GUI (egui), designed to run all the time: the window folds into the menu bar /
system tray, the app stays reachable and uses almost nothing when idle.

## Features

- **Conversations**: public "All" channel, private messages, group rooms with member management
- **Rich messages**: Markdown, emojis (picker + `:shortcodes:`), reactions, quoted replies, typing indicator
- **Receipts**: delivered (grey ✓✓) and read (blue ✓✓) in private chats, with automatic retransmission
- **Files and media**: files and folders up to 2 GiB, explicit acceptance above 50 MB, thumbnails and viewer
- **GIFs, memes, stickers**: built-in Klipy picker
- **Security**: X25519 identity per machine, Noise XX encryption, key pinning (TOFU), optional room passphrase
- **Always on**: closing folds into the tray, system notifications, unread badge, autostart at login
- **Bilingual**: French/English interface, light/dark theme
- **Offline**: a message written to an absent peer is sent automatically when it reconnects
- **Local-first**: plain-text export of a conversation and database compaction from Settings

## Quick start

Requirements: stable Rust (`rustup`). Machines must be on the same LAN, with ports
9000-9001/tcp and 9001/udp open between them.

```bash
# Once after cloning: enable the pre-commit hook (cargo fmt)
git config core.hooksPath .githooks

# Run
cargo run --release -- <nickname>

# Try P2P locally: two instances on the same machine
ABCOM_INSTANCE=1 cargo run --release -- alice   # terminal 1
ABCOM_INSTANCE=2 cargo run --release -- bob     # terminal 2
# or: make run2  (shortcut for scripts/run-multi.sh)
```

Without an argument, the nickname defaults to `USER` / `USERNAME`. Installation per
platform (Linux/systemd, Windows, Docker) is described in
[docs/06-installation.md](docs/06-installation.md).

## Documentation

The detailed documentation is in French, in [docs/](docs/): presentation, architecture,
network and security (threat model), storage, features, installation, development,
history and audits, known limits, test plan.

To contribute: [CONTRIBUTING.md](CONTRIBUTING.md) (in French). Changes are tracked in
[CHANGELOG.md](CHANGELOG.md).

## Project status

Version **1.0.0-beta.1**, licensed under the **AGPL-3.0**. 329 automated tests, including
an external headless P2P scenario and a headless render of the whole interface, and
GitHub Actions CI on Linux, macOS and Windows, with MSRV, `cargo audit`, `cargo deny`
and coverage.

Open items are listed in [docs/09 — Limits and next steps](docs/09-limites-et-pistes.md) (in French).
