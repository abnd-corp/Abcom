//! Intégration bureau, par nature spécifique au système : icône résidente,
//! notifications natives et lancement au démarrage.

pub mod autostart;
pub(crate) mod icons;
pub mod notify;
pub mod tray;
