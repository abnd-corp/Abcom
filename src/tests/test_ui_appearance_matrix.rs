//! L'application entière rejouée sous chaque apparence, à chaque taille de
//! texte, de fenêtre et de densité : rien ne panique, rien ne sort de la
//! fenêtre, et les parcours courants restent possibles.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::Harness;
use tokio::sync::mpsc;

use super::appearance::{Appearance, Mode, TextSettings};
use super::appearance_tab::AppearanceTab;
use super::fonts::{FontChoice, SuggestedFont};
use super::themes::BuiltinTheme;
use super::{i18n, AbcomApp, SettingsTab, UiLanguage};

use super::appearance_e2e_tests::new_app;

const WINDOWS: [(f32, f32); 3] = [(560.0, 360.0), (860.0, 600.0), (1440.0, 900.0)];

fn fr(entry: i18n::Entry) -> &'static str {
    entry.get(UiLanguage::French)
}

/// Une combinaison à éprouver, nommée pour que l'échec dise laquelle.
#[derive(Clone)]
struct Combo {
    name: String,
    appearance: Appearance,
    size: (f32, f32),
    ppp: f32,
    custom_theme: bool,
}

struct Run {
    harness: Harness<'static>,
    app: Rc<RefCell<AbcomApp>>,
    send_rx: mpsc::Receiver<crate::ui::NetworkSendRequest>,
    screen: egui::Rect,
    name: String,
}

