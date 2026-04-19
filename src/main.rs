mod clipboard;
mod server;
mod storage;
mod tray;

use crate::{clipboard::Clipboard, server::Server, storage::Storage, tray::Tray};
use anyhow::Result;
use std::sync::mpsc::Sender;
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

fn main() -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let proxy = event_loop.create_proxy();
    let mut clipclip = Clipclip::new(proxy);
    event_loop.run_app(&mut clipclip)?;

    Ok(())
}

struct Clipclip {
    proxy: EventLoopProxy<UserEvent>,
    tray: Option<Tray>,
    clipboard: Option<Clipboard>,
    storage: Option<Storage>,
    server: Option<Server>,
}

enum UserEvent {
    Exit,
    SaveClip(String),
    RecvClip(String),
    SendClip(Sender<Result<String>>),
    UpdateAddr(String),
}

impl Clipclip {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self {
            proxy,
            tray: None,
            clipboard: None,
            storage: None,
            server: None,
        }
    }

    fn handle_exit(event_loop: &ActiveEventLoop) {
        event_loop.exit();
    }

    fn handle_save_clip(&mut self, clip: String) {
        let storage = self
            .storage
            .as_mut()
            .expect("Storage has not been initialized");
        if let Err(_) = storage.save_clip(clip) {
            // TODO(Log Err)
        }
    }

    fn handle_recv_clip(&self, clip: String) {
        let clipboard = self
            .clipboard
            .as_ref()
            .expect("Clipboard has not been initialized");
        if let Err(_) = clipboard.set_clip(clip) {
            // TODO(Log Err)
        }
    }

    fn handle_send_clip(&self, sender: Sender<Result<String>>) {
        let clipboard = self
            .clipboard
            .as_ref()
            .expect("Clipboard has not been initialized");
        if let Err(_) = sender.send(clipboard.get_clip()) {
            // TODO(Log Err)
        }
    }

    fn handle_update_addr(&self, addr: String) {
        println!("handle_update_addr: {}", addr);
    }
}

impl ApplicationHandler<UserEvent> for Clipclip {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            let tray = Tray::new();
            tray.start_listening_events(self.proxy.clone());
            self.tray = Some(tray);

            let mut clipboard = Clipboard::new();
            clipboard.start_listening_clip_change(self.proxy.clone());
            self.clipboard = Some(clipboard);

            let storage = Storage::new();
            self.storage = Some(storage);

            let mut server = Server::new();
            server.start_listening_request(self.proxy.clone());
            self.server = Some(server);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Exit => Clipclip::handle_exit(event_loop),
            UserEvent::SaveClip(clip) => self.handle_save_clip(clip),
            UserEvent::RecvClip(clip) => self.handle_recv_clip(clip),
            UserEvent::SendClip(sender) => self.handle_send_clip(sender),
            UserEvent::UpdateAddr(addr) => self.handle_update_addr(addr),
        };
    }
}
