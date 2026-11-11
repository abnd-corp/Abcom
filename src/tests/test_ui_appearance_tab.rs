use std::collections::HashSet;

use eframe::egui::Color32;

use super::*;
use crate::ui::theme::{DARK, LIGHT};

#[test]
fn hex_codes_round_trip() {
    for color in [
        Color32::BLACK,
        Color32::WHITE,
        Color32::from_rgb(1, 2, 3),
        Color32::from_rgb(255, 128, 0),
    ] {
        assert_eq!(parse_hex(&hex_of(color)), Some(color));
    }
    assert_eq!(hex_of(Color32::from_rgb(10, 255, 0)), "#0AFF00");
}

#[test]
fn hex_parsing_accepts_the_usual_spellings() {
    let orange = Some(Color32::from_rgb(255, 136, 0));
    assert_eq!(parse_hex("#ff8800"), orange);
    assert_eq!(parse_hex("FF8800"), orange);
    assert_eq!(parse_hex("  #F80  "), orange);
    assert_eq!(parse_hex("f80"), orange);
}

#[test]
fn hex_parsing_rejects_everything_else_without_panicking() {
    for raw in [
        "",
        "#",
        "##",
        "#12",
        "#1234",
        "#12345",
        "#1234567",
        "zzz",
        "#gg0000",
        "rouge",
        "é",
        "#é12",
        "#ééé",
        "🙂",
        "#🙂🙂",
        "0x112233",
        "#11 22 33",
        "\u{0}",
        "-#112233",
    ] {
        assert_eq!(parse_hex(raw), None, "{raw:?}");
    }
}

#[test]
fn a_very_long_input_is_rejected() {
    assert_eq!(parse_hex(&"f".repeat(10_000)), None);
}

#[test]
fn ratios_are_written_for_the_language() {
    assert_eq!(format_ratio(7.24, true), "7,2:1");
    assert_eq!(format_ratio(7.24, false), "7.2:1");
    assert_eq!(format_ratio(1.0, true), "1,0:1");
    assert_eq!(format_ratio(21.0, false), "21.0:1");
}

#[test]
fn main_and_advanced_roles_are_disjoint_and_cover_the_editable_palette() {
    let all: Vec<_> = Role::MAIN.into_iter().chain(Role::ADVANCED).collect();
    let unique: HashSet<_> = all.iter().copied().collect();
    assert_eq!(unique.len(), all.len(), "un rôle est listé deux fois");
    assert_eq!(all.len(), 21);
}

#[test]
fn every_role_reads_and_writes_its_own_field() {
    for role in Role::MAIN.into_iter().chain(Role::ADVANCED) {
        let mut palette = DARK;
        let marker = Color32::from_rgb(1, 2, 3);
        role.set(&mut palette, marker);
        assert_eq!(role.get(&palette), marker, "{role:?}");
    }
}

#[test]
fn a_role_only_changes_what_it_names_apart_from_its_documented_pairs() {
    // Le texte entraîne le texte des blocs, l'accent la bordure de vos réactions.
    let marker = Color32::from_rgb(9, 9, 9);
    for role in Role::MAIN.into_iter().chain(Role::ADVANCED) {
        let mut palette = LIGHT;
        role.set(&mut palette, marker);
        let changed = [
            palette.surface != LIGHT.surface,
            palette.surface_strong != LIGHT.surface_strong,
            palette.surface_hover != LIGHT.surface_hover,
            palette.separator != LIGHT.separator,
            palette.text != LIGHT.text,
            palette.text_muted != LIGHT.text_muted,
            palette.accent != LIGHT.accent,
            palette.accent_soft != LIGHT.accent_soft,
            palette.danger != LIGHT.danger,
            palette.success != LIGHT.success,
            palette.receipt_read != LIGHT.receipt_read,
            palette.link != LIGHT.link,
            palette.block_bg != LIGHT.block_bg,
            palette.block_stripe != LIGHT.block_stripe,
            palette.block_border != LIGHT.block_border,
            palette.block_text != LIGHT.block_text,
            palette.background != LIGHT.background,
            palette.input_bg != LIGHT.input_bg,
            palette.chip != LIGHT.chip,
            palette.chip_mine != LIGHT.chip_mine,
            palette.chip_mine_border != LIGHT.chip_mine_border,
        ]
        .into_iter()
        .filter(|changed| *changed)
        .count();
        let expected = match role {
            Role::Text | Role::Accent => 2,
            _ => 1,
        };
        assert_eq!(changed, expected, "{role:?}");
    }
}

#[test]
fn every_role_has_a_distinct_label_in_both_languages() {
    use crate::ui::UiLanguage;
    for language in [UiLanguage::French, UiLanguage::English] {
        let labels: HashSet<_> = Role::MAIN
            .into_iter()
            .chain(Role::ADVANCED)
            .map(|role| role.label().get(language))
            .collect();
        assert_eq!(labels.len(), 21, "libellés en double");
    }
}
