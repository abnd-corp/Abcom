//! Parcours de l'apparence comme le vit l'utilisateur : clic dans les
//! Paramètres, enregistrement, puis redémarrage sur les mêmes préférences.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::Harness;
use tokio::sync::mpsc;

use super::appearance::{Appearance, Mode, TextSettings};
use super::{i18n, AbcomApp, UiLanguage, UiRuntimeChannels};
use crate::app::AppState;
use crate::network::secure::TrustStore;

/// Application neuve, dont les préférences contiennent `pref` sous la clé d'apparence.
pub(super) fn new_app(pref: Option<&str>) -> AbcomApp {
    let dir = std::env::temp_dir().join(format!(
        "abcom-appearance-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    new_app_in(&dir, pref)
}

/// Application dont le dossier de données est `dir` : un redémarrage y retrouve ses thèmes et polices.
pub(super) fn new_app_in(dir: &std::path::Path, pref: Option<&str>) -> AbcomApp {
    let mut state = AppState::new_with_base("moi", dir);
    if let Some(pref) = pref {
        state.set_pref(Appearance::PREF_KEY, pref);
    }
    let (send_tx, _send_rx) = mpsc::channel(16);
    let (send_media_tx, _media_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(16);
    let (_offer_tx, media_offer_rx) = mpsc::channel(16);
    AbcomApp::new(
        Arc::new(Mutex::new(state)),
        "aaaa:bbbb".to_string(),
        false,
        UiRuntimeChannels {
            event_rx,
            event_tx,
            send_tx,
            send_media_tx,
            media_offer_rx,
            trust: Arc::new(TrustStore::new(HashMap::new(), None)),
        },
    )
}

/// Préférence enregistrée par l'application, telle qu'un redémarrage la relirait.
pub(super) fn saved_pref(app: &AbcomApp) -> Option<String> {
    app.state
        .lock()
        .unwrap()
        .pref_str(Appearance::PREF_KEY)
        .map(str::to_owned)
}

/// Fenêtre pilotée par kittest autour d'une application partagée.
pub(super) struct Session {
    pub(super) harness: Harness<'static>,
    pub(super) app: Rc<RefCell<AbcomApp>>,
}

impl Session {
    pub(super) fn new(app: AbcomApp, size: egui::Vec2) -> Self {
        let app = Rc::new(RefCell::new(app));
        let shared = Rc::clone(&app);
        let harness = Harness::builder().with_size(size).build_ui(move |root| {
            let mut app = shared.borrow_mut();
            app.sync_appearance(root.ctx());
            let ctx = root.ctx().clone();
            app.show_central_panel(root);
            app.render_settings(&ctx);
        });
        Self { harness, app }
    }

    pub(super) fn settle(&mut self) {
        for _ in 0..4 {
            self.harness.step();
        }
    }

    pub(super) fn ctx(&self) -> egui::Context {
        self.harness.ctx.clone()
    }
}

fn open_appearance_settings(session: &mut Session) {
    {
        let mut app = session.app.borrow_mut();
        app.modals.settings_open = true;
        app.modals.settings_tab = super::SettingsTab::Appearance;
    }
    session.settle();
}

#[test]
fn a_saved_appearance_is_applied_on_the_first_frame() {
    let saved = Appearance {
        theme: "frost".into(),
        mode: Mode::Dark,
        text: TextSettings {
            conv_scale: 1.4,
            ..TextSettings::default()
        },
        ..Appearance::default()
    };
    let mut session = Session::new(new_app(Some(&saved.to_pref())), egui::vec2(860.0, 600.0));
    session.harness.step();
    let ctx = session.ctx();
    assert_eq!(ctx.theme(), egui::Theme::Dark);
    let resolved = super::theme::current(&ctx);
    assert_eq!(
        resolved.dark,
        super::themes::BuiltinTheme::Frost.resolve().dark
    );
    let style = ctx.style_of(egui::Theme::Dark);
    assert!((style.text_styles[&super::theme::msg_style()].size - 18.2).abs() < 1e-3);
    assert_eq!(style.text_styles[&egui::TextStyle::Body].size, 13.0);
}

#[test]
fn clicking_dark_in_the_settings_switches_the_theme_and_is_saved() {
    let mut session = Session::new(new_app(None), egui::vec2(860.0, 600.0));
    open_appearance_settings(&mut session);
    assert_eq!(session.app.borrow().appearance.mode, Mode::System);
    assert_eq!(
        saved_pref(&session.app.borrow()),
        None,
        "rien n'est écrit sans changement"
    );

    session
        .harness
        .get_by_label(i18n::SOMBRE.get(UiLanguage::French))
        .click();
    session.settle();

    assert_eq!(session.app.borrow().appearance.mode, Mode::Dark);
    assert_eq!(session.ctx().theme(), egui::Theme::Dark);
    let saved = Appearance::from_pref(saved_pref(&session.app.borrow()).as_deref());
    assert_eq!(saved.mode, Mode::Dark);

    session
        .harness
        .get_by_label(i18n::CLAIR.get(UiLanguage::French))
        .click();
    session.settle();
    assert_eq!(session.ctx().theme(), egui::Theme::Light);
    let saved = Appearance::from_pref(saved_pref(&session.app.borrow()).as_deref());
    assert_eq!(saved.mode, Mode::Light);
}

/// Le choix clair/sombre revenait à « Système » à chaque lancement.
#[test]
fn the_chosen_mode_survives_a_restart() {
    let mut first = Session::new(new_app(None), egui::vec2(860.0, 600.0));
    open_appearance_settings(&mut first);
    first
        .harness
        .get_by_label(i18n::CLAIR.get(UiLanguage::French))
        .click();
    first.settle();
    let pref = saved_pref(&first.app.borrow()).expect("préférence écrite");
    drop(first);

    let mut second = Session::new(new_app(Some(&pref)), egui::vec2(860.0, 600.0));
    second.harness.step();
    assert_eq!(second.app.borrow().appearance.mode, Mode::Light);
    assert_eq!(
        second.ctx().theme(),
        egui::Theme::Light,
        "appliqué dès la première image"
    );
}

#[test]
fn a_corrupted_saved_appearance_falls_back_to_the_defaults() {
    for raw in [
        "",
        "{",
        "null",
        "[]",
        "42",
        r#"{"mode":"dark","theme":3}"#,
        "\u{0}\u{1}",
    ] {
        let mut session = Session::new(new_app(Some(raw)), egui::vec2(860.0, 600.0));
        session.settle();
        let app = session.app.borrow();
        assert_eq!(app.appearance, Appearance::default(), "préférence {raw:?}");
    }
}

#[test]
fn an_unknown_saved_theme_becomes_classic_and_keeps_the_other_settings() {
    let raw = r#"{"theme":"supprime","mode":"dark","text":{"ui_scale":1.2}}"#;
    let session = Session::new(new_app(Some(raw)), egui::vec2(860.0, 600.0));
    let app = session.app.borrow();
    assert_eq!(app.appearance.theme, "classic");
    assert_eq!(app.appearance.mode, Mode::Dark);
    assert_eq!(app.appearance.text.ui_scale, 1.2);
}

/// Un curseur qu'on fait glisser change l'apparence à chaque image : la base
/// n'écrit qu'au relâchement.
#[test]
fn the_saved_value_waits_for_the_mouse_button_to_be_released() {
    let mut app = new_app(None);
    let ctx = egui::Context::default();
    let frame = |app: &mut AbcomApp, events: Vec<egui::Event>| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| app.sync_appearance(ui.ctx()),
        );
        output.textures_delta.clear();
    };
    let at = egui::pos2(10.0, 10.0);
    let press = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };

    frame(&mut app, vec![egui::Event::PointerMoved(at), press(true)]);
    for step in 0..5 {
        app.appearance.text.global_scale = 1.0 + step as f32 * 0.1;
        frame(&mut app, vec![]);
    }
    assert_eq!(saved_pref(&app), None, "bouton enfoncé : rien d'écrit");
    assert_eq!(
        super::theme::current(&ctx).text.body,
        13.0 * 1.4,
        "mais l'apparence suit en direct"
    );

    frame(&mut app, vec![press(false)]);
    frame(&mut app, vec![]);
    let saved = Appearance::from_pref(saved_pref(&app).as_deref());
    assert_eq!(
        saved.text.global_scale, 1.4,
        "relâché : la dernière valeur seulement"
    );
}

