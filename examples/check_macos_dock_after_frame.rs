//! Runs eframe before inspecting the real NSApplication icon, including bundle and fallback paths.

#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "../src/platform/icons.rs"]
mod actual_icons;

#[cfg(target_os = "macos")]
mod native {
    use std::sync::{Arc, Mutex};

    use abcom::util::MutexExt;
    use objc2::AnyThread;
    use objc2_app_kit::{NSApplication, NSBitmapImageFileType, NSBitmapImageRep};
    use objc2_foundation::{MainThreadMarker, NSDictionary};

    struct Probe {
        frames: u8,
        legacy: bool,
        failure: Arc<Mutex<Option<String>>>,
    }

    fn check_icon(legacy: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mtm = MainThreadMarker::new().ok_or("AppKit main thread required")?;
        let icon = NSApplication::sharedApplication(mtm)
            .applicationIconImage()
            .ok_or("missing application icon after eframe rendering")?;
        let tiff = icon
            .TIFFRepresentation()
            .ok_or("cannot rasterize the live Dock icon")?;
        let bitmap = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)
            .ok_or("cannot read live icon bitmap")?;
        // SAFETY : le dictionnaire typé est vide ; aucune propriété invalide n'est fournie.
        let png = unsafe {
            bitmap.representationUsingType_properties(
                NSBitmapImageFileType::PNG,
                &NSDictionary::new(),
            )
        }
        .ok_or("cannot export the live icon")?
        .to_vec();
        let rgba = image::load_from_memory(&png)?.to_rgba8();
        let visible: Vec<_> = rgba
            .enumerate_pixels()
            .filter(|(_, _, p)| p[3] > 128)
            .collect();
        let left = visible
            .iter()
            .map(|(x, _, _)| *x)
            .min()
            .ok_or("empty icon")?;
        let right = visible
            .iter()
            .map(|(x, _, _)| *x)
            .max()
            .ok_or("empty icon")?;
        let ratio = f64::from(right - left + 1) / f64::from(rgba.width());
        let variant = if legacy { "legacy" } else { "fixed" };
        let output = std::env::temp_dir().join(format!("abcom-dock-after-eframe-{variant}.png"));
        std::fs::write(&output, png)?;
        println!(
            "After 3 eframe frames: {}x{} px, visible width {} px ({:.1}%), {} native representations; {}",
            rgba.width(), rgba.height(), right - left + 1, ratio * 100.0,
            icon.representations().len(), output.display(),
        );
        if !(0.75..=0.86).contains(&ratio) {
            return Err(format!(
                "the live Dock icon lost its optical margins: {:.1}%",
                ratio * 100.0
            )
            .into());
        }
        Ok(())
    }

    impl eframe::App for Probe {
        fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
            root.label("Vérification de l’icône réelle du Dock…");
            self.frames += 1;
            if self.frames == 3 {
                *self.failure.lock_safe() =
                    check_icon(self.legacy).err().map(|error| error.to_string());
                root.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                root.ctx().request_repaint();
            }
        }
    }

    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let legacy = std::env::args().any(|arg| arg == "--reproduce-old");
        let icon = if legacy {
            eframe::icon_data::from_png_bytes(include_bytes!("../assets/app_icon.png"))?
        } else {
            super::actual_icons::window_icon_data().ok_or("missing viewport icon policy")?
        };
        let failure = Arc::new(Mutex::new(Some(
            "the native probe did not render".to_owned(),
        )));
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([340.0, 120.0])
                .with_icon(icon),
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };
        let result = failure.clone();
        eframe::run_native(
            "Abcom Dock verification",
            options,
            Box::new(move |_cc| {
                let mtm = MainThreadMarker::new().ok_or("AppKit main thread required")?;
                if !super::actual_icons::restore_dock_icon(mtm) {
                    return Err("cannot restore native Dock icon".into());
                }
                Ok(Box::new(Probe {
                    frames: 0,
                    legacy,
                    failure: result,
                }))
            }),
        )
        .map_err(|error| error.to_string())?;
        let error = failure.lock_safe().take();
        if let Some(error) = error {
            return Err(error.into());
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}

#[cfg(not(target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("run this native Dock diagnostic on macOS".into())
}
