use super::*;

#[cfg(target_os = "macos")]
#[test]
fn macos_window_icon_explicitly_disables_eframe_dock_replacement() {
    let icon = window_icon_data().expect("None déclencherait le logo par défaut d'eframe");
    assert_eq!(icon, egui::IconData::default());
}

#[cfg(not(target_os = "macos"))]
#[test]
fn desktop_window_keeps_the_coloured_application_icon() {
    let icon = window_icon_data().unwrap();
    assert_eq!((icon.width, icon.height), (512, 512));
    assert!(icon.rgba.as_chunks::<4>().0.contains(&[255, 138, 96, 255]));
}
