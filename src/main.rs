mod clipboard;
mod server;
mod storage;
mod tray;

use crate::{clipboard::Clipboard, server::Server, storage::Storage, tray::Tray};
use eframe::{
    Renderer, UserEvent,
    egui::{CentralPanel, Ui, ViewportBuilder, ViewportCommand},
};
use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender, channel},
};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        renderer: Renderer::Glow,
        ..Default::default()
    };

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let mut app = eframe::create_native(
        "Clipclip",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            let (save_clip_tx, save_clip_rx) = channel::<String>();
            let (set_clip_tx, set_clip_rx) = channel::<String>();
            let (get_clip_tx, get_clip_rx) = channel::<Sender<String>>();
            let (exited_tx, exited_rx) = channel::<()>();

            let mut storage = Storage::new();
            storage.start_listening_save_clip(save_clip_rx);

            let mut clipboard = Clipboard::new();
            clipboard.start_listening_clip_change(save_clip_tx.clone());
            clipboard.start_listening_set_clip(set_clip_rx);
            clipboard.start_listening_get_clip(get_clip_rx);

            let mut server = Server::new();
            server.start_listening(
                save_clip_tx.clone(),
                set_clip_tx.clone(),
                get_clip_tx.clone(),
            );

            let tray = Tray::new();
            tray.start_listening_events(ctx, exited_tx);

            let clipclip = Clipclip::new(exited_rx, tray, storage, clipboard, server);

            Ok(Box::new(clipclip))
        }),
        &event_loop,
    );

    event_loop.run_app(&mut app)?;

    Ok(())
}

struct Clipclip {
    status: String,
    exited_rx: Receiver<()>,
    _tray: Tray,
    storage: Storage,
    _clipboard: Clipboard,
    _server: Server,
}

impl Clipclip {
    fn new(
        exited_rx: Receiver<()>,
        tray: Tray,
        storage: Storage,
        clipboard: Clipboard,
        server: Server,
    ) -> Self {
        Self {
            status: "".to_string(),
            exited_rx,
            _tray: tray,
            storage,
            _clipboard: clipboard,
            _server: server,
        }
    }
}

impl eframe::App for Clipclip {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.viewport().close_requested()) && self.exited_rx.try_recv().is_err() {
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
                if ui.button("Load").clicked() {
                    match self.storage.get_all_clips() {
                        Ok(it) => self.status = it.len().to_string(),
                        Err(e) => self.status = format!("{:?}", e),
                    };
                }
            });
            ui.label(format!("Status: {}", &self.status));
        });
    }
}
