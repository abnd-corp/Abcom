use eframe::egui;

use super::*;
use crate::ui::theme::{self, FontKind, TextMetrics, Typography};
use crate::ui::themes::BuiltinTheme;

fn settings(global: f32, ui: f32, conv: f32) -> TextSettings {
    TextSettings {
        global_scale: global,
        ui_scale: ui,
        conv_scale: conv,
        ..TextSettings::default()
    }
}

fn metrics(settings: &TextSettings) -> TextMetrics {
    TextMetrics::resolve(&Typography::CLASSIC, settings)
}

#[test]
fn defaults_are_neutral() {
    let d = TextSettings::default();
    assert_eq!((d.global_scale, d.ui_scale, d.conv_scale), (1.0, 1.0, 1.0));
    assert_eq!((d.conv_line_spacing, d.conv_letter_spacing), (None, None));
    assert_eq!(d.sanitized(), d);
}

#[test]
fn sanitized_clamps_out_of_range_values() {
    let wild = TextSettings {
        global_scale: 9.0,
        ui_scale: 0.0,
        conv_scale: -3.0,
        conv_line_spacing: Some(50.0),
        conv_letter_spacing: Some(-50.0),
    }
    .sanitized();
    assert_eq!(wild.global_scale, 1.6);
    assert_eq!(wild.ui_scale, 0.8);
    assert_eq!(wild.conv_scale, 0.8);
    assert_eq!(wild.conv_line_spacing, Some(2.0));
    assert_eq!(wild.conv_letter_spacing, Some(-0.5));
}

#[test]
fn sanitized_replaces_nan_and_infinity() {
    let broken = TextSettings {
        global_scale: f32::NAN,
        ui_scale: f32::INFINITY,
        conv_scale: f32::NEG_INFINITY,
        conv_line_spacing: Some(f32::NAN),
        conv_letter_spacing: Some(f32::INFINITY),
    }
    .sanitized();
    assert_eq!(broken, TextSettings::default());
}

