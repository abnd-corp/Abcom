//! Thème clair/sombre du bureau Linux.
//!
//! winit ne remonte pas le thème du système sous Linux : egui retombait alors sur le
//! sombre, même sur un bureau clair. On lit la préférence publiée par le portail XDG
//! (`org.freedesktop.appearance color-scheme`, utilisée par GNOME et KDE), puis, à
//! défaut, les réglages KDE, XFCE et GTK. Sous macOS et Windows, winit s'en charge.

#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::sync::atomic::{AtomicU8, Ordering};

/// Clair/sombre détecté ; `None` tant qu'aucune détection n'a abouti.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTheme {
    Light,
    Dark,
}

const UNKNOWN: u8 = 0;
const LIGHT: u8 = 1;
const DARK: u8 = 2;

static CURRENT: AtomicU8 = AtomicU8::new(UNKNOWN);

/// Dernier thème détecté par la tâche de fond (toujours `None` hors Linux).
pub fn current() -> Option<SystemTheme> {
    match CURRENT.load(Ordering::Relaxed) {
        LIGHT => Some(SystemTheme::Light),
        DARK => Some(SystemTheme::Dark),
        _ => None,
    }
}

/// Enregistre une détection ; renvoie `true` si le thème a changé.
fn store(theme: SystemTheme) -> bool {
    let value = match theme {
        SystemTheme::Light => LIGHT,
        SystemTheme::Dark => DARK,
    };
    CURRENT.swap(value, Ordering::Relaxed) != value
}

/// Valeur `color-scheme` du portail : 1 sombre, 2 clair, 0 sans préférence.
pub(crate) fn from_portal(value: u32) -> Option<SystemTheme> {
    match value {
        1 => Some(SystemTheme::Dark),
        2 => Some(SystemTheme::Light),
        _ => None,
    }
}

/// Un nom de thème ou de jeu de couleurs (« Adwaita-dark », « BreezeDark ») trahit le sombre.
fn name_is_dark(name: &str) -> bool {
    name.to_ascii_lowercase().contains("dark")
}

/// `kdeglobals` : `ColorScheme=` de la section `[General]`.
pub(crate) fn from_kdeglobals(content: &str) -> Option<SystemTheme> {
    let mut general = false;
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            general = line == "[General]";
        } else if general {
            if let Some(scheme) = line.strip_prefix("ColorScheme=") {
                return Some(if name_is_dark(scheme) {
                    SystemTheme::Dark
                } else {
                    SystemTheme::Light
                });
            }
        }
    }
    None
}

/// `xsettings.xml` de XFCE : `<property name="ThemeName" type="string" value="…"/>`.
pub(crate) fn from_xfce_xsettings(content: &str) -> Option<SystemTheme> {
    let line = content
        .lines()
        .find(|line| line.contains("name=\"ThemeName\""))?;
    let value = line.split("value=\"").nth(1)?.split('"').next()?;
    Some(if name_is_dark(value) {
        SystemTheme::Dark
    } else {
        SystemTheme::Light
    })
}

/// `gtk-3.0/settings.ini` : préférence sombre explicite, sinon nom du thème.
pub(crate) fn from_gtk_settings(content: &str) -> Option<SystemTheme> {
    let mut theme = None;
    for line in content.lines().map(str::trim) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match (key.trim(), value.trim()) {
            ("gtk-application-prefer-dark-theme", "1" | "true") => return Some(SystemTheme::Dark),
            ("gtk-theme-name", name) => {
                theme = Some(if name_is_dark(name) {
                    SystemTheme::Dark
                } else {
                    SystemTheme::Light
                })
            }
            _ => {}
        }
    }
    theme
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Fréquence de relecture : un changement de thème du bureau s'applique en direct.
    const POLL: Duration = Duration::from_secs(3);

    fn config_home() -> Option<PathBuf> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
    }

    async fn from_portal_bus(connection: &zbus::Connection) -> Option<u32> {
        let proxy = zbus::proxy::Builder::<zbus::Proxy>::new(connection)
            .destination("org.freedesktop.portal.Desktop")
            .path("/org/freedesktop/portal/desktop")
            .interface("org.freedesktop.portal.Settings")
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await
            .ok()?;
        let reply: zbus::OwnedVariant = proxy
            .call("ReadOne", &("org.freedesktop.appearance", "color-scheme"))
            .await
            .ok()?;
        u32::try_from(reply).ok()
    }

    fn from_files() -> Option<SystemTheme> {
        let config = config_home()?;
        let read = |path: PathBuf| std::fs::read_to_string(path).ok();
        read(config.join("kdeglobals"))
            .and_then(|content| from_kdeglobals(&content))
            .or_else(|| {
                read(config.join("xfce4/xfconf/xfce-perchannel-xml/xsettings.xml"))
                    .and_then(|content| from_xfce_xsettings(&content))
            })
            .or_else(|| {
                read(config.join("gtk-3.0/settings.ini"))
                    .and_then(|content| from_gtk_settings(&content))
            })
    }

    /// Portail d'abord (préférence explicite), puis fichiers du bureau ; clair par défaut,
    /// comme les bureaux Linux eux-mêmes.
    pub(super) async fn detect(connection: Option<&zbus::Connection>) -> SystemTheme {
        let portal = match connection {
            Some(connection) => from_portal_bus(connection).await.and_then(from_portal),
            None => None,
        };
        portal.or_else(from_files).unwrap_or(SystemTheme::Light)
    }

    pub(super) async fn watch(on_change: impl Fn()) {
        let connection = zbus::Connection::session().await.ok();
        loop {
            if store(detect(connection.as_ref()).await) {
                tracing::debug!(theme = ?current(), "thème du système détecté");
                on_change();
            }
            tokio::time::sleep(POLL).await;
        }
    }
}

/// Suit le thème du bureau Linux ; `on_change` réveille l'interface. Sans effet ailleurs.
pub async fn watch(on_change: impl Fn()) {
    #[cfg(target_os = "linux")]
    linux::watch(on_change).await;
    #[cfg(not(target_os = "linux"))]
    let _ = on_change;
}

#[cfg(test)]
#[path = "../tests/test_platform_system_theme.rs"]
mod tests;
