pub struct Clipboard {
    clip: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            clip: arboard::Clipboard::new().unwrap(),
        }
    }

    pub fn read_latest(&mut self) -> Result<String, arboard::Error> {
        let text = self.clip.get_text()?;
        Ok(text)
    }

    pub fn write_latest(&mut self) -> Result<(), arboard::Error> {
        self.clip.set_text("")?;
        Ok(())
    }
}
