use crate::error::Error;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open() -> Result<Self, Error> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS key_counts (
                key_code INTEGER NOT NULL,
                date TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (key_code, date)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT,
                keystroke_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    fn db_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyheat")
            .join("keyheat.db")
    }

    pub fn flush_counts(&mut self, counts: &HashMap<u16, u64>, date: &str) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO key_counts (key_code, date, count)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (key_code, date)
                 DO UPDATE SET count = count + excluded.count",
            )?;

            for (&key_code, &count) in counts {
                stmt.execute(params![key_code, date, count])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn start_session(&mut self, start_time: DateTime<Utc>) -> Result<i64, Error> {
        self.conn.execute(
            "INSERT INTO sessions (start_time, keystroke_count) VALUES (?1, 0)",
            params![start_time.to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_session_keystrokes(&mut self, session_id: i64, count: u64) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE sessions SET keystroke_count = ?1 WHERE id = ?2",
            params![count, session_id],
        )?;
        Ok(())
    }

    pub fn end_session(
        &mut self,
        session_id: i64,
        end_time: DateTime<Utc>,
        keystroke_count: u64,
    ) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE sessions SET end_time = ?1, keystroke_count = ?2 WHERE id = ?3",
            params![end_time.to_rfc3339(), keystroke_count, session_id],
        )?;
        Ok(())
    }
}
