//! Signaux hors de la fenêtre : notifications système au nom d'Abcom et pastille de non-lus
//! sur l'icône de l'application (Dock macOS, barre des tâches Windows, lanceurs Linux).

/// Identifiant de l'application : bundle macOS, AppUserModelID Windows, nom Flatpak.
pub(crate) const APP_ID: &str = "corp.abnd.abcom";

/// Au-delà, la pastille affiche « 99+ » : un compteur plus long déborde du Dock.
const BADGE_MAX: usize = 99;

/// Texte de la pastille ; `None` quand il n'y a rien à signaler.
pub(crate) fn badge_label(count: usize) -> Option<String> {
    match count {
        0 => None,
        n if n > BADGE_MAX => Some(format!("{BADGE_MAX}+")),
        n => Some(n.to_string()),
    }
}

/// Pastille ronde rouge en RGBA, lissée sur le bord : superposition de la barre des tâches.
#[cfg(any(windows, test))]
pub(crate) fn badge_dot_rgba(size: u32) -> Vec<u8> {
    const RED: [u8; 3] = [229, 72, 77];
    let radius = size as f32 / 2.0;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - radius;
            let dy = y as f32 + 0.5 - radius;
            // Couverture du pixel par le disque, sur un pixel de transition.
            let coverage = (radius - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
            pixels.extend_from_slice(&RED);
            pixels.push((coverage * 255.0).round() as u8);
        }
    }
    pixels
}

/// À appeler une fois au démarrage, avant la première notification : sans cela, macOS les
/// attribue au Finder et Windows à PowerShell.
pub(crate) fn init_notification_identity(data_dir: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = data_dir;
        // `cargo run` n'a pas de bundle : l'identité par défaut reste celle du système.
        let bundle = objc2_foundation::NSBundle::mainBundle().bundleIdentifier();
        if bundle.is_some_and(|id| id.to_string() == APP_ID) {
            if let Err(error) = notify_rust::set_application(APP_ID) {
                tracing::warn!(%error, "identité des notifications macOS");
            }
        }
    }
    #[cfg(windows)]
    if let Err(error) = windows::register_app_user_model_id(data_dir) {
        tracing::warn!(%error, "enregistrement de l'AppUserModelID");
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    let _ = data_dir;
}

/// Notification système, envoyée d'un thread détaché : l'appel peut bloquer selon l'OS.
/// Silencieuse partout : le son d'Abcom est réglé à part et joué par l'application.
pub(crate) fn show_notification(summary: String, body: String) {
    let spawned = std::thread::Builder::new()
        .name("abcom-notify".into())
        .spawn(move || {
            let mut notification = notify_rust::Notification::new();
            notification.appname("Abcom").summary(&summary).body(&body);
            #[cfg(windows)]
            notification.app_id(APP_ID);
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                let id = super::icons::desktop_app_id(super::in_flatpak());
                notification
                    .icon(id)
                    .hint(notify_rust::Hint::DesktopEntry(id.to_string()))
                    .hint(notify_rust::Hint::SuppressSound(true));
            }
            if let Err(error) = notification.show() {
                tracing::warn!(%error, "notification système non affichée");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "fil de notification non créé");
    }
}

/// Pastille de non-lus sur l'icône de l'application. Ne touche au système qu'au changement.
pub(crate) struct AppBadge {
    shown: usize,
    #[cfg(target_os = "linux")]
    launcher: tokio::sync::watch::Sender<usize>,
}

impl AppBadge {
    /// Sous Linux, la pastille passe par D-Bus : la tâche d'émission vit dans le runtime.
    /// Sans runtime (tests), elle reste locale.
    pub(crate) fn new(runtime: Option<&tokio::runtime::Handle>) -> Self {
        #[cfg(target_os = "linux")]
        {
            let (launcher, updates) = tokio::sync::watch::channel(0);
            if let Some(runtime) = runtime {
                runtime.spawn(linux::publish_launcher_count(updates));
            }
            Self { shown: 0, launcher }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = runtime;
            Self { shown: 0 }
        }
    }

