//! Réglages d'apparence choisis par l'utilisateur, et leur application au contexte egui.

use std::sync::Arc;

use eframe::egui;
use serde::{Deserialize, Serialize};

use super::custom_themes::{ThemeStore, CUSTOM_PREFIX};
use super::fonts::{resolve_choice, FontChoice, FontPlan, FontPlans, FontStore, SuggestedFont};
use super::theme::{self, FontKind, ResolvedTheme};
use super::themes::BuiltinTheme;
use super::PreviewFont;

/// Bornes des curseurs de taille.
pub(crate) const SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.8..=1.6;
/// Bornes de l'interligne des conversations, en multiple de la taille du texte.
pub(crate) const LINE_SPACING_RANGE: std::ops::RangeInclusive<f32> = 1.0..=2.0;
/// Bornes de l'espacement des lettres des conversations, en points.
pub(crate) const LETTER_SPACING_RANGE: std::ops::RangeInclusive<f32> = -0.5..=1.5;
/// Facteur combiné (général × zone) : au-delà, l'interface cesse d'être utilisable.
pub(crate) const COMBINED_SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.7..=2.0;

/// Réglages de texte persistés.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct TextSettings {
    /// Taille générale : agrandit tout d'un coup.
    pub(crate) global_scale: f32,
    /// Taille de l'interface seule.
    pub(crate) ui_scale: f32,
    /// Taille des conversations seules.
    pub(crate) conv_scale: f32,
    /// Interligne des conversations ; `None` garde celui du thème.
    pub(crate) conv_line_spacing: Option<f32>,
    /// Espacement des lettres des conversations ; `None` garde celui du thème.
    pub(crate) conv_letter_spacing: Option<f32>,
}

impl Default for TextSettings {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            ui_scale: 1.0,
            conv_scale: 1.0,
            conv_line_spacing: None,
            conv_letter_spacing: None,
        }
    }
}

fn bounded(value: f32, range: &std::ops::RangeInclusive<f32>, default: f32) -> f32 {
    if value.is_finite() {
        value.clamp(*range.start(), *range.end())
    } else {
        default
    }
}

fn bounded_option(value: Option<f32>, range: &std::ops::RangeInclusive<f32>) -> Option<f32> {
    value
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(*range.start(), *range.end()))
}

impl TextSettings {
    /// Ramène chaque valeur dans ses bornes ; une valeur illisible revient au défaut.
    pub(crate) fn sanitized(self) -> Self {
        Self {
            global_scale: bounded(self.global_scale, &SCALE_RANGE, 1.0),
            ui_scale: bounded(self.ui_scale, &SCALE_RANGE, 1.0),
            conv_scale: bounded(self.conv_scale, &SCALE_RANGE, 1.0),
            conv_line_spacing: bounded_option(self.conv_line_spacing, &LINE_SPACING_RANGE),
            conv_letter_spacing: bounded_option(self.conv_letter_spacing, &LETTER_SPACING_RANGE),
        }
    }

    /// Facteur appliqué aux tailles de l'interface.
    pub(crate) fn ui_factor(&self) -> f32 {
        combined(self.global_scale, self.ui_scale)
    }

    /// Facteur appliqué aux tailles des conversations.
    pub(crate) fn conv_factor(&self) -> f32 {
        combined(self.global_scale, self.conv_scale)
    }
}

fn combined(global: f32, part: f32) -> f32 {
    (global * part).clamp(*COMBINED_SCALE_RANGE.start(), *COMBINED_SCALE_RANGE.end())
}

/// Mode clair, sombre ou celui du système.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Light,
    Dark,
    /// Suit le système ; une valeur inconnue y revient aussi.
    #[default]
    #[serde(other)]
    System,
}

impl Mode {
    pub(crate) fn preference(self) -> egui::ThemePreference {
        match self {
            Self::Light => egui::ThemePreference::Light,
            Self::Dark => egui::ThemePreference::Dark,
            Self::System => egui::ThemePreference::System,
        }
    }
}

/// Apparence choisie par l'utilisateur, persistée sous une seule clé.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Appearance {
    /// Identifiant du thème (cf. [`BuiltinTheme::id`]).
    pub(crate) theme: String,
    pub(crate) mode: Mode,
    pub(crate) text: TextSettings,
    /// Police de l'interface.
    pub(crate) ui_font: FontChoice,
    /// Police des conversations.
    pub(crate) conv_font: FontChoice,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: BuiltinTheme::Classic.id().to_owned(),
            mode: Mode::default(),
            text: TextSettings::default(),
            ui_font: FontChoice::default(),
            conv_font: FontChoice::default(),
        }
    }
}

impl Appearance {
    /// Clé de la table des préférences.
    pub(crate) const PREF_KEY: &'static str = "appearance";

