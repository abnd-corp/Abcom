use super::{current, install, palette, typography, ResolvedTheme, DARK, LIGHT};
use eframe::egui::{self, Color32};

/// Un rôle doit rester lisible dans les deux thèmes : c'est précisément ce qui
/// manquait quand les couleurs étaient écrites en dur pour un fond sombre.
fn luminance(color: eframe::egui::Color32) -> f32 {
    let [r, g, b, _] = color.to_array();
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0
}

#[test]
fn both_palettes_are_selectable() {
    let classic = ResolvedTheme::classic();
    assert_eq!(classic.palette(true).text, DARK.text);
    assert_eq!(classic.palette(false).text, LIGHT.text);
}

#[test]
fn text_contrasts_with_its_surface_in_both_themes() {
    for (name, palette) in [("sombre", DARK), ("clair", LIGHT)] {
        let text = luminance(palette.text);
        let surface = luminance(palette.surface);
        assert!(
            (text - surface).abs() > 0.4,
            "thème {name} : texte et fond trop proches ({text:.2} vs {surface:.2})"
        );
        let muted = luminance(palette.text_muted);
        assert!(
            (muted - surface).abs() > 0.15,
            "thème {name} : texte discret illisible ({muted:.2} vs {surface:.2})"
        );
    }
}

#[test]
fn dark_and_light_are_actually_inverted() {
    // Sans cela, une palette « claire » copiée de la sombre passerait inaperçue.
    assert!(
        luminance(DARK.surface) < 0.5,
        "le fond sombre doit être sombre"
    );
    assert!(
        luminance(LIGHT.surface) > 0.5,
        "le fond clair doit être clair"
    );
    assert!(luminance(DARK.text) > 0.5);
    assert!(luminance(LIGHT.text) < 0.5);
}

#[test]
fn classic_theme_keeps_the_historical_values() {
    let classic = ResolvedTheme::classic();
    assert_eq!(classic.revision, 0);
    assert_eq!(classic.dark, DARK);
    assert_eq!(classic.light, LIGHT);
    assert_eq!(classic.typography.composer_line_height, 22.0);
    assert_eq!(DARK.background, Color32::from_gray(27));
    assert_eq!(DARK.input_bg, Color32::from_gray(10));
    assert_eq!(LIGHT.background, Color32::from_gray(248));
    assert_eq!(LIGHT.input_bg, Color32::from_gray(255));
}

#[test]
fn dark_chips_and_row_tint_match_the_former_hardcoded_colors() {
    assert_eq!(DARK.chip, Color32::from_rgb(50, 52, 58));
    assert_eq!(DARK.chip_mine, Color32::from_rgb(70, 90, 140));
    assert_eq!(DARK.chip_mine_border, Color32::from_rgb(120, 150, 255));
    assert_eq!(
        DARK.row_tint,
        Color32::from_rgba_unmultiplied(255, 255, 255, 7)
    );
    assert_eq!(LIGHT.row_tint, Color32::from_rgba_unmultiplied(0, 0, 0, 10));
    assert_eq!(DARK.accent, Color32::from_rgb(88, 101, 242));
}

#[test]
fn chips_stay_readable_in_both_themes() {
    for (name, palette) in [("sombre", DARK), ("clair", LIGHT)] {
        for chip in [palette.chip, palette.chip_mine] {
            assert!(
                (luminance(palette.text) - luminance(chip)).abs() > 0.4,
                "thème {name} : texte illisible sur une pastille de réaction"
            );
        }
    }
}

#[test]
fn without_an_installed_theme_the_classic_one_applies() {
    let ctx = egui::Context::default();
    assert_eq!(current(&ctx).revision, 0);
    assert_eq!(current(&ctx).dark, DARK);
}

#[test]
fn an_installed_theme_replaces_the_classic_one() {
    let ctx = egui::Context::default();
    let mut theme = ResolvedTheme::classic();
    theme.revision = 7;
    theme.dark.accent = Color32::from_rgb(1, 2, 3);
    theme.typography.composer_line_height = 30.0;
    install(&ctx, theme);
    let current = current(&ctx);
    assert_eq!(current.revision, 7);
    assert_eq!(current.palette(true).accent, Color32::from_rgb(1, 2, 3));
    assert_eq!(current.palette(false), LIGHT);
}

#[test]
fn palette_and_typography_follow_the_installed_theme_from_a_ui() {
    let ctx = egui::Context::default();
    let mut theme = ResolvedTheme::classic();
    theme.dark.text = Color32::from_rgb(9, 9, 9);
    theme.typography.composer_line_height = 31.0;
    install(&ctx, theme);
    ctx.set_visuals(egui::Visuals::dark());
    let mut seen = None;
    let mut output = ctx.run_ui(Default::default(), |ui| {
        seen = Some((palette(ui).text, typography(ui).composer_line_height));
    });
    // epaint panique si les deltas de textures d'une frame sont abandonnés.
    output.textures_delta.clear();
    assert_eq!(seen, Some((Color32::from_rgb(9, 9, 9), 31.0)));
}