    /// Affiche `count` non-lus (0 efface la pastille). Sur le thread principal : AppKit l'exige.
    pub(crate) fn set(&mut self, count: usize, description: &str) {
        if count == self.shown {
            return;
        }
        self.shown = count;
        #[cfg(target_os = "macos")]
        {
            let _ = description;
            macos::set_dock_badge(badge_label(count).as_deref());
        }
        #[cfg(windows)]
        windows::set_taskbar_overlay(count > 0, description);
        #[cfg(target_os = "linux")]
        {
            let _ = description;
            self.launcher.send_replace(count);
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2_app_kit::NSApplication;
    use objc2_foundation::{MainThreadMarker, NSString};

    pub(super) fn set_dock_badge(label: Option<&str>) {
        let Some(mtm) = MainThreadMarker::new() else {
            tracing::warn!("pastille du Dock hors du thread principal");
            return;
        };
        let label = label.map(NSString::from_str);
        NSApplication::sharedApplication(mtm)
            .dockTile()
            .setBadgeLabel(label.as_deref());
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;

    use zbus::zvariant::Value;

    /// Interface lue par KDE Plasma, Ubuntu Dock, Dash to Dock et Plank, entre autres.
    const INTERFACE: &str = "com.canonical.Unity.LauncherEntry";

    pub(super) async fn publish_launcher_count(mut updates: tokio::sync::watch::Receiver<usize>) {
        let Ok(connection) = zbus::Connection::session().await else {
            tracing::debug!("pas de bus de session : pastille du lanceur indisponible");
            return;
        };
        let id = super::super::icons::desktop_app_id(super::super::in_flatpak());
        let app_uri = format!("application://{id}.desktop");
        while updates.changed().await.is_ok() {
            let count = *updates.borrow_and_update();
            let properties = HashMap::from([
                ("count", Value::from(count as i64)),
                ("count-visible", Value::from(count > 0)),
            ]);
            let sent = connection
                .emit_signal(
                    None::<&str>,
                    "/corp/abnd/abcom",
                    INTERFACE,
                    "Update",
                    &(app_uri.as_str(), properties),
                )
                .await;
            if let Err(error) = sent {
                tracing::debug!(%error, "pastille du lanceur non publiée");
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use windows_sys::core::GUID;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows_sys::Win32::System::Registry::{RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateIcon, DestroyIcon, GetSystemMetrics, HICON, SM_CXSMICON,
    };

    const CLSID_TASKBAR_LIST: GUID = GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);
    const IID_ITASKBAR_LIST3: GUID = GUID::from_u128(0xea1afb91_9e28_4b86_90e9_9e9f8a5eefaf);

    /// Table virtuelle d'`ITaskbarList3`, jusqu'à `SetOverlayIcon` (emplacement 18).
    #[repr(C)]
    struct TaskbarListVtbl {
        _query_interface: usize,
        _add_ref: usize,
        release: unsafe extern "system" fn(*mut TaskbarList) -> u32,
        hr_init: unsafe extern "system" fn(*mut TaskbarList) -> i32,
        // AddTab … ThumbBarSetImageList : emplacements 4 à 17, inutilisés.
        _unused: [usize; 14],
        set_overlay_icon:
            unsafe extern "system" fn(*mut TaskbarList, HWND, HICON, *const u16) -> i32,
    }

    #[repr(C)]
    struct TaskbarList {
        vtbl: *const TaskbarListVtbl,
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Notifications d'une application non empaquetée : Windows lit son nom et son icône
    /// sous `HKCU\Software\Classes\AppUserModelId\<id>`, sans droits administrateur.
    pub(super) fn register_app_user_model_id(data_dir: &std::path::Path) -> std::io::Result<()> {
        let icon = data_dir.join("notification-icon.png");
        let png: &[u8] = include_bytes!("../../assets/app_icon.png");
        if std::fs::read(&icon).ok().as_deref() != Some(png) {
            std::fs::create_dir_all(data_dir)?;
            std::fs::write(&icon, png)?;
        }
        let key = wide(&format!(
            r"Software\Classes\AppUserModelId\{}",
            super::APP_ID
        ));
        for (name, value) in [
            ("DisplayName", "Abcom".to_string()),
            ("IconUri", icon.to_string_lossy().into_owned()),
        ] {
            let name = wide(name);
            let value = wide(&value);
            // SAFETY : chaînes UTF-16 terminées par NUL, taille en octets NUL compris.
            let status = unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    value.as_ptr().cast(),
                    (value.len() * 2) as u32,
                )
            };
            if status != 0 {
                return Err(std::io::Error::from_raw_os_error(status as i32));
            }
        }
        Ok(())
    }

    /// Pastille rouge en surimpression sur le bouton de la barre des tâches. COM vit dans un
    /// fil à part : son appartement ne doit pas interférer avec les sélecteurs de fichiers.
    pub(super) fn set_taskbar_overlay(shown: bool, description: &str) {
        let Some(hwnd) = crate::platform::tray::win::handle() else {
            return;
        };
        let hwnd = hwnd as isize;
        let description = wide(description);
        let spawned = std::thread::Builder::new()
            .name("abcom-taskbar".into())
            .spawn(move || apply_overlay(hwnd as HWND, shown, &description));
        if let Err(error) = spawned {
            tracing::debug!(%error, "fil de la barre des tâches non créé");
        }
    }

    fn apply_overlay(hwnd: HWND, shown: bool, description: &[u16]) {
        // SAFETY : COM est initialisé puis libéré dans ce fil ; l'objet obtenu est utilisé
        // selon la table d'ITaskbarList3 puis relâché ; la barre des tâches copie l'icône
        // avant qu'elle soit détruite.
        unsafe {
            let initialized = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            let mut object: *mut TaskbarList = std::ptr::null_mut();
            let created = CoCreateInstance(
                &CLSID_TASKBAR_LIST,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_ITASKBAR_LIST3,
                (&mut object as *mut *mut TaskbarList).cast(),
            );
            if created >= 0 && !object.is_null() {
                let vtbl = &*(*object).vtbl;
                if (vtbl.hr_init)(object) >= 0 {
                    let icon = if shown {
                        dot_icon()
                    } else {
                        std::ptr::null_mut()
                    };
                    let result = (vtbl.set_overlay_icon)(object, hwnd, icon, description.as_ptr());
                    if result < 0 {
                        tracing::debug!(result, "pastille de la barre des tâches refusée");
                    }
                    if !icon.is_null() {
                        DestroyIcon(icon);
                    }
                }
                (vtbl.release)(object);
            } else {
                tracing::debug!(created, "ITaskbarList3 indisponible");
            }
            if initialized >= 0 {
                CoUninitialize();
            }
        }
    }

    /// Icône 32 bits à la taille des petites icônes du système (DPI compris).
    fn dot_icon() -> HICON {
        // SAFETY : simple lecture d'une métrique système.
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64) as u32;
        let mut bgra = super::badge_dot_rgba(size);
        for pixel in bgra.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        // Masque ET à zéro : l'alpha du bitmap couleur fait foi. Lignes alignées sur 16 bits.
        let mask = vec![0u8; size.div_ceil(16) as usize * 2 * size as usize];
        // SAFETY : tampons aux dimensions annoncées (32 bits par pixel, masque 1 bit aligné).
        unsafe {
            CreateIcon(
                std::ptr::null_mut(),
                size as i32,
                size as i32,
                1,
                32,
                mask.as_ptr(),
                bgra.as_ptr(),
            )
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_platform_attention.rs"]
mod tests;
