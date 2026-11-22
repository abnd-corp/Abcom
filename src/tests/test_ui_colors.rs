use eframe::egui::Color32;

use super::*;

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

#[test]
fn black_on_white_is_the_maximum_contrast() {
    let ratio = contrast_ratio(Color32::BLACK, Color32::WHITE);
    assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
}

#[test]
fn a_color_against_itself_has_no_contrast() {
    for color in [Color32::WHITE, rgb(10, 20, 30), rgb(200, 100, 50)] {
        assert!((contrast_ratio(color, color) - 1.0).abs() < 1e-6);
    }
}

#[test]
fn contrast_is_symmetric() {
    let (a, b) = (rgb(88, 101, 242), rgb(30, 31, 34));
    assert_eq!(contrast_ratio(a, b), contrast_ratio(b, a));
}

#[test]
fn known_contrast_values() {
    // Gris 119 sur blanc : la valeur de référence des 4,5:1 WCAG (≈ 4,48).
    let ratio = contrast_ratio(rgb(119, 119, 119), Color32::WHITE);
    assert!((ratio - 4.48).abs() < 0.02, "{ratio}");
}

#[test]
fn mix_endpoints_and_midpoint() {
    let (a, b) = (rgb(0, 0, 0), rgb(200, 100, 50));
    assert_eq!(mix(a, b, 0.0), a);
    assert_eq!(mix(a, b, 1.0), b);
    assert_eq!(mix(a, b, 0.5), rgb(100, 50, 25));
    assert_eq!(mix(a, b, 2.0), b, "la part est bornée");
    assert_eq!(mix(a, b, -1.0), a, "la part est bornée");
}

#[test]
fn ensure_contrast_leaves_a_conforming_color_untouched() {
    let fg = rgb(240, 240, 240);
    assert_eq!(ensure_contrast(fg, rgb(20, 20, 20), 4.5), fg);
}

#[test]
fn ensure_contrast_reaches_the_minimum_on_dark_and_light_backgrounds() {
    for (fg, bg) in [
        (rgb(110, 110, 120), rgb(30, 30, 34)),
        (rgb(150, 150, 150), rgb(245, 245, 245)),
        (rgb(80, 80, 200), rgb(20, 20, 30)),
    ] {
        let fixed = ensure_contrast(fg, bg, 4.5);
        assert!(contrast_ratio(fixed, bg) >= 4.5, "{fg:?} sur {bg:?}");
    }
}

#[test]
fn ensure_contrast_moves_away_from_the_background() {
    let dark_bg = rgb(20, 20, 20);
    assert!(luminance(ensure_contrast(rgb(60, 60, 60), dark_bg, 4.5)) > luminance(rgb(60, 60, 60)));
    let light_bg = rgb(240, 240, 240);
    assert!(
        luminance(ensure_contrast(rgb(200, 200, 200), light_bg, 4.5))
            < luminance(rgb(200, 200, 200))
    );
}

#[test]
fn ensure_contrast_is_deterministic() {
    let (fg, bg) = (rgb(120, 130, 140), rgb(35, 36, 40));
    assert_eq!(ensure_contrast(fg, bg, 4.5), ensure_contrast(fg, bg, 4.5));
}

#[test]
fn an_unreachable_contrast_ends_on_black_or_white_without_looping() {
    let mid = rgb(128, 128, 128);
    let fixed = ensure_contrast(mid, mid, 21.0);
    assert!(
        fixed == Color32::BLACK || fixed == Color32::WHITE,
        "{fixed:?}"
    );
}

fn dark_seed() -> Seed {
    Seed {
        background: rgb(46, 52, 64),
        surface: rgb(59, 66, 82),
        input_bg: rgb(67, 76, 94),
        border: rgb(76, 86, 106),
        text: rgb(236, 239, 244),
        text_muted: rgb(216, 222, 233),
        accent: rgb(136, 192, 208),
        link: rgb(129, 161, 193),
        danger: rgb(191, 97, 106),
        success: rgb(163, 190, 140),
    }
}

#[test]
fn derived_palette_keeps_the_seed_roles() {
    let seed = dark_seed();
    let palette = palette_from_seed(&seed);
    assert_eq!(palette.background, seed.background);
    assert_eq!(palette.surface, seed.surface);
    assert_eq!(palette.input_bg, seed.input_bg);
    assert_eq!(palette.separator, seed.border);
    assert_eq!(palette.block_border, seed.border);
    assert_eq!(palette.chip_mine_border, palette.accent);
    assert_eq!(palette.receipt_read, palette.link);
}

#[test]
fn derived_palette_guarantees_the_contrast_floors() {
    let mut seed = dark_seed();
    // Une graine volontairement illisible : tout doit être redressé.
    seed.text = rgb(70, 76, 90);
    seed.text_muted = rgb(66, 70, 80);
    seed.link = rgb(60, 66, 80);
    seed.danger = rgb(70, 40, 40);
    seed.success = rgb(40, 70, 40);
    seed.accent = rgb(50, 56, 70);
    let p = palette_from_seed(&seed);
    for (name, color) in [("text", p.text), ("text_muted", p.text_muted)] {
        assert!(
            contrast_ratio(color, p.background) >= MIN_TEXT_CONTRAST,
            "{name}/fond"
        );
        assert!(
            contrast_ratio(color, p.surface) >= MIN_TEXT_CONTRAST,
            "{name}/surface"
        );
    }
    for (name, color) in [
        ("link", p.link),
        ("danger", p.danger),
        ("success", p.success),
    ] {
        assert!(
            contrast_ratio(color, p.background) >= MIN_TEXT_CONTRAST,
            "{name}"
        );
    }
    assert!(contrast_ratio(p.accent, p.background) >= MIN_GRAPHIC_CONTRAST);
}

#[test]
fn derived_palette_does_not_alter_a_conforming_seed() {
    let seed = dark_seed();
    let p = palette_from_seed(&seed);
    assert_eq!(p.text, seed.text);
    assert_eq!(p.accent, seed.accent);
}

#[test]
fn derived_row_tint_is_translucent_in_both_modes() {
    let dark = palette_from_seed(&dark_seed());
    assert!(dark.row_tint.a() <= 12 && dark.row_tint.a() > 0);
    let mut light_seed = dark_seed();
    light_seed.background = rgb(236, 239, 244);
    light_seed.surface = rgb(229, 233, 240);
    light_seed.text = rgb(46, 52, 64);
    light_seed.text_muted = rgb(76, 86, 106);
    let light = palette_from_seed(&light_seed);
    assert!(light.row_tint.a() <= 12 && light.row_tint.a() > 0);
}