    /// Relit l'apparence enregistrée. Une valeur absente, illisible ou d'une
    /// autre forme donne l'apparence par défaut, jamais une erreur.
    pub(crate) fn from_pref(raw: Option<&str>) -> Self {
        let Some(raw) = raw.map(str::trim).filter(|raw| !raw.is_empty()) else {
            return Self::default();
        };
        match serde_json::from_str::<Self>(raw) {
            Ok(appearance) => appearance.sanitized(),
            Err(err) => {
                tracing::warn!("apparence illisible, valeurs par défaut : {err}");
                Self::default()
            }
        }
    }

    /// Forme enregistrée dans les préférences.
    pub(crate) fn to_pref(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Identifiant du thème personnel choisi, s'il y en a un.
    pub(crate) fn custom_id(&self) -> Option<&str> {
        self.theme.strip_prefix(CUSTOM_PREFIX)
    }

    /// Choisit un thème personnel.
    pub(crate) fn select_custom(&mut self, id: &str) {
        self.theme = format!("{CUSTOM_PREFIX}{id}");
    }

    /// Choisit un thème intégré.
    pub(crate) fn select_builtin(&mut self, theme: BuiltinTheme) {
        self.theme = theme.id().to_owned();
    }

    /// Le thème personnel choisi a-t-il disparu ? Si oui, retour à Classique.
    /// Renvoie vrai quand le thème a changé.
    pub(crate) fn drop_missing_custom(&mut self, store: &ThemeStore) -> bool {
        let missing = self.custom_id().is_some_and(|id| store.get(id).is_none());
        if missing {
            self.select_builtin(BuiltinTheme::Classic);
        }
        missing
    }

    /// Ramène chaque valeur dans ses bornes ; un thème inconnu redevient Classique.
    pub(crate) fn sanitized(mut self) -> Self {
        self.text = self.text.sanitized();
        if self.custom_id().is_some_and(|id| !id.is_empty()) {
            return self;
        }
        if BuiltinTheme::from_id(&self.theme).is_none() {
            tracing::warn!("thème inconnu « {} », retour à Classique", self.theme);
            self.theme = BuiltinTheme::Classic.id().to_owned();
        }
        self
    }

    /// Thème intégré choisi.
    pub(crate) fn builtin(&self) -> BuiltinTheme {
        BuiltinTheme::from_id(&self.theme).unwrap_or(BuiltinTheme::Classic)
    }
}

/// Installe l'apparence dans le contexte egui : mode, thème, tailles et polices.
///
/// Renvoie le nombre de zones dont la police choisie n'a pas pu être chargée :
/// elles retombent sur la police du thème.
pub(crate) fn apply_appearance(
    ctx: &egui::Context,
    appearance: &Appearance,
    store: &ThemeStore,
    fonts: &FontStore,
    previews: bool,
) -> usize {
    ctx.set_theme(appearance.mode.preference());
    let text = appearance.text.sanitized();
    let mut resolved = match appearance.custom_id().and_then(|id| store.get(id)) {
        Some(custom) => custom.resolve(&text),
        None => appearance.builtin().resolve_with(&text),
    };
    let theme_font = match &resolved.fonts.ui {
        FontPlan::Theme(kind) => *kind,
        FontPlan::Files { .. } => FontKind::NotoSans,
    };
    let mut failures = 0;
    let mut zone = |choice: &FontChoice| {
        resolve_choice(choice, theme_font, fonts).unwrap_or_else(|err| {
            tracing::warn!("police {} indisponible : {err:?}", choice.to_id());
            failures += 1;
            FontPlan::Theme(theme_font)
        })
    };
    resolved.fonts = FontPlans {
        ui: zone(&appearance.ui_font),
        msg: zone(&appearance.conv_font),
    };
    resolved.text = resolved.text.with_font_cuts(
        resolved.fonts.msg.real_bold(),
        resolved.fonts.msg.real_italic(),
    );
    let preview_fonts = if previews {
        preview_fonts(fonts)
    } else {
        Vec::new()
    };
    install_resolved(ctx, resolved, preview_fonts, fonts.revision);
    failures
}

/// Polices proposées et importées, pour écrire chaque nom dans sa propre police.
fn preview_fonts(fonts: &FontStore) -> Vec<PreviewFont> {
    let mut previews: Vec<PreviewFont> = SuggestedFont::ALL
        .into_iter()
        .filter_map(|font| {
            font.decoded().ok().map(|decoded| PreviewFont {
                family: format!("suggested-{}", font.id()),
                files: decoded.regular,
            })
        })
        .collect();
    previews.extend([
        PreviewFont {
            family: "builtin-noto".to_owned(),
            files: vec![Arc::new(
                include_bytes!("../../assets/fonts/NotoSans-Regular.ttf").to_vec(),
            )],
        },
        PreviewFont {
            family: "builtin-inter".to_owned(),
            files: vec![Arc::new(
                include_bytes!("../../assets/fonts/Inter-Regular.ttf").to_vec(),
            )],
        },
    ]);
    previews.extend(fonts.fonts().iter().filter_map(|font| {
        fonts.read(&font.id).ok().map(|bytes| PreviewFont {
            family: format!("imported-{}", font.id),
            files: vec![bytes],
        })
    }));
    previews
}

/// Installe un thème et ses réglages de texte dans le contexte egui.
///
/// Les polices ne sont reconstruites que si le choix de police change : c'est
/// le seul changement coûteux, une couleur ou une taille n'y touche pas.
pub(crate) fn apply(ctx: &egui::Context, theme: BuiltinTheme, text: &TextSettings) {
    install_resolved(ctx, theme.resolve_with(&text.sanitized()), Vec::new(), 0);
}

fn install_resolved(
    ctx: &egui::Context,
    mut resolved: ResolvedTheme,
    previews: Vec<PreviewFont>,
    store_revision: u64,
) {
    resolved.revision = theme::current(ctx).revision + 1;
    let key = format!(
        "{}|{}",
        resolved.fonts.key(),
        if previews.is_empty() {
            "-".to_owned()
        } else {
            format!("p{store_revision}:{}", previews.len())
        }
    );
    if font_choice_changed(ctx, &key) {
        ctx.set_fonts(super::build_fonts(&resolved.fonts, &previews));
        remember_font_choice(ctx, key);
    }
    apply_text_styles(ctx, &resolved);
    apply_visuals(ctx, &resolved);
    theme::install(ctx, resolved);
}

fn font_choice_id() -> egui::Id {
    egui::Id::new("abcom_font_choice")
}

fn font_choice_changed(ctx: &egui::Context, key: &str) -> bool {
    ctx.data(|d| d.get_temp::<String>(font_choice_id()))
        .as_deref()
        != Some(key)
}

fn remember_font_choice(ctx: &egui::Context, key: String) {
    ctx.data_mut(|d| d.insert_temp(font_choice_id(), key));
}

/// Tailles de tous les styles de texte : ceux de l'interface, puis ceux des conversations.
fn apply_text_styles(ctx: &egui::Context, resolved: &ResolvedTheme) {
    let t = &resolved.text;
    ctx.all_styles_mut(|style| {
        for (text_style, size) in [
            (egui::TextStyle::Small, t.small),
            (egui::TextStyle::Body, t.body),
            (egui::TextStyle::Button, t.button),
            (egui::TextStyle::Heading, t.heading),
            (egui::TextStyle::Monospace, t.monospace),
        ] {
            if let Some(font_id) = style.text_styles.get_mut(&text_style) {
                font_id.size = size;
            }
        }
        let message =
            |size| egui::FontId::new(size, egui::FontFamily::Name(theme::MSG_FAMILY.into()));
        style
            .text_styles
            .insert(theme::msg_style(), message(t.msg_body));
        style
            .text_styles
            .insert(theme::msg_small_style(), message(t.msg_small));
        style.text_styles.insert(
            theme::msg_mono_style(),
            egui::FontId::new(t.msg_mono, egui::FontFamily::Monospace),
        );
    });
}

/// Couleurs des widgets. Classique laisse celles d'egui, qui n'ont jamais été les siennes.
fn apply_visuals(ctx: &egui::Context, resolved: &ResolvedTheme) {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let dark = theme == egui::Theme::Dark;
        let p = resolved.palette(dark);
        ctx.style_mut_of(theme, |style| {
            // On repart des réglages d'egui : changer de thème ne doit rien laisser du précédent.
            style.visuals = if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            let v = &mut style.visuals;
            v.panel_fill = p.background;
            v.window_fill = p.background;
            v.extreme_bg_color = p.input_bg;
            if resolved.styled_widgets {
                v.faint_bg_color = p.block_stripe;
                v.code_bg_color = p.block_bg;
                v.hyperlink_color = p.link;
                v.error_fg_color = p.danger;
                v.selection.bg_fill = p.accent.gamma_multiply(0.45);
                v.selection.stroke = egui::Stroke::new(1.0, p.accent);
                v.widgets.noninteractive.bg_fill = p.background;
                v.widgets.noninteractive.weak_bg_fill = p.surface;
                v.widgets.noninteractive.bg_stroke.color = p.separator;
                v.widgets.noninteractive.fg_stroke.color = p.text;
                v.widgets.inactive.bg_fill = p.surface_strong;
                v.widgets.inactive.weak_bg_fill = p.surface_strong;
                v.widgets.inactive.fg_stroke.color = p.text;
                v.widgets.hovered.bg_fill = p.surface_hover;
                v.widgets.hovered.weak_bg_fill = p.surface_hover;
                v.widgets.hovered.fg_stroke.color = p.text;
                v.widgets.active.bg_fill = p.accent_soft;
                v.widgets.active.weak_bg_fill = p.accent_soft;
                v.widgets.active.fg_stroke.color = p.text;
                v.widgets.open.bg_fill = p.surface_strong;
                v.widgets.open.weak_bg_fill = p.surface_strong;
            }
            if let Some(gamma) = resolved.text.dark_text_gamma.filter(|_| dark) {
                v.text_options.color_transfer_function =
                    egui::epaint::FontColorTransferFunction::Gamma(gamma);
            }
        });
    }
}

#[cfg(test)]
#[path = "../tests/test_ui_appearance.rs"]
mod tests;
