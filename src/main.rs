mod clipboard;
mod server;
mod storage;
mod tray;

use crate::{clipboard::Clipboard, server::Server, storage::Storage, tray::Tray};
use anyhow::{Result, anyhow};
use eframe::{
    NativeOptions, Renderer, UserEvent, create_native,
    egui::{
        CentralPanel, FontData, FontDefinitions, FontFamily, Ui, ViewportBuilder, ViewportCommand,
    },
};
use std::{
    error::Error,
    fs::read,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};
use system_fonts::{FontStyle, FoundFontSource, find_for_system_locale};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        renderer: Renderer::Glow,
        ..Default::default()
    };

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let mut app = create_native(
        "Clipclip",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();

            if let Ok(font_definitions) = gen_font_definitions() {
                ctx.set_fonts(font_definitions);
            }

            let (save_clip_tx, save_clip_rx) = channel::<String>();
            let (set_clip_tx, set_clip_rx) = channel::<String>();
            let (get_clip_tx, get_clip_rx) = channel::<Sender<String>>();
            let (exited_tx, exited_rx) = channel::<()>();
            let (copied_tx, copied_rx) = channel::<String>();

            let mut storage = Storage::new();
            storage.start_listening_save_clip(save_clip_rx);

            let mut clipboard = Clipboard::new();
            clipboard.start_listening_clip_change(
                ctx.clone(),
                save_clip_tx.clone(),
                copied_tx.clone(),
            );
            clipboard.start_listening_set_clip(set_clip_rx);
            clipboard.start_listening_get_clip(get_clip_rx);

            let mut server = Server::new();
            server.start_listening(set_clip_tx.clone(), get_clip_tx.clone());

            let tray = Tray::new();
            tray.start_listening_events(ctx, exited_tx);

            let mut clipclip =
                Clipclip::new(exited_rx, copied_rx, tray, storage, clipboard, server);
            clipclip.status = "Service running".to_string();

            Ok(Box::new(clipclip))
        }),
        &event_loop,
    );

    event_loop.run_app(&mut app)?;

    Ok(())
}

fn gen_font_definitions() -> Result<FontDefinitions> {
    let (_, _, found_fonts) = find_for_system_locale(FontStyle::Sans);
    let default_font = found_fonts.first().ok_or(anyhow!("Not found font"))?;
    let font_path = match &default_font.source {
        FoundFontSource::Path(it) => it,
        _ => return Err(anyhow!("Not found font")),
    };
    let font_bytes = read(font_path)?;
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "default".to_owned(),
        Arc::new(FontData::from_owned(font_bytes)),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "default".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("default".to_owned());

    Ok(fonts)
}

struct Clipclip {
    status: String,
    exited_rx: Receiver<()>,
    copied_rx: Receiver<String>,
    _tray: Tray,
    _storage: Storage,
    _clipboard: Clipboard,
    server: Server,
}

impl Clipclip {
    fn new(
        exited_rx: Receiver<()>,
        copied_rx: Receiver<String>,
        tray: Tray,
        storage: Storage,
        clipboard: Clipboard,
        server: Server,
    ) -> Self {
        Self {
            status: "".to_string(),
            exited_rx,
            copied_rx,
            _tray: tray,
            _storage: storage,
            _clipboard: clipboard,
            server: server,
        }
    }
}

impl eframe::App for Clipclip {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.viewport().close_requested()) && self.exited_rx.try_recv().is_err() {
            ui.send_viewport_cmd(ViewportCommand::CancelClose);
            ui.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        if let Ok(clip) = self.copied_rx.try_recv() {
            self.status = clip;
        }

        CentralPanel::default().show_inside(ui, |ui| {
            ui.label(format!("Addr: {}", &self.server.addr));
            ui.label(format!("Status: {}", &self.status));
            if ui.button("Load all clips").clicked() {}
        });
    }
}