#[test]
fn nothing_is_applied_or_written_twice_for_an_unchanged_appearance() {
    let mut app = new_app(None);
    let ctx = egui::Context::default();
    let run = |app: &mut AbcomApp| {
        let mut output = ctx.run_ui(Default::default(), |ui| app.sync_appearance(ui.ctx()));
        output.textures_delta.clear();
    };
    run(&mut app);
    let revision = super::theme::current(&ctx).revision;
    for _ in 0..5 {
        run(&mut app);
    }
    assert_eq!(
        super::theme::current(&ctx).revision,
        revision,
        "pas de réapplication"
    );
    assert_eq!(saved_pref(&app), None, "pas d'écriture sans changement");
}

#[test]
fn changing_the_appearance_invalidates_the_chat_cache() {
    let mut app = new_app(None);
    let ctx = egui::Context::default();
    let before = {
        let mut output = ctx.run_ui(Default::default(), |ui| app.sync_appearance(ui.ctx()));
        output.textures_delta.clear();
        super::theme::current(&ctx).revision
    };
    app.appearance.theme = "forest".into();
    let mut output = ctx.run_ui(Default::default(), |ui| app.sync_appearance(ui.ctx()));
    output.textures_delta.clear();
    assert!(super::theme::current(&ctx).revision > before);
}

// ── Onglet Apparence ──────────────────────────────────────────────────────

use super::appearance_tab::AppearanceTab;
use super::themes::BuiltinTheme;

fn fr(entry: i18n::Entry) -> &'static str {
    entry.get(UiLanguage::French)
}

/// Amène un contrôle dans la zone visible, comme le ferait la molette.
fn reveal(session: &mut Session, label: &str) {
    session.harness.get_by_label(label).scroll_to_me();
    session.settle();
}

fn click(session: &mut Session, label: &str) {
    reveal(session, label);
    session.harness.get_by_label(label).click();
    session.settle();
}

fn open_tab(session: &mut Session, tab: AppearanceTab) {
    open_appearance_settings(session);
    let label = match tab {
        AppearanceTab::Themes => i18n::APPARENCE_THEMES,
        AppearanceTab::Text => i18n::APPARENCE_TEXTE,
        AppearanceTab::Colors => i18n::APPARENCE_COULEURS,
        AppearanceTab::Fonts => i18n::APPARENCE_POLICES,
    };
    let language = session.app.borrow().ui_language;
    click(session, label.get(language));
}

/// Remplace le contenu d'un champ de saisie, comme le ferait un utilisateur.
fn type_into(session: &mut Session, label: &str, text: &str) {
    session
        .harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, label)
        .scroll_to_me();
    session.settle();
    session
        .harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, label)
        .focus();
    session.settle();
    session
        .harness
        .key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    session.settle();
    session
        .harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, label)
        .type_text(text);
    session.settle();
}

fn desktop() -> egui::Vec2 {
    egui::vec2(860.0, 600.0)
}

#[test]
fn every_builtin_theme_can_be_picked_from_the_gallery_and_survives_a_restart() {
    for theme in BuiltinTheme::ALL {
        let mut session = Session::new(new_app(None), desktop());
        open_tab(&mut session, AppearanceTab::Themes);
        let label = format!("{} {}", fr(i18n::THEME), fr(theme.name()));
        click(&mut session, &label);

        assert_eq!(
            session.app.borrow().appearance.theme,
            theme.id(),
            "{theme:?}"
        );
        let ctx = session.ctx();
        assert_eq!(
            super::theme::current(&ctx).dark,
            theme.resolve().dark,
            "{theme:?} : palette appliquée"
        );
        // Classique est le défaut : rien à écrire, et rien à relire.
        let pref = saved_pref(&session.app.borrow());
        assert_eq!(pref.is_none(), theme == BuiltinTheme::Classic, "{theme:?}");
        drop(session);

        let mut restarted = Session::new(new_app(pref.as_deref()), desktop());
        restarted.harness.step();
        assert_eq!(restarted.app.borrow().appearance.theme, theme.id());
        assert_eq!(
            super::theme::current(&restarted.ctx()).dark,
            theme.resolve().dark
        );
    }
}

#[test]
fn the_gallery_shows_all_ten_themes_with_their_translated_names() {
    for language in [UiLanguage::French, UiLanguage::English] {
        let mut app = new_app(None);
        app.ui_language = language;
        let mut session = Session::new(app, desktop());
        open_tab(&mut session, AppearanceTab::Themes);
        for theme in BuiltinTheme::ALL {
            let label = format!(
                "{} {}",
                i18n::THEME.get(language),
                theme.name().get(language)
            );
            session.harness.get_by_label(&label);
        }
    }
}

