use crate::storage::Storage;
use std::error::Error;

pub struct Clipboard {
    arclipboard: arboard::Clipboard,
    storage: Storage,
}

pub struct Clip {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            arclipboard: arboard::Clipboard::new().unwrap(),
            storage: Storage::new(),
        }
    }

    pub fn load_latest(&self) -> Result<String, Box<dyn Error>> {
        let clips = self.storage.query(1)?;
        let clip = clips.into_iter().next().ok_or("No clip")?;
        let text = clip.content;

        Ok(text)
    }

    pub fn load_all(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let texts = self
            .storage
            .query(100)?
            .into_iter()
            .map(|it| it.content)
            .collect::<Vec<String>>();

        Ok(texts)
    }

    pub fn save_latest(&mut self) -> Result<(), Box<dyn Error>> {
        let text = self.arclipboard.get_text()?;
        self.storage.insert(&text)?;

        Ok(())
    }
}
