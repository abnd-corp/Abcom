use super::*;

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
