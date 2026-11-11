//! Palette de l'interface, dérivée du thème actif.
//!
//! Les couleurs étaient auparavant écrites en dur à leur point d'usage, toutes
//! choisies pour un fond sombre : le sélecteur clair/sombre existait mais
//! n'était tenu qu'à moitié. Chaque rôle a désormais ses deux teintes ici, et
//! un seul endroit à modifier pour ajuster l'apparence.

use std::sync::Arc;

use eframe::egui::{self, Color32};

use super::appearance::TextSettings;
use super::fonts::FontPlans;

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

/// Police de texte embarquée d'un thème.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontKind {
    NotoSans,
    Inter,
}

/// Réglages de texte du thème actif.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Typography {
    pub(crate) small: f32,
    pub(crate) body: f32,
    pub(crate) button: f32,
    pub(crate) heading: f32,
    pub(crate) monospace: f32,
    /// Hauteur de ligne du composeur, qui place aussi les emojis de la saisie.
    pub(crate) composer_line_height: f32,
    /// Hauteur de ligne du fil ; `None` laisse celle de la police.
    pub(crate) body_line_height: Option<f32>,
    /// Approche des lettres du fil, en points.
    pub(crate) tracking: f32,
    /// Gamma des glyphes clairs sur fond sombre ; `None` garde celui d'egui.
    pub(crate) dark_text_gamma: Option<f32>,
    pub(crate) font: FontKind,
}

impl Typography {
    /// Réglages d'avant la personnalisation : les défauts d'egui.
    pub(crate) const CLASSIC: Self = Self {
        small: 9.0,
        body: 13.0,
        button: 13.0,
        heading: 18.0,
        monospace: 13.0,
        composer_line_height: CLASSIC_COMPOSER_LINE_HEIGHT,
        body_line_height: None,
        tracking: 0.0,
        dark_text_gamma: None,
        font: FontKind::NotoSans,
    };

    /// Corps plus grand en Inter, à l'échelle des clients de discussion.
    pub(crate) const LOUNGE: Self = Self {
        small: 12.0,
        body: 15.0,
        button: 14.0,
        heading: 18.0,
        monospace: 13.0,
        composer_line_height: 25.5,
        body_line_height: Some(20.0),
        tracking: 0.0,
        dark_text_gamma: Some(1.0),
        font: FontKind::Inter,
    };

    /// Texte plus grand et plus aéré, resserré par l'approche des lettres.
    pub(crate) const READABLE: Self = Self {
        small: 11.0,
        body: 15.0,
        button: 15.0,
        heading: 18.0,
        monospace: 13.0,
        composer_line_height: 25.5,
        body_line_height: Some(17.0),
        tracking: 0.15,
        dark_text_gamma: Some(1.0),
        font: FontKind::Inter,
    };
}

/// Famille de police du texte des conversations.
pub(crate) const MSG_FAMILY: &str = "msg";
/// Gras des conversations : noms d'auteur et `**gras**`.
pub(crate) const MSG_BOLD_FAMILY: &str = "msg-bold";
/// Italique des conversations.
pub(crate) const MSG_ITALIC_FAMILY: &str = "msg-italic";

/// Style de texte du corps des messages.
pub(crate) fn msg_style() -> egui::TextStyle {
    egui::TextStyle::Name("msg".into())
}

/// Style du petit texte des messages (heure, mentions secondaires).
pub(crate) fn msg_small_style() -> egui::TextStyle {
    egui::TextStyle::Name("msg-small".into())
}

/// Style du code des messages.
pub(crate) fn msg_mono_style() -> egui::TextStyle {
    egui::TextStyle::Name("msg-mono".into())
}

/// Tailles effectives : réglages du thème, mises à l'échelle par l'utilisateur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextMetrics {
    pub(crate) small: f32,
    pub(crate) body: f32,
    pub(crate) button: f32,
    pub(crate) heading: f32,
    pub(crate) monospace: f32,
    pub(crate) msg_small: f32,
    pub(crate) msg_body: f32,
    pub(crate) msg_mono: f32,
    /// Hauteur de ligne du fil ; `None` laisse celle de la police.
    pub(crate) msg_line_height: Option<f32>,
    pub(crate) msg_tracking: f32,
    /// Hauteur de ligne du composeur, qui place aussi les emojis de la saisie.
    pub(crate) composer_line_height: f32,
    pub(crate) dark_text_gamma: Option<f32>,
    /// La police des conversations a une vraie coupe grasse.
    pub(crate) msg_real_bold: bool,
    /// La police des conversations a une vraie coupe italique.
    pub(crate) msg_real_italic: bool,
}

