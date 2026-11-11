//! Palette de l'interface, dérivée du thème actif.
//!
//! Les couleurs étaient auparavant écrites en dur à leur point d'usage, toutes
//! choisies pour un fond sombre : le sélecteur clair/sombre existait mais
//! n'était tenu qu'à moitié. Chaque rôle a désormais ses deux teintes ici, et
//! un seul endroit à modifier pour ajuster l'apparence.

use std::sync::Arc;

use eframe::egui::{self, Color32};

/// Hauteur de ligne du composeur dans le thème classique (curseur et emojis).
const CLASSIC_COMPOSER_LINE_HEIGHT: f32 = 22.0;

/// Clé du thème résolu dans la mémoire du contexte egui.
const THEME_ID: &str = "abcom_resolved_theme";

/// Couleurs de l'interface pour le thème courant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    /// Fond des cartes, bulles et blocs de code.
    pub(crate) surface: Color32,
    /// Fond des champs de saisie et boutons d'action.
    pub(crate) surface_strong: Color32,
    /// Fond survolé d'un élément de liste.
    pub(crate) surface_hover: Color32,
    /// Liseré de séparation et bordure de cadre.
    pub(crate) separator: Color32,
    /// Texte principal.
    pub(crate) text: Color32,
    /// Texte secondaire discret (tailles, horodatages, libellés de repli).
    pub(crate) text_muted: Color32,
    /// Accent de l'application (sélection, boutons actifs).
    pub(crate) accent: Color32,
    /// Accent atténué (survol d'un bouton accentué).
    pub(crate) accent_soft: Color32,
    /// Erreur, échec de livraison, alerte.
    pub(crate) danger: Color32,
    /// Confirmation, pair en ligne.
    pub(crate) success: Color32,
    /// Accusé de lecture.
    pub(crate) receipt_read: Color32,
    /// Liens et citations de réponse.
    pub(crate) link: Color32,
    /// Fond des blocs de code, citations et en-têtes de tableau.
    pub(crate) block_bg: Color32,
    /// Fond alterné des lignes de tableau.
    pub(crate) block_stripe: Color32,
    /// Bordure des blocs et tableaux.
    pub(crate) block_border: Color32,
    /// Texte à l'intérieur d'un bloc de code.
    pub(crate) block_text: Color32,
    /// Fond de l'application (panneaux et fenêtres).
    pub(crate) background: Color32,
    /// Fond des champs de saisie.
    pub(crate) input_bg: Color32,
    /// Fond d'une pastille de réaction.
    pub(crate) chip: Color32,
    /// Fond d'une pastille de réaction posée par soi-même.
    pub(crate) chip_mine: Color32,
    /// Bordure d'une pastille de réaction posée par soi-même.
    pub(crate) chip_mine_border: Color32,
    /// Voile posé sur la ligne de message survolée.
    pub(crate) row_tint: Color32,
}

/// Palette sombre — l'apparence historique de l'application.
pub(crate) const DARK: Palette = Palette {
    surface: Color32::from_rgb(43, 45, 49),
    surface_strong: Color32::from_rgb(66, 66, 70),
    surface_hover: Color32::from_rgb(60, 63, 68),
    separator: Color32::from_rgb(96, 96, 100),
    text: Color32::from_rgb(244, 245, 247),
    text_muted: Color32::from_gray(150),
    accent: Color32::from_rgb(88, 101, 242),
    accent_soft: Color32::from_rgb(88, 122, 255),
    danger: Color32::from_rgb(220, 80, 80),
    success: Color32::from_rgb(80, 200, 120),
    receipt_read: Color32::from_rgb(100, 180, 255),
    link: Color32::from_rgb(125, 211, 252),
    block_bg: Color32::from_rgb(30, 41, 59),
    block_stripe: Color32::from_rgb(23, 31, 46),
    block_border: Color32::from_rgb(71, 85, 105),
    block_text: Color32::from_rgb(248, 250, 252),
    background: Color32::from_gray(27),
    input_bg: Color32::from_gray(10),
    chip: Color32::from_rgb(50, 52, 58),
    chip_mine: Color32::from_rgb(70, 90, 140),
    chip_mine_border: Color32::from_rgb(120, 150, 255),
    row_tint: Color32::from_rgba_premultiplied(7, 7, 7, 7),
};

