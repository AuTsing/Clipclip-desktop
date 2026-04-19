use crate::UserEvent;
use anyhow::{Result, anyhow};
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
use winit::event_loop::EventLoopProxy;

pub struct Clipboard {
    listening_clip_change_handle: Option<JoinHandle<()>>,
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> Self {
        let arclipboard = Arc::new(Mutex::new(arboard::Clipboard::new().unwrap()));

        Self {
            listening_clip_change_handle: None,
            arclipboard,
        }
    }

    pub fn start_listening_clip_change(&mut self, proxy: EventLoopProxy<UserEvent>) {
        let arclipboard = self.arclipboard.clone();
        self.listening_clip_change_handle = Some(thread::spawn(move || {
            let listener = Listener::new(arclipboard, proxy);

            let mut master = match Master::new(listener) {
                Ok(it) => it,
                Err(_) => {
                    // TODO(Log err)
                    return;
                }
            };

            if let Err(_) = master.run() {
                // TODO(Log err)
                return;
            }
        }));
    }

    pub fn get_clip(&self) -> Result<String> {
        let mut arclipboard_guard = self
            .arclipboard
            .lock()
            .map_err(|_| anyhow!("Poison error"))?;
        let clip = arclipboard_guard.get_text()?;

        Ok(clip)
    }

    pub fn set_clip(&self, clip: String) -> Result<()> {
        let mut arclipboard_guard = self
            .arclipboard
            .lock()
            .map_err(|_| anyhow!("Poison error"))?;
        arclipboard_guard.set_text(clip)?;

        Ok(())
    }
}

struct Listener {
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
    proxy: EventLoopProxy<UserEvent>,
}

impl Listener {
    fn new(arclipboard: Arc<Mutex<arboard::Clipboard>>, proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { arclipboard, proxy }
    }
}

impl ClipboardHandler for Listener {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let mut arclipboard_guard = match self.arclipboard.lock() {
            Ok(it) => it,
            Err(_) => {
                // TODO(Log err)
                return CallbackResult::Next;
            }
        };

        let clip = match arclipboard_guard.get_text() {
            Ok(it) => it,
            Err(_) => {
                // TODO(Log err)
                return CallbackResult::Next;
            }
        };

        let _ = self.proxy.send_event(UserEvent::SaveClip(clip));

        CallbackResult::Next
    }
}
