//! Onglet Apparence des Paramètres : thèmes, texte et couleurs.

use std::collections::HashMap;

use eframe::egui::{self, Color32};

use super::appearance::{
    Appearance, Mode, TextSettings, LETTER_SPACING_RANGE, LINE_SPACING_RANGE, SCALE_RANGE,
};
use super::colors::{contrast_ratio, MIN_TEXT_CONTRAST};
use super::custom_themes::clean_name;
use super::i18n::{self, Entry};
use super::theme::{self, Palette};
use super::themes::BuiltinTheme;
use super::{markdown, AbcomApp};

/// Largeur d'une carte de thème, en points.
const CARD_WIDTH: f32 = 148.0;
const CARD_HEIGHT: f32 = 104.0;
/// Hauteur de la bande du nom, au bas d'une carte.
const CARD_NAME_HEIGHT: f32 = 26.0;

/// Sous-onglet ouvert dans Apparence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum AppearanceTab {
    #[default]
    Themes,
    Text,
    Colors,
}

/// État de l'onglet, gardé tant que l'application tourne.
#[derive(Default)]
pub(crate) struct AppearanceUi {
    pub(crate) tab: AppearanceTab,
    confirm_reset: bool,
    confirm_delete: bool,
    /// Nom en cours de saisie, avec l'identifiant du thème renommé.
    name_buffer: Option<(String, String)>,
    /// Codes couleur en cours de saisie.
    hex_buffers: HashMap<(Role, bool), String>,
}

/// Rôles de couleur modifiables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Role {
    Background,
    Surface,
    Text,
    Accent,
    SurfaceStrong,
    SurfaceHover,
    Separator,
    TextMuted,
    AccentSoft,
    Danger,
    Success,
    ReceiptRead,
    Link,
    BlockBg,
    BlockStripe,
    BlockBorder,
    BlockText,
    InputBg,
    Chip,
    ChipMine,
    ChipMineBorder,
}

impl Role {
    /// Les quatre couleurs qui font l'ambiance d'un thème.
    pub(crate) const MAIN: [Self; 4] = [Self::Background, Self::Surface, Self::Text, Self::Accent];

    /// Les autres, dans la section Avancé.
    pub(crate) const ADVANCED: [Self; 17] = [
        Self::SurfaceStrong,
        Self::SurfaceHover,
        Self::InputBg,
        Self::Separator,
        Self::TextMuted,
        Self::AccentSoft,
        Self::Link,
        Self::Danger,
        Self::Success,
        Self::ReceiptRead,
        Self::BlockBg,
        Self::BlockStripe,
        Self::BlockBorder,
        Self::BlockText,
        Self::Chip,
        Self::ChipMine,
        Self::ChipMineBorder,
    ];

    pub(crate) fn label(self) -> Entry {
        match self {
            Self::Background => i18n::ROLE_FOND,
            Self::Surface => i18n::ROLE_SURFACES,
            Self::Text => i18n::ROLE_COULEUR_DU_TEXTE,
            Self::Accent => i18n::ROLE_ACCENT,
            Self::SurfaceStrong => i18n::ROLE_BOUTONS_D_ACTION,
            Self::SurfaceHover => i18n::ROLE_SURVOL_D_UNE_LIGNE,
            Self::Separator => i18n::ROLE_SEPARATEURS,
            Self::TextMuted => i18n::ROLE_TEXTE_DISCRET,
            Self::AccentSoft => i18n::ROLE_ACCENT_SURVOLE,
            Self::Danger => i18n::ROLE_ERREURS,
            Self::Success => i18n::ROLE_SUCCES,
            Self::ReceiptRead => i18n::ROLE_ACCUSE_DE_LECTURE,
            Self::Link => i18n::ROLE_LIENS,
            Self::BlockBg => i18n::ROLE_FOND_DES_BLOCS_DE_CODE,
            Self::BlockStripe => i18n::ROLE_LIGNES_ALTERNEES,
            Self::BlockBorder => i18n::ROLE_BORDURES_DES_BLOCS,
            Self::BlockText => i18n::ROLE_TEXTE_DES_BLOCS_DE_CODE,
            Self::InputBg => i18n::ROLE_CHAMPS_DE_SAISIE,
            Self::Chip => i18n::ROLE_PASTILLES_DE_REACTION,
            Self::ChipMine => i18n::ROLE_VOS_REACTIONS,
            Self::ChipMineBorder => i18n::ROLE_BORDURE_DE_VOS_REACTIONS,
        }
    }