/// Palette claire — mêmes rôles, contrastes inversés.
pub(crate) const LIGHT: Palette = Palette {
    surface: Color32::from_rgb(241, 245, 249),
    surface_strong: Color32::from_rgb(226, 232, 240),
    surface_hover: Color32::from_rgb(214, 222, 233),
    separator: Color32::from_rgb(190, 198, 208),
    text: Color32::from_rgb(15, 23, 42),
    text_muted: Color32::from_rgb(71, 85, 105),
    accent: Color32::from_rgb(59, 74, 214),
    accent_soft: Color32::from_rgb(70, 100, 235),
    danger: Color32::from_rgb(180, 40, 40),
    success: Color32::from_rgb(21, 128, 61),
    receipt_read: Color32::from_rgb(3, 105, 161),
    link: Color32::from_rgb(3, 105, 161),
    block_bg: Color32::from_rgb(226, 232, 240),
    block_stripe: Color32::from_rgb(241, 245, 249),
    block_border: Color32::from_rgb(148, 163, 184),
    block_text: Color32::from_rgb(15, 23, 42),
    background: Color32::from_gray(248),
    input_bg: Color32::from_gray(255),
    chip: Color32::from_rgb(226, 232, 240),
    chip_mine: Color32::from_rgb(199, 210, 254),
    chip_mine_border: Color32::from_rgb(59, 74, 214),
    row_tint: Color32::from_rgba_premultiplied(0, 0, 0, 10),
};

/// Réglages de texte du thème actif.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Typography {
    /// Hauteur de ligne du composeur, qui place aussi les emojis de la saisie.
    pub(crate) composer_line_height: f32,
}

impl Typography {
    /// Réglages d'avant la personnalisation.
    pub(crate) const CLASSIC: Self = Self {
        composer_line_height: CLASSIC_COMPOSER_LINE_HEIGHT,
    };
}

/// Thème résolu : palettes des deux modes et réglages de texte.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedTheme {
    /// Change à chaque modification, pour invalider les caches qui en dérivent.
    pub(crate) revision: u64,
    pub(crate) dark: Palette,
    pub(crate) light: Palette,
    pub(crate) typography: Typography,
}

impl ResolvedTheme {
    /// Apparence historique de l'application.
    pub(crate) const fn classic() -> Self {
        Self {
            revision: 0,
            dark: DARK,
            light: LIGHT,
            typography: Typography::CLASSIC,
        }
    }

    /// Palette du mode demandé.
    pub(crate) fn palette(&self, dark_mode: bool) -> Palette {
        if dark_mode {
            self.dark
        } else {
            self.light
        }
    }
}

/// Range le thème résolu dans le contexte : toute l'interface le lit de là.
pub(crate) fn install(ctx: &egui::Context, theme: ResolvedTheme) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new(THEME_ID), Arc::new(theme)));
}

/// Thème résolu du contexte, ou le classique tant qu'aucun n'est installé.
pub(crate) fn current(ctx: &egui::Context) -> Arc<ResolvedTheme> {
    ctx.data(|data| data.get_temp::<Arc<ResolvedTheme>>(egui::Id::new(THEME_ID)))
        .unwrap_or_else(|| Arc::new(ResolvedTheme::classic()))
}

/// Palette du thème actif.
pub(crate) fn palette(ui: &egui::Ui) -> Palette {
    current(ui.ctx()).palette(ui.visuals().dark_mode)
}

/// Réglages de texte du thème actif.
pub(crate) fn typography(ui: &egui::Ui) -> Typography {
    current(ui.ctx()).typography
}

#[cfg(test)]
#[path = "../tests/test_ui_theme.rs"]
mod tests;
