use super::*;

const ALL_ON: NotificationSettings = NotificationSettings {
    system: true,
    preview: true,
    sound: true,
    badge: true,
    attention: true,
};

#[test]
fn focused_window_only_beeps_for_another_conversation() {
    let other = alert_for(&ALL_ON, WindowPresence::Focused, false, false);
    assert_eq!(
        other,
        Alert {
            sound: true,
            ..Alert::default()
        }
    );
    let open = alert_for(&ALL_ON, WindowPresence::Focused, true, false);
    assert_eq!(open, Alert::default());
}

#[test]
fn background_window_uses_every_enabled_signal_even_for_the_open_conversation() {
    let expected = Alert {
        system: true,
        sound: true,
        attention: true,
    };
    assert_eq!(
        alert_for(&ALL_ON, WindowPresence::Background, true, false),
        expected
    );
    assert_eq!(
        alert_for(&ALL_ON, WindowPresence::Background, false, false),
        expected
    );
}

#[test]
fn hidden_window_has_no_button_to_flash() {
    let alert = alert_for(&ALL_ON, WindowPresence::Hidden, false, false);
    assert!(alert.system && alert.sound && !alert.attention);
}

#[test]
fn muted_conversation_never_alerts() {
    for presence in [
        WindowPresence::Focused,
        WindowPresence::Background,
        WindowPresence::Hidden,
    ] {
        assert_eq!(alert_for(&ALL_ON, presence, false, true), Alert::default());
    }
}

#[test]
fn each_setting_switches_off_its_own_signal() {
    let quiet = NotificationSettings {
        system: false,
        sound: false,
        attention: false,
        ..ALL_ON
    };
    assert_eq!(
        alert_for(&quiet, WindowPresence::Background, false, false),
        Alert::default()
    );
    let no_sound = NotificationSettings {
        sound: false,
        ..ALL_ON
    };
    let alert = alert_for(&no_sound, WindowPresence::Background, false, false);
    assert!(alert.system && !alert.sound && alert.attention);
}

fn temp_state() -> AppState {
    let dir = std::env::temp_dir().join(format!(
        "abcom-notif-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    AppState::new_with_base("moi", &dir)
}

#[test]
fn settings_default_to_on_and_round_trip_through_preferences() {
    let mut state = temp_state();
    assert_eq!(
        NotificationSettings::load(&state),
        NotificationSettings::default()
    );
    let changed = NotificationSettings {
        preview: false,
        badge: false,
        ..NotificationSettings::default()
    };
    changed.save(&mut state);
    assert_eq!(NotificationSettings::load(&state), changed);
}

#[test]
fn muted_conversations_round_trip_including_everyone() {
    let mut state = temp_state();
    assert!(load_muted(&state).is_empty());
    let muted: MutedConversations = [None, Some("bob".to_string()), Some("#salon".to_string())]
        .into_iter()
        .collect();
    save_muted(&mut state, &muted);
    assert_eq!(load_muted(&state), muted);
}
