//! Parcours de l'apparence comme le vit l'utilisateur : clic dans les
//! Paramètres, enregistrement, puis redémarrage sur les mêmes préférences.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_kittest::kittest::Queryable as _;
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
    let mut state = AppState::new_with_base("moi", &dir);
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

fn open_general_settings(session: &mut Session) {
    {
        let mut app = session.app.borrow_mut();
        app.modals.settings_open = true;
        app.modals.settings_tab = super::SettingsTab::General;
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
    open_general_settings(&mut session);
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
    open_general_settings(&mut first);
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
