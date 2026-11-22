use super::*;

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