#[test]
fn json_round_trip_and_tolerance() {
    let original = TextSettings {
        global_scale: 1.25,
        conv_line_spacing: Some(1.4),
        ..TextSettings::default()
    };
    let json = serde_json::to_string(&original).unwrap();
    assert_eq!(
        serde_json::from_str::<TextSettings>(&json).unwrap(),
        original
    );
    // Champs manquants, champs inconnus d'une version future.
    let partial: TextSettings = serde_json::from_str(r#"{"ui_scale":1.2,"futur":true}"#).unwrap();
    assert_eq!(partial.ui_scale, 1.2);
    assert_eq!(partial.global_scale, 1.0);
    // Un mauvais type est une erreur, que l'appelant ramène au défaut.
    assert!(serde_json::from_str::<TextSettings>(r#"{"global_scale":"grand"}"#).is_err());
}

#[test]
fn classic_metrics_are_the_egui_defaults() {
    let m = metrics(&TextSettings::default());
    assert_eq!(
        (m.small, m.body, m.button, m.heading, m.monospace),
        (9.0, 13.0, 13.0, 18.0, 13.0)
    );
    assert_eq!((m.msg_small, m.msg_body, m.msg_mono), (9.0, 13.0, 13.0));
    assert_eq!(m.composer_line_height, 22.0);
    assert_eq!(m.msg_line_height, None);
    assert_eq!(m.msg_tracking, 0.0);
    assert_eq!(m.dark_text_gamma, None);
}

#[test]
fn global_scale_moves_the_interface_and_the_conversations_together() {
    let m = metrics(&settings(1.5, 1.0, 1.0));
    assert_eq!(m.body, 19.5);
    assert_eq!(m.msg_body, 19.5);
    assert_eq!(m.heading, 27.0);
    assert_eq!(m.msg_mono, 19.5);
}

#[test]
fn interface_and_conversation_scales_are_independent() {
    let m = metrics(&settings(1.0, 0.8, 1.4));
    assert!((m.body - 10.4).abs() < 1e-4);
    assert!((m.small - 7.2).abs() < 1e-4);
    assert!((m.msg_body - 18.2).abs() < 1e-4);
    assert!((m.msg_small - 12.6).abs() < 1e-4);
    // Le seul réglage de l'interface ne touche pas les conversations.
    let only_ui = metrics(&settings(1.0, 1.5, 1.0));
    assert_eq!(only_ui.msg_body, 13.0);
    let only_conv = metrics(&settings(1.0, 1.0, 1.5));
    assert_eq!(only_conv.body, 13.0);
}

#[test]
fn combined_scale_is_bounded() {
    let huge = metrics(&settings(1.6, 1.6, 1.6));
    assert_eq!(huge.body, 13.0 * 2.0, "2,56 est ramené à 2,0");
    let tiny = metrics(&settings(0.8, 0.8, 0.8));
    assert!(
        (tiny.body - 13.0 * 0.7).abs() < 1e-4,
        "0,64 est ramené à 0,7"
    );
}

#[test]
fn line_spacing_is_a_multiple_of_the_body_size() {
    let custom = TextSettings {
        conv_line_spacing: Some(1.5),
        ..TextSettings::default()
    };
    assert_eq!(metrics(&custom).msg_line_height, Some(13.0 * 1.5));
    let scaled = TextSettings {
        conv_scale: 1.2,
        conv_line_spacing: Some(1.5),
        ..TextSettings::default()
    };
    assert!((metrics(&scaled).msg_line_height.unwrap() - 13.0 * 1.2 * 1.5).abs() < 1e-4);
}

#[test]
fn the_theme_line_height_follows_the_conversation_scale() {
    let m = TextMetrics::resolve(&Typography::LOUNGE, &settings(1.0, 1.0, 1.5));
    assert_eq!(m.msg_line_height, Some(30.0));
    let unchanged = TextMetrics::resolve(&Typography::LOUNGE, &TextSettings::default());
    assert_eq!(unchanged.msg_line_height, Some(20.0));
}

#[test]
fn letter_spacing_overrides_the_theme() {
    let custom = TextSettings {
        conv_letter_spacing: Some(0.4),
        ..TextSettings::default()
    };
    assert_eq!(
        TextMetrics::resolve(&Typography::READABLE, &custom).msg_tracking,
        0.4
    );
    assert_eq!(
        TextMetrics::resolve(&Typography::READABLE, &TextSettings::default()).msg_tracking,
        0.15
    );
}

#[test]
fn composer_line_height_never_drops_below_the_classic_value() {
    let small = metrics(&settings(0.8, 0.8, 0.8));
    assert_eq!(small.composer_line_height, 22.0);
    let big = metrics(&settings(1.0, 1.0, 1.5));
    assert_eq!(big.composer_line_height, 33.0);
}

#[test]
fn apply_installs_classic_styles_identical_to_egui_defaults() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Classic, &TextSettings::default());
    let style = ctx.style_of(egui::Theme::Dark);
    for (text_style, size) in [
        (egui::TextStyle::Small, 9.0),
        (egui::TextStyle::Body, 13.0),
        (egui::TextStyle::Button, 13.0),
        (egui::TextStyle::Heading, 18.0),
        (egui::TextStyle::Monospace, 13.0),
    ] {
        assert_eq!(style.text_styles[&text_style].size, size, "{text_style:?}");
    }
    assert_eq!(style.visuals.panel_fill, egui::Color32::from_gray(27));
    assert_eq!(style.visuals.extreme_bg_color, egui::Color32::from_gray(10));
    assert_eq!(
        style.visuals.widgets.noninteractive.fg_stroke.color,
        egui::Visuals::dark().widgets.noninteractive.fg_stroke.color,
        "Classique laisse les couleurs de widgets d'egui"
    );
    let light = ctx.style_of(egui::Theme::Light);
    assert_eq!(light.visuals.panel_fill, egui::Color32::from_gray(248));
}

#[test]
fn apply_defines_the_conversation_text_styles() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Classic, &settings(1.0, 1.0, 1.5));
    let style = ctx.style_of(egui::Theme::Dark);
    let msg = &style.text_styles[&theme::msg_style()];
    assert_eq!(msg.size, 19.5);
    assert_eq!(msg.family, egui::FontFamily::Name(theme::MSG_FAMILY.into()));
    assert_eq!(style.text_styles[&theme::msg_small_style()].size, 13.5);
    let mono = &style.text_styles[&theme::msg_mono_style()];
    assert_eq!(mono.size, 19.5);
    assert_eq!(mono.family, egui::FontFamily::Monospace);
    assert_eq!(style.text_styles[&egui::TextStyle::Body].size, 13.0);
}

#[test]
fn apply_registers_the_message_font_families() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Lounge, &TextSettings::default());
    // `set_fonts` ne prend effet qu'à la frame suivante.
    let mut output = ctx.run_ui(Default::default(), |_| {});
    output.textures_delta.clear();
    let families: Vec<_> = ctx.fonts(|f| f.families());
    for name in [
        theme::MSG_FAMILY,
        theme::MSG_BOLD_FAMILY,
        theme::MSG_ITALIC_FAMILY,
    ] {
        assert!(
            families.contains(&egui::FontFamily::Name(name.into())),
            "famille {name} absente"
        );
    }
}

#[test]
fn apply_bumps_the_revision_every_time() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Classic, &TextSettings::default());
    let first = theme::current(&ctx).revision;
    apply(&ctx, BuiltinTheme::Classic, &TextSettings::default());
    apply(&ctx, BuiltinTheme::Frost, &TextSettings::default());
    assert_eq!(theme::current(&ctx).revision, first + 2);
}

