use super::init_preference;

#[test]
fn an_enabled_preference_refreshes_the_path_after_install_or_upgrade() {
    let mut refreshed = false;
    assert!(init_preference(Some(true), true, || {
        refreshed = true;
        Ok(())
    }));
    assert!(refreshed);
}

#[test]
fn a_disabled_preference_and_debug_builds_do_not_touch_the_system() {
    for (saved, release, expected) in [
        (Some(false), true, false),
        (None, false, false),
        (Some(true), false, true),
    ] {
        assert_eq!(
            init_preference(saved, release, || panic!("unexpected OS write")),
            expected
        );
    }
}

#[test]
fn a_failed_refresh_preserves_an_explicit_preference() {
    assert!(init_preference(Some(true), true, || anyhow::bail!(
        "unavailable"
    )));
    assert!(!init_preference(None, true, || anyhow::bail!(
        "unavailable"
    )));
}

#[test]
fn a_first_release_launch_enables_autostart() {
    assert!(init_preference(None, true, || Ok(())));
}
