//! Intégration bureau, par nature spécifique au système : icône résidente,
//! notifications natives et lancement au démarrage.

pub mod autostart;
pub(crate) mod icons;
pub mod notify;
pub mod tray;

pub(crate) fn in_flatpak() -> bool {
    cfg!(target_os = "linux") && std::path::Path::new("/.flatpak-info").is_file()
}