    pub(crate) fn get(self, p: &Palette) -> Color32 {
        match self {
            Self::Background => p.background,
            Self::Surface => p.surface,
            Self::Text => p.text,
            Self::Accent => p.accent,
            Self::SurfaceStrong => p.surface_strong,
            Self::SurfaceHover => p.surface_hover,
            Self::Separator => p.separator,
            Self::TextMuted => p.text_muted,
            Self::AccentSoft => p.accent_soft,
            Self::Danger => p.danger,
            Self::Success => p.success,
            Self::ReceiptRead => p.receipt_read,
            Self::Link => p.link,
            Self::BlockBg => p.block_bg,
            Self::BlockStripe => p.block_stripe,
            Self::BlockBorder => p.block_border,
            Self::BlockText => p.block_text,
            Self::InputBg => p.input_bg,
            Self::Chip => p.chip,
            Self::ChipMine => p.chip_mine,
            Self::ChipMineBorder => p.chip_mine_border,
        }
    }

    pub(crate) fn set(self, p: &mut Palette, color: Color32) {
        match self {
            Self::Background => p.background = color,
            Self::Surface => p.surface = color,
            Self::Text => {
                p.text = color;
                p.block_text = color;
            }
            Self::Accent => {
                p.accent = color;
                p.chip_mine_border = color;
            }
            Self::SurfaceStrong => p.surface_strong = color,
            Self::SurfaceHover => p.surface_hover = color,
            Self::Separator => p.separator = color,
            Self::TextMuted => p.text_muted = color,
            Self::AccentSoft => p.accent_soft = color,
            Self::Danger => p.danger = color,
            Self::Success => p.success = color,
            Self::ReceiptRead => p.receipt_read = color,
            Self::Link => p.link = color,
            Self::BlockBg => p.block_bg = color,
            Self::BlockStripe => p.block_stripe = color,
            Self::BlockBorder => p.block_border = color,
            Self::BlockText => p.block_text = color,
            Self::InputBg => p.input_bg = color,
            Self::Chip => p.chip = color,
            Self::ChipMine => p.chip_mine = color,
            Self::ChipMineBorder => p.chip_mine_border = color,
        }
    }
}