fn build(combo: &Combo) -> Run {
    let mut app = new_app(None);
    let (send_tx, send_rx) = mpsc::channel(64);
    app.net.send_tx = send_tx;
    {
        let mut state = app.state.lock().unwrap();
        state.add_peer("alice".into(), "127.0.0.1:9000".parse().unwrap());
        state.selected_conversation = Some("alice".into());
        let texts = [
            "Bonjour de la matrice",
            "Un message un peu plus long, avec **du gras**, de l'*italique*, un [lien](https://example.org) et du `code` en ligne.",
            "# Un titre\n\n- premier point\n- second point\n\n> une citation",
            "```\nlet x = 42;\n```",
            "😀 😀",
            "Ça s'écrit très bien : Łódź, 日本語, Привет",
        ];
        for (i, text) in texts.iter().enumerate() {
            state.messages.push(crate::message::ChatMessage {
                from: if i % 2 == 0 {
                    "alice".into()
                } else {
                    "moi".into()
                },
                content: (*text).to_owned(),
                timestamp: "12:00".into(),
                timestamp_epoch: Some(i as u64),
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
    app.appearance = combo.appearance.clone();
    if combo.custom_theme {
        let base = BuiltinTheme::Frost;
        let id = app.themes.create(base, &base.resolve(), "Matrice");
        app.appearance.select_custom(&id);
    }
    let app = Rc::new(RefCell::new(app));
    let shared = Rc::clone(&app);
    let (width, height) = combo.size;
    let ready = std::cell::Cell::new(false);
    let harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_pixels_per_point(combo.ppp)
        .build_ui(move |root| {
            let mut app = shared.borrow_mut();
            let ctx = root.ctx().clone();
            app.sync_appearance(&ctx);
            // Les styles posés pendant une image ne valent qu'à la suivante : comme
            // au lancement, où l'apparence est installée avant la première image.
            if !ready.replace(true) {
                return;
            }
            app.refresh_caches(&ctx);
            app.apply_picker_outcomes();
            app.show_app(root);
        });
    Run {
        harness,
        app,
        send_rx,
        screen: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, height)),
        name: combo.name.clone(),
    }
}

impl Run {
    fn settle(&mut self) {
        for _ in 0..4 {
            self.harness.step();
        }
    }

    /// Aucun contrôle interactif ne dépasse la fenêtre à droite ou à gauche, et
    /// la fenêtre modale tient entière dans la fenêtre.
    fn check_layout(&self, step: &str) {
        use egui::accesskit::Role;
        let slack = 1.0;
        for node in self.harness.root().children_recursive() {
            let role = node.accesskit_node().role();
            let interactive = matches!(
                role,
                Role::Button
                    | Role::TextInput
                    | Role::MultilineTextInput
                    | Role::Slider
                    | Role::RadioButton
                    | Role::CheckBox
                    | Role::ComboBox
                    | Role::Link
            );
            if !interactive || node.accesskit_node().is_hidden() {
                continue;
            }
            let rect = node.rect();
            if rect.width() <= 0.0 || rect.height() <= 0.0 {
                continue;
            }
            assert!(
                rect.min.x >= self.screen.min.x - slack && rect.max.x <= self.screen.max.x + slack,
                "[{}] {step} : le contrôle {role:?} « {} » sort de la fenêtre en largeur : {rect:?} dans {:?}",
                self.name,
                node.accesskit_node().label().unwrap_or_default(),
                self.screen
            );
        }
        let ctx = self.harness.ctx.clone();
        if let Some(rect) = ctx.memory(|m| m.area_rect(egui::Id::new("settings_modal"))) {
            assert!(
                self.screen.expand(slack).contains_rect(rect),
                "[{}] {step} : les Paramètres {rect:?} sortent de la fenêtre {:?}",
                self.name,
                self.screen
            );
        }
    }

    fn open_settings(&mut self, tab: SettingsTab) {
        {
            let mut app = self.app.borrow_mut();
            app.modals.settings_open = true;
            app.modals.settings_tab = tab;
        }
        self.settle();
    }

    fn click(&mut self, label: &str) {
        self.harness.get_by_label(label).scroll_to_me();
        self.settle();
        self.harness.get_by_label(label).click();
        self.settle();
    }
}

/// Parcours commun : le fil, l'envoi, les surcouches, puis chaque onglet des Paramètres.
fn exercise(run: &mut Run) {
    run.settle();
    run.check_layout("démarrage");

    // Le fil affiche les messages.
    run.harness.get_by_label("Bonjour de la matrice");

    // Envoyer : frappe puis Cmd+Entrée.
    {
        let mut app = run.app.borrow_mut();
        app.composer.text = "bonjour matrice".to_owned();
        app.composer.cursor_char = app.composer.text.chars().count();
        app.composer.has_focus = true;
    }
    run.settle();
    run.harness
        .key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Enter);
    run.settle();
    assert!(
        run.send_rx.try_recv().is_ok(),
        "[{}] Cmd+Entrée n'a rien envoyé",
        run.name
    );
    run.check_layout("après envoi");

    // Sélecteur d'emojis, puis Échap.
    run.app.borrow_mut().show_emoji_picker = true;
    run.settle();
    run.check_layout("sélecteur d'emojis");
    run.harness.key_press(egui::Key::Escape);
    run.settle();
    assert!(
        !run.app.borrow().show_emoji_picker,
        "[{}] Échap ferme le sélecteur",
        run.name
    );

    // Recherche : Cmd+F l'ouvre, Échap la ferme.
    run.harness
        .key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::F);
    run.settle();
    assert!(
        run.app.borrow().search.open,
        "[{}] Cmd+F ouvre la recherche",
        run.name
    );
    run.check_layout("recherche");
    run.harness.key_press(egui::Key::Escape);
    run.settle();
    assert!(!run.app.borrow().search.open);

    // Échap ferme les surcouches une par une, la plus haute d'abord.
    {
        let mut app = run.app.borrow_mut();
        app.modals.settings_open = true;
        app.show_emoji_picker = true;
    }
    run.settle();
    run.harness.key_press(egui::Key::Escape);
    run.settle();
    assert!(
        !run.app.borrow().show_emoji_picker,
        "[{}] le sélecteur se ferme en premier",
        run.name
    );
    assert!(
        run.app.borrow().modals.settings_open,
        "[{}] pas les Paramètres en même temps",
        run.name
    );
    run.harness.key_press(egui::Key::Escape);
    run.settle();
    assert!(
        !run.app.borrow().modals.settings_open,
        "[{}] le second Échap ferme les Paramètres",
        run.name
    );

    // Chaque onglet des Paramètres.
    for (tab, name) in [
        (SettingsTab::Profile, "Profil"),
        (SettingsTab::General, "Général"),
        (SettingsTab::Storage, "Stockage"),
        (SettingsTab::Credits, "Crédits"),
        (SettingsTab::License, "Licence"),
    ] {
        run.open_settings(tab);
        run.check_layout(&format!("paramètres, onglet {name}"));
    }

    // Chaque sous-onglet d'Apparence, et son dernier contrôle atteignable.
    run.open_settings(SettingsTab::Appearance);
    for (tab, label, last) in [
        (
            AppearanceTab::Themes,
            i18n::APPARENCE_THEMES,
            i18n::DUPLIQUER_LE_THEME_ACTIF,
        ),
        (
            AppearanceTab::Text,
            i18n::APPARENCE_TEXTE,
            i18n::REVENIR_AUX_TAILLES_DU_THEME,
        ),
        (
            AppearanceTab::Colors,
            i18n::APPARENCE_COULEURS,
            i18n::OPTIONS_AVANCEES,
        ),
        (
            AppearanceTab::Fonts,
            i18n::APPARENCE_POLICES,
            i18n::FORMATS_DE_POLICE_ACCEPTES,
        ),
    ] {
        run.click(fr(label));
        assert_eq!(run.app.borrow().appearance_ui.tab, tab);
        run.check_layout("apparence");
        run.harness.get_by_label(fr(last)).scroll_to_me();
        run.settle();
        run.check_layout("apparence, fin de page");
    }
    run.harness
        .get_by_label(fr(i18n::REINITIALISER_L_APPARENCE));
}

