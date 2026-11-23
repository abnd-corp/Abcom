//! Onglet Crédits des paramètres : les sections sont des données, le rendu est unique.

use eframe::egui;

use super::i18n::{self, Entry};
use super::UiLanguage;

/// Texte d'une cellule : nom propre ou licence invariables, ou libellé traduit.
#[derive(Clone, Copy)]
enum Text {
    Fixed(&'static str),
    Translated(&'static Entry),
}

impl Text {
    fn resolve(self, language: UiLanguage) -> &'static str {
        match self {
            Text::Fixed(text) => text,
            Text::Translated(entry) => entry.get(language),
        }
    }
}

struct Section {
    /// Identifiant egui de la grille ; unique dans l'onglet.
    id: &'static str,
    title: Text,
    rows: &'static [(&'static Entry, Text)],
    /// Mention en petits caractères sous la grille.
    note: Option<&'static Entry>,
}

const OFL: Text = Text::Fixed("SIL Open Font License v1.1");
const NOTO_AUTHORS: Text = Text::Fixed("The Noto Project Authors");

const SECTIONS: &[Section] = &[
    Section {
        id: "credits_abcom",
        title: Text::Fixed("Abcom"),
        rows: &[
            (&i18n::VERSION, Text::Fixed(env!("CARGO_PKG_VERSION"))),
            (
                &i18n::DESCRIPTION,
                Text::Translated(&i18n::MESSAGERIE_PAIR_A_PAIR_LOCALE_DECOUVERTE),
            ),
            (
                &i18n::DEVELOPPEURS,
                Text::Fixed("Hugo Lagouardat Massiroles, Rudy Alves"),
            ),
            (&i18n::COPYRIGHT, Text::Fixed("Abnd © 2026")),
            (
                &i18n::LICENCE,
                Text::Fixed("GNU Affero General Public License v3"),
            ),
        ],
        note: Some(&i18n::LOGICIEL_DISTRIBUE_SANS_GARANTIE_VOIR_LA),
    },
    Section {
        id: "credits_klipy",
        title: Text::Fixed("Klipy"),
        rows: &[
            (&i18n::FOURNISSEUR, Text::Fixed("KLIPY (klipy.com)")),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::GIF_ANIMES_MEMES_STATIQUES_ET_STICKERS),
            ),
            (&i18n::LICENCE, Text::Fixed("Klipy API Terms of Service")),
            (
                &i18n::ATTRIBUTION,
                Text::Fixed("© KLIPY — Powered by KLIPY"),
            ),
        ],
        note: None,
    },
    Section {
        id: "credits_openmoji",
        title: Text::Fixed("OpenEmoji"),
        rows: &[
            (&i18n::SOURCE, Text::Fixed("OpenEmoji (openmoji.org)")),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::JEU_D_EMOJIS_UTILISE_DANS_LE),
            ),
            (&i18n::LICENCE, Text::Fixed("CC BY 4.0")),
            (
                &i18n::AUTEURS,
                Text::Fixed("HfG Schwäbisch Gmünd & contributeurs OpenEmoji"),
            ),
        ],
        note: None,
    },
    Section {
        id: "credits_noto_sans",
        title: Text::Fixed("Noto Sans (police de texte)"),
        rows: &[
            (&i18n::AUTEURS, NOTO_AUTHORS),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::POLICE_DE_TEXTE_DE_L_INTERFACE),
            ),
            (&i18n::LICENCE, OFL),
        ],
        note: None,
    },
    Section {
        id: "credits_inter",
        title: Text::Fixed("Inter (police d'écriture)"),
        rows: &[
            (&i18n::AUTEUR, Text::Fixed("Rasmus Andersson")),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::POLICE_D_ECRITURE_EN_GRAS_UTILISEE),
            ),
            (&i18n::LICENCE, OFL),
        ],
        note: None,
    },
    Section {
        id: "credits_noto_symbols",
        title: Text::Fixed("Noto Sans Symbols 2 (police de symboles)"),
        rows: &[
            (&i18n::AUTEURS, NOTO_AUTHORS),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::POLICE_DE_SYMBOLES_UTILISEE_EN_REPLI),
            ),
            (&i18n::LICENCE, OFL),
        ],
        note: None,
    },
    Section {
        id: "credits_unifont",
        title: Text::Fixed("GNU Unifont (dernier recours)"),
        rows: &[
            (
                &i18n::AUTEURS,
                Text::Fixed("Roman Czyborra, Paul Hardy & contributeurs"),
            ),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::POLICE_DE_DERNIER_RECOURS),
            ),
            (
                &i18n::LICENCE,
                Text::Fixed("SIL Open Font License v1.1 / GPLv2+ avec exception d'embarquement"),
            ),
        ],
        note: None,
    },
    Section {
        id: "credits_suggested_fonts",
        title: Text::Translated(&i18n::POLICES_PROPOSEES),
        rows: &[
            (
                &i18n::AUTEURS,
                Text::Fixed(
                    "Atkinson Hyperlegible (Braille Institute of America), \
                     Lexend (The Lexend Project Authors), \
                     OpenDyslexic (Abbie Gonzalez), \
                     Literata (The Literata Project Authors), \
                     Nunito (The Nunito Project Authors), \
                     JetBrains Mono (The JetBrains Mono Project Authors)",
                ),
            ),
            (&i18n::ROLE, Text::Translated(&i18n::POLICES_PROPOSEES_ROLE)),
            (&i18n::LICENCE, OFL),
        ],
        note: None,
    },
    Section {
        id: "credits_palettes",
        title: Text::Translated(&i18n::PALETTES_DE_COULEURS),
        rows: &[
            (
                &i18n::AUTEURS,
                Text::Fixed(
                    "Nord (Arctic Ice Studio), Dracula (Zeno Rocha), \
                     Solarized (Ethan Schoonover), One Dark (Atom), \
                     Catppuccin, GitHub Primer",
                ),
            ),
            (
                &i18n::ROLE,
                Text::Translated(&i18n::PALETTES_INSPIREES_DE_CES_PROJETS),
            ),
            (&i18n::LICENCE, Text::Fixed("MIT")),
        ],
        note: None,
    },
];

pub(super) fn show_credits(ui: &mut egui::Ui, language: UiLanguage) {
    for (index, section) in SECTIONS.iter().enumerate() {
        if index > 0 {
            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);
        }
        show_section(ui, section, language);
    }
    ui.add_space(8.0);
}

fn show_section(ui: &mut egui::Ui, section: &Section, language: UiLanguage) {
    ui.label(
        egui::RichText::new(section.title.resolve(language))
            .strong()
            .heading(),
    );
    ui.add_space(4.0);
    egui::Grid::new(section.id)
        .max_col_width((ui.available_width() - 12.0) / 2.0)
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            for (label, value) in section.rows {
                ui.label(egui::RichText::new(label.get(language)).strong());
                ui.add(
                    egui::Label::new(value.resolve(language)).wrap_mode(egui::TextWrapMode::Wrap),
                );
                ui.end_row();
            }
        });
    if let Some(note) = section.note {
        ui.add_space(2.0);
        ui.label(egui::RichText::new(note.get(language)).small().weak());
    }
}

#[cfg(test)]
#[path = "../tests/test_ui_credits.rs"]
mod tests;