/// Code `#RRGGBB` d'une couleur.
pub(crate) fn hex_of(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

/// Lit `#RGB`, `#RRGGBB`, avec ou sans dièse.
pub(crate) fn parse_hex(raw: &str) -> Option<Color32> {
    let digits = raw.trim().trim_start_matches('#');
    if !digits.is_ascii() {
        return None;
    }
    let channel = |s: &str| u8::from_str_radix(s, 16).ok();
    match digits.len() {
        3 => {
            let d = |i: usize| channel(&digits[i..=i]).map(|v| v * 17);
            Some(Color32::from_rgb(d(0)?, d(1)?, d(2)?))
        }
        6 => Some(Color32::from_rgb(
            channel(&digits[0..2])?,
            channel(&digits[2..4])?,
            channel(&digits[4..6])?,
        )),
        _ => None,
    }
}

/// Rapport de contraste écrit à la française : « 7,2:1 ».
pub(crate) fn format_ratio(ratio: f32, comma: bool) -> String {
    let text = format!("{ratio:.1}:1");
    if comma {
        text.replace('.', ",")
    } else {
        text
    }
}

impl AbcomApp {
    pub(crate) fn render_appearance_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (AppearanceTab::Themes, i18n::APPARENCE_THEMES),
                (AppearanceTab::Text, i18n::APPARENCE_TEXTE),
                (AppearanceTab::Colors, i18n::APPARENCE_COULEURS),
            ] {
                if ui
                    .selectable_label(self.appearance_ui.tab == tab, self.t(label))
                    .clicked()
                {
                    self.appearance_ui.tab = tab;
                }
            }
        });
        ui.separator();

        let tab = self.appearance_ui.tab;
        let footer = 44.0;
        egui::ScrollArea::vertical()
            .id_salt(("appearance_scroll", tab))
            .auto_shrink([false, false])
            .max_height((ui.available_height() - footer).max(80.0))
            .show(ui, |ui| match tab {
                AppearanceTab::Themes => self.themes_tab(ui),
                AppearanceTab::Text => self.text_tab(ui),
                AppearanceTab::Colors => self.colors_tab(ui),
            });
        ui.separator();
        self.reset_footer(ui);
    }

    fn heading(&self, ui: &mut egui::Ui, entry: Entry) {
        ui.add_space(6.0);
        ui.label(egui::RichText::new(self.t(entry)).strong());
        ui.add_space(2.0);
    }

    // ── Thèmes ────────────────────────────────────────────────────────────

    fn themes_tab(&mut self, ui: &mut egui::Ui) {
        self.heading(ui, i18n::MODE_D_AFFICHAGE);
        ui.horizontal_wrapped(|ui| {
            for (mode, label) in [
                (Mode::System, self.t(i18n::SUIVRE_LE_SYSTEME)),
                (Mode::Light, self.t(i18n::CLAIR)),
                (Mode::Dark, self.t(i18n::SOMBRE)),
            ] {
                ui.radio_value(&mut self.appearance.mode, mode, label);
            }
        });

        self.heading(ui, i18n::THEME);
        let dark = ui.visuals().dark_mode;
        let mut chosen: Option<BuiltinTheme> = None;
        ui.horizontal_wrapped(|ui| {
            for theme in BuiltinTheme::ALL {
                let selected = self.appearance.theme == theme.id();
                let palette = theme.resolve().palette(dark);
                let name = self.t(theme.name());
                let label = format!("{} {name}", self.t(i18n::THEME));
                let card = theme_card(ui, &label, name, &palette, selected)
                    .on_hover_text(self.t(theme.description()));
                if card.clicked() {
                    chosen = Some(theme);
                }
            }
        });
        if let Some(theme) = chosen {
            self.appearance.select_builtin(theme);
        }

        self.heading(ui, i18n::MES_THEMES);
        self.custom_themes_section(ui, dark);
    }

    fn custom_themes_section(&mut self, ui: &mut egui::Ui, dark: bool) {
        let customs = self.themes.themes().to_vec();
        if customs.is_empty() {
            ui.label(egui::RichText::new(self.t(i18n::AUCUN_THEME_PERSONNEL)).weak());
        }
        let mut chosen: Option<String> = None;
        ui.horizontal_wrapped(|ui| {
            for custom in &customs {
                let selected = self.appearance.custom_id() == Some(custom.id.as_str());
                let palette = if dark { custom.dark } else { custom.light };
                let label = format!("{} {}", self.t(i18n::THEME), custom.name);
                if theme_card(ui, &label, &custom.name, &palette, selected).clicked() {
                    chosen = Some(custom.id.clone());
                }
            }
        });
        if let Some(id) = chosen {
            self.appearance.select_custom(&id);
        }

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.t(i18n::DUPLIQUER_LE_THEME_ACTIF)).clicked() {
                self.duplicate_active_theme();
            }
        });
        if let Some(id) = self.appearance.custom_id().map(str::to_owned) {
            self.active_custom_controls(ui, &id);
        } else {
            self.appearance_ui.confirm_delete = false;
        }
    }

    fn active_custom_controls(&mut self, ui: &mut egui::Ui, id: &str) {
        let Some(current) = self.themes.get(id).map(|t| t.name.clone()) else {
            return;
        };
        let buffer_is_stale = self
            .appearance_ui
            .name_buffer
            .as_ref()
            .is_none_or(|(buffer_id, _)| buffer_id != id);
        if buffer_is_stale {
            self.appearance_ui.name_buffer = Some((id.to_owned(), current.clone()));
        }
        let label = self.t(i18n::NOM_DU_THEME);
        let mut rename: Option<String> = None;
        let mut reset_buffer = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(label);
            if let Some((_, buffer)) = self.appearance_ui.name_buffer.as_mut() {
                let response = ui.add(
                    egui::TextEdit::singleline(buffer)
                        .char_limit(super::custom_themes::MAX_NAME_CHARS * 2),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label)
                });
                if response.lost_focus() {
                    if clean_name(buffer).is_ok() {
                        rename = Some(buffer.clone());
                    } else {
                        reset_buffer = true;
                    }
                }
            }
        });
        if let Some(raw) = rename {
            if let Ok(name) = self.themes.rename(id, &raw) {
                self.appearance_ui.name_buffer = Some((id.to_owned(), name));
            }
        }
        if reset_buffer {
            self.appearance_ui.name_buffer = Some((id.to_owned(), current));
        }

        ui.horizontal_wrapped(|ui| {
            if !self.appearance_ui.confirm_delete {
                if ui.button(self.t(i18n::SUPPRIMER_LE_THEME)).clicked() {
                    self.appearance_ui.confirm_delete = true;
                }
            } else {
                ui.label(self.t(i18n::SUPPRIMER_CE_THEME_QUESTION));
                if ui.button(self.t(i18n::CONFIRMER)).clicked() {
                    self.delete_custom_theme(id);
                }
                if ui.button(self.t(i18n::ANNULER)).clicked() {
                    self.appearance_ui.confirm_delete = false;
                }
            }
        });
    }

    fn delete_custom_theme(&mut self, id: &str) {
        self.appearance_ui.confirm_delete = false;
        self.appearance_ui.name_buffer = None;
        if let Some(base) = self.themes.delete(id) {
            if self.appearance.custom_id() == Some(id) {
                self.appearance.select_builtin(base);
            }
        }
    }

    fn duplicate_active_theme(&mut self) {
        let id = match self.appearance.custom_id().map(str::to_owned) {
            Some(id) => self.themes.duplicate(&id),
            None => {
                let base = self.appearance.builtin();
                let name = self.t(base.name());
                Some(self.themes.create(base, &base.resolve(), name))
            }
        };
        if let Some(id) = id {
            self.appearance.select_custom(&id);
        }
    }

    // ── Texte ─────────────────────────────────────────────────────────────

    fn text_tab(&mut self, ui: &mut egui::Ui) {
        let metrics = theme::text(ui);
        let theme_line_multiple = metrics
            .msg_line_height
            .map_or(1.2, |height| height / metrics.msg_body);
        let theme_tracking = metrics.msg_tracking;
        let narrow = ui.available_width() < 520.0;
        ui.spacing_mut().slider_width = if narrow {
            (ui.available_width() - 80.0).max(80.0)
        } else {
            240.0
        };

        let labels = TextLabels {
            overall: self.t(i18n::TAILLE_GENERALE),
            interface: self.t(i18n::TAILLE_DE_L_INTERFACE),
            conversations: self.t(i18n::TAILLE_DES_CONVERSATIONS),
            line: self.t(i18n::INTERLIGNE),
            line_theme: self.t(i18n::INTERLIGNE_DU_THEME),
            letter: self.t(i18n::ESPACEMENT_DES_LETTRES),
            letter_theme: self.t(i18n::ESPACEMENT_DU_THEME),
        };
        let (help, interface_title, conversations_title, back) = (
            self.t(i18n::TAILLE_GENERALE_AIDE),
            self.t(i18n::SECTION_INTERFACE),
            self.t(i18n::SECTION_CONVERSATIONS),
            self.t(i18n::REVENIR_AUX_TAILLES_DU_THEME),
        );

        let s = &mut self.appearance.text;
        ui.add_space(6.0);
        percent_slider(ui, labels.overall, &mut s.global_scale);
        ui.label(egui::RichText::new(help).weak());

        ui.add_space(10.0);
        ui.label(egui::RichText::new(interface_title).strong());
        percent_slider(ui, labels.interface, &mut s.ui_scale);

        ui.add_space(10.0);
        ui.label(egui::RichText::new(conversations_title).strong());
        percent_slider(ui, labels.conversations, &mut s.conv_scale);
        optional_slider(
            ui,
            labels.line_theme,
            labels.line,
            &mut s.conv_line_spacing,
            theme_line_multiple,
            LINE_SPACING_RANGE,
            |v| format!("{v:.2} ×"),
        );
        optional_slider(
            ui,
            labels.letter_theme,
            labels.letter,
            &mut s.conv_letter_spacing,
            theme_tracking,
            LETTER_SPACING_RANGE,
            |v| format!("{v:.2} pt"),
        );

        ui.add_space(8.0);
        if ui.button(back).clicked() {
            *s = TextSettings::default();
        }
        ui.add_space(8.0);
        self.appearance_preview(ui);
    }

    // ── Couleurs ──────────────────────────────────────────────────────────

    fn colors_tab(&mut self, ui: &mut egui::Ui) {
        let dark = ui.visuals().dark_mode;
        let palette = theme::palette(ui);
        let french = self.ui_language == super::UiLanguage::French;

        let theme_name = match self
            .appearance
            .custom_id()
            .and_then(|id| self.themes.get(id))
        {
            Some(custom) => custom.name.clone(),
            None => self.t(self.appearance.builtin().name()).to_owned(),
        };
        ui.add_space(6.0);
        ui.label(egui::RichText::new(&theme_name).strong());
        if self.appearance.custom_id().is_none() {
            ui.label(
                egui::RichText::new(format!("{} {theme_name}", self.t(i18n::COPIE_PERSO_DE)))
                    .weak(),
            );
        }
        ui.label(
            egui::RichText::new(self.t(if dark {
                i18n::MODIFICATION_DU_MODE_SOMBRE
            } else {
                i18n::MODIFICATION_DU_MODE_CLAIR
            }))
            .weak(),
        );
        ui.add_space(6.0);

        let base_palette = self.base_palette(dark);
        let mut edits: Vec<(Role, Color32)> = Vec::new();
        for role in Role::MAIN {
            self.color_row(ui, role, dark, &palette, base_palette.as_ref(), &mut edits);
        }

        let ratio = contrast_ratio(palette.text, palette.background);
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(self.t(i18n::CONTRASTE_DU_TEXTE));
            ui.label(egui::RichText::new(format_ratio(ratio, french)).strong());
        });
        if ratio < MIN_TEXT_CONTRAST {
            ui.label(
                egui::RichText::new(self.t(i18n::CONTRASTE_INSUFFISANT)).color(palette.danger),
            );
        }

        ui.add_space(6.0);
        egui::CollapsingHeader::new(self.t(i18n::OPTIONS_AVANCEES))
            .id_salt("appearance_advanced")
            .show(ui, |ui| {
                for role in Role::ADVANCED {
                    self.color_row(ui, role, dark, &palette, base_palette.as_ref(), &mut edits);
                }
            });

        for (role, color) in edits {
            self.edit_color(role, dark, color);
        }
        ui.add_space(8.0);
        self.appearance_preview(ui);
    }

    /// Palette du thème dont dérive le thème personnel actif, pour « Par défaut ».
    fn base_palette(&self, dark: bool) -> Option<Palette> {
        let custom = self.themes.get(self.appearance.custom_id()?)?;
        Some(custom.base.resolve().palette(dark))
    }

    fn color_row(
        &mut self,
        ui: &mut egui::Ui,
        role: Role,
        dark: bool,
        palette: &Palette,
        base: Option<&Palette>,
        edits: &mut Vec<(Role, Color32)>,
    ) {
        let name = self.t(role.label());
        let current = role.get(palette);
        let hex_label = format!("{name} : {}", self.t(i18n::CODE_COULEUR));
        let default_label = format!("{name} : {}", self.t(i18n::PAR_DEFAUT_POUR));
        ui.horizontal_wrapped(|ui| {
            ui.label(name);
            let mut rgb = [current.r(), current.g(), current.b()];
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                edits.push((role, Color32::from_rgb(rgb[0], rgb[1], rgb[2])));
            }
            let id = egui::Id::new(("appearance_hex", role, dark));
            let focused = ui.memory(|m| m.has_focus(id));
            let buffer = self
                .appearance_ui
                .hex_buffers
                .entry((role, dark))
                .or_default();
            if !focused {
                *buffer = hex_of(current);
            }
            let response = ui.add(
                egui::TextEdit::singleline(buffer)
                    .id(id)
                    .desired_width(78.0)
                    .char_limit(8),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, &hex_label)
            });
            if response.changed() {
                if let Some(color) = parse_hex(buffer) {
                    edits.push((role, color));
                }
            }
            let base_color = base.map(|b| role.get(b));
            let differs = base_color.is_some_and(|b| b != current);
            let button = ui.add_enabled(differs, egui::Button::new(self.t(i18n::PAR_DEFAUT)));
            button.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, differs, &default_label)
            });
            if button.clicked() {
                if let Some(color) = base_color {
                    edits.push((role, color));
                }
            }
        });
    }

    /// Applique une couleur ; toucher un thème intégré en crée d'abord une copie.
    pub(crate) fn edit_color(&mut self, role: Role, dark: bool, color: Color32) {
        let id = match self.appearance.custom_id().map(str::to_owned) {
            Some(id) => id,
            None => {
                let base = self.appearance.builtin();
                let name = format!(
                    "{} ({})",
                    self.t(base.name()),
                    self.t(i18n::THEME_PERSO_SUFFIXE)
                );
                let id = self.themes.create(base, &base.resolve(), &name);
                self.appearance.select_custom(&id);
                self.notify(self.t(i18n::COPIE_PERSONNELLE_CREEE));
                id
            }
        };
        self.themes.update(&id, |theme| {
            role.set(theme.palette_mut(dark), color);
        });
    }

    // ── Aperçu et réinitialisation ────────────────────────────────────────

    fn appearance_preview(&self, ui: &mut egui::Ui) {
        let palette = theme::palette(ui);
        ui.label(egui::RichText::new(self.t(i18n::APERCU)).strong());
        egui::Frame::NONE
            .fill(palette.background)
            .stroke(egui::Stroke::new(1.0, palette.separator))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    egui::Frame::NONE
                        .fill(palette.surface)
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Alice").color(palette.accent));
                                ui.label(egui::RichText::new("Bob").color(palette.text_muted));
                            });
                        });
                    ui.vertical(|ui| {
                        let text = self.t(i18n::APERCU_MESSAGE);
                        let parsed = markdown::parse_message(text, &self.emoji.map);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Alice")
                                    .text_style(theme::msg_style())
                                    .color(palette.accent)
                                    .family(egui::FontFamily::Name(theme::MSG_BOLD_FAMILY.into())),
                            );
                            ui.label(
                                egui::RichText::new("12:00")
                                    .text_style(theme::msg_small_style())
                                    .color(palette.text_muted),
                            );
                        });
                        markdown::render_parsed_markdown(
                            ui,
                            &parsed,
                            &self.emoji.map,
                            &self.emoji.textures,
                        );
                    });
                });
            });
    }

    fn reset_footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if !self.appearance_ui.confirm_reset {
                if ui.button(self.t(i18n::REINITIALISER_L_APPARENCE)).clicked() {
                    self.appearance_ui.confirm_reset = true;
                }
            } else {
                ui.label(self.t(i18n::REINITIALISER_QUESTION));
                if ui.button(self.t(i18n::CONFIRMER)).clicked() {
                    self.appearance = Appearance::default();
                    self.appearance_ui.confirm_reset = false;
                }
                if ui.button(self.t(i18n::ANNULER)).clicked() {
                    self.appearance_ui.confirm_reset = false;
                }
            }
        });
    }
}

