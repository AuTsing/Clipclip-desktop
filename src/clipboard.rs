use clipboard_master::{CallbackResult, ClipboardHandler, Master};

use crate::storage::Storage;
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Clipboard {
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
    storage: Arc<Mutex<Storage>>,
    last_clip: Arc<Mutex<String>>,
    _listener: JoinHandle<()>,
}

pub struct Clip {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
}

impl Clipboard {
    pub fn new() -> Self {
        let arclipboard = Arc::new(Mutex::new(arboard::Clipboard::new().unwrap()));
        let storage = Arc::new(Mutex::new(Storage::new()));
        let last_clip = Arc::new(Mutex::new("".to_string()));

        let listener = Listener::new(arclipboard.clone(), storage.clone(), last_clip.clone());
        let _listener = thread::spawn(move || {
            let mut master = Master::new(listener).unwrap();
            master.run().unwrap();
        });

        Self {
            arclipboard,
            storage,
            last_clip,
            _listener,
        }
    }

    pub fn load_all(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let texts = self
            .storage
            .lock()
            .unwrap()
            .query(100)?
            .into_iter()
            .map(|it| it.content)
            .collect::<Vec<String>>();

        Ok(texts)
    }
}

struct Listener {
    arclipboard: Arc<Mutex<arboard::Clipboard>>,
    storage: Arc<Mutex<Storage>>,
    last_clip: Arc<Mutex<String>>,
}

impl Listener {
    fn new(
        arclipboard: Arc<Mutex<arboard::Clipboard>>,
        storage: Arc<Mutex<Storage>>,
        last_clip: Arc<Mutex<String>>,
    ) -> Self {
        Self {
            arclipboard,
            storage,
            last_clip,
        }
    }

    fn save_latest(&mut self) -> Result<(), Box<dyn Error>> {
        let text = self.arclipboard.lock().unwrap().get_text()?;
        self.storage.lock().unwrap().insert(&text)?;
        *self.last_clip.lock().unwrap() = text;

        Ok(())
    }
}

impl ClipboardHandler for Listener {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        println!("Clipboard change happened!");
        self.save_latest().unwrap();
        CallbackResult::Next
    }
}