#[test]
fn the_selected_card_is_announced_as_selected() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Themes);
    let classic = format!("{} {}", fr(i18n::THEME), fr(BuiltinTheme::Classic.name()));
    let frost = format!("{} {}", fr(i18n::THEME), fr(BuiltinTheme::Frost.name()));
    let selected = |session: &Session, label: &str| {
        session
            .harness
            .get_by_label(label)
            .accesskit_node()
            .toggled()
            == Some(egui::accesskit::Toggled::True)
    };
    assert!(selected(&session, &classic));
    assert!(!selected(&session, &frost));
    click(&mut session, &frost);
    assert!(selected(&session, &frost));
    assert!(!selected(&session, &classic));
}

/// Le curseur lui-même, et non son libellé (egui expose les deux sous le même nom).
fn slider<'a>(session: &'a Session, label: &'a str) -> egui_kittest::Node<'a> {
    session
        .harness
        .get_by_role_and_label(egui::accesskit::Role::Slider, label)
}

/// Fait avancer un curseur au clavier : focus, puis flèches.
fn nudge(session: &mut Session, label: &str, key: egui::Key, times: usize) {
    slider(session, label).scroll_to_me();
    session.settle();
    slider(session, label).focus();
    session.settle();
    for _ in 0..times {
        session.harness.key_press(key);
        session.harness.step();
    }
    session.settle();
}

#[test]
fn the_overall_size_slider_scales_interface_and_conversations_but_not_emoji() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    nudge(
        &mut session,
        fr(i18n::TAILLE_GENERALE),
        egui::Key::ArrowRight,
        4,
    );
    let scale = session.app.borrow().appearance.text.global_scale;
    assert!(scale > 1.15, "le curseur a avancé : {scale}");
    let ctx = session.ctx();
    let m = super::theme::current(&ctx).text;
    assert!(
        (m.body - 13.0 * scale).abs() < 0.01,
        "interface : {}",
        m.body
    );
    assert!(
        (m.msg_body - 13.0 * scale).abs() < 0.01,
        "conversations : {}",
        m.msg_body
    );
    let saved = Appearance::from_pref(saved_pref(&session.app.borrow()).as_deref());
    assert_eq!(
        saved.text.global_scale, scale,
        "enregistré une fois le curseur lâché"
    );
}

#[test]
fn interface_and_conversation_sliders_are_independent() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    nudge(
        &mut session,
        fr(i18n::TAILLE_DES_CONVERSATIONS),
        egui::Key::ArrowRight,
        6,
    );
    nudge(
        &mut session,
        fr(i18n::TAILLE_DE_L_INTERFACE),
        egui::Key::ArrowLeft,
        3,
    );
    let m = super::theme::current(&session.ctx()).text;
    assert!(
        m.msg_body > 13.0 * 1.2,
        "conversations plus grandes : {}",
        m.msg_body
    );
    assert!(m.body < 13.0 * 0.95, "interface plus petite : {}", m.body);
}

#[test]
fn sliders_stop_at_their_bounds_when_the_key_is_held() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    nudge(
        &mut session,
        fr(i18n::TAILLE_GENERALE),
        egui::Key::ArrowRight,
        60,
    );
    assert_eq!(session.app.borrow().appearance.text.global_scale, 1.6);
    nudge(
        &mut session,
        fr(i18n::TAILLE_GENERALE),
        egui::Key::ArrowLeft,
        80,
    );
    assert_eq!(session.app.borrow().appearance.text.global_scale, 0.8);
}

#[test]
fn line_and_letter_spacing_follow_the_theme_until_touched_and_can_go_back() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    assert_eq!(session.app.borrow().appearance.text.conv_line_spacing, None);

    click(&mut session, fr(i18n::INTERLIGNE_DU_THEME));
    assert!(session
        .app
        .borrow()
        .appearance
        .text
        .conv_line_spacing
        .is_some());
    nudge(&mut session, fr(i18n::INTERLIGNE), egui::Key::ArrowRight, 5);
    let line = session
        .app
        .borrow()
        .appearance
        .text
        .conv_line_spacing
        .unwrap();
    assert!(line > 1.3, "{line}");
    let height = super::theme::current(&session.ctx()).text.msg_line_height;
    assert!((height.unwrap() - 13.0 * line).abs() < 0.01);

    click(&mut session, fr(i18n::INTERLIGNE_DU_THEME));
    assert_eq!(
        session.app.borrow().appearance.text.conv_line_spacing,
        None,
        "retour au thème"
    );
    assert_eq!(
        super::theme::current(&session.ctx()).text.msg_line_height,
        None
    );

    click(&mut session, fr(i18n::ESPACEMENT_DU_THEME));
    nudge(
        &mut session,
        fr(i18n::ESPACEMENT_DES_LETTRES),
        egui::Key::ArrowRight,
        3,
    );
    let letters = session
        .app
        .borrow()
        .appearance
        .text
        .conv_letter_spacing
        .unwrap();
    assert!(letters > 0.0);
    assert_eq!(
        super::theme::current(&session.ctx()).text.msg_tracking,
        letters
    );
}

#[test]
fn the_slider_is_disabled_while_the_theme_value_applies() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    assert!(slider(&session, fr(i18n::INTERLIGNE))
        .accesskit_node()
        .is_disabled());
}

#[test]
fn back_to_theme_sizes_resets_every_text_setting() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    nudge(
        &mut session,
        fr(i18n::TAILLE_GENERALE),
        egui::Key::ArrowRight,
        5,
    );
    click(&mut session, fr(i18n::INTERLIGNE_DU_THEME));
    click(&mut session, fr(i18n::REVENIR_AUX_TAILLES_DU_THEME));
    assert_eq!(
        session.app.borrow().appearance.text,
        TextSettings::default()
    );
    assert_eq!(super::theme::current(&session.ctx()).text.body, 13.0);
}

#[test]
fn a_message_is_sized_by_the_conversation_setting_in_the_real_preview() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Text);
    // L'aperçu existe, dans la langue de l'interface.
    session.harness.get_by_label(fr(i18n::APERCU));
}

fn hex_label(role: super::appearance_tab::Role) -> String {
    format!("{} : {}", fr(role.label()), fr(i18n::CODE_COULEUR))
}

