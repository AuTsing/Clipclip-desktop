mod clipboard;
mod storage;

use crate::clipboard::Clipboard;
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
    history: String,
    clipboard: Clipboard,
}

impl Default for Clipclip {
    fn default() -> Self {
        Self {
            status: "".to_string(),
            history: "".to_string(),
            clipboard: Clipboard::new(),
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
                if ui.button("Test Read").clicked() {
                    match self.clipboard.read_latest() {
                        Ok(it) => self.history = it,
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
                if ui.button("Test Write").clicked() {
                    match self.clipboard.write_latest() {
                        Ok(_) => self.status = "Write success".to_string(),
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
            });
            ui.label(format!("Status: {}", &self.status));
            ui.label(format!("Clipboard: {}", &self.history));
        });
    }
}