fn appearance(theme: BuiltinTheme, mode: Mode, text: TextSettings) -> Appearance {
    Appearance {
        theme: theme.id().to_owned(),
        mode,
        text,
        ..Appearance::default()
    }
}

fn scaled(global: f32, conv: f32) -> TextSettings {
    TextSettings {
        global_scale: global,
        conv_scale: conv,
        ..TextSettings::default()
    }
}

/// Matrice A : tous les thèmes, clairs et sombres.
#[test]
fn matrix_every_theme_in_both_modes() {
    for theme in BuiltinTheme::ALL {
        for mode in [Mode::Light, Mode::Dark] {
            let combo = Combo {
                name: format!("{theme:?} {mode:?}"),
                appearance: appearance(theme, mode, TextSettings::default()),
                size: (860.0, 600.0),
                ppp: 1.0,
                custom_theme: false,
            };
            exercise(&mut build(&combo));
        }
    }
}

/// Matrice B : un thème, toutes les tailles de texte, de fenêtre et de densité.
fn sizes_windows_and_densities(theme_name: &str, base: Appearance, custom: bool) {
    let sizes = [
        ("80 %", scaled(0.8, 1.0)),
        ("100 %", scaled(1.0, 1.0)),
        ("160 % général", scaled(1.6, 1.0)),
        ("160 % × 160 % conversations", scaled(1.6, 1.6)),
    ];
    for (size_name, text) in &sizes {
        for window in WINDOWS {
            for ppp in [1.0, 2.0] {
                let mut appearance = base.clone();
                appearance.text = *text;
                let combo = Combo {
                    name: format!(
                        "{theme_name} / {size_name} / {}×{} / ppp {ppp}",
                        window.0, window.1
                    ),
                    appearance,
                    size: window,
                    ppp,
                    custom_theme: custom,
                };
                exercise(&mut build(&combo));
            }
        }
    }
}

#[test]
fn matrix_sizes_windows_and_densities_classic() {
    let base = appearance(BuiltinTheme::Classic, Mode::Dark, TextSettings::default());
    sizes_windows_and_densities("Classique", base, false);
}

#[test]
fn matrix_sizes_windows_and_densities_light_sand() {
    let base = appearance(BuiltinTheme::Sand, Mode::Light, TextSettings::default());
    sizes_windows_and_densities("Sable clair", base, false);
}

#[test]
fn matrix_sizes_windows_and_densities_dark_nocturne() {
    let base = appearance(BuiltinTheme::Nocturne, Mode::Dark, TextSettings::default());
    sizes_windows_and_densities("Nocturne sombre", base, false);
}

#[test]
fn matrix_sizes_windows_and_densities_custom_theme() {
    let base = appearance(BuiltinTheme::Frost, Mode::Dark, TextSettings::default());
    sizes_windows_and_densities("perso", base, true);
}