#[test]
fn fonts_are_only_rebuilt_when_the_font_choice_changes() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Classic, &TextSettings::default());
    assert!(!font_choice_changed(
        &ctx,
        (FontKind::NotoSans, FontKind::NotoSans)
    ));
    // Couleurs, tailles et autre thème de même police : pas de reconstruction.
    apply(&ctx, BuiltinTheme::Frost, &settings(1.4, 1.0, 1.0));
    assert!(!font_choice_changed(
        &ctx,
        (FontKind::NotoSans, FontKind::NotoSans)
    ));
    apply(&ctx, BuiltinTheme::Lounge, &TextSettings::default());
    assert!(font_choice_changed(
        &ctx,
        (FontKind::NotoSans, FontKind::NotoSans)
    ));
    assert!(!font_choice_changed(
        &ctx,
        (FontKind::Inter, FontKind::Inter)
    ));
}

#[test]
fn switching_back_to_classic_leaves_nothing_of_the_previous_theme() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Lounge, &TextSettings::default());
    let styled = ctx.style_of(egui::Theme::Dark);
    assert_ne!(
        styled.visuals.widgets.inactive.bg_fill,
        egui::Visuals::dark().widgets.inactive.bg_fill
    );
    assert_eq!(
        styled.visuals.text_options.color_transfer_function,
        egui::epaint::FontColorTransferFunction::Gamma(1.0)
    );
    apply(&ctx, BuiltinTheme::Classic, &TextSettings::default());
    let back = ctx.style_of(egui::Theme::Dark);
    assert_eq!(
        back.visuals.widgets.inactive.bg_fill,
        egui::Visuals::dark().widgets.inactive.bg_fill
    );
    assert_eq!(
        back.visuals.hyperlink_color,
        egui::Visuals::dark().hyperlink_color
    );
    assert_eq!(
        back.visuals.text_options.color_transfer_function,
        egui::Visuals::dark().text_options.color_transfer_function
    );
}

#[test]
fn derived_themes_paint_widgets_with_their_palette() {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Frost, &TextSettings::default());
    let resolved = theme::current(&ctx);
    let style = ctx.style_of(egui::Theme::Dark);
    assert_eq!(style.visuals.panel_fill, resolved.dark.background);
    assert_eq!(style.visuals.extreme_bg_color, resolved.dark.input_bg);
    assert_eq!(style.visuals.hyperlink_color, resolved.dark.link);
    assert_eq!(
        style.visuals.widgets.noninteractive.fg_stroke.color,
        resolved.dark.text
    );
    let light = ctx.style_of(egui::Theme::Light);
    assert_eq!(light.visuals.panel_fill, resolved.light.background);
}

#[test]
fn out_of_range_settings_are_bounded_before_use() {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        BuiltinTheme::Classic,
        &TextSettings {
            global_scale: f32::NAN,
            ui_scale: 99.0,
            ..TextSettings::default()
        },
    );
    let body = ctx.style_of(egui::Theme::Dark).text_styles[&egui::TextStyle::Body].size;
    assert!(
        body.is_finite() && (13.0 * 0.7..=13.0 * 2.0).contains(&body),
        "{body}"
    );
}

/// Largeur d'un texte posé tel quel, puis posé comme corps d'un message.
fn widths(settings: TextSettings) -> (f32, f32) {
    let ctx = egui::Context::default();
    apply(&ctx, BuiltinTheme::Classic, &settings);
    let (mut interface, mut message) = (0.0, 0.0);
    let emoji_map = std::collections::HashMap::new();
    let textures = crate::ui::EmojiTextures::default();
    let parsed = crate::ui::markdown::parse_message("Bonjour tout le monde", &emoji_map);
    // La première frame charge les polices ; la mesure se fait à la suivante.
    for _ in 0..2 {
        let mut output = ctx.run_ui(Default::default(), |ui| {
            interface = ui.label("Bonjour tout le monde").rect.width();
            let before = ui.min_rect().height();
            crate::ui::markdown::render_parsed_markdown(ui, &parsed, &emoji_map, &textures);
            message = ui.min_rect().height() - before;
        });
        output.textures_delta.clear();
    }
    (interface, message)
}

#[test]
fn message_text_follows_the_conversation_scale_and_not_the_interface_one() {
    let (interface_base, message_base) = widths(TextSettings::default());
    let (interface_big, message_big) = widths(settings(1.0, 1.0, 1.6));
    assert_eq!(interface_big, interface_base, "l'interface ne bouge pas");
    assert!(
        message_big > message_base * 1.3,
        "le message grossit : {message_base} → {message_big}"
    );
    let (interface_ui, message_ui) = widths(settings(1.0, 1.6, 1.0));
    assert!(interface_ui > interface_base * 1.3);
    assert_eq!(message_ui, message_base, "les messages ne bougent pas");
}

#[test]
fn emoji_sizes_never_depend_on_the_text_settings() {
    // Les tailles d'emoji sont des constantes du rendu : elles ne figurent pas dans les métriques.
    let a = metrics(&TextSettings::default());
    let b = metrics(&settings(1.6, 1.6, 1.6));
    assert_ne!(a, b, "le texte, lui, change");
    assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
}
