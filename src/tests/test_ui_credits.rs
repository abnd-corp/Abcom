use super::SECTIONS;
use crate::ui::i18n::LICENCE;
use crate::ui::UiLanguage;
use std::collections::HashSet;

#[test]
fn every_grid_id_is_unique_and_every_row_has_text_in_both_languages() {
    let mut ids = HashSet::new();
    for section in SECTIONS {
        assert!(
            ids.insert(section.id),
            "id de grille en double : {}",
            section.id
        );
        assert!(!section.rows.is_empty(), "section vide : {}", section.id);
        for language in [UiLanguage::French, UiLanguage::English] {
            assert!(!section.title.resolve(language).is_empty());
            for (label, value) in section.rows {
                assert!(!label.get(language).is_empty(), "{}", section.id);
                assert!(!value.resolve(language).is_empty(), "{}", section.id);
            }
        }
    }
}

#[test]
fn every_section_names_its_licence() {
    for section in SECTIONS {
        let has_licence = section.rows.iter().any(|(label, _)| label.fr == LICENCE.fr);
        assert!(has_licence, "licence manquante : {}", section.id);
    }
}
