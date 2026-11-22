//! Politique d'icône de fenêtre et de Dock : le bundle macOS garde son ICNS.

pub(crate) fn desktop_app_id(flatpak: bool) -> &'static str {
    if flatpak {
        "corp.abnd.abcom"
    } else {
        "abcom"
    }
}

pub(crate) fn window_icon_data() -> Option<egui::IconData> {
    #[cfg(target_os = "macos")]
    {
        // eframe 0.37 distingue Some(vide), qui préserve le Dock, de None, qui choisit son logo.
        Some(egui::IconData::default())
    }
    #[cfg(not(target_os = "macos"))]
    {
        match image::load_from_memory(include_bytes!("../../assets/app_icon.png")) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                Some(egui::IconData {
                    width: rgba.width(),
                    height: rgba.height(),
                    rgba: rgba.into_raw(),
                })
            }
            Err(error) => {
                tracing::warn!(%error, "icône de fenêtre illisible");
                None
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn restore_dock_icon(mtm: objc2_foundation::MainThreadMarker) -> bool {
    use objc2::AnyThread;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::{NSBundle, NSData, NSString};

    let app = NSApplication::sharedApplication(mtm);
    let bundled_icon = NSBundle::mainBundle().pathForResource_ofType(
        Some(&NSString::from_str("Abcom")),
        Some(&NSString::from_str("icns")),
    );
    if bundled_icon.is_some() {
        // SAFETY : AppKit documente nil pour restaurer l'icône d'origine du bundle.
        unsafe { app.setApplicationIconImage(None) };
        return true;
    }
    // `cargo run` n'a pas de bundle : garder le même ICNS multi-résolution comme repli.
    let data = NSData::with_bytes(include_bytes!("../../assets/app_icon.icns"));
    if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
        // SAFETY : image valide, appel sur le thread principal garanti par mtm.
        unsafe { app.setApplicationIconImage(Some(&image)) };
        true
    } else {
        tracing::warn!("ICNS embarqué illisible");
        false
    }
}

#[cfg(test)]
#[path = "../tests/test_platform_icons.rs"]
mod tests;

/// Marque nos propres entrées de menu : seules celles-ci sont mises à jour.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const GENERATED_MARKER: &str = "X-Abcom-Generated=true";

/// Échappe un chemin pour `Exec=` (spécification Desktop Entry : guillemets, `\`, `$`, `` ` ``).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn desktop_exec_path(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len() + 2);
    escaped.push('"');
    for c in path.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped.push('"');
    escaped
}

/// Entrée de menu de l'application : son nom doit être l'`app_id` de la fenêtre pour que
/// le compositeur Wayland affiche notre icône (Wayland ne transmet pas l'icône de fenêtre).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn desktop_entry(exec: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Abcom\nComment=Encrypted LAN messenger\n\
         Comment[fr]=Messagerie chiffrée pour le réseau local\nExec={} %U\nIcon=abcom\n\
         Terminal=false\nCategories=Network;Chat;InstantMessaging;\nStartupWMClass=abcom\n\
         {GENERATED_MARKER}\n",
        desktop_exec_path(exec)
    )
}

/// Décide quoi faire d'une entrée existante : rien si elle vient d'un paquet (Nix,
/// installation système), réécriture si c'est la nôtre et qu'elle pointe ailleurs.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn desktop_entry_needs_write(existing: Option<&str>, wanted: &str) -> bool {
    match existing {
        None => true,
        Some(content) => content.contains(GENERATED_MARKER) && content != wanted,
    }
}

/// Binaire nu ou AppImage hors Flatpak : installe l'entrée de menu et l'icône si aucun
/// paquet ne les fournit, pour que GNOME et KDE sous Wayland affichent notre logo.
#[cfg(target_os = "linux")]
pub(crate) fn ensure_desktop_entry() {
    use std::path::PathBuf;
    if crate::platform::in_flatpak()
        || !crate::platform::autostart::owns_system_entry(crate::config::instance_id())
    {
        return;
    }
    let Some(data_home) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/share")))
    else {
        return;
    };
    let ours = data_home.join("applications/abcom.desktop");
    let system_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    if !ours.exists()
        && system_dirs.split(':').any(|dir| {
            PathBuf::from(dir)
                .join("applications/abcom.desktop")
                .exists()
        })
    {
        return;
    }
    let Ok(exec) = crate::platform::autostart::current_launch_path() else {
        return;
    };
    let wanted = desktop_entry(&exec.to_string_lossy());
    let existing = std::fs::read_to_string(&ours).ok();
    if !desktop_entry_needs_write(existing.as_deref(), &wanted) {
        return;
    }
    let icon = data_home.join("icons/hicolor/512x512/apps/abcom.png");
    let written = ours
        .parent()
        .map(std::fs::create_dir_all)
        .unwrap_or(Ok(()))
        .and_then(|()| std::fs::write(&ours, &wanted))
        .and_then(|()| icon.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())))
        .and_then(|()| std::fs::write(&icon, include_bytes!("../../assets/app_icon.png")));
    match written {
        Ok(()) => tracing::info!(path = %ours.display(), "entrée de menu installée"),
        Err(error) => tracing::warn!(%error, "entrée de menu impossible à écrire"),
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn ensure_desktop_entry() {}
