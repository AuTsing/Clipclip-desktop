use rusqlite::Connection;
use std::{
    error::Error,
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

    pub fn insert(&self, text: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute(
                "INSERT INTO clips (content, created_at) VALUES (?1, strftime('%s','now'))",
                [text],
            )
            .unwrap();

        Ok(())
    }

    pub fn query(&self, limit: i64) -> Result<Vec<Clip>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, created_at FROM clips ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows_iter = stmt.query_map([limit], |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        let clips = rows_iter.collect::<Result<Vec<Clip>, rusqlite::Error>>()?;

        Ok(clips)
    }
}

pub struct Clip {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
}

pub struct Storage {
    saving_clip_handle: Option<JoinHandle<()>>,
    last_clip: Arc<Mutex<String>>,
    sqlc: Arc<Mutex<Sqlc>>,
}

impl Storage {
    pub fn new() -> Self {
        let last_clip = Arc::new(Mutex::new("".to_string()));
        let sqlc = Arc::new(Mutex::new(Sqlc::new()));

        Self {
            saving_clip_handle: None,
            last_clip,
            sqlc,
        }
    }

    pub fn start_saving_clip(&mut self, save_clip_rx: Receiver<String>) {
        let last_clip = self.last_clip.clone();
        let sqlc = self.sqlc.clone();
        self.saving_clip_handle = Some(thread::spawn(move || {
            loop {
                let clip = save_clip_rx.recv().unwrap();
                if clip == *last_clip.lock().unwrap() {
                    continue;
                }
                sqlc.lock().unwrap().insert(&clip).unwrap();
                *last_clip.lock().unwrap() = clip;
            }
        }));
    }

    pub fn get_all_clips(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let clips = self
            .sqlc
            .lock()
            .unwrap()
            .query(100)?
            .into_iter()
            .map(|it| it.content)
            .collect::<Vec<String>>();

        Ok(clips)
    }
}
