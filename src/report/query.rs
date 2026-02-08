use crate::error::Error;
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RawSession {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub keystroke_count: u64,
    pub avg_wpm: Option<f64>,
    pub peak_wpm: Option<f64>,
}

#[derive(Debug)]
#[allow(dead_code)] // timestamp used by HTML renderer for time-series
pub struct RawWpmSample {
    pub timestamp: DateTime<Utc>,
    pub wpm: f64,
}

pub fn key_counts_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<HashMap<String, u64>, Error> {
    let mut stmt = conn.prepare(
        "SELECT key_code, SUM(count) as total
         FROM key_counts
         WHERE date >= ?1 AND date <= ?2
         GROUP BY key_code
         ORDER BY total DESC",
    )?;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let rows = stmt.query_map(params![start_str, end_str], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    })?;

    let mut counts = HashMap::new();
    for row in rows {
        let (key, count) = row?;
        counts.insert(key, count);
    }

    Ok(counts)
}

pub fn total_keystrokes_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<u64, Error> {
    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let total: u64 = conn
        .query_row(
            "SELECT COALESCE(SUM(count), 0) FROM key_counts WHERE date >= ?1 AND date <= ?2",
            params![start_str, end_str],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(total)
}

pub fn all_time_keystrokes(conn: &Connection) -> Result<u64, Error> {
    let total: u64 = conn
        .query_row(
            "SELECT COALESCE(SUM(count), 0) FROM key_counts",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(total)
}

pub fn sessions_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<RawSession>, Error> {
    let mut stmt = conn.prepare(
        "SELECT id, start_time, end_time, keystroke_count, avg_wpm, peak_wpm
         FROM sessions
         WHERE date(start_time) >= ?1 AND date(start_time) <= ?2
         ORDER BY start_time",
    )?;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let rows = stmt.query_map(params![start_str, end_str], |row| {
        let start_str: String = row.get(1)?;
        let end_str: Option<String> = row.get(2)?;

        let start_time = DateTime::parse_from_rfc3339(&start_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let end_time = end_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok(RawSession {
            id: row.get(0)?,
            start_time,
            end_time,
            keystroke_count: row.get::<_, i64>(3)? as u64,
            avg_wpm: row.get(4)?,
            peak_wpm: row.get(5)?,
        })
    })?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }

    Ok(sessions)
}

pub fn wpm_samples_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<RawWpmSample>, Error> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, wpm
         FROM wpm_samples
         WHERE date(timestamp) >= ?1 AND date(timestamp) <= ?2
         ORDER BY timestamp",
    )?;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let rows = stmt.query_map(params![start_str, end_str], |row| {
        let ts_str: String = row.get(0)?;
        let timestamp = DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(RawWpmSample {
            timestamp,
            wpm: row.get(1)?,
        })
    })?;

    let mut samples = Vec::new();
    for row in rows {
        samples.push(row?);
    }

    Ok(samples)
}

pub fn shortcuts_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<(String, u64)>, Error> {
    let mut stmt = conn.prepare(
        "SELECT combo, SUM(count) as total
         FROM shortcut_counts
         WHERE date >= ?1 AND date <= ?2
         GROUP BY combo
         ORDER BY total DESC",
    )?;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let rows = stmt.query_map(params![start_str, end_str], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    })?;

    let mut shortcuts = Vec::new();
    for row in rows {
        shortcuts.push(row?);
    }

    Ok(shortcuts)
}

pub fn peak_wpm_session_for_range(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Option<(f64, DateTime<Utc>)>, Error> {
    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let result = conn.query_row(
        "SELECT peak_wpm, start_time
         FROM sessions
         WHERE date(start_time) >= ?1 AND date(start_time) <= ?2
           AND peak_wpm IS NOT NULL
         ORDER BY peak_wpm DESC
         LIMIT 1",
        params![start_str, end_str],
        |row| {
            let wpm: f64 = row.get(0)?;
            let ts_str: String = row.get(1)?;
            Ok((wpm, ts_str))
        },
    );

    match result {
        Ok((wpm, ts_str)) => {
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(Some((wpm, timestamp)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
