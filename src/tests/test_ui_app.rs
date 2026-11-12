//! Rendu headless de l'arbre d'interface complet : détecte panique, identifiants
//! egui dupliqués et ordre de panneaux invalide sans ouvrir de fenêtre.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::egui;
use tokio::sync::mpsc;

use super::{i18n, AbcomApp, UiLanguage, UiRuntimeChannels};
use crate::app::AppState;
use crate::network::secure::TrustStore;

#[test]
fn application_png_contains_the_coral_brand_and_transparent_corners() {
    let rgba = image::load_from_memory(include_bytes!("../../assets/app_icon.png"))
        .unwrap()
        .to_rgba8();
    let icon = egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    };
    assert_eq!((icon.width, icon.height), (512, 512));
    assert_eq!(icon.rgba[3], 0);
    assert!(icon.rgba.as_chunks::<4>().0.contains(&[255, 138, 96, 255]));
    assert!(icon.rgba.as_chunks::<4>().0.contains(&[48, 40, 48, 255]));
}

#[test]
fn legacy_macos_dock_icon_has_its_own_optical_margins() {
    let icon = image::load_from_memory(include_bytes!(
        "../../assets/branding/png/abcom-dock-512.png"
    ))
    .unwrap()
    .to_rgba8();
    let ink: Vec<_> = icon
        .enumerate_pixels()
        .filter(|(_, _, p)| p[3] > 128)
        .collect();
    let left = ink.iter().map(|(x, _, _)| *x).min().unwrap();
    let right = ink.iter().map(|(x, _, _)| *x).max().unwrap();
    let top = ink.iter().map(|(_, y, _)| *y).min().unwrap();
    let bottom = ink.iter().map(|(_, y, _)| *y).max().unwrap();
    assert_eq!(icon.dimensions(), (512, 512));
    assert!((49..=51).contains(&left) && (460..=463).contains(&right));
    assert_eq!((left, right), (top, bottom));
    assert!(
        (410..=414).contains(&(right - left + 1)),
        "le fond ICNS ne doit pas remplir son canevas"
    );
}

