use crate::aggregator::WpmSample;
use crate::error::Error;
use crate::keycode::KeyCode;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SCHEMA_VERSION: i32 = 3;

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

        let mut storage = Self { conn };
        storage.migrate()?;

        Ok(storage)
    }

    fn migrate(&mut self) -> Result<(), Error> {
        let version: i32 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap_or(0);

        if version < 1 {
            self.conn.execute(
                "CREATE TABLE IF NOT EXISTS sessions (
                    id INTEGER PRIMARY KEY,
                    start_time TEXT NOT NULL,
                    end_time TEXT,
                    keystroke_count INTEGER NOT NULL DEFAULT 0
                )",
                [],
            )?;

            self.conn.execute(
                "CREATE TABLE IF NOT EXISTS shortcut_counts (
                    combo TEXT NOT NULL,
                    date TEXT NOT NULL,
                    count INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (combo, date)
                )",
                [],
            )?;
        }

        if version < 2 {
            self.migrate_to_v2()?;
        }

        if version < 3 {
            self.migrate_to_v3()?;
        }

        self.conn
            .pragma_update(None, "user_version", SCHEMA_VERSION)?;

        Ok(())
    }

    fn migrate_to_v2(&mut self) -> Result<(), Error> {
        let has_old_table: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='key_counts'",
            [],
            |row| row.get::<_, i32>(0),
        )? > 0;

        if has_old_table {
            let is_integer_format: bool = self
                .conn
                .query_row(
                    "SELECT type FROM pragma_table_info('key_counts') WHERE name='key_code'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .map(|t| t.to_uppercase() == "INTEGER")
                .unwrap_or(false);

            if is_integer_format {
                eprintln!("migrating key_counts from integer to string format...");

                self.conn.execute(
                    "CREATE TABLE key_counts_new (
                        key_code TEXT NOT NULL,
                        date TEXT NOT NULL,
                        count INTEGER NOT NULL DEFAULT 0,
                        PRIMARY KEY (key_code, date)
                    )",
                    [],
                )?;

                let tx = self.conn.transaction()?;
                {
                    let mut read_stmt =
                        tx.prepare("SELECT key_code, date, count FROM key_counts")?;
                    let mut write_stmt = tx.prepare(
                        "INSERT INTO key_counts_new (key_code, date, count) VALUES (?1, ?2, ?3)",
                    )?;

                    let rows = read_stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, i32>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, i64>(2)?,
                        ))
                    })?;

                    for row in rows {
                        let (code, date, count) = row?;
                        let key_str = evdev_code_to_string(code as u32);
                        write_stmt.execute(params![key_str, date, count])?;
                    }
                }
                tx.commit()?;

                self.conn.execute("DROP TABLE key_counts", [])?;
                self.conn
                    .execute("ALTER TABLE key_counts_new RENAME TO key_counts", [])?;

                eprintln!("migration complete");
            }
        } else {
            self.conn.execute(
                "CREATE TABLE key_counts (
                    key_code TEXT NOT NULL,
                    date TEXT NOT NULL,
                    count INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (key_code, date)
                )",
                [],
            )?;
        }

        Ok(())
    }

    fn migrate_to_v3(&mut self) -> Result<(), Error> {
        eprintln!("migrating to v3: adding WPM tracking...");

        // add WPM columns to sessions
        let has_avg_wpm: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name='avg_wpm'",
            [],
            |row| row.get::<_, i32>(0),
        )? > 0;

        if !has_avg_wpm {
            self.conn
                .execute("ALTER TABLE sessions ADD COLUMN avg_wpm REAL", [])?;
            self.conn
                .execute("ALTER TABLE sessions ADD COLUMN peak_wpm REAL", [])?;
        }

        // create wpm_samples table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS wpm_samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                wpm REAL NOT NULL,
                keystrokes_in_window INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wpm_samples_session ON wpm_samples(session_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wpm_samples_timestamp ON wpm_samples(timestamp)",
            [],
        )?;

        eprintln!("v3 migration complete");
        Ok(())
    }

    fn db_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyheat")
            .join("keyheat.db")
    }

    pub fn flush_counts(
        &mut self,
        counts: &HashMap<KeyCode, u64>,
        date: &str,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO key_counts (key_code, date, count)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (key_code, date)
                 DO UPDATE SET count = count + excluded.count",
            )?;

            for (key_code, &count) in counts {
                stmt.execute(params![key_code.to_string(), date, count])?;
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
        avg_wpm: Option<f64>,
        peak_wpm: Option<f64>,
    ) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE sessions SET end_time = ?1, keystroke_count = ?2, avg_wpm = ?3, peak_wpm = ?4 WHERE id = ?5",
            params![end_time.to_rfc3339(), keystroke_count, avg_wpm, peak_wpm, session_id],
        )?;
        Ok(())
    }

    pub fn flush_shortcuts(
        &mut self,
        shortcuts: &HashMap<String, u64>,
        date: &str,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO shortcut_counts (combo, date, count)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (combo, date)
                 DO UPDATE SET count = count + excluded.count",
            )?;

            for (combo, &count) in shortcuts {
                stmt.execute(params![combo, date, count])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn flush_wpm_samples(&mut self, samples: &[WpmSample]) -> Result<(), Error> {
        if samples.is_empty() {
            return Ok(());
        }

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO wpm_samples (session_id, timestamp, wpm, keystrokes_in_window)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for sample in samples {
                stmt.execute(params![
                    sample.session_id,
                    sample.timestamp.to_rfc3339(),
                    sample.wpm,
                    sample.keystrokes_in_window,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

fn evdev_code_to_string(code: u32) -> String {
    match code {
        30 => "a",
        48 => "b",
        46 => "c",
        32 => "d",
        18 => "e",
        33 => "f",
        34 => "g",
        35 => "h",
        23 => "i",
        36 => "j",
        37 => "k",
        38 => "l",
        50 => "m",
        49 => "n",
        24 => "o",
        25 => "p",
        16 => "q",
        19 => "r",
        31 => "s",
        20 => "t",
        22 => "u",
        47 => "v",
        17 => "w",
        45 => "x",
        21 => "y",
        44 => "z",

        11 => "0",
        2 => "1",
        3 => "2",
        4 => "3",
        5 => "4",
        6 => "5",
        7 => "6",
        8 => "7",
        9 => "8",
        10 => "9",

        59 => "f1",
        60 => "f2",
        61 => "f3",
        62 => "f4",
        63 => "f5",
        64 => "f6",
        65 => "f7",
        66 => "f8",
        67 => "f9",
        68 => "f10",
        87 => "f11",
        88 => "f12",

        42 => "lshift",
        54 => "rshift",
        29 => "lctrl",
        97 => "rctrl",
        56 => "lalt",
        100 => "ralt",
        125 => "lsuper",
        126 => "rsuper",

        57 => "space",
        15 => "tab",
        28 => "enter",
        1 => "escape",
        14 => "backspace",
        111 => "delete",
        58 => "capslock",
        69 => "numlock",
        70 => "scrolllock",
        110 => "insert",
        99 => "printscreen",
        119 => "pause",

        103 => "up",
        108 => "down",
        105 => "left",
        106 => "right",
        102 => "home",
        107 => "end",
        104 => "pageup",
        109 => "pagedown",

        12 => "minus",
        13 => "equal",
        26 => "leftbracket",
        27 => "rightbracket",
        39 => "semicolon",
        40 => "apostrophe",
        41 => "grave",
        43 => "backslash",
        51 => "comma",
        52 => "period",
        53 => "slash",

        82 => "numpad0",
        79 => "numpad1",
        80 => "numpad2",
        81 => "numpad3",
        75 => "numpad4",
        76 => "numpad5",
        77 => "numpad6",
        71 => "numpad7",
        72 => "numpad8",
        73 => "numpad9",
        78 => "numpadadd",
        74 => "numpadsubtract",
        55 => "numpadmultiply",
        98 => "numpaddivide",
        96 => "numpadenter",
        83 => "numpaddecimal",

        127 => "menu",

        _ => return format!("unknown_{code}"),
    }
    .to_string()
}
