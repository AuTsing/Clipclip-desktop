use anyhow::Result;
use rusqlite::Connection;

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
    sqlc: Sqlc,
    last_clip: String,
}

impl Storage {
    pub fn new() -> Self {
        let sqlc = Sqlc::new();
        let last_clip = "".to_string();

        Self { sqlc, last_clip }
    }

    pub fn save_clip(&mut self, clip: String) -> Result<()> {
        if clip == self.last_clip {
            return Ok(());
        }

        self.sqlc.insert(&clip)?;
        self.last_clip = clip;

        Ok(())
    }

    pub fn get_all_clips(&self) -> Result<Vec<String>> {
        let clips = self.sqlc.query(100)?;

        Ok(clips)
    }
}