fn test_app() -> AbcomApp {
    let dir = std::env::temp_dir().join(format!(
        "abcom-ui-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let state = Arc::new(Mutex::new(AppState::new_with_base("moi", &dir)));
    let (send_tx, _send_rx) = mpsc::channel(16);
    let (send_media_tx, _media_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(16);
    let (_offer_tx, media_offer_rx) = mpsc::channel(16);
    AbcomApp::new(
        state,
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

/// Peint `frames` frames de l'arbre complet, dans les deux thèmes.
fn render(app: &mut AbcomApp, frames: usize) {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        render_with_theme(app, frames, theme);
    }
}

fn render_with_theme(app: &mut AbcomApp, frames: usize, theme: egui::Theme) {
    let ctx = egui::Context::default();
    ctx.set_theme(theme);
    for _ in 0..frames {
        let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
            app.show_sidebar_panel(root);
            let (emoji, gif) = app.show_input_bar(root);
            let ctx = root.ctx().clone();
            app.show_notification(&ctx);
            app.show_emoji_picker_window(&ctx, emoji);
            app.show_gif_picker_window(&ctx, gif);
            app.render_group_modal(&ctx);
            app.render_group_manage_modal(&ctx);
            app.show_central_panel(root);
            app.show_reaction_emoji_picker(&ctx);
            app.render_settings(&ctx);
            app.show_media_viewer(&ctx);
            app.show_search(&ctx);
        });
        // epaint panique si les deltas de textures d'une frame sont abandonnés.
        output.textures_delta.clear();
    }
}

#[test]
fn renders_the_whole_tree_without_panicking() {
    let mut app = test_app();
    render(&mut app, 3);
}

#[test]
fn renders_with_messages_and_a_selected_conversation() {
    let mut app = test_app();
    {
        let mut s = app.state.lock().unwrap();
        s.add_peer("alice".into(), "127.0.0.1:9000".parse().unwrap());
        s.add_message(crate::message::ChatMessage {
            from: "alice".into(),
            content: "**gras** et `code` :tada:".into(),
            timestamp: "12:00".into(),
            timestamp_epoch: Some(1),
            to_user: Some("moi".into()),
            media: None,
            reply_to: None,
            nonce: None,
        });
        s.selected_conversation = Some("alice".into());
    }
    render(&mut app, 3);
}

#[test]
fn survives_a_hide_and_restore_cycle() {
    let mut app = test_app();
    let ctx = egui::Context::default();
    render(&mut app, 1);
    // Libère textures et images du chargeur.
    app.hide_to_tray(&ctx);
    app.show_from_tray(&ctx);
    // Tout doit se reconstruire : un rendu qui panique ici signalerait qu'on a
    // libéré une ressource qu'egui croyait encore allouée.
    render(&mut app, 2);
}

#[test]
fn renders_every_modal_and_picker_open() {
    let mut app = test_app();
    app.modals.settings_open = true;
    app.modals.group_modal_open = true;
    app.modals.key_mismatch = Some(("alice".into(), vec![7u8; 32]));
    app.modals.rename_target = Some("alice".into());
    app.modals.participants_open = true;
    app.show_emoji_picker = true;
    app.gif_picker.show = true;
    app.last_notification = Some("coucou".into());
    app.search.open = true;
    app.search.query = "bureau".into();
    render(&mut app, 3);
}

/// Les onglets de Paramètres ne s'ouvraient dans aucun test : Crédits et
/// Licence n'étaient jamais peints, donc ni panique ni identifiant egui
/// dupliqué n'y auraient été détectés.
#[test]
fn renders_every_settings_tab() {
    use crate::ui::SettingsTab;

    let mut app = test_app();
    app.modals.settings_open = true;
    for tab in [
        SettingsTab::Profile,
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Storage,
        SettingsTab::Credits,
        SettingsTab::License,
    ] {
        app.modals.settings_tab = tab;
        render(&mut app, 2);
    }
}

/// `process_events` tient le verrou d'état pendant toute sa boucle : un
/// gestionnaire qui le reprend fige l'application entière, fenêtre comprise —
/// c'est arrivé sur le compte rendu de purge, l'utilisateur ne pouvait plus ni
/// fermer ni ouvrir quoi que ce soit.
///
/// `AbcomApp` n'est pas `Send` (le tray), donc pas de chien de garde sur un
/// autre thread : un interblocage se manifeste ici en ne rendant jamais la
/// main. Les assertions vérifient que l'événement a bien été traité.
#[test]
fn a_purge_report_is_handled_without_retaking_the_state_lock() {
    use crate::app::media::GcReport;
    use crate::message::AppEvent;

    let mut app = test_app();
    let report = |dry_run| {
        AppEvent::MediaPurged(GcReport {
            freed_bytes: 1024,
            freed_files: 2,
            dry_run,
        })
    };

    app.net.event_tx.try_send(report(true)).unwrap();
    app.process_events();
    assert_eq!(
        app.purge_preview.map(|r| r.freed_bytes),
        Some(1024),
        "une simulation doit alimenter l'aperçu"
    );
    assert!(!app.purge_preview_pending);

    app.net.event_tx.try_send(report(false)).unwrap();
    app.process_events();
    assert!(
        app.purge_preview.is_none(),
        "une purge réelle périme l'aperçu"
    );
    let notice = app.last_notification.expect("purge annoncée");
    assert!(notice.contains("1.0 ko"), "compte rendu chiffré : {notice}");
}

/// Le rattrapage d'accusés de lecture doit fonctionner sans changer de
/// conversation — c'est ce que `logic` déclenche au retour du focus — et ne
/// pas réémettre à chaque passage.
#[test]
fn read_receipts_are_swept_without_switching_conversation() {
    let mut app = test_app();
    let (send_tx, mut send_rx) = mpsc::channel(64);
    app.net.send_tx = send_tx;

    let conv = {
        let mut s = app.state.lock().unwrap();
        s.peers.push(crate::app::Peer {
            username: "alice".to_string(),
            addr: "127.0.0.1:9000".parse().unwrap(),
            last_seen: 0,
            online: true,
        });
        let group = s
            .create_group("equipe".to_string(), vec!["alice".to_string()])
            .expect("création du salon");
        let conv = AppState::group_conv_key(&group.id);
        // Message reçu pendant que la fenêtre n'avait pas le focus.
        s.add_message(crate::message::ChatMessage {
            from: "alice".to_string(),
            content: "coucou".to_string(),
            timestamp: "12:00".to_string(),
            timestamp_epoch: Some(1),
            to_user: Some(conv.clone()),
            media: None,
            reply_to: None,
            nonce: Some(1),
        });
        s.selected_conversation = Some(conv.clone());
        conv
    };

    app.send_read_receipts_for_conversation(Some(conv.clone()));
    assert!(
        send_rx.try_recv().is_ok(),
        "l'accusé doit partir sans quitter puis rouvrir le salon"
    );

    app.send_read_receipts_for_conversation(Some(conv));
    assert!(
        send_rx.try_recv().is_err(),
        "et ne pas être réémis au passage suivant"
    );
}

/// Entrée insère une nouvelle ligne, l'envoi passe par Cmd/Ctrl+Entrée. egui
/// active pourtant un bouton qui a le focus sur Entrée comme sur Espace : il
/// suffisait qu'un clic ait laissé le focus sur « Envoyer » — au retour d'un
/// sélecteur de fichiers, par exemple — pour que la touche envoie le message au
/// lieu d'aller à la ligne.
#[test]
fn enter_on_the_focused_send_button_goes_to_the_line_instead_of_sending() {
    let mut app = test_app();
    let (send_tx, mut send_rx) = mpsc::channel(16);
    app.net.send_tx = send_tx;
    {
        let mut s = app.state.lock().unwrap();
        s.add_peer("alice".into(), "127.0.0.1:9000".parse().unwrap());
        s.selected_conversation = Some("alice".into());
    }
    app.composer.text = "bonjour".to_string();
    app.composer.cursor_char = app.composer.text.chars().count();

    let ready = std::cell::Cell::new(false);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(700.0, 340.0))
        .build_ui(move |root| {
            if !ready.get() {
                // Les familles grasse et italique ne sont liées qu'ici : sans
                // elles, le premier nom d'auteur peint fait paniquer epaint.
                crate::ui::appearance::apply(
                    root.ctx(),
                    crate::ui::themes::BuiltinTheme::Classic,
                    &Default::default(),
                );
                ready.set(true);
                return;
            }
            {
                let s = app.state.lock().unwrap();
                app.sidebar_cache.refresh(&s);
                app.chat_cache.refresh(
                    &s,
                    app.ui_language,
                    &app.emoji.map,
                    true,
                    &crate::ui::theme::ResolvedTheme::classic(),
                );
            }
            app.show_sidebar_panel(root);
            let _ = app.show_input_bar(root);
            app.show_central_panel(root);
        });
    for _ in 0..3 {
        harness.step();
    }

    use egui_kittest::kittest::Queryable as _;
    harness
        .get_by_label(i18n::ENVOYER.get(UiLanguage::French))
        .focus();
    harness.step();
    harness.key_press(egui::Key::Enter);
    harness.step();
    harness.step();
    assert!(
        send_rx.try_recv().is_err(),
        "Entrée seule sur le bouton focalisé doit aller à la ligne, pas envoyer"
    );

    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Enter);
    harness.step();
    harness.step();
    assert!(
        send_rx.try_recv().is_ok(),
        "Cmd+Entrée doit continuer d'envoyer, focus ou pas"
    );
}

