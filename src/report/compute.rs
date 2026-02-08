use super::query::{self, RawSession};
use super::{ReportData, SessionSummary, ShortcutInsight, WeekRange};
use crate::error::Error;
use chrono::{Datelike, Duration, NaiveDate, Timelike, Utc, Weekday};
use rusqlite::Connection;
use std::collections::HashMap;

const KEY_PITCH_MM: f64 = 19.05;

pub fn build_report(conn: &Connection, week: Option<NaiveDate>) -> Result<ReportData, Error> {
    let week_range = compute_week_range(week);
    let prev_week_range = compute_prev_week_range(&week_range);

    let key_counts = query::key_counts_for_range(conn, week_range.start, week_range.end)?;
    let total_keystrokes =
        query::total_keystrokes_for_range(conn, week_range.start, week_range.end)?;
    let prev_week_keystrokes =
        query::total_keystrokes_for_range(conn, prev_week_range.start, prev_week_range.end).ok();
    let all_time_keystrokes = query::all_time_keystrokes(conn)?;

    let sessions = query::sessions_for_range(conn, week_range.start, week_range.end)?;
    let wpm_samples = query::wpm_samples_for_range(conn, week_range.start, week_range.end)?;
    let shortcuts = query::shortcuts_for_range(conn, week_range.start, week_range.end)?;
    let prev_week_shortcuts =
        query::shortcuts_for_range(conn, prev_week_range.start, prev_week_range.end)
            .unwrap_or_default();

    let peak_wpm_info = query::peak_wpm_session_for_range(conn, week_range.start, week_range.end)?;

    let prev_sessions = query::sessions_for_range(conn, prev_week_range.start, prev_week_range.end)
        .unwrap_or_default();
    let prev_week_avg_wpm = compute_avg_wpm(&prev_sessions);

    let key_frequencies = compute_key_frequencies(&key_counts);
    let session_summaries = compute_session_summaries(&sessions);
    let longest_session = find_longest_session(&session_summaries);
    let total_typing_minutes = compute_total_typing_minutes(&sessions);
    let avg_wpm = compute_avg_wpm(&sessions).unwrap_or(0.0);
    let (peak_wpm, peak_wpm_time) = peak_wpm_info.unzip();
    let daily_wpm = compute_daily_wpm(&sessions, &week_range);
    let wpm_distribution = compute_wpm_distribution(&wpm_samples);
    let hourly_activity = compute_hourly_activity(&sessions, &week_range);
    let finger_travel_mm = compute_finger_travel(&key_counts);
    let backspace_ratio = compute_backspace_ratio(&key_counts, total_keystrokes);
    let fastest_day = compute_fastest_day(&sessions);
    let peak_hour = compute_peak_hour(&hourly_activity);
    let night_owl_pct = compute_night_owl_pct(&sessions);
    let shortcut_insight = generate_shortcut_insight(&shortcuts);

    Ok(ReportData {
        week: week_range,
        total_keystrokes,
        prev_week_keystrokes,
        key_frequencies,
        avg_wpm,
        peak_wpm: peak_wpm.unwrap_or(0.0),
        peak_wpm_time,
        prev_week_avg_wpm,
        daily_wpm,
        wpm_distribution,
        sessions: session_summaries,
        total_typing_minutes,
        longest_session,
        hourly_activity,
        shortcuts,
        prev_week_shortcuts,
        all_time_keystrokes,
        finger_travel_mm,
        backspace_ratio,
        fastest_day,
        peak_hour,
        night_owl_pct,
        shortcut_insight,
    })
}

fn compute_week_range(date: Option<NaiveDate>) -> WeekRange {
    let target = date.unwrap_or_else(|| Utc::now().date_naive());

    let days_since_monday = target.weekday().num_days_from_monday();
    let start = target - Duration::days(days_since_monday as i64);
    let end = start + Duration::days(6);

    let label = format!(
        "Week of {} - {}",
        start.format("%b %d"),
        end.format("%b %d, %Y")
    );

    WeekRange { start, end, label }
}

fn compute_prev_week_range(current: &WeekRange) -> WeekRange {
    let start = current.start - Duration::days(7);
    let end = current.end - Duration::days(7);

    let label = format!(
        "Week of {} - {}",
        start.format("%b %d"),
        end.format("%b %d, %Y")
    );

    WeekRange { start, end, label }
}

fn compute_key_frequencies(counts: &HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut freqs: Vec<_> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    freqs.sort_by(|a, b| b.1.cmp(&a.1));
    freqs
}