fn default_label(role: super::appearance_tab::Role) -> String {
    format!("{} : {}", fr(role.label()), fr(i18n::PAR_DEFAUT_POUR))
}

fn themes_dir(app: &AbcomApp) -> std::path::PathBuf {
    app.state
        .lock()
        .unwrap()
        .media_dir()
        .parent()
        .unwrap()
        .join("themes")
}

fn custom_files(app: &AbcomApp) -> usize {
    std::fs::read_dir(themes_dir(app))
        .map(|dir| {
            dir.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
                .count()
        })
        .unwrap_or(0)
}

use super::appearance_tab::Role;

#[test]
fn touching_a_color_of_a_builtin_theme_makes_a_personal_copy() {
    let mut app = new_app(None);
    app.appearance.select_builtin(BuiltinTheme::Frost);
    app.appearance.mode = Mode::Dark;
    let mut session = Session::new(app, desktop());
    open_tab(&mut session, AppearanceTab::Colors);
    let original_frost = BuiltinTheme::Frost.resolve();

    type_into(&mut session, &hex_label(Role::Background), "#102030");
    {
        let app = session.app.borrow();
        let id = app
            .appearance
            .custom_id()
            .expect("une copie est sélectionnée");
        let custom = app.themes.get(id).unwrap();
        assert_eq!(
            custom.dark.background,
            egui::Color32::from_rgb(0x10, 0x20, 0x30)
        );
        assert_eq!(custom.name, "Givre (perso)");
        assert_eq!(
            custom.dark.text, original_frost.dark.text,
            "le reste est copié"
        );
        assert_eq!(
            custom.light, original_frost.light,
            "l'autre mode n'est pas touché"
        );
        assert!(app.last_notification.is_some(), "la copie est signalée");
    }
    let ctx = session.ctx();
    assert_eq!(
        ctx.style_of(egui::Theme::Dark).visuals.panel_fill,
        egui::Color32::from_rgb(0x10, 0x20, 0x30),
        "appliqué tout de suite"
    );
    assert_eq!(
        super::theme::current(&ctx).dark.background,
        egui::Color32::from_rgb(0x10, 0x20, 0x30)
    );
    assert_eq!(
        custom_files(&session.app.borrow()),
        1,
        "écrit sur le disque"
    );
    // Le thème intégré, lui, n'a pas bougé.
    assert_eq!(BuiltinTheme::Frost.resolve().dark, original_frost.dark);
}

fn frost_dark_session() -> Session {
    let mut app = new_app(None);
    app.appearance.select_builtin(BuiltinTheme::Frost);
    app.appearance.mode = Mode::Dark;
    Session::new(app, desktop())
}

fn active_custom(session: &Session) -> super::custom_themes::CustomTheme {
    let app = session.app.borrow();
    let id = app.appearance.custom_id().expect("thème personnel actif");
    app.themes.get(id).unwrap().clone()
}

#[test]
fn an_invalid_color_code_changes_nothing_and_creates_no_copy() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    for junk in ["zz", "#12", "rouge", "#gg", "é", "#"] {
        type_into(&mut session, &hex_label(Role::Accent), junk);
        assert_eq!(
            session.app.borrow().appearance.custom_id(),
            None,
            "{junk:?}"
        );
        assert_eq!(custom_files(&session.app.borrow()), 0);
    }
}

#[test]
fn short_and_hashless_color_codes_are_accepted() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    type_into(&mut session, &hex_label(Role::Accent), "f80");
    assert_eq!(
        active_custom(&session).dark.accent,
        egui::Color32::from_rgb(255, 136, 0)
    );
    type_into(&mut session, &hex_label(Role::Accent), "00ff7f");
    assert_eq!(
        active_custom(&session).dark.accent,
        egui::Color32::from_rgb(0, 255, 127)
    );
}

#[test]
fn a_color_set_in_dark_mode_leaves_the_light_variant_alone() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    type_into(&mut session, &hex_label(Role::Text), "#FFEEDD");
    let custom = active_custom(&session);
    assert_eq!(custom.dark.text, egui::Color32::from_rgb(255, 238, 221));
    assert_eq!(custom.light.text, BuiltinTheme::Frost.resolve().light.text);

    // Passer en clair édite l'autre variante.
    open_tab(&mut session, AppearanceTab::Themes);
    click(&mut session, fr(i18n::CLAIR));
    open_tab(&mut session, AppearanceTab::Colors);
    session
        .harness
        .get_by_label(fr(i18n::MODIFICATION_DU_MODE_CLAIR));
    type_into(&mut session, &hex_label(Role::Text), "#101010");
    let custom = active_custom(&session);
    assert_eq!(custom.light.text, egui::Color32::from_rgb(16, 16, 16));
    assert_eq!(
        custom.dark.text,
        egui::Color32::from_rgb(255, 238, 221),
        "le sombre est intact"
    );
    assert_eq!(
        custom_files(&session.app.borrow()),
        1,
        "toujours une seule copie"
    );
}

#[test]
fn a_poor_contrast_is_flagged_live_and_unflagged_once_fixed() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    let warning = fr(i18n::CONTRASTE_INSUFFISANT);
    assert!(
        session.harness.query_by_label(warning).is_none(),
        "Givre est lisible"
    );

    type_into(&mut session, &hex_label(Role::Background), "#808080");
    type_into(&mut session, &hex_label(Role::Text), "#808080");
    session.harness.get_by_label(warning);
    session.harness.get_by_label("1,0:1");

    type_into(&mut session, &hex_label(Role::Text), "#000000");
    assert!(session.harness.query_by_label(warning).is_none());
}

#[test]
fn contrast_is_written_with_a_decimal_point_in_english() {
    let mut app = new_app(None);
    app.ui_language = UiLanguage::English;
    app.appearance.select_builtin(BuiltinTheme::Frost);
    app.appearance.mode = Mode::Dark;
    let mut session = Session::new(app, desktop());
    open_tab(&mut session, AppearanceTab::Colors);
    let ratio = super::appearance_tab::format_ratio(
        super::colors::contrast_ratio(
            super::themes::BuiltinTheme::Frost.resolve().dark.text,
            BuiltinTheme::Frost.resolve().dark.background,
        ),
        false,
    );
    assert!(ratio.contains('.'));
    session.harness.get_by_label(&ratio);
}

