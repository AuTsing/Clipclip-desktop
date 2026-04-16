use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

pub struct Clipboard {
    save_latest_running_handle: Option<JoinHandle<()>>,
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> Self {
        let arclipboard = Arc::new(Mutex::new(arboard::Clipboard::new().unwrap()));

        Self {
            save_latest_running_handle: None,
            arclipboard,
        }
    }

    pub fn start_listening_clip_change(&mut self, save_clip_tx: Sender<String>) {
        let arclipboard = self.arclipboard.clone();
        self.save_latest_running_handle = Some(thread::spawn(move || {
            let listener = Listener::new(save_clip_tx, arclipboard);
            let mut master = Master::new(listener).unwrap();
            master.run().unwrap();
        }));
    }

    pub fn start_listening_get_clip(&mut self, get_clip_rx: Receiver<Sender<String>>) {
        let arclipboard = self.arclipboard.clone();
        self.save_latest_running_handle = Some(thread::spawn(move || {
            loop {
                let tx = get_clip_rx.recv().unwrap();
                let clip = arclipboard.lock().unwrap().get_text().unwrap();
                tx.send(clip).unwrap();
            }
        }));
    }
}

struct Listener {
    save_latest_tx: Sender<String>,
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
}

impl Listener {
    fn new(save_latest_tx: Sender<String>, arclipboard: Arc<Mutex<arboard::Clipboard>>) -> Self {
        Self {
            save_latest_tx,
            arclipboard,
        }
    }
}

impl ClipboardHandler for Listener {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let clip = self.arclipboard.lock().unwrap().get_text().unwrap();
        self.save_latest_tx.send(clip).unwrap();
        CallbackResult::Next
    }
}