fn compute_session_summaries(sessions: &[RawSession]) -> Vec<SessionSummary> {
    sessions
        .iter()
        .map(|s| {
            let duration_minutes = s
                .end_time
                .map(|end| (end - s.start_time).num_seconds() as f64 / 60.0)
                .unwrap_or(0.0);

            SessionSummary {
                id: s.id,
                start_time: s.start_time,
                end_time: s.end_time,
                duration_minutes,
                keystroke_count: s.keystroke_count,
                avg_wpm: s.avg_wpm,
                peak_wpm: s.peak_wpm,
            }
        })
        .collect()
}

fn find_longest_session(sessions: &[SessionSummary]) -> Option<SessionSummary> {
    sessions
        .iter()
        .max_by(|a, b| {
            a.duration_minutes
                .partial_cmp(&b.duration_minutes)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

fn compute_total_typing_minutes(sessions: &[RawSession]) -> f64 {
    sessions
        .iter()
        .filter_map(|s| {
            s.end_time
                .map(|end| (end - s.start_time).num_seconds() as f64 / 60.0)
        })
        .sum()
}

fn compute_avg_wpm(sessions: &[RawSession]) -> Option<f64> {
    let valid: Vec<_> = sessions.iter().filter_map(|s| s.avg_wpm).collect();
    if valid.is_empty() {
        None
    } else {
        Some(valid.iter().sum::<f64>() / valid.len() as f64)
    }
}

fn compute_daily_wpm(sessions: &[RawSession], week: &WeekRange) -> Vec<(NaiveDate, f64)> {
    let mut daily: HashMap<NaiveDate, Vec<f64>> = HashMap::new();

    for session in sessions {
        if let Some(wpm) = session.avg_wpm {
            let date = session.start_time.date_naive();
            daily.entry(date).or_default().push(wpm);
        }
    }

    let mut result = Vec::new();
    let mut date = week.start;
    while date <= week.end {
        let avg = daily
            .get(&date)
            .map(|wpms| wpms.iter().sum::<f64>() / wpms.len() as f64)
            .unwrap_or(0.0);
        result.push((date, avg));
        date += Duration::days(1);
    }

    result
}

fn compute_wpm_distribution(samples: &[query::RawWpmSample]) -> Vec<u32> {
    // buckets: 0-20, 20-40, 40-60, 60-80, 80-100, 100-120, 120+
    let mut buckets = vec![0u32; 7];

    for sample in samples {
        let idx = match sample.wpm as u32 {
            0..=19 => 0,
            20..=39 => 1,
            40..=59 => 2,
            60..=79 => 3,
            80..=99 => 4,
            100..=119 => 5,
            _ => 6,
        };
        buckets[idx] += 1;
    }

    buckets
}

fn compute_hourly_activity(sessions: &[RawSession], week: &WeekRange) -> [[u64; 24]; 7] {
    let mut grid = [[0u64; 24]; 7];

    for session in sessions {
        let date = session.start_time.date_naive();
        if date < week.start || date > week.end {
            continue;
        }

        let day_idx = (date - week.start).num_days() as usize;
        if day_idx >= 7 {
            continue;
        }

        let hour = session.start_time.hour() as usize;
        grid[day_idx][hour] += session.keystroke_count;
    }

    grid
}

fn compute_finger_travel(counts: &HashMap<String, u64>) -> f64 {
    let distances = finger_distance_map();
    let mut total_mm = 0.0;

    for (key, &count) in counts {
        if let Some(&distance) = distances.get(key.as_str()) {
            total_mm += distance * count as f64;
        }
    }

    total_mm
}

fn finger_distance_map() -> HashMap<&'static str, f64> {
    let mut map = HashMap::new();

    // home row keys have 0 distance
    for key in ["a", "s", "d", "f", "j", "k", "l", "semicolon"] {
        map.insert(key, 0.0);
    }

    // one row up
    for key in ["q", "w", "e", "r", "u", "i", "o", "p"] {
        map.insert(key, KEY_PITCH_MM);
    }
    map.insert("t", KEY_PITCH_MM * 1.5); // index stretch
    map.insert("y", KEY_PITCH_MM * 1.5);

    // one row down
    for key in ["z", "x", "c", "v", "m", "comma", "period", "slash"] {
        map.insert(key, KEY_PITCH_MM);
    }
    map.insert("b", KEY_PITCH_MM * 1.5);
    map.insert("n", KEY_PITCH_MM * 1.5);

    // number row (2 rows up)
    for key in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
        map.insert(key, KEY_PITCH_MM * 2.0);
    }
    map.insert("grave", KEY_PITCH_MM * 2.5);
    map.insert("minus", KEY_PITCH_MM * 2.0);
    map.insert("equal", KEY_PITCH_MM * 2.0);

    // right side home row extensions
    map.insert("apostrophe", KEY_PITCH_MM);
    map.insert("leftbracket", KEY_PITCH_MM * 1.5);
    map.insert("rightbracket", KEY_PITCH_MM * 2.0);
    map.insert("backslash", KEY_PITCH_MM * 2.5);

    // special keys
    map.insert("space", KEY_PITCH_MM * 0.5); // thumbs move less
    map.insert("enter", KEY_PITCH_MM);
    map.insert("backspace", KEY_PITCH_MM * 2.5);
    map.insert("tab", KEY_PITCH_MM * 1.5);

    // g and h are index finger stretches
    map.insert("g", KEY_PITCH_MM * 0.5);
    map.insert("h", KEY_PITCH_MM * 0.5);

    map
}

fn compute_backspace_ratio(counts: &HashMap<String, u64>, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    let backspace_count = counts.get("backspace").copied().unwrap_or(0);
    backspace_count as f64 / total as f64
}

fn compute_fastest_day(sessions: &[RawSession]) -> Option<String> {
    let mut day_wpms: HashMap<Weekday, Vec<f64>> = HashMap::new();

    for session in sessions {
        if let Some(wpm) = session.avg_wpm {
            let weekday = session.start_time.weekday();
            day_wpms.entry(weekday).or_default().push(wpm);
        }
    }

    day_wpms
        .iter()
        .map(|(day, wpms)| {
            let avg = wpms.iter().sum::<f64>() / wpms.len() as f64;
            (*day, avg)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(day, _)| format!("{day:?}"))
}

fn compute_peak_hour(grid: &[[u64; 24]; 7]) -> Option<u8> {
    let mut hourly_totals = [0u64; 24];

    for day in grid {
        for (hour, &count) in day.iter().enumerate() {
            hourly_totals[hour] += count;
        }
    }

    hourly_totals
        .iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .filter(|(_, &count)| count > 0)
        .map(|(hour, _)| hour as u8)
}

fn compute_night_owl_pct(sessions: &[RawSession]) -> f64 {
    let total: u64 = sessions.iter().map(|s| s.keystroke_count).sum();
    if total == 0 {
        return 0.0;
    }

    let night_keystrokes: u64 = sessions
        .iter()
        .filter(|s| {
            let hour = s.start_time.hour();
            !(6..18).contains(&hour)
        })
        .map(|s| s.keystroke_count)
        .sum();

    night_keystrokes as f64 / total as f64 * 100.0
}

fn generate_shortcut_insight(shortcuts: &[(String, u64)]) -> Option<ShortcutInsight> {
    let shortcut_map: HashMap<&str, u64> =
        shortcuts.iter().map(|(k, v)| (k.as_str(), *v)).collect();

    let top_3: Vec<&str> = shortcuts.iter().take(3).map(|(k, _)| k.as_str()).collect();

    // high undo usage
    if let Some(&undo_count) = shortcut_map.get("ctrl+z") {
        if top_3.contains(&"ctrl+z") && undo_count > 50 {
            return Some(ShortcutInsight {
                message: format!(
                    "You undid {undo_count} actions this week. Experimenting or second-guessing?"
                ),
            });
        }
    }

    // paste more than copy
    let copy_count = shortcut_map.get("ctrl+c").copied().unwrap_or(0);
    let paste_count = shortcut_map.get("ctrl+v").copied().unwrap_or(0);
    if paste_count > copy_count && copy_count > 0 {
        let ratio = paste_count / copy_count;
        if ratio >= 2 {
            return Some(ShortcutInsight {
                message: format!("You pasted {ratio}x more than you copied. Template warrior."),
            });
        }
    }

    // frequent saves
    if let Some(&save_count) = shortcut_map.get("ctrl+s") {
        if save_count > 100 {
            return Some(ShortcutInsight {
                message: format!(
                    "You saved {save_count} times this week. Trust issues with your editor?"
                ),
            });
        }
    }

    // window switching
    if let Some(&alt_tab) = shortcut_map.get("alt+tab") {
        if top_3.contains(&"alt+tab") && alt_tab > 100 {
            return Some(ShortcutInsight {
                message: format!("You switched windows {alt_tab} times. Context switching much?"),
            });
        }
    }

    // zero undos (conviction)
    if !shortcut_map.contains_key("ctrl+z") && !shortcuts.is_empty() {
        return Some(ShortcutInsight {
            message: "Zero undos this week. You type with conviction.".to_string(),
        });
    }

    None
}
