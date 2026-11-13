use std::path::PathBuf;

use super::{attachment_label, push_unique_paths, should_send_message};

#[test]
fn toolbar_text_and_icons_have_identical_hit_boxes() {
    use eframe::egui;
    for scale in [1.0, 1.5, 2.0] {
        let ctx = egui::Context::default();
        ctx.set_zoom_factor(scale);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.horizontal(|ui| {
                let gif = super::widgets::action_button(
                    ui,
                    egui::RichText::new("GIF").size(10.5),
                    "GIF",
                    false,
                );
                let icon = super::widgets::icon_button(ui, "Ajouter", false, |_, _, _| {});
                assert_eq!(gif.rect.size(), icon.rect.size());
                assert_eq!(gif.rect.size(), egui::vec2(28.0, 28.0));
            });
        });
        output.textures_delta.clear();
    }
}

#[test]
fn gif_button_keeps_its_frame_and_lettering_still_on_hover() {
    use eframe::egui;

    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        for pixels_per_point in [1.0, 1.5, 2.0] {
            for selected in [false, true] {
                let ctx = egui::Context::default();
                ctx.set_theme(theme);
                ctx.set_pixels_per_point(pixels_per_point);
                let mut baseline = None;
                let mut button = egui::Rect::NOTHING;
                for frame in 0..24 {
                    let pointer = match frame {
                        0..=3 | 20.. => egui::pos2(300.0, 100.0),
                        4..=11 => button.center(),
                        _ => button.left_center() + egui::vec2(0.5, 0.0),
                    };
                    let mut response = None;
                    let mut output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(400.0, 200.0),
                            )),
                            time: Some(f64::from(frame) / 60.0),
                            events: vec![egui::Event::PointerMoved(pointer)],
                            ..Default::default()
                        },
                        |root| {
                            root.horizontal(|ui| {
                                response = Some(super::widgets::action_button(
                                    ui,
                                    egui::RichText::new("GIF").size(10.5),
                                    "GIF",
                                    selected,
                                ));
                            });
                        },
                    );
                    output.textures_delta.clear();
                    let response = response.unwrap();
                    button = response.rect;
                    let text = output.shapes.iter().find_map(|shape| {
                        if let egui::epaint::Shape::Text(text) = &shape.shape {
                            (text.galley.text() == "GIF").then_some((text.pos, text.galley.size()))
                        } else {
                            None
                        }
                    });
                    let geometry = (button, text.expect("GIF lettering must be visible"));
                    if frame == 3 {
                        baseline = Some(geometry);
                    } else if frame > 3 {
                        assert_eq!(
                            geometry,
                            baseline.unwrap(),
                            "GIF moved at frame {frame}, {theme:?}, {pixels_per_point}×, selected={selected}"
                        );
                        assert_eq!(response.hovered(), frame < 20);
                    }
                }
            }
        }
    }
}

#[test]
fn enter_from_composer_sends_when_shortcode_menu_is_closed() {
    assert!(should_send_message(true, false, false, "hello"));
}

#[test]
fn enter_fallback_does_not_send_when_shortcode_menu_is_open() {
    assert!(!should_send_message(false, true, true, ":jo"));
}

#[test]
fn enter_fallback_sends_when_shortcode_menu_is_closed() {
    assert!(should_send_message(false, true, false, "hello"));
}

#[test]
fn empty_message_never_sends() {
    assert!(!should_send_message(true, true, false, "   "));
}

#[test]
fn push_unique_paths_ignores_duplicates() {
    let mut paths = vec![PathBuf::from("/tmp/alpha.txt")];

    push_unique_paths(
        &mut paths,
        [
            PathBuf::from("/tmp/alpha.txt"),
            PathBuf::from("/tmp/beta.txt"),
            PathBuf::from("/tmp/beta.txt"),
        ],
    );

    assert_eq!(
        paths,
        vec![
            PathBuf::from("/tmp/alpha.txt"),
            PathBuf::from("/tmp/beta.txt")
        ]
    );
}

#[test]
fn attachment_label_prefers_file_name() {
    assert_eq!(
        attachment_label(PathBuf::from("/tmp/subdir/report.pdf").as_path()),
        "report.pdf"
    );
}

#[test]
fn attachment_label_falls_back_to_full_path_when_needed() {
    let path = PathBuf::from("/");
    assert_eq!(attachment_label(path.as_path()), "/");
}

// ── Garde-fou de taille filaire à l'envoi ─────────────────────────────────

fn sized_message(content_len: usize) -> crate::message::ChatMessage {
    crate::message::ChatMessage {
        from: "alice".to_string(),
        content: "a".repeat(content_len),
        timestamp: "12:00".to_string(),
        timestamp_epoch: Some(1_750_000_000),
        to_user: None,
        media: None,
        reply_to: None,
        nonce: Some(42),
    }
}

#[test]
fn normal_message_fits_on_wire() {
    let msg = sized_message(crate::ui::composer::MAX_INPUT_CHARS);
    assert!(super::chat_wire_size(&msg) <= crate::network::secure::MAX_LOGICAL_MESSAGE);
}

#[test]
fn oversized_message_is_detected_before_send() {
    // Au-delà de la limite que la réception rejette (connexion coupée) :
    // l'envoi doit être refusé en amont.
    let msg = sized_message(9 * 1024 * 1024);
    assert!(super::chat_wire_size(&msg) > crate::network::secure::MAX_LOGICAL_MESSAGE);
}
