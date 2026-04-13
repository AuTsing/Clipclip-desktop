use crate::clipboard::Clip;
use rusqlite::Connection;

pub struct Storage {
    conn: Connection,
}

impl Storage {
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
