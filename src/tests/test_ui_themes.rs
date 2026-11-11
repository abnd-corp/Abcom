use std::collections::HashSet;

use eframe::egui::Color32;

use super::*;
use crate::ui::colors::{contrast_ratio, luminance, MIN_GRAPHIC_CONTRAST, MIN_TEXT_CONTRAST};
use crate::ui::theme::{FontKind, Typography, DARK, LIGHT};
use crate::ui::UiLanguage;

fn palettes(theme: BuiltinTheme) -> [(&'static str, crate::ui::theme::Palette); 2] {
    let resolved = theme.resolve();
    [("sombre", resolved.dark), ("clair", resolved.light)]
}

#[test]
fn there_are_ten_distinct_themes_with_stable_ids() {
    assert_eq!(BuiltinTheme::ALL.len(), 10);
    let ids: HashSet<_> = BuiltinTheme::ALL.iter().map(|t| t.id()).collect();
    assert_eq!(ids.len(), 10);
    for theme in BuiltinTheme::ALL {
        assert_eq!(BuiltinTheme::from_id(theme.id()), Some(theme));
    }
    assert_eq!(BuiltinTheme::from_id("inconnu"), None);
    assert_eq!(BuiltinTheme::from_id(""), None);
}

#[test]
fn classic_is_first_and_readable_last() {
    assert_eq!(BuiltinTheme::ALL[0], BuiltinTheme::Classic);
    assert_eq!(BuiltinTheme::ALL[9], BuiltinTheme::Readable);
}

#[test]
fn names_are_original_and_translated() {
    for theme in BuiltinTheme::ALL {
        for language in [UiLanguage::French, UiLanguage::English] {
            let name = theme.name().get(language);
            assert!(!name.trim().is_empty());
            for brand in [
                "Discord",
                "Nord",
                "Dracula",
                "Solarized",
                "Catppuccin",
                "GitHub",
                "One",
            ] {
                assert!(!name.contains(brand), "{name} reprend un nom de marque");
            }
            assert!(!theme.description().get(language).trim().is_empty());
        }
    }
}

#[test]
fn classic_keeps_the_original_palettes_and_typography() {
    let classic = BuiltinTheme::Classic.resolve();
    assert_eq!(classic.dark, DARK);
    assert_eq!(classic.light, LIGHT);
    assert_eq!(BuiltinTheme::Classic.typography(), Typography::CLASSIC);
    assert_eq!(classic.text.body, 13.0);
    assert_eq!(classic.text.small, 9.0);
    assert_eq!(classic.text.heading, 18.0);
    assert_eq!(classic.text.composer_line_height, 22.0);
    assert_eq!(classic.text.ui_font, FontKind::NotoSans);
    assert!(
        !classic.styled_widgets,
        "Classique garde les widgets d'egui"
    );
}

#[test]
fn readable_is_classic_with_larger_text_and_a_softer_white() {
    let readable = BuiltinTheme::Readable.resolve();
    assert_eq!(readable.dark.text, Color32::from_rgb(232, 235, 239));
    assert_eq!(readable.dark.surface, DARK.surface);
    assert_eq!(readable.light, LIGHT);
    let t = BuiltinTheme::Readable.typography();
    assert_eq!((t.small, t.body, t.button), (11.0, 15.0, 15.0));
    assert_eq!(t.body_line_height, Some(17.0));
    assert_eq!(t.tracking, 0.15);
    assert_eq!(t.dark_text_gamma, Some(1.0));
    assert_eq!(t.font, FontKind::Inter);
}

#[test]
fn lounge_uses_inter_with_chat_client_sizes() {
    let t = BuiltinTheme::Lounge.typography();
    assert_eq!((t.small, t.body, t.button), (12.0, 15.0, 14.0));
    assert_eq!(t.body_line_height, Some(20.0));
    assert_eq!(t.font, FontKind::Inter);
}

#[test]
fn other_themes_keep_the_classic_typography() {
    for theme in [
        BuiltinTheme::Frost,
        BuiltinTheme::Nocturne,
        BuiltinTheme::Sand,
        BuiltinTheme::Slate,
        BuiltinTheme::Marshmallow,
        BuiltinTheme::Ink,
        BuiltinTheme::Forest,
    ] {
        assert_eq!(theme.typography(), Typography::CLASSIC, "{theme:?}");
    }
}

#[test]
fn every_dark_variant_is_dark_and_every_light_variant_is_light() {
    for theme in BuiltinTheme::ALL {
        let resolved = theme.resolve();
        assert!(
            luminance(resolved.dark.background) < 0.2,
            "{theme:?} sombre"
        );
        assert!(
            luminance(resolved.light.background) > 0.5,
            "{theme:?} clair"
        );
        assert!(
            luminance(resolved.dark.text) > 0.3,
            "{theme:?} texte sombre"
        );
        assert!(
            luminance(resolved.light.text) < 0.3,
            "{theme:?} texte clair"
        );
    }
}

#[test]
fn text_is_readable_on_the_background_and_the_cards_in_every_theme() {
    for theme in BuiltinTheme::ALL {
        for (mode, p) in palettes(theme) {
            for (name, bg) in [("fond", p.background), ("surface", p.surface)] {
                let ratio = contrast_ratio(p.text, bg);
                assert!(
                    ratio >= MIN_TEXT_CONTRAST,
                    "{theme:?} {mode} : texte sur {name} {ratio:.2}:1"
                );
            }
        }
    }
}

#[test]
fn secondary_colors_meet_their_contrast_floors_in_every_derived_theme() {
    for theme in BuiltinTheme::ALL {
        if matches!(theme, BuiltinTheme::Classic | BuiltinTheme::Readable) {
            continue;
        }
        for (mode, p) in palettes(theme) {
            for (name, color) in [
                ("texte discret", p.text_muted),
                ("lien", p.link),
                ("erreur", p.danger),
                ("succès", p.success),
            ] {
                let ratio = contrast_ratio(color, p.background);
                assert!(
                    ratio >= MIN_TEXT_CONTRAST,
                    "{theme:?} {mode} : {name} {ratio:.2}:1"
                );
            }
            assert!(
                contrast_ratio(p.accent, p.background) >= MIN_GRAPHIC_CONTRAST,
                "{theme:?} {mode} : accent"
            );
        }
    }
}

#[test]
fn text_stays_readable_on_reaction_chips_in_every_derived_theme() {
    for theme in BuiltinTheme::ALL {
        if matches!(theme, BuiltinTheme::Classic | BuiltinTheme::Readable) {
            continue;
        }
        for (mode, p) in palettes(theme) {
            for (name, chip) in [("pastille", p.chip), ("pastille perso", p.chip_mine)] {
                let ratio = contrast_ratio(p.text, chip);
                assert!(
                    ratio >= MIN_TEXT_CONTRAST,
                    "{theme:?} {mode} : {name} {ratio:.2}:1"
                );
            }
        }
    }
}

#[test]
fn themes_are_visibly_different_from_each_other() {
    let mut backgrounds = HashSet::new();
    for theme in BuiltinTheme::ALL {
        if theme == BuiltinTheme::Readable {
            continue; // Même fond que Classique par construction.
        }
        let background = theme.resolve().dark.background;
        assert!(
            backgrounds.insert(background.to_array()),
            "{theme:?} : fond sombre en double"
        );
    }
}

#[test]
fn every_derived_theme_defines_both_modes_with_an_accent() {
    for theme in BuiltinTheme::ALL {
        let resolved = theme.resolve();
        assert_ne!(resolved.dark.accent, Color32::TRANSPARENT);
        assert_ne!(
            resolved.dark.background, resolved.light.background,
            "{theme:?}"
        );
    }
}

#[test]
fn resolved_themes_start_at_revision_zero() {
    for theme in BuiltinTheme::ALL {
        assert_eq!(theme.resolve().revision, 0);
    }
}
