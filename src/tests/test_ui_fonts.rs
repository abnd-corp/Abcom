//! Couverture des polices : un caractère sans glyphe s'affiche en carré vide,
//! sans le moindre avertissement. Ce qui est écrit doit être dessinable.
//!
//! La mesure passe par le **glyphe réellement peint**, pas par
//! `Fonts::has_glyphs` : celui-ci répond « absent » pour tout caractère porté
//! par la police de remplacement de la famille, ce qui en fait un faux négatif
//! sur une bonne part des symboles.

use eframe::egui;

/// Caractères que l'interface écrit elle-même, et ceux qu'un texte collé
/// apporte couramment (documentation, notes, README).
const EXPECTED: &[char] = &[
    '☐', '☑', // cases à cocher des listes de tâches Markdown
    '✓', '✔', '✘', // coches et croix des accusés et des tableaux
    '→', '←', '↑', '↓', // flèches
    '⏎', '␣', // touches
    '⌘', // raccourcis macOS
    '•', '–', '—', '…', // ponctuation typographique
    '«', '»', '“', '”', // guillemets
    '≥', '≤', '×', '÷', '±', // mathématiques courantes
    '€', '£', '©', '®', '°',
];

/// Codets non assignés par Unicode : aucune police ne peut les dessiner, ils
/// donnent donc la signature du carré vide.
const UNASSIGNED: [char; 2] = ['\u{0378}', '\u{05FF}'];

/// Signature du glyphe peint pour `c` : sa position dans l'atlas et sa chasse.
/// Deux caractères qui la partagent sont peints à l'identique.
fn glyph_signature(ctx: &egui::Context, c: char) -> Option<(u16, u16, u16, u16, u32)> {
    let galley = ctx.fonts_mut(|fonts| {
        fonts.layout_no_wrap(
            c.to_string(),
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
        )
    });
    let glyph = galley.rows.first()?.glyphs.first()?;
    let uv = glyph.uv_rect;
    Some((
        uv.min[0],
        uv.min[1],
        uv.max[0],
        uv.max[1],
        glyph.advance_width.to_bits(),
    ))
}

struct Probe {
    ctx: egui::Context,
    tofu: Option<(u16, u16, u16, u16, u32)>,
}

impl Probe {
    /// Sonde la chaîne **privée de son dernier recours**.
    ///
    /// Unifont dessine une case hexadécimale pour n'importe quel codet, y
    /// compris non assigné : plus rien n'est jamais « absent », et la question
    /// utile devient « une *vraie* police le dessine-t-elle ? ». On la retire
    /// donc pour mesurer.
    fn without_last_resort() -> Self {
        let mut definitions = super::build_fonts(
            crate::ui::theme::FontKind::NotoSans,
            crate::ui::theme::FontKind::NotoSans,
        );
        for family in definitions.families.values_mut() {
            family.retain(|name| name != "unifont");
        }
        let ctx = Self::context(definitions);
        let tofu = glyph_signature(&ctx, UNASSIGNED[0]);
        assert_eq!(
            tofu,
            glyph_signature(&ctx, UNASSIGNED[1]),
            "deux codets non assignés doivent produire le même carré vide, \
             sans quoi la référence de la mesure ne vaut rien"
        );
        Self { ctx, tofu }
    }

    /// Sonde la chaîne complète. La référence n'est plus le carré vide mais la
    /// case hexadécimale qu'Unifont réserve à un codet non assigné : un
    /// caractère qui s'en distingue a bien un glyphe à lui.
    fn complete() -> Self {
        let ctx = Self::context(super::build_fonts(
            crate::ui::theme::FontKind::NotoSans,
            crate::ui::theme::FontKind::NotoSans,
        ));
        let tofu = glyph_signature(&ctx, UNASSIGNED[0]);
        Self { ctx, tofu }
    }

    fn context(definitions: egui::FontDefinitions) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_fonts(definitions);
        // Les polices ne sont interrogeables qu'à l'intérieur d'une passe.
        ctx.begin_pass(egui::RawInput::default());
        ctx
    }

    fn is_drawn(&self, c: char) -> bool {
        let signature = glyph_signature(&self.ctx, c);
        signature.is_some() && signature != self.tofu
    }
}

#[test]
fn every_character_the_interface_writes_is_drawn() {
    let probe = Probe::without_last_resort();
    let missing: Vec<char> = EXPECTED
        .iter()
        .copied()
        .filter(|c| !probe.is_drawn(*c))
        .collect();
    assert!(
        missing.is_empty(),
        "caractères peints en carré vide : {missing:?}"
    );
}