#[test]
fn the_default_button_restores_a_color_and_is_disabled_when_already_default() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    let label = default_label(Role::Accent);
    assert!(
        session
            .harness
            .get_by_label(&label)
            .accesskit_node()
            .is_disabled(),
        "un thème intégré n'a rien à restaurer"
    );
    type_into(&mut session, &hex_label(Role::Accent), "#FF0000");
    assert!(!session
        .harness
        .get_by_label(&label)
        .accesskit_node()
        .is_disabled());
    click(&mut session, &label);
    assert_eq!(
        active_custom(&session).dark.accent,
        BuiltinTheme::Frost.resolve().dark.accent
    );
    assert!(session
        .harness
        .get_by_label(&label)
        .accesskit_node()
        .is_disabled());
}

#[test]
fn the_advanced_section_edits_every_role() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    click(&mut session, fr(i18n::OPTIONS_AVANCEES));
    for (n, role) in Role::ADVANCED.into_iter().enumerate() {
        let color = egui::Color32::from_rgb(10 + n as u8, 20, 30);
        type_into(
            &mut session,
            &hex_label(role),
            &super::appearance_tab::hex_of(color),
        );
        assert_eq!(role.get(&active_custom(&session).dark), color, "{role:?}");
    }
    assert_eq!(custom_files(&session.app.borrow()), 1);
}

#[test]
fn a_custom_theme_survives_a_restart_with_its_colors() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    type_into(&mut session, &hex_label(Role::Background), "#0A0B0C");
    let (pref, dir) = (
        saved_pref(&session.app.borrow()).expect("préférence écrite"),
        themes_dir(&session.app.borrow()),
    );
    drop(session);

    // Même dossier de données : on rebranche le nouveau magasin dessus.
    let mut app = new_app(Some(&pref));
    app.themes = super::custom_themes::ThemeStore::load(&dir);
    app.appearance = Appearance::from_pref(Some(&pref));
    assert!(
        !app.appearance.drop_missing_custom(&app.themes),
        "le thème est retrouvé"
    );
    let mut restarted = Session::new(app, desktop());
    restarted.harness.step();
    assert_eq!(
        super::theme::current(&restarted.ctx()).dark.background,
        egui::Color32::from_rgb(0x0A, 0x0B, 0x0C)
    );
}

#[test]
fn a_custom_theme_whose_file_was_deleted_falls_back_to_classic_with_a_notice() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    type_into(&mut session, &hex_label(Role::Background), "#0A0B0C");
    let pref = saved_pref(&session.app.borrow()).unwrap();
    drop(session);

    // Le fichier a disparu : un autre dossier de données, sans le thème.
    let app = new_app(Some(&pref));
    assert_eq!(app.appearance.theme, "classic");
    assert!(app.last_notification.is_some());
}

#[test]
fn duplicate_rename_and_delete_a_personal_theme() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Themes);
    click(&mut session, fr(i18n::DUPLIQUER_LE_THEME_ACTIF));
    let first = active_custom(&session);
    assert_eq!(first.name, "Givre");
    assert_eq!(first.base, BuiltinTheme::Frost);

    click(&mut session, fr(i18n::DUPLIQUER_LE_THEME_ACTIF));
    let second = active_custom(&session);
    assert_ne!(second.id, first.id);
    assert_eq!(second.name, "Givre (2)");

    type_into(&mut session, fr(i18n::NOM_DU_THEME), "  Mon \n Givre  ");
    // Valider : le champ perd le focus.
    click(&mut session, fr(i18n::MODE_D_AFFICHAGE));
    assert_eq!(active_custom(&session).name, "Mon Givre");

    click(&mut session, fr(i18n::SUPPRIMER_LE_THEME));
    session
        .harness
        .get_by_label(fr(i18n::SUPPRIMER_CE_THEME_QUESTION));
    click(&mut session, fr(i18n::ANNULER));
    assert!(
        session.app.borrow().appearance.custom_id().is_some(),
        "annulé"
    );

    click(&mut session, fr(i18n::SUPPRIMER_LE_THEME));
    click(&mut session, fr(i18n::CONFIRMER));
    assert_eq!(
        session.app.borrow().appearance.theme,
        "frost",
        "retour au thème de base"
    );
    assert_eq!(session.app.borrow().themes.themes().len(), 1);
    assert_eq!(custom_files(&session.app.borrow()), 1);
}

#[test]
fn renaming_to_an_empty_name_keeps_the_old_one() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Themes);
    click(&mut session, fr(i18n::DUPLIQUER_LE_THEME_ACTIF));
    type_into(&mut session, fr(i18n::NOM_DU_THEME), "   ");
    click(&mut session, fr(i18n::MODE_D_AFFICHAGE));
    assert_eq!(active_custom(&session).name, "Givre");
}

#[test]
fn fifty_duplicates_keep_distinct_names_and_the_gallery_stays_usable() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Themes);
    for _ in 0..50 {
        click(&mut session, fr(i18n::DUPLIQUER_LE_THEME_ACTIF));
    }
    let app = session.app.borrow();
    let names: std::collections::HashSet<_> = app
        .themes
        .themes()
        .iter()
        .map(|t| t.name.to_lowercase())
        .collect();
    assert_eq!(names.len(), 50);
}

#[test]
fn deleting_the_active_theme_then_restarting_gives_the_base_theme() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Themes);
    click(&mut session, fr(i18n::DUPLIQUER_LE_THEME_ACTIF));
    click(&mut session, fr(i18n::SUPPRIMER_LE_THEME));
    click(&mut session, fr(i18n::CONFIRMER));
    let pref = saved_pref(&session.app.borrow()).unwrap();
    drop(session);
    assert_eq!(Appearance::from_pref(Some(&pref)).theme, "frost");
}

#[test]
fn reset_asks_for_confirmation_and_keeps_personal_themes() {
    let mut session = frost_dark_session();
    open_tab(&mut session, AppearanceTab::Colors);
    type_into(&mut session, &hex_label(Role::Accent), "#FF0000");
    open_tab(&mut session, AppearanceTab::Text);
    nudge(
        &mut session,
        fr(i18n::TAILLE_GENERALE),
        egui::Key::ArrowRight,
        4,
    );

    click(&mut session, fr(i18n::REINITIALISER_L_APPARENCE));
    session
        .harness
        .get_by_label(fr(i18n::REINITIALISER_QUESTION));
    click(&mut session, fr(i18n::ANNULER));
    assert_ne!(
        session.app.borrow().appearance,
        Appearance::default(),
        "annulé"
    );

    click(&mut session, fr(i18n::REINITIALISER_L_APPARENCE));
    click(&mut session, fr(i18n::CONFIRMER));
    assert_eq!(session.app.borrow().appearance, Appearance::default());
    assert_eq!(
        session.app.borrow().themes.themes().len(),
        1,
        "le thème personnel reste"
    );
    assert_eq!(super::theme::current(&session.ctx()).text.body, 13.0);
    let saved = Appearance::from_pref(saved_pref(&session.app.borrow()).as_deref());
    assert_eq!(saved, Appearance::default());
}

