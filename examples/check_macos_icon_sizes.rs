//! Native, main-thread geometry check; run on a logged-in Mac, outside the unit-test harness.

#[cfg(target_os = "macos")]
use abcom::{platform, util};

#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "../src/platform/tray.rs"]
mod actual_tray;

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use objc2_app_kit::{NSApplication, NSSquareStatusItemLength};
    use objc2_foundation::MainThreadMarker;
    use tray_icon::{Icon, TrayIconBuilder};

    let mtm = MainThreadMarker::new().ok_or("a main-thread macOS session is required")?;
    NSApplication::sharedApplication(mtm).finishLaunching();
    let (normal, badge) = actual_tray::tray_images(true, 36).ok_or("missing template")?;
    let make_icon = |image: &image::RgbaImage| {
        Icon::from_rgba(image.as_raw().clone(), image.width(), image.height())
    };
    let icon = TrayIconBuilder::new()
        .with_icon_templated(make_icon(&normal)?)
        .build()?;
    for (name, bitmap) in [("normal", &normal), ("unread", &badge), ("read", &normal)] {
        icon.set_icon_templated(Some(make_icon(bitmap)?))?;
        actual_tray::configure_macos_image(&icon);
        let item = icon.ns_status_item().ok_or("missing status item")?;
        let button = item.button(mtm).ok_or("missing status button")?;
        let image = button.image().ok_or("missing native image")?;
        let size = image.size();
        let pixels = image.representations().objectAtIndex(0);
        let button_size = button.frame().size;
        assert_eq!((size.width, size.height), (18.0, 18.0));
        assert_eq!((pixels.pixelsWide(), pixels.pixelsHigh()), (36, 36));
        assert!(image.isTemplate());
        assert_eq!(item.length(), NSSquareStatusItemLength);
        assert_eq!(button_size.width, button_size.height);
        assert!(button_size.width >= size.width);
        println!(
            "{name}: {}x{} pt, {}x{} px, system button {}x{} pt",
            size.width,
            size.height,
            pixels.pixelsWide(),
            pixels.pixelsHigh(),
            button_size.width,
            button_size.height,
        );
    }
    drop(icon);

    // Optional diagnostic inputs compare optical bounds without assuming a fixed Dock size.
    for path in std::env::args().skip(1) {
        let image = image::open(&path)?.to_rgba8();
        let visible: Vec<_> = image
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
        println!(
            "{path}: canvas {} px, visible width {} px",
            image.width(),
            right - left + 1
        );
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("run this native AppKit diagnostic on macOS".into())
}
