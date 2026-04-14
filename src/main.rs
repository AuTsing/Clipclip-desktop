mod clipboard;
mod storage;
mod tray;

use crate::{clipboard::Clipboard, tray::Tray};
use eframe::{
    Renderer, UserEvent,
    egui::{CentralPanel, Context, Pos2, Ui, ViewportBuilder, ViewportCommand, pos2},
};
use std::error::Error;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_position(pos2(300.0, 300.0)),
        renderer: Renderer::Glow,
        ..Default::default()
    };

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let mut app = eframe::create_native(
        "Clipclip",
        options,
        Box::new(|cc| {
            let clipclip = Clipclip::new(cc.egui_ctx.clone());
            Ok(Box::new(clipclip))
        }),
        &event_loop,
    );

    event_loop.run_app(&mut app)?;

    Ok(())
}

struct Clipclip {
    status: String,
    exited: bool,
    clipboard: Clipboard,
    tray: Tray,
}

impl Clipclip {
    fn new(ctx: Context) -> Self {
        Self {
            status: "".to_string(),
            exited: false,
            clipboard: Clipboard::new(),
            tray: Tray::new(ctx),
        }
    }
}

impl eframe::App for Clipclip {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.tray.on_double_click.try_recv() {
            ui.send_viewport_cmd(ViewportCommand::Visible(true));
        }

        if let Ok(_) = self.tray.on_exit.try_recv() {
            self.exited = true;
            ui.send_viewport_cmd(ViewportCommand::Close);
        }

        if ui.input(|i| i.viewport().close_requested() && !self.exited) {
            ui.send_viewport_cmd(ViewportCommand::CancelClose);
            ui.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        CentralPanel::default().show_inside(ui, |ui| {
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
    }
}