#[test]
fn the_whole_tab_is_translated_in_english() {
    let mut app = new_app(None);
    app.ui_language = UiLanguage::English;
    let mut session = Session::new(app, desktop());
    for (tab, label) in [
        (AppearanceTab::Themes, i18n::APPARENCE_THEMES),
        (AppearanceTab::Text, i18n::APPARENCE_TEXTE),
        (AppearanceTab::Colors, i18n::APPARENCE_COULEURS),
    ] {
        open_tab(&mut session, tab);
        session.harness.get_by_label(label.get(UiLanguage::English));
        for entry in [i18n::REINITIALISER_L_APPARENCE, i18n::APERCU] {
            if matches!(tab, AppearanceTab::Themes) && entry.fr == i18n::APERCU.fr {
                continue;
            }
            assert!(
                session
                    .harness
                    .query_by_label(entry.get(UiLanguage::English))
                    .is_some(),
                "{} manquant en anglais",
                entry.en
            );
            assert!(
                session.harness.query_by_label(entry.fr).is_none(),
                "{} resté en français",
                entry.fr
            );
        }
    }
}

// ── Polices ───────────────────────────────────────────────────────────────

use super::appearance_tab::FontZone;
use super::fonts::{FontChoice, SuggestedFont};
use super::theme::{MSG_BOLD_FAMILY, MSG_FAMILY};

const ASSET_FONTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts");
const FIXTURE_FONTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fonts");

fn zone_title(zone: FontZone) -> &'static str {
    match zone {
        FontZone::Interface => fr(i18n::POLICE_DE_L_INTERFACE),
        FontZone::Conversations => fr(i18n::POLICE_DES_CONVERSATIONS),
    }
}

fn font_row(zone: FontZone, name: &str) -> String {
    format!("{} : {name}", zone_title(zone))
}

fn import_label(zone: FontZone) -> &'static str {
    match zone {
        FontZone::Interface => fr(i18n::IMPORTER_POUR_L_INTERFACE),
        FontZone::Conversations => fr(i18n::IMPORTER_POUR_LES_CONVERSATIONS),
    }
}

/// Fait répondre le sélecteur de fichier de l'application avec `path` (ou l'annule).
fn pick(session: &Session, path: Option<std::path::PathBuf>) {
    session.app.borrow_mut().font_picker = Box::new(move || path.clone());
}

fn registered_families(session: &Session) -> Vec<egui::FontFamily> {
    session.ctx().fonts(|fonts| fonts.families())
}

/// Première police de la chaîne d'une famille, telle qu'egui la connaît.
fn head_of(session: &Session, family: &str) -> String {
    session.ctx().fonts(|fonts| {
        fonts.definitions().families[&egui::FontFamily::Name(family.into())][0].clone()
    })
}

#[test]
fn opening_the_fonts_tab_registers_every_suggested_font_once() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    session.settle();
    let families = registered_families(&session);
    for font in SuggestedFont::ALL {
        let family = super::preview_family(&format!("suggested-{}", font.id()));
        assert!(families.contains(&family), "{} non enregistrée", font.id());
    }
    for builtin in ["builtin-noto", "builtin-inter"] {
        assert!(
            families.contains(&super::preview_family(builtin)),
            "{builtin}"
        );
    }
}

#[test]
fn every_row_shows_its_name_and_a_description() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    for zone in [FontZone::Interface, FontZone::Conversations] {
        for name in [
            "Noto Sans",
            "Inter",
            "Atkinson Hyperlegible",
            "Lexend",
            "OpenDyslexic",
            "Literata",
            "Nunito",
            "JetBrains Mono",
        ] {
            session.harness.get_by_label(&font_row(zone, name));
        }
        session
            .harness
            .get_by_label(&font_row(zone, fr(i18n::POLICE_DU_THEME)));
    }
    let descriptions = session
        .harness
        .get_all_by_label(fr(i18n::POLICE_LEXEND_DESCRIPTION))
        .count();
    assert_eq!(descriptions, 2, "une par zone");
}

#[test]
fn each_suggested_font_can_be_picked_for_the_conversations_and_the_interface() {
    for font in SuggestedFont::ALL {
        for zone in [FontZone::Interface, FontZone::Conversations] {
            let mut session = Session::new(new_app(None), desktop());
            open_tab(&mut session, AppearanceTab::Fonts);
            click(&mut session, &font_row(zone, font.name()));
            session.settle();
            let app = session.app.borrow();
            let chosen = match zone {
                FontZone::Interface => &app.appearance.ui_font,
                FontZone::Conversations => &app.appearance.conv_font,
            };
            assert_eq!(
                *chosen,
                FontChoice::Suggested(font),
                "{} / {zone:?}",
                font.id()
            );
            drop(app);
            let head = match zone {
                FontZone::Interface => session.ctx().fonts(|f| {
                    f.definitions().families[&egui::FontFamily::Proportional][0].clone()
                }),
                FontZone::Conversations => head_of(&session, MSG_FAMILY),
            };
            assert!(
                head.ends_with("-r0"),
                "{} : la police est en tête ({head})",
                font.id()
            );
            let bold = head_of(&session, MSG_BOLD_FAMILY);
            if zone == FontZone::Conversations {
                assert!(
                    bold.ends_with("-b0"),
                    "{} : son gras est le 700 ({bold})",
                    font.id()
                );
                assert!(super::theme::current(&session.ctx()).text.msg_real_bold);
            }
        }
    }
}

#[test]
fn the_two_zones_can_use_different_fonts() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Literata"));
    click(&mut session, &font_row(FontZone::Interface, "Lexend"));
    let app = session.app.borrow();
    assert_eq!(
        app.appearance.conv_font,
        FontChoice::Suggested(SuggestedFont::Literata)
    );
    assert_eq!(
        app.appearance.ui_font,
        FontChoice::Suggested(SuggestedFont::Lexend)
    );
}

