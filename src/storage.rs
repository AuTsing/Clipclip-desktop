use anyhow::{Result, anyhow};
use rusqlite::Connection;
use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

pub struct Sqlc {
    conn: Connection,
}

impl Sqlc {
    pub fn new() -> Self {
        let conn = Connection::open("clips.db").unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clips (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
            [],
        )
        .unwrap();

        Self { conn }
    }

    pub fn insert(&self, text: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO clips (content, created_at) VALUES (?1, strftime('%s','now'))",
            [text],
        )?;
        self.conn.execute(
            "DELETE FROM clips WHERE id NOT IN (SELECT id FROM clips ORDER BY id DESC LIMIT 100)",
            [],
        )?;

        Ok(())
    }

    pub fn query(&self, limit: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT content FROM clips ORDER BY id DESC LIMIT ?1")?;
        let rows_iter = stmt.query_map([limit], |row| Ok(row.get(0)?))?;
        let clips = rows_iter.collect::<Result<Vec<String>, rusqlite::Error>>()?;

        Ok(clips)
    }
}

pub struct Storage {
    listening_save_clip_handle: Option<JoinHandle<()>>,
    last_clip: Arc<Mutex<String>>,
    sqlc: Arc<Mutex<Sqlc>>,
}

impl Storage {
    pub fn new() -> Self {
        let last_clip = Arc::new(Mutex::new("".to_string()));
        let sqlc = Arc::new(Mutex::new(Sqlc::new()));

        Self {
            listening_save_clip_handle: None,
            last_clip,
            sqlc,
        }
    }

    pub fn start_listening_save_clip(&mut self, save_clip_rx: Receiver<String>) {
        let last_clip = self.last_clip.clone();
        let sqlc = self.sqlc.clone();
        self.listening_save_clip_handle = Some(thread::spawn(move || {
            loop {
                let clip = match save_clip_rx.recv() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };
                let mut last_chip_guard = match last_clip.lock() {
                    Ok(it) => it,
                    Err(e) => e.into_inner(),
                };
                if clip == *last_chip_guard {
                    continue;
                }
                let sqlc_guard = match sqlc.lock() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };
                if let Err(_) = sqlc_guard.insert(&clip) {
                    // TODO(Log err)
                    continue;
                }
                *last_chip_guard = clip;
            }
        }));
    }

    pub fn get_all_clips(&self) -> Result<Vec<String>> {
        let sqlc_guard = self.sqlc.lock().map_err(|_| anyhow!("Got poison sqlc"))?;
        let clips = sqlc_guard.query(100)?;

        Ok(clips)
    }
}
