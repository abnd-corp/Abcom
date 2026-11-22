//! Entrée de menu Linux : sous Wayland, le compositeur prend l'icône de fenêtre dans
//! l'entrée nommée comme l'`app_id` (Wayland ne transmet pas l'icône). Module séparé
//! d'`icons.rs`, que les exemples incluent seul par chemin.

#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

/// Marque nos propres entrées de menu : seules celles-ci sont mises à jour.
const GENERATED_MARKER: &str = "X-Abcom-Generated=true";

/// Échappe un chemin pour `Exec=` (spécification Desktop Entry : guillemets, `\`, `$`, `` ` ``).
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
pub(crate) fn desktop_entry_needs_write(existing: Option<&str>, wanted: &str) -> bool {
    match existing {
        None => true,
        Some(content) => content.contains(GENERATED_MARKER) && content != wanted,
    }
}

/// Binaire nu ou AppImage hors Flatpak : installe l'entrée de menu et l'icône si aucun
/// paquet ne les fournit, pour que GNOME et KDE sous Wayland affichent notre logo.
#[cfg(target_os = "linux")]
pub(crate) fn ensure() {
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
pub(crate) fn ensure() {}

#[cfg(test)]
#[path = "../tests/test_platform_desktop_entry.rs"]
mod tests;
