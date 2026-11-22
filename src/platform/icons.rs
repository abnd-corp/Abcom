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
