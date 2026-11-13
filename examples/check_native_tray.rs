//! Native tray/window probe for disposable visual checks, without personal data.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use abcom::{platform, util};
use eframe::egui;
use util::MutexExt;

#[allow(dead_code)]
#[path = "../src/platform/icons.rs"]
mod actual_icons;
#[allow(dead_code)]
#[path = "../src/platform/tray.rs"]
mod actual_tray;

struct Probe {
    tray: Option<actual_tray::Tray>,
    initialized: bool,
    unread: bool,
    started: Instant,
    failure: Arc<Mutex<Option<String>>>,
}

impl eframe::App for Probe {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.initialized {
            self.tray = actual_tray::Tray::new("Open", "Quit");
            if let Some(tray) = &mut self.tray {
                tray.set_unread(self.unread);
                println!("Native tray created; unread={}", self.unread);
            } else {
                *self.failure.lock_safe() = Some("native tray creation failed".into());
            }
            self.initialized = true;
        }
        egui::CentralPanel::default().show(root, |ui| {
            ui.heading("Abcom — native icon check");
            ui.label(if self.unread {
                "Unread badge"
            } else {
                "No unread messages"
            });
            ui.label("Disposable diagnostic: no account, network or personal data.");
        });
        if self.started.elapsed() > Duration::from_secs(8) {
            root.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            root.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let unread = std::env::args().any(|arg| arg == "--unread");
    let dark = std::env::args().any(|arg| arg == "--dark");
    let failure = Arc::new(Mutex::new(None));
    let result = failure.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 150.0])
            .with_icon(actual_icons::window_icon_data().ok_or("missing window icon")?),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Abcom native icons",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            Ok(Box::new(Probe {
                tray: None,
                initialized: false,
                unread,
                started: Instant::now(),
                failure: result,
            }))
        }),
    )?;
    if let Some(error) = failure.lock_safe().take() {
        return Err(error.into());
    }
    Ok(())
}