/// Rend `frames` images dans une fenêtre de `size` points à la densité `ppp`,
/// avec les Paramètres ouverts, et renvoie le rectangle de leur boîte modale.
fn settings_rect_in_window(
    app: &mut AbcomApp,
    size: egui::Vec2,
    ppp: f32,
    frames: usize,
) -> Option<egui::Rect> {
    let ctx = egui::Context::default();
    ctx.set_pixels_per_point(ppp);
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        ..Default::default()
    };
    for _ in 0..frames {
        let mut output = ctx.run_ui(raw.clone(), |root| {
            app.show_sidebar_panel(root);
            app.show_central_panel(root);
            let ctx = root.ctx().clone();
            app.render_settings(&ctx);
        });
        output.textures_delta.clear();
    }
    ctx.memory(|m| m.area_rect(egui::Id::unique("settings_modal")))
}

/// La fenêtre des Paramètres faisait 640 × 480 quelle que soit la fenêtre : dans
/// la plus petite permise (560 × 360), elle en sortait de partout.
#[test]
fn settings_window_fits_in_every_window_size_and_tab() {
    use crate::ui::SettingsTab;

    let mut app = test_app();
    app.modals.settings_open = true;
    for (width, height) in [(560.0, 360.0), (860.0, 600.0), (1440.0, 900.0)] {
        for ppp in [1.0, 2.0] {
            for tab in [
                SettingsTab::Profile,
                SettingsTab::General,
                SettingsTab::Appearance,
                SettingsTab::Storage,
                SettingsTab::Credits,
                SettingsTab::License,
            ] {
                app.modals.settings_tab = tab;
                let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, height));
                let rect = settings_rect_in_window(&mut app, screen.size(), ppp, 3)
                    .unwrap_or_else(|| panic!("Paramètres absents ({width}×{height})"));
                assert!(
                    screen.contains_rect(rect),
                    "Paramètres {rect:?} hors de la fenêtre {screen:?} (onglet {}, ppp {ppp})",
                    match tab {
                        SettingsTab::Profile => "Profil",
                        SettingsTab::General => "Général",
                        SettingsTab::Appearance => "Apparence",
                        SettingsTab::Storage => "Stockage",
                        SettingsTab::Credits => "Crédits",
                        SettingsTab::License => "Licence",
                    }
                );
            }
        }
    }
}

#[test]
fn settings_window_keeps_its_size_on_a_large_screen() {
    let mut app = test_app();
    app.modals.settings_open = true;
    let rect = settings_rect_in_window(&mut app, egui::vec2(1440.0, 900.0), 1.0, 3)
        .expect("Paramètres ouverts");
    assert!(
        rect.width() >= 760.0 && rect.height() >= 560.0,
        "sur grand écran la fenêtre garde sa taille nominale : {rect:?}"
    );
}
