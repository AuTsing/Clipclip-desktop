use rusqlite::{Connection, params};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY,
            content TEXT,
            time INTEGER
        )",
            [],
        )
        .unwrap();
        Self { conn }
    }

    pub fn insert(&self, text: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute(
                "INSERT INTO clipboard (content, time) VALUES (?1, strftime('%s','now'))",
                params![text],
            )
            .unwrap();
        Ok(())
    }
}
