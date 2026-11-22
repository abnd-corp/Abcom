use super::*;

#[test]
fn flatpak_uses_the_exported_desktop_identifier() {
    assert_eq!(desktop_app_id(true), "corp.abnd.abcom");
    assert_eq!(desktop_app_id(false), "abcom");
}

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

#[test]
fn the_desktop_exec_path_escapes_quotes_dollars_and_spaces() {
    assert_eq!(
        desktop_exec_path("/opt/Abcom 1/abcom"),
        "\"/opt/Abcom 1/abcom\""
    );
    assert_eq!(desktop_exec_path("/a/$b\"c"), "\"/a/\\$b\\\"c\"");
}

#[test]
fn only_our_own_desktop_entry_is_rewritten_when_it_points_elsewhere() {
    let wanted = desktop_entry("/home/alice/Abcom.AppImage");
    assert!(desktop_entry_needs_write(None, &wanted));
    assert!(!desktop_entry_needs_write(Some(&wanted), &wanted));
    let moved = desktop_entry("/tmp/old/Abcom.AppImage");
    assert!(desktop_entry_needs_write(Some(&moved), &wanted));
    // Une entrée fournie par un paquet (Nix, distribution) n'est jamais touchée.
    let packaged = "[Desktop Entry]\nName=Abcom\nExec=abcom\nIcon=abcom\n";
    assert!(!desktop_entry_needs_write(Some(packaged), &wanted));
    assert!(wanted.contains("Icon=abcom") && wanted.contains("StartupWMClass=abcom"));
}