struct TextLabels {
    overall: &'static str,
    interface: &'static str,
    conversations: &'static str,
    line: &'static str,
    line_theme: &'static str,
    letter: &'static str,
    letter_theme: &'static str,
}

/// Curseur de taille en pourcentage.
fn percent_slider(ui: &mut egui::Ui, label: &'static str, value: &mut f32) {
    ui.add(
        egui::Slider::new(value, SCALE_RANGE)
            .step_by(0.05)
            .custom_formatter(|v, _| format!("{:.0} %", v * 100.0))
            .custom_parser(|s| {
                s.trim()
                    .trim_end_matches('%')
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .map(|p| p / 100.0)
            })
            .text(label),
    );
}

/// Réglage qui garde la valeur du thème tant qu'on ne le touche pas.
fn optional_slider(
    ui: &mut egui::Ui,
    theme_label: &'static str,
    label: &'static str,
    value: &mut Option<f32>,
    theme_value: f32,
    range: std::ops::RangeInclusive<f32>,
    format: impl Fn(f64) -> String + 'static,
) {
    let mut follows_theme = value.is_none();
    if ui.checkbox(&mut follows_theme, theme_label).changed() {
        *value = if follows_theme {
            None
        } else {
            Some(theme_value.clamp(*range.start(), *range.end()))
        };
    }
    let mut shown = value
        .unwrap_or(theme_value)
        .clamp(*range.start(), *range.end());
    let response = ui.add_enabled(
        value.is_some(),
        egui::Slider::new(&mut shown, range)
            .step_by(0.05)
            .custom_formatter(move |v, _| format(v))
            .text(label),
    );
    if response.changed() {
        *value = Some(shown);
    }
}