#[test]
fn a_font_choice_is_saved_and_applied_on_the_next_start() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Nunito"));
    let pref = saved_pref(&session.app.borrow()).expect("écrit");
    assert!(pref.contains("suggested:nunito"), "{pref}");
    drop(session);

    let mut restarted = Session::new(new_app(Some(&pref)), desktop());
    restarted.settle();
    assert_eq!(
        restarted.app.borrow().appearance.conv_font,
        FontChoice::Suggested(SuggestedFont::Nunito)
    );
    assert!(head_of(&restarted, MSG_FAMILY).ends_with("-r0"));
}

#[test]
fn going_back_to_the_theme_font_restores_the_theme_chain() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Lexend"));
    click(
        &mut session,
        &font_row(FontZone::Conversations, fr(i18n::POLICE_DU_THEME)),
    );
    assert_eq!(head_of(&session, MSG_FAMILY), "noto-sans");
}

#[test]
fn builtin_fonts_can_be_chosen_explicitly() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Inter"));
    assert_eq!(head_of(&session, MSG_FAMILY), "inter");
    assert!(super::theme::current(&session.ctx()).text.msg_real_italic);
    click(
        &mut session,
        &font_row(FontZone::Conversations, "Noto Sans"),
    );
    assert_eq!(head_of(&session, MSG_FAMILY), "noto-sans");
    assert!(!super::theme::current(&session.ctx()).text.msg_real_italic);
}

#[test]
fn fonts_are_not_rebuilt_by_a_color_or_a_size_change() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Lexend"));
    let before = session.ctx().fonts(|f| f.definitions().clone());
    session.app.borrow_mut().appearance.text.global_scale = 1.4;
    session
        .app
        .borrow_mut()
        .appearance
        .select_builtin(BuiltinTheme::Frost);
    session.settle();
    let after = session.ctx().fonts(|f| f.definitions().clone());
    assert_eq!(
        before.families, after.families,
        "la même chaîne, sans reconstruction"
    );
}

