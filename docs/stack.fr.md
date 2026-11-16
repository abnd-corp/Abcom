[English](stack.md) · **Français**

# Stack et entretien des dépendances

`Cargo.lock` contient les versions exactes résolues ; `Cargo.toml` fixe les dépendances directes et fonctionnalités. Lire les notes amont avant de modifier ces contraintes.

| Composant | Contrainte actuelle | Changements amont |
| --- | --- | --- |
| Rust | 1.95 minimum | [Notes de version](https://doc.rust-lang.org/stable/releases.html) |
| egui / eframe | 0.37.0 | [Changelog](https://github.com/emilk/egui/blob/main/CHANGELOG.md) |
| Tokio | 1.54.1 | [Changelog](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md) |
| rusqlite | 0.42.0, SQLite embarqué | [Changelog](https://github.com/rusqlite/rusqlite/blob/master/CHANGELOG.md) |
| tray-icon | 0.26.1, backend KSNI | [Versions](https://github.com/tauri-apps/tray-icon/releases) |
| Accessibilité Linux / D-Bus | AccessKit, AT-SPI et KSNI portés vers zbus 6 | [Correctifs locaux et provenance](../vendor/README.fr.md) |

La décision D-Bus et les conditions de retrait figurent dans [l'ADR 0001](adr/0001-dbus-maintained.fr.md). Configuration et résultats des VM restent dans le workspace partagé Test-local, hors de ce dépôt.