/// Un caractère témoin par police de la chaîne, qu'elle seule fournit : si
/// l'une d'elles se débranche, le témoin retombe en carré vide.
#[test]
fn each_font_of_the_chain_carries_its_share() {
    let probe = Probe::without_last_resort();
    for (character, font) in [
        ('✓', "Noto Sans"),
        ('✘', "Noto Sans Symbols 2"),
        ('→', "Inter"),
        ('😀', "polices d'emoji d'egui"),
    ] {
        assert!(
            probe.is_drawn(character),
            "{character} est peint en carré vide : {font} n'est plus consultée"
        );
    }

    // Le CJK n'existe que dans le dernier recours : sans lui, une phrase
    // entière en chinois, japonais ou coréen est illisible.
    let complete = Probe::complete();
    for character in ['好', '日', '한'] {
        assert!(
            complete.is_drawn(character),
            "{character} n'a pas de glyphe : Unifont n'est plus consultée"
        );
    }
}

/// Chaîne de Noto Sans, dans l'ordre historique de l'application.
const NOTO_CHAIN: [&str; 4] = ["noto-sans", "noto-symbols2", "inter-bold", "unifont"];

/// La chaîne doit être consultée dans l'ordre déclaré, et Unifont fermer la
/// marche : placée plus haut, son rendu tramé remplacerait des glyphes que
/// d'autres polices dessinent proprement.
#[test]
fn the_chain_is_wired_in_declared_order() {
    use crate::ui::theme::FontKind;
    let definitions = super::build_fonts(FontKind::NotoSans, FontKind::NotoSans);
    for name in NOTO_CHAIN {
        assert!(
            definitions.font_data.contains_key(name),
            "{name} n'est pas embarquée"
        );
    }
    let proportional = &definitions.families[&egui::FontFamily::Proportional];
    assert_eq!(
        &proportional[..NOTO_CHAIN.len()],
        &NOTO_CHAIN.map(str::to_owned),
        "la chaîne doit être en tête du proportionnel, dans l'ordre"
    );
    assert_eq!(
        NOTO_CHAIN.last(),
        Some(&"unifont"),
        "Unifont est le dernier recours, pas un choix par défaut"
    );
}

/// Chaque famille se ferme sur Unifont, quelle que soit la police de tête, et
/// ne cite que des polices embarquées : un nom inconnu ferait paniquer egui.
#[test]
fn every_family_ends_on_unifont_and_only_names_embedded_fonts() {
    use crate::ui::theme::{FontKind, MSG_BOLD_FAMILY, MSG_FAMILY, MSG_ITALIC_FAMILY};
    for (ui, messages) in [
        (FontKind::NotoSans, FontKind::NotoSans),
        (FontKind::Inter, FontKind::Inter),
        (FontKind::NotoSans, FontKind::Inter),
        (FontKind::Inter, FontKind::NotoSans),
    ] {
        let definitions = super::build_fonts(ui, messages);
        for name in [
            super::BOLD_FAMILY,
            MSG_FAMILY,
            MSG_BOLD_FAMILY,
            MSG_ITALIC_FAMILY,
        ] {
            let chain = &definitions.families[&egui::FontFamily::Name(name.into())];
            assert_eq!(chain.last().map(String::as_str), Some("unifont"), "{name}");
            for font in chain {
                assert!(
                    definitions.font_data.contains_key(font),
                    "{name} cite {font}, non embarquée"
                );
            }
        }
    }
}

#[test]
fn the_message_and_interface_fonts_can_differ() {
    use crate::ui::theme::{FontKind, MSG_FAMILY};
    let definitions = super::build_fonts(FontKind::NotoSans, FontKind::Inter);
    let proportional = &definitions.families[&egui::FontFamily::Proportional];
    let messages = &definitions.families[&egui::FontFamily::Name(MSG_FAMILY.into())];
    assert_eq!(proportional[0], "noto-sans");
    assert_eq!(messages[0], "inter");
}

#[test]
fn inter_has_a_real_italic_and_noto_sans_does_not() {
    use crate::ui::theme::{FontKind, MSG_ITALIC_FAMILY};
    let italic = |kind| {
        super::build_fonts(FontKind::NotoSans, kind).families
            [&egui::FontFamily::Name(MSG_ITALIC_FAMILY.into())][0]
            .clone()
    };
    assert_eq!(italic(FontKind::Inter), "inter-italic");
    assert_eq!(italic(FontKind::NotoSans), "noto-sans");
}
