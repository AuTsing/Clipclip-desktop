mod clipboard;
mod storage;
mod tray;

use crate::clipboard::Clipboard;
use crate::tray::Tray;
use eframe::Renderer;
use eframe::egui;
use eframe::egui::ViewportCommand;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        renderer: Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Clipclip",
        options,
        Box::new(|_cc| Ok(Box::<Clipclip>::default())),
    )
}

struct Clipclip {
    status: String,
    clipboard: Clipboard,
    tray: Tray,
}

impl Default for Clipclip {
    fn default() -> Self {
        Self {
            status: "".to_string(),
            clipboard: Clipboard::new(),
            tray: Tray::new(),
        }
    }
}

impl eframe::App for Clipclip {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Up").clicked() {
                    self.status = "Clipclip up".to_string();
                }
                if ui.button("Down").clicked() {
                    self.status = "Clipclip down".to_string();
                }
                if ui.button("Test Load").clicked() {
                    match self.clipboard.load_all() {
                        Ok(it) => self.status = it.len().to_string(),
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
                if ui.button("Test Save").clicked() {
                    match self.clipboard.save_latest() {
                        Ok(_) => self.status = "Save success".to_string(),
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
            });
            ui.label(format!("Status: {}", &self.status));
        });

        if ui.input(|i| i.viewport().close_requested()) {
            ui.send_viewport_cmd(ViewportCommand::CancelClose);
            ui.send_viewport_cmd(ViewportCommand::Visible(false));
        }
    }
}
