//! Lancement automatique à l'ouverture de session (Launch Agent macOS,
//! registre Windows, `~/.config/autostart` Linux) via la crate `auto-launch`.
//!
//! Politique : activé par défaut au **premier lancement d'un build release**
//! (jamais en debug/`cargo run`), désactivable dans Paramètres → Général.
//! La préférence est persistée dans la table kv (`autostart`).

use auto_launch::AutoLaunchBuilder;
use std::path::{Path, PathBuf};

pub fn is_supported() -> bool {
    !super::in_flatpak()
}

/// L'entrée système est unique : seule l'instance principale la gère. Les instances
/// secondaires (`ABCOM_INSTANCE`, tests, `make run2`) ne doivent pas détourner celle de l'utilisateur.
fn owns_system_entry(instance: u32) -> bool {
    instance == 0
}

fn launch_path(executable: PathBuf, appimage: Option<&Path>) -> PathBuf {
    appimage
        .filter(|path| path.is_absolute())
        .map_or(executable, Path::to_path_buf)
}

fn launcher() -> anyhow::Result<auto_launch::AutoLaunch> {
    let appimage = if cfg!(target_os = "linux") {
        std::env::var_os("APPIMAGE").map(PathBuf::from)
    } else {
        None
    };
    let exe = launch_path(std::env::current_exe()?, appimage.as_deref());
    let exe = exe.to_string_lossy().to_string();
    let mut builder = AutoLaunchBuilder::new();
    builder.set_app_name("Abcom").set_app_path(&exe);
    #[cfg(target_os = "macos")]
    builder.set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent);
    builder
        .build()
        .map_err(|e| anyhow::anyhow!("autostart : {e}"))
}

/// Active/désactive le lancement au démarrage au niveau système.
pub fn set_enabled(enabled: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        is_supported(),
        "autostart unavailable in this Flatpak build"
    );
    anyhow::ensure!(
        owns_system_entry(crate::config::instance_id()),
        "autostart is managed by the main instance only (ABCOM_INSTANCE=0)"
    );
    let auto = launcher()?;
    if enabled {
        auto.enable().map_err(|e| anyhow::anyhow!("{e}"))?;
    } else {
        auto.disable().map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    Ok(())
}

/// Applique la politique de premier lancement : en release, si aucune
/// préférence n'existe encore, active l'autostart et renvoie la valeur
/// effective à persister. En debug ou depuis une instance secondaire, ne touche jamais au système.
/// Une préférence active réécrit le lanceur pour suivre les déplacements et mises à jour.
pub fn init_default(existing_pref: Option<bool>) -> bool {
    if !is_supported() {
        return false;
    }
    let manages_entry = !cfg!(debug_assertions) && owns_system_entry(crate::config::instance_id());
    init_preference(existing_pref, manages_entry, || set_enabled(true))
}

fn init_preference(
    existing_pref: Option<bool>,
    release: bool,
    enable: impl FnOnce() -> anyhow::Result<()>,
) -> bool {
    if !release {
        return existing_pref.unwrap_or(false);
    }
    match existing_pref {
        Some(false) => false,
        saved => match enable() {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("activation impossible : {e}");
                saved.unwrap_or(false)
            }
        },
    }
}

#[cfg(test)]
#[path = "../tests/test_platform_autostart.rs"]
mod tests;
