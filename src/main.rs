use std::sync::Arc;

use arboard::Clipboard;
use eframe::egui::{self, mutex::Mutex};

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
    clipboard: Arc<Mutex<Clipboard>>,
}

impl Default for Clipclip {
    fn default() -> Self {
        Self {
            status: "".to_string(),
            history: "".to_string(),
            clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
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
                if ui.button("Read").clicked() {
                    match Clipclip::read(&self) {
                        Ok(it) => self.history = it,
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
                if ui.button("Write").clicked() {
                    match Clipclip::write(&self) {
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

trait Readable {
    fn read(&self) -> Result<String, arboard::Error>;
}

impl Readable for Clipclip {
    fn read(&self) -> Result<String, arboard::Error> {
        let mut clipboard = self.clipboard.lock();
        let text = clipboard.get_text()?;
        Ok(text)
    }
}

trait Writable {
    fn write(&self) -> Result<(), arboard::Error>;
}

impl Writable for Clipclip {
    fn write(&self) -> Result<(), arboard::Error> {
        let mut clipboard = self.clipboard.lock();
        clipboard.set_text("")?;
        Ok(())
    }
}