/// Carte de la galerie : miniature du thème et son nom.
fn theme_card(
    ui: &mut egui::Ui,
    accessible_label: &str,
    name: &str,
    palette: &Palette,
    selected: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(CARD_WIDTH, CARD_HEIGHT), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            true,
            selected,
            accessible_label,
        )
    });
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let round = egui::CornerRadius::same(8);
        painter.rect_filled(rect, round, palette.background);
        let preview = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.max.x, rect.max.y - CARD_NAME_HEIGHT),
        );
        let sidebar = egui::Rect::from_min_max(
            preview.min,
            egui::pos2(preview.min.x + preview.width() * 0.28, preview.max.y),
        );
        painter.rect_filled(
            sidebar,
            egui::CornerRadius {
                nw: 8,
                sw: 0,
                ne: 0,
                se: 0,
            },
            palette.surface,
        );
        let bubble = egui::Rect::from_min_max(
            egui::pos2(sidebar.max.x + 8.0, preview.min.y + 12.0),
            egui::pos2(preview.max.x - 8.0, preview.min.y + 38.0),
        );
        painter.rect_filled(bubble, egui::CornerRadius::same(6), palette.surface_strong);
        painter.text(
            bubble.left_center() + egui::vec2(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "Aa",
            egui::FontId::proportional(14.0),
            palette.text,
        );
        painter.circle_filled(
            egui::pos2(bubble.right() - 10.0, bubble.center().y),
            4.0,
            palette.accent,
        );
        let line = egui::Rect::from_min_size(
            egui::pos2(bubble.min.x, bubble.max.y + 8.0),
            egui::vec2(bubble.width() * 0.6, 5.0),
        );
        painter.rect_filled(line, egui::CornerRadius::same(2), palette.text_muted);
        let name_band = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, rect.max.y - CARD_NAME_HEIGHT),
            rect.max,
        );
        painter.rect_filled(
            name_band,
            egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: 8,
                se: 8,
            },
            palette.surface,
        );
        let shown: String = if name.chars().count() > 20 {
            format!("{}…", name.chars().take(19).collect::<String>())
        } else {
            name.to_owned()
        };
        painter.text(
            name_band.center(),
            egui::Align2::CENTER_CENTER,
            shown,
            egui::FontId::proportional(12.0),
            palette.text,
        );
        let stroke = if selected {
            egui::Stroke::new(2.5, palette.accent)
        } else if response.hovered() {
            egui::Stroke::new(1.5, palette.text_muted)
        } else {
            egui::Stroke::new(1.0, palette.separator)
        };
        painter.rect_stroke(rect, round, stroke, egui::StrokeKind::Inside);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
#[path = "../tests/test_ui_appearance_tab.rs"]
mod tests;
