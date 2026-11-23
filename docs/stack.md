**English** · [Français](stack.fr.md)

# Stack and upstream maintenance

The exact resolved versions are in `Cargo.lock`; direct requirements and features are in `Cargo.toml`. Read upstream release notes before changing these requirements.

| Component | Current requirement | Upstream changes |
| --- | --- | --- |
| Rust | 1.95 minimum | [Release notes](https://doc.rust-lang.org/stable/releases.html) |
| egui / eframe | 0.37.0 | [Changelog](https://github.com/emilk/egui/blob/main/CHANGELOG.md) |
| Tokio | 1.54.1 | [Changelog](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md) |
| rusqlite | 0.42.0, bundled SQLite | [Changelog](https://github.com/rusqlite/rusqlite/blob/master/CHANGELOG.md) |
| tray-icon | 0.26.1, KSNI backend | [Releases](https://github.com/tauri-apps/tray-icon/releases) |
| Linux accessibility / D-Bus | AccessKit, AT-SPI and KSNI ported to zbus 6 | [Local patches and provenance](../vendor/README.md) |

The D-Bus maintenance decision and removal conditions are recorded in [ADR 0001](adr/0001-dbus-maintained.md). General VM configuration and results belong to the shared Test-local workspace, not this repository.