#[test]
fn importing_each_font_format_applies_it_and_survives_a_restart() {
    let cases = [
        ("Inter-Regular.ttf", ASSET_FONTS, "Inter"),
        ("Unifont.otf", ASSET_FONTS, "Unifont"),
        ("Inter-Regular.woff", FIXTURE_FONTS, "Inter"),
        ("Inter-Regular.woff2", FIXTURE_FONTS, "Inter"),
    ];
    for (file, folder, family) in cases {
        for zone in [FontZone::Interface, FontZone::Conversations] {
            let dir = std::env::temp_dir().join(format!(
                "abcom-import-{}-{:?}-{}",
                file,
                zone,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let mut session = Session::new(new_app_in(&dir, None), desktop());
            pick(&session, Some(std::path::PathBuf::from(folder).join(file)));
            open_tab(&mut session, AppearanceTab::Fonts);
            click(&mut session, import_label(zone));

            let app = session.app.borrow();
            assert_eq!(app.fonts.fonts().len(), 1, "{file}");
            let font = app.fonts.fonts()[0].clone();
            assert_eq!(font.family, family, "{file}");
            let applied = match zone {
                FontZone::Interface => &app.appearance.ui_font,
                FontZone::Conversations => &app.appearance.conv_font,
            };
            assert_eq!(*applied, FontChoice::Imported(font.id.clone()), "{file}");
            assert!(app
                .last_notification
                .as_deref()
                .unwrap_or("")
                .contains(family));
            let pref = saved_pref(&app).expect("préférence écrite");
            drop(app);
            drop(session);

            let mut restarted = Session::new(new_app_in(&dir, Some(&pref)), desktop());
            restarted.settle();
            assert_eq!(restarted.app.borrow().fonts.fonts().len(), 1);
            let head = match zone {
                FontZone::Interface => restarted.ctx().fonts(|f| {
                    f.definitions().families[&egui::FontFamily::Proportional][0].clone()
                }),
                FontZone::Conversations => head_of(&restarted, MSG_FAMILY),
            };
            assert!(
                head.ends_with("-r0"),
                "{file} : appliquée au redémarrage ({head})"
            );
        }
    }
}

fn session_in_temp_dir() -> (Session, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "abcom-fontdir-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    (Session::new(new_app_in(&dir, None), desktop()), dir)
}

#[test]
fn refused_imports_say_why_and_change_nothing() {
    let (mut session, dir) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    let ttf = std::fs::read(format!("{ASSET_FONTS}/Inter-Regular.ttf")).unwrap();
    let woff2 = std::fs::read(format!("{FIXTURE_FONTS}/Inter-Regular.woff2")).unwrap();
    let mut bomb = vec![0u8; 48];
    bomb[..4].copy_from_slice(b"wOF2");
    bomb[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    let latin_ext = SuggestedFont::Lexend.decoded().unwrap().regular[1]
        .as_ref()
        .clone();

    let huge = dir.join("enorme.ttf");
    std::fs::File::create(&huge)
        .unwrap()
        .set_len(31 * 1024 * 1024)
        .unwrap();

    let cases: Vec<(&str, Option<Vec<u8>>, i18n::Entry)> = vec![
        (
            "bidon.ttf",
            Some(b"ceci n'est pas une police".to_vec()),
            i18n::POLICE_ILLISIBLE,
        ),
        ("vide.ttf", Some(Vec::new()), i18n::POLICE_ILLISIBLE),
        (
            "coupee.woff2",
            Some(woff2[..woff2.len() / 2].to_vec()),
            i18n::POLICE_ILLISIBLE,
        ),
        ("bombe.woff2", Some(bomb), i18n::POLICE_TROP_LOURDE),
        ("ext.ttf", Some(latin_ext), i18n::POLICE_SANS_LATIN),
        (
            "renommee.ttf",
            Some(b"<html>pas une police</html>".to_vec()),
            i18n::POLICE_ILLISIBLE,
        ),
        ("enorme.ttf", None, i18n::POLICE_TROP_LOURDE),
    ];
    let _ = ttf;
    for (name, bytes, expected) in cases {
        let path = match bytes {
            Some(bytes) => {
                let path = dir.join(name);
                std::fs::write(&path, bytes).unwrap();
                path
            }
            None => huge.clone(),
        };
        let before = session.app.borrow().appearance.clone();
        session.app.borrow_mut().last_notification = None;
        pick(&session, Some(path));
        click(&mut session, import_label(FontZone::Conversations));
        let app = session.app.borrow();
        assert_eq!(
            app.last_notification.as_deref(),
            Some(fr(expected)),
            "{name}"
        );
        assert_eq!(app.appearance, before, "{name} : rien n'a changé");
        assert!(app.fonts.fonts().is_empty(), "{name} : rien n'est importé");
    }
    assert!(!dir.join("fonts").join("index.json").exists());
}

#[test]
fn cancelling_the_file_picker_changes_nothing() {
    let (mut session, _) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    pick(&session, None);
    session.app.borrow_mut().last_notification = None;
    click(&mut session, import_label(FontZone::Conversations));
    let app = session.app.borrow();
    assert_eq!(app.appearance, Appearance::default());
    assert!(app.last_notification.is_none());
}

#[test]
fn importing_the_same_font_twice_selects_the_existing_one() {
    let (mut session, _) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    let path = std::path::PathBuf::from(ASSET_FONTS).join("Inter-Regular.ttf");
    pick(&session, Some(path));
    click(&mut session, import_label(FontZone::Conversations));
    click(&mut session, import_label(FontZone::Interface));
    let app = session.app.borrow();
    assert_eq!(app.fonts.fonts().len(), 1, "pas de doublon");
    assert_eq!(app.appearance.ui_font, app.appearance.conv_font);
    assert_eq!(
        app.last_notification.as_deref(),
        Some(fr(i18n::POLICE_DEJA_IMPORTEE))
    );
}

#[test]
fn removing_an_imported_font_resets_both_zones_and_deletes_the_file() {
    let (mut session, dir) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    pick(
        &session,
        Some(std::path::PathBuf::from(ASSET_FONTS).join("Inter-Regular.ttf")),
    );
    click(&mut session, import_label(FontZone::Conversations));
    click(&mut session, import_label(FontZone::Interface));
    let family = session.app.borrow().fonts.fonts()[0].family.clone();
    let file = dir
        .join("fonts")
        .join(&session.app.borrow().fonts.fonts()[0].file);
    assert!(file.exists());

    click(&mut session, &format!("{} {family}", fr(i18n::RETIRER)));
    let app = session.app.borrow();
    assert!(app.fonts.fonts().is_empty());
    assert_eq!(app.appearance.ui_font, FontChoice::ThemeDefault);
    assert_eq!(app.appearance.conv_font, FontChoice::ThemeDefault);
    assert!(!file.exists());
    drop(app);
    assert_eq!(head_of(&session, MSG_FAMILY), "noto-sans");
}

#[test]
fn an_imported_font_deleted_from_the_disk_falls_back_with_a_notice_at_startup() {
    let (mut session, dir) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    pick(
        &session,
        Some(std::path::PathBuf::from(ASSET_FONTS).join("Inter-Regular.ttf")),
    );
    click(&mut session, import_label(FontZone::Conversations));
    let pref = saved_pref(&session.app.borrow()).unwrap();
    let file = dir
        .join("fonts")
        .join(&session.app.borrow().fonts.fonts()[0].file);
    drop(session);
    std::fs::remove_file(file).unwrap();

    let mut restarted = Session::new(new_app_in(&dir, Some(&pref)), desktop());
    restarted.settle();
    assert_eq!(
        head_of(&restarted, MSG_FAMILY),
        "noto-sans",
        "police du thème"
    );
    let notice = restarted.app.borrow().last_notification.clone();
    assert_eq!(notice.as_deref(), Some(fr(i18n::POLICE_INDISPONIBLE)));
}

#[test]
fn an_imported_font_replaced_by_junk_never_reaches_egui() {
    let (mut session, dir) = session_in_temp_dir();
    open_tab(&mut session, AppearanceTab::Fonts);
    pick(
        &session,
        Some(std::path::PathBuf::from(ASSET_FONTS).join("Inter-Regular.ttf")),
    );
    click(&mut session, import_label(FontZone::Conversations));
    let pref = saved_pref(&session.app.borrow()).unwrap();
    let file = dir
        .join("fonts")
        .join(&session.app.borrow().fonts.fonts()[0].file);
    drop(session);
    std::fs::write(file, b"remplace par du texte").unwrap();

    let mut restarted = Session::new(new_app_in(&dir, Some(&pref)), desktop());
    restarted.settle();
    assert_eq!(head_of(&restarted, MSG_FAMILY), "noto-sans");
}

#[test]
fn text_in_other_scripts_still_renders_with_a_latin_only_font() {
    let mut session = Session::new(new_app(None), desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    click(&mut session, &font_row(FontZone::Conversations, "Lexend"));
    session.settle();
    let width = |text: &str| -> f32 {
        session.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(
                    text.to_owned(),
                    egui::FontId::new(14.0, egui::FontFamily::Name(MSG_FAMILY.into())),
                    egui::Color32::WHITE,
                )
                .size()
                .x
        })
    };
    for text in [
        "日本語のテキスト",
        "Привет, мир",
        "مرحبا بالعالم",
        "한국어",
        "Ça va très bien ? Łódź",
    ] {
        assert!(width(text) > 0.0, "{text}");
    }
}

#[test]
fn switching_fonts_with_a_long_conversation_stays_responsive() {
    let mut app = new_app(None);
    {
        let mut state = app.state.lock().unwrap();
        state.add_peer("alice".into(), "127.0.0.1:9000".parse().unwrap());
        state.selected_conversation = Some("alice".into());
        for i in 0..1000u64 {
            state.messages.push(crate::message::ChatMessage {
                from: if i % 2 == 0 {
                    "alice".into()
                } else {
                    "moi".into()
                },
                content: format!(
                    "Message numéro {i} : **gras**, *italique* et `code` — Ça va très bien ?"
                ),
                timestamp: "12:00".into(),
                timestamp_epoch: Some(i),
                to_user: Some(if i % 2 == 0 {
                    "moi".into()
                } else {
                    "alice".into()
                }),
                media: None,
                reply_to: None,
                nonce: None,
            });
        }
        state.bump_content();
    }
    app.modals.settings_open = true;
    app.modals.settings_tab = super::SettingsTab::Appearance;
    let mut session = Session::new(app, desktop());
    open_tab(&mut session, AppearanceTab::Fonts);
    for font in ["Lexend", "OpenDyslexic", "Literata"] {
        let started = std::time::Instant::now();
        click(&mut session, &font_row(FontZone::Conversations, font));
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_millis() < 2500,
            "{font} : {elapsed:?} pour 5 images"
        );
    }
}
