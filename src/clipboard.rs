use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use eframe::egui::Context;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

pub struct Clipboard {
    listening_clip_change_handle: Option<JoinHandle<()>>,
    listening_set_clip_handle: Option<JoinHandle<()>>,
    listening_get_clip_handle: Option<JoinHandle<()>>,
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> Self {
        let arclipboard = Arc::new(Mutex::new(arboard::Clipboard::new().unwrap()));

        Self {
            listening_clip_change_handle: None,
            listening_set_clip_handle: None,
            listening_get_clip_handle: None,
            arclipboard,
        }
    }

    pub fn start_listening_clip_change(
        &mut self,
        ctx: Context,
        save_clip_tx: Sender<String>,
        copied_tx: Sender<String>,
    ) {
        let arclipboard = self.arclipboard.clone();
        self.listening_clip_change_handle = Some(thread::spawn(move || {
            let listener = Listener::new(ctx, arclipboard, save_clip_tx, copied_tx);

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

    pub fn start_listening_set_clip(&mut self, set_clip_rx: Receiver<String>) {
        let arclipboard = self.arclipboard.clone();
        self.listening_set_clip_handle = Some(thread::spawn(move || {
            loop {
                let clip = match set_clip_rx.recv() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                let mut arclipboard_guard = match arclipboard.lock() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                if let Err(_) = arclipboard_guard.set_text(clip) {
                    // TODO(Log err)
                    continue;
                }
            }
        }));
    }

    pub fn start_listening_get_clip(&mut self, get_clip_rx: Receiver<Sender<String>>) {
        let arclipboard = self.arclipboard.clone();
        self.listening_get_clip_handle = Some(thread::spawn(move || {
            loop {
                let tx = match get_clip_rx.recv() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                let mut arclipboard_guard = match arclipboard.lock() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                let clip = match arclipboard_guard.get_text() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                if let Err(_) = tx.send(clip) {
                    // TODO(Log err)
                    continue;
                }
            }
        }));
    }
}

struct Listener {
    ctx: Context,
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
    save_clip_tx: Sender<String>,
    copied_tx: Sender<String>,
}

impl Listener {
    fn new(
        ctx: Context,
        arclipboard: Arc<Mutex<arboard::Clipboard>>,
        save_clip_tx: Sender<String>,
        copied_tx: Sender<String>,
    ) -> Self {
        Self {
            ctx,
            arclipboard,
            save_clip_tx,
            copied_tx,
        }
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

        if let Err(_) = self.save_clip_tx.send(clip.clone()) {
            // TODO(Log err)
        }

        if let Err(_) = self.copied_tx.send(clip) {
            // TODO(Log err)
        }

        self.ctx.request_repaint();

        CallbackResult::Next
    }
}