/// Matrice C : chaque police proposée, pour l'interface et pour les conversations, au pire cas.
#[test]
fn matrix_every_font_in_the_smallest_window_at_the_largest_size() {
    let fonts = [
        FontChoice::NotoSans,
        FontChoice::Inter,
        FontChoice::Suggested(SuggestedFont::Atkinson),
        FontChoice::Suggested(SuggestedFont::Lexend),
        FontChoice::Suggested(SuggestedFont::OpenDyslexic),
        FontChoice::Suggested(SuggestedFont::Literata),
        FontChoice::Suggested(SuggestedFont::Nunito),
        FontChoice::Suggested(SuggestedFont::JetBrainsMono),
    ];
    for font in fonts {
        for zone in ["interface", "conversations"] {
            let mut appearance = appearance(BuiltinTheme::Classic, Mode::Dark, scaled(1.6, 1.6));
            if zone == "interface" {
                appearance.ui_font = font.clone();
            } else {
                appearance.conv_font = font.clone();
            }
            let combo = Combo {
                name: format!("{} / {zone}", font.to_id()),
                appearance,
                size: (560.0, 360.0),
                ppp: 1.0,
                custom_theme: false,
            };
            exercise(&mut build(&combo));
        }
    }
}

// ── Cas limites ───────────────────────────────────────────────────────────

fn default_combo(name: &str, size: (f32, f32)) -> Combo {
    Combo {
        name: name.to_owned(),
        appearance: Appearance::default(),
        size,
        ppp: 1.0,
        custom_theme: false,
    }
}

/// Les Paramètres se réajustent quand on redimensionne la fenêtre.
#[test]
fn resizing_the_window_with_the_settings_open_keeps_them_inside() {
    let mut run = build(&default_combo("redimensionnement", (1440.0, 900.0)));
    run.open_settings(SettingsTab::Appearance);
    run.check_layout("grande fenêtre");
    for size in [
        (860.0, 600.0),
        (560.0, 360.0),
        (1000.0, 500.0),
        (560.0, 360.0),
        (1440.0, 900.0),
    ] {
        run.harness.set_size(egui::vec2(size.0, size.1));
        run.screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(size.0, size.1));
        run.settle();
        run.check_layout(&format!("{size:?}"));
    }
}

/// Fenêtre déplacée d'un écran Retina vers un écran ordinaire, et inversement.
#[test]
fn changing_the_pixel_density_does_not_break_the_layout() {
    let mut run = build(&default_combo("densité", (860.0, 600.0)));
    run.open_settings(SettingsTab::Appearance);
    for ppp in [2.0, 1.0, 1.5, 2.0, 1.0] {
        run.harness.ctx.set_pixels_per_point(ppp);
        run.settle();
        run.check_layout(&format!("ppp {ppp}"));
    }
}

/// Un message très long reste replié, et son bouton accessible, à la plus grande taille.
#[test]
fn a_very_long_message_still_collapses_at_the_largest_size() {
    let mut combo = default_combo("message long", (860.0, 600.0));
    combo.appearance.text = scaled(1.6, 1.6);
    let mut run = build(&combo);
    {
        let app = run.app.borrow();
        let mut state = app.state.lock().unwrap();
        state.messages.push(crate::message::ChatMessage {
            from: "alice".into(),
            content: "mot ".repeat(1500),
            timestamp: "12:01".into(),
            timestamp_epoch: Some(99),
            to_user: Some("moi".into()),
            media: None,
            reply_to: None,
            nonce: None,
        });
        state.bump_content();
    }
    run.settle();
    run.check_layout("message long");
    let show_more = run
        .harness
        .query_all_by_label_contains("Afficher la suite")
        .count();
    assert_eq!(show_more, 1, "[{}] le repli est proposé", run.name);
}

/// Au clavier seul : on atteint les cartes de thème, on en choisit une.
#[test]
fn the_theme_gallery_is_usable_with_the_keyboard_alone() {
    let mut run = build(&default_combo("clavier", (860.0, 600.0)));
    run.open_settings(SettingsTab::Appearance);
    let classic = format!("{} {}", fr(i18n::THEME), fr(BuiltinTheme::Classic.name()));
    run.harness.get_by_label(&classic).focus();
    run.settle();
    run.harness.key_press(egui::Key::Tab);
    run.settle();
    run.harness.key_press(egui::Key::Space);
    run.settle();
    assert_eq!(run.app.borrow().appearance.theme, BuiltinTheme::Lounge.id());
    run.harness.key_press(egui::Key::Tab);
    run.settle();
    run.harness.key_press(egui::Key::Enter);
    run.settle();
    assert_eq!(run.app.borrow().appearance.theme, BuiltinTheme::Frost.id());
}

