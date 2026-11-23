use super::*;
use crate::app::usage::{Entry, Usage};
use crate::ui::appearance_e2e_tests::{new_app, Session};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

fn session(tab: SettingsTab, size: egui::Vec2, scale: f32, language: UiLanguage) -> Session {
    let mut app = new_app(None);
    app.modals.settings_open = true;
    app.modals.settings_tab = tab;
    app.appearance.text.global_scale = scale;
    app.ui_language = language;
    app.storage_usage = Some(Usage {
        media_received: Entry {
            bytes: 1_234_567_890,
            files: 12345,
        },
        media_sent: Entry {
            bytes: 987_654_321,
            files: 9876,
        },
        ..Usage::default()
    });
    app.purge_preview = Some(crate::app::media::GcReport::default());
    let app = std::rc::Rc::new(std::cell::RefCell::new(app));
    let shared = std::rc::Rc::clone(&app);
    let harness = egui_kittest::Harness::builder()
        .with_size(size)
        .build_ui(move |ui| {
            let mut app = shared.borrow_mut();
            app.sync_appearance(ui.ctx());
            app.render_settings(ui.ctx());
        });
    let mut session = Session { harness, app };
    session.settle();
    session
}

#[test]
fn loaded_settings_text_and_controls_fit_the_content_width() {
    for language in [UiLanguage::French, UiLanguage::English] {
        for scale in [1.0, 1.6] {
            for size in [egui::vec2(560.0, 360.0), egui::vec2(860.0, 600.0)] {
                for tab in [
                    SettingsTab::Profile,
                    SettingsTab::General,
                    SettingsTab::Storage,
                    SettingsTab::Credits,
                    SettingsTab::License,
                ] {
                    let s = session(tab, size, scale, language);
                    let modal = s
                        .ctx()
                        .memory(|m| m.area_rect(egui::Id::unique("settings_modal")))
                        .unwrap();
                    for node in s.harness.root().children_recursive() {
                        if !node.accesskit_node().has_bounds()
                            || node.accesskit_node().is_hidden()
                            || node.rect().height() <= 0.0
                        {
                            continue;
                        }
                        let rect = node.rect();
                        if node.accesskit_node().label().is_none() {
                            continue;
                        }
                        assert!(
                            rect.min.x >= modal.min.x - 1.0 && rect.max.x <= modal.max.x + 1.0,
                            "onglet {}, langue {}, {scale}, {size:?}: {:?} {:?} hors de {modal:?}",
                            tab as u8,
                            language as u8,
                            node.accesskit_node().role(),
                            node.accesskit_node().label()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn storage_category_labels_remain_readable() {
    let s = session(
        SettingsTab::Storage,
        egui::vec2(860.0, 600.0),
        1.0,
        UiLanguage::French,
    );
    let label = s.harness.get_by_label(i18n::TRANSFERTS_INACHEVES.fr);
    assert!(
        label.rect().width() >= 120.0,
        "colonne de catégories écrasée : {:?}",
        label.rect()
    );
    assert!(
        label.rect().height() < 50.0,
        "catégorie découpée sur trop de lignes : {:?}",
        label.rect()
    );
}

#[test]
fn credits_labels_remain_on_one_line_at_normal_size() {
    let s = session(
        SettingsTab::Credits,
        egui::vec2(860.0, 600.0),
        1.0,
        UiLanguage::French,
    );
    assert!(
        s.harness
            .get_by_label(i18n::DEVELOPPEURS.fr)
            .rect()
            .height()
            < 25.0
    );
}

#[test]
fn license_reflow_preserves_every_word() {
    assert_eq!(
        LICENSE_TEXT.split_whitespace().collect::<Vec<_>>(),
        license_text_for_display()
            .split_whitespace()
            .collect::<Vec<_>>()
    );
    assert!(license_text_for_display()
        .lines()
        .all(|line| line == line.trim()));
}

#[test]
fn license_text_follows_the_general_text_size() {
    fn font_size(scale: f32) -> f32 {
        let s = session(
            SettingsTab::License,
            egui::vec2(860.0, 600.0),
            scale,
            UiLanguage::French,
        );
        s.harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    if text.galley.job.text == license_text_for_display() {
                        return Some(text.galley.job.sections[0].format.font_id.size);
                    }
                }
                None
            })
            .expect("texte de licence peint")
    }
    assert!(font_size(1.6) > font_size(1.0) * 1.5);
}

#[test]
fn storage_purge_controls_are_reachable_and_preferences_are_saved() {
    let mut s = session(
        SettingsTab::Storage,
        egui::vec2(560.0, 360.0),
        1.6,
        UiLanguage::French,
    );
    s.harness
        .get_by_label(i18n::INCLURE_LES_IMAGES.fr)
        .scroll_to_me();
    s.settle();
    s.harness.get_by_label(i18n::INCLURE_LES_IMAGES.fr).click();
    s.settle();
    let app = s.app.borrow();
    assert!(app.purge_includes_images);
    assert_eq!(
        app.state.lock_safe().pref_str("media_purge_images"),
        Some("1")
    );
    drop(app);
    s.harness
        .get_by_label(i18n::OUVRIR_LE_DOSSIER.fr)
        .scroll_to_me();
    s.settle();
    let rect = s.harness.get_by_label(i18n::OUVRIR_LE_DOSSIER.fr).rect();
    assert!(
        s.ctx().content_rect().contains_rect(rect),
        "bouton inaccessible : {rect:?}"
    );
}
