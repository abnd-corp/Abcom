use super::*;

#[cfg(feature = "tray")]
#[test]
fn macos_tray_and_unread_badge_remain_alpha_templates() {
    let (normal, badge) = tray_images(true).unwrap();
    assert_eq!(normal.dimensions(), badge.dimensions());
    assert!(
        normal.width() > normal.height(),
        "le masque ne doit pas réduire le symbole à un carré vide"
    );
    for icon in [&normal, &badge] {
        assert!(icon.pixels().any(|p| p[3] > 0));
        assert!(icon.pixels().any(|p| p[3] == 0));
        assert!(icon.pixels().all(|p| p[3] == 0 || p.0[..3] == [0, 0, 0]));
    }
    assert_ne!(
        normal, badge,
        "les non-lus doivent rester identifiables en mode template"
    );
}

#[cfg(feature = "tray")]
#[test]
fn desktop_tray_uses_colour_artwork_and_a_visible_unread_badge() {
    let (normal, badge) = tray_images(false).unwrap();
    assert_eq!(normal.dimensions(), (64, 64));
    assert!(normal.pixels().any(|p| p.0 == [255, 138, 96, 255]));
    assert!(normal.pixels().any(|p| p.0 == [48, 40, 48, 255]));
    assert_ne!(normal, badge);
    assert_eq!(normal.dimensions(), badge.dimensions());
}

#[cfg(feature = "tray")]
#[test]
fn menu_actions_are_routed_by_id() {
    let open = MenuId::new("ouvrir");
    let quit = MenuId::new("quitter");
    assert_eq!(
        action_for(RawEvent::Menu(open.clone()), &open, &quit),
        Some(TrayAction::Open)
    );
    assert_eq!(
        action_for(RawEvent::Menu(quit.clone()), &open, &quit),
        Some(TrayAction::Quit)
    );
}

#[cfg(feature = "tray")]
#[test]
fn unrelated_menu_events_are_ignored() {
    assert_eq!(
        action_for(
            RawEvent::Menu(MenuId::new("autre")),
            &MenuId::new("ouvrir"),
            &MenuId::new("quitter")
        ),
        None
    );
}

#[cfg(feature = "tray")]
#[test]
fn icon_activation_opens_the_window() {
    assert_eq!(
        action_for(
            RawEvent::Click,
            &MenuId::new("ouvrir"),
            &MenuId::new("quitter")
        ),
        Some(TrayAction::Open)
    );
}

#[cfg(not(feature = "tray"))]
#[test]
fn disabled_tray_has_no_actions_and_no_icon() {
    assert!(Tray::new("Ouvrir", "Quitter").is_none());
    let mut tray = Tray;
    tray.set_unread(true);
    assert!(tray.poll().is_empty());
}

#[cfg(all(feature = "tray", target_os = "linux"))]
#[test]
#[ignore = "requires a private D-Bus session; run by Linux CI"]
fn ksni_creates_menu_and_updates_badge_without_gtk() {
    assert!(std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some());
    let mut tray = Tray::new("Ouvrir", "Quitter").expect("KSNI sur le bus de test");
    assert_ne!(tray.open_id, tray.quit_id);
    tray.set_unread(true);
    assert!(tray.badge_shown);
    tray.set_unread(false);
    assert!(!tray.badge_shown);
}
