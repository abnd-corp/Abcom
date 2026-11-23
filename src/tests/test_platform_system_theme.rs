use super::{from_gtk_settings, from_kdeglobals, from_portal, from_xfce_xsettings, SystemTheme};

#[test]
fn the_portal_preference_maps_to_light_or_dark_and_ignores_no_preference() {
    assert_eq!(from_portal(1), Some(SystemTheme::Dark));
    assert_eq!(from_portal(2), Some(SystemTheme::Light));
    assert_eq!(from_portal(0), None);
    assert_eq!(from_portal(7), None);
}

#[test]
fn kde_color_scheme_is_read_from_the_general_section_only() {
    let dark = "[Colors:View]\nColorScheme=Ignored\n[General]\nColorScheme=BreezeDark\n";
    assert_eq!(from_kdeglobals(dark), Some(SystemTheme::Dark));
    assert_eq!(
        from_kdeglobals("[General]\nColorScheme=BreezeLight\n"),
        Some(SystemTheme::Light)
    );
    assert_eq!(from_kdeglobals("[KDE]\nSingleClick=false\n"), None);
}

#[test]
fn xfce_theme_name_reveals_a_dark_theme() {
    let dark = r#"<property name="ThemeName" type="string" value="Greybird-dark"/>"#;
    let light = r#"<property name="ThemeName" type="string" value="Greybird"/>"#;
    assert_eq!(from_xfce_xsettings(dark), Some(SystemTheme::Dark));
    assert_eq!(from_xfce_xsettings(light), Some(SystemTheme::Light));
    assert_eq!(from_xfce_xsettings("<channel/>"), None);
}

#[test]
fn gtk_prefers_the_explicit_dark_flag_over_the_theme_name() {
    let flag = "[Settings]\ngtk-theme-name=Adwaita\ngtk-application-prefer-dark-theme=1\n";
    assert_eq!(from_gtk_settings(flag), Some(SystemTheme::Dark));
    assert_eq!(
        from_gtk_settings("[Settings]\ngtk-theme-name=Adwaita-dark\n"),
        Some(SystemTheme::Dark)
    );
    assert_eq!(
        from_gtk_settings("[Settings]\ngtk-theme-name=Adwaita\n"),
        Some(SystemTheme::Light)
    );
    assert_eq!(from_gtk_settings("[Settings]\n"), None);
}