/// Chaque contrôle de l'onglet a un nom pour un lecteur d'écran.
#[test]
fn every_control_of_the_appearance_tab_has_an_accessible_name() {
    use egui::accesskit::Role;
    let mut run = build(&default_combo("accessibilité", (860.0, 600.0)));
    run.open_settings(SettingsTab::Appearance);
    for label in [
        i18n::APPARENCE_THEMES,
        i18n::APPARENCE_TEXTE,
        i18n::APPARENCE_COULEURS,
        i18n::APPARENCE_POLICES,
    ] {
        let is_colors = label.fr == i18n::APPARENCE_COULEURS.fr;
        let title = fr(label);
        run.click(title);
        if is_colors {
            run.click(fr(i18n::OPTIONS_AVANCEES));
        }
        for node in run.harness.root().children_recursive() {
            let access = node.accesskit_node();
            if !matches!(
                access.role(),
                Role::Button | Role::Slider | Role::RadioButton | Role::CheckBox | Role::TextInput
            ) || access.is_hidden()
            {
                continue;
            }
            let name = access.label().unwrap_or_default();
            assert!(
                !name.trim().is_empty() || access.role() == Role::Button,
                "[{}] un contrôle {:?} sans nom dans « {} »",
                run.name,
                access.role(),
                title
            );
        }
    }
}

/// Dossier des thèmes en lecture seule : le thème sert pour la session, un seul avis.
#[cfg(unix)]
#[test]
fn a_read_only_data_folder_keeps_the_theme_for_the_session_and_warns_once() {
    use std::os::unix::fs::PermissionsExt;
    let mut run = build(&default_combo("lecture seule", (860.0, 600.0)));
    let themes_dir = {
        let app = run.app.borrow();
        let dir = app
            .state
            .lock()
            .unwrap()
            .media_dir()
            .parent()
            .unwrap()
            .to_path_buf();
        dir
    };
    std::fs::set_permissions(&themes_dir, std::fs::Permissions::from_mode(0o500)).unwrap();
    if std::fs::write(themes_dir.join("sonde"), "x").is_ok() {
        std::fs::set_permissions(&themes_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        return;
    }
    run.open_settings(SettingsTab::Appearance);
    run.click(fr(i18n::APPARENCE_COULEURS));
    run.harness
        .get_by_role_and_label(
            egui::accesskit::Role::TextInput,
            &format!("{} : {}", fr(i18n::ROLE_FOND), fr(i18n::CODE_COULEUR)),
        )
        .focus();
    run.settle();
    run.harness
        .key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    run.harness
        .get_by_role_and_label(
            egui::accesskit::Role::TextInput,
            &format!("{} : {}", fr(i18n::ROLE_FOND), fr(i18n::CODE_COULEUR)),
        )
        .type_text("#123456");
    run.settle();
    let app = run.app.borrow();
    assert!(app.appearance.custom_id().is_some(), "le thème reste actif");
    assert_eq!(
        app.last_notification.as_deref(),
        Some(fr(i18n::THEME_NON_ENREGISTRE))
    );
    assert!(app.themes.has_unsaved());
    drop(app);
    std::fs::set_permissions(&themes_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
}

/// Mode Système : l'application suit le thème du système quand il change en cours de route.
#[test]
fn the_system_mode_follows_the_operating_system_while_running() {
    let mut app = new_app(None);
    assert_eq!(app.appearance.mode, Mode::System);
    let ctx = egui::Context::default();
    let mut previous_revision = 0;
    for theme in [
        egui::Theme::Dark,
        egui::Theme::Light,
        egui::Theme::Dark,
        egui::Theme::Light,
    ] {
        let raw = egui::RawInput {
            system_theme: Some(theme),
            ..Default::default()
        };
        let mut output = ctx.run_ui(raw, |ui| {
            app.sync_appearance(ui.ctx());
            app.refresh_caches(ui.ctx());
        });
        output.textures_delta.clear();
        assert_eq!(ctx.theme(), theme, "le thème suit celui du système");
        previous_revision = super::theme::current(&ctx).revision.max(previous_revision);
    }
    // Un thème forcé ne le suit plus.
    app.appearance.mode = Mode::Dark;
    let raw = egui::RawInput {
        system_theme: Some(egui::Theme::Light),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw, |ui| app.sync_appearance(ui.ctx()));
    output.textures_delta.clear();
    assert_eq!(ctx.theme(), egui::Theme::Dark);
}