impl TextMetrics {
    /// Le composeur ne descend jamais sous cette hauteur : ses emojis y tiennent.
    const MIN_COMPOSER_LINE_HEIGHT: f32 = CLASSIC_COMPOSER_LINE_HEIGHT;

    pub(crate) fn resolve(typography: &Typography, settings: &TextSettings) -> Self {
        let (ui, conv) = (settings.ui_factor(), settings.conv_factor());
        let msg_body = typography.body * conv;
        let msg_line_height = match settings.conv_line_spacing {
            Some(multiple) => Some(msg_body * multiple),
            None => typography.body_line_height.map(|height| height * conv),
        };
        Self {
            small: typography.small * ui,
            body: typography.body * ui,
            button: typography.button * ui,
            heading: typography.heading * ui,
            monospace: typography.monospace * ui,
            msg_small: typography.small * conv,
            msg_body,
            msg_mono: typography.monospace * conv,
            msg_line_height,
            msg_tracking: settings.conv_letter_spacing.unwrap_or(typography.tracking),
            composer_line_height: (typography.composer_line_height * conv)
                .max(Self::MIN_COMPOSER_LINE_HEIGHT),
            dark_text_gamma: typography.dark_text_gamma,
            msg_real_bold: typography.font == FontKind::Inter,
            msg_real_italic: typography.font == FontKind::Inter,
        }
    }

    /// Reporte sur les métriques les coupes réellement disponibles.
    pub(crate) fn with_font_cuts(mut self, bold: bool, italic: bool) -> Self {
        self.msg_real_bold = bold;
        self.msg_real_italic = italic;
        self
    }
}

/// Thème résolu : palettes des deux modes et tailles de texte.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedTheme {
    /// Change à chaque modification, pour invalider les caches qui en dérivent.
    pub(crate) revision: u64,
    pub(crate) dark: Palette,
    pub(crate) light: Palette,
    pub(crate) text: TextMetrics,
    /// Polices de l'interface et des conversations.
    pub(crate) fonts: FontPlans,
    /// Les widgets egui prennent les couleurs du thème ; Classique garde celles d'egui.
    pub(crate) styled_widgets: bool,
}

impl ResolvedTheme {
    /// Apparence historique de l'application.
    pub(crate) fn classic() -> Self {
        Self {
            revision: 0,
            dark: DARK,
            light: LIGHT,
            text: TextMetrics::resolve(&Typography::CLASSIC, &TextSettings::default()),
            fonts: FontPlans::theme(FontKind::NotoSans),
            styled_widgets: false,
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
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique(THEME_ID), Arc::new(theme)));
}

/// Thème résolu du contexte, ou le classique tant qu'aucun n'est installé.
pub(crate) fn current(ctx: &egui::Context) -> Arc<ResolvedTheme> {
    ctx.data(|data| data.get_temp::<Arc<ResolvedTheme>>(egui::Id::unique(THEME_ID)))
        .unwrap_or_else(|| Arc::new(ResolvedTheme::classic()))
}

/// Palette du thème actif.
pub(crate) fn palette(ui: &egui::Ui) -> Palette {
    current(ui.ctx()).palette(ui.visuals().dark_mode)
}

/// Texte du corps d'un message : approche des lettres et hauteur de ligne du thème.
pub(crate) fn message_text(ui: &egui::Ui, text: impl Into<String>) -> egui::RichText {
    let metrics = self::text(ui);
    let mut rich = egui::RichText::new(text);
    if metrics.msg_tracking != 0.0 {
        rich = rich.extra_letter_spacing(metrics.msg_tracking);
    }
    if let Some(height) = metrics.msg_line_height {
        rich = rich.line_height(Some(height));
    }
    rich
}

/// Tailles de texte du thème actif.
pub(crate) fn text(ui: &egui::Ui) -> TextMetrics {
    current(ui.ctx()).text
}

#[cfg(test)]
#[path = "../tests/test_ui_theme.rs"]
mod tests;
