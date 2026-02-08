mod compute;
mod html;
mod layout;
mod query;

pub use compute::build_report;
pub use html::render as render_html;

use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone)]
pub struct WeekRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub label: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // fields used by HTML renderer in Phase 3
pub struct SessionSummary {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: f64,
    pub keystroke_count: u64,
    pub avg_wpm: Option<f64>,
    pub peak_wpm: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ShortcutInsight {
    pub message: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // fields used by HTML renderer in Phase 3
pub struct ReportData {
    pub week: WeekRange,
    pub total_keystrokes: u64,
    pub prev_week_keystrokes: Option<u64>,
    pub key_frequencies: Vec<(String, u64)>,
    pub avg_wpm: f64,
    pub peak_wpm: f64,
    pub peak_wpm_time: Option<DateTime<Utc>>,
    pub prev_week_avg_wpm: Option<f64>,
    pub daily_wpm: Vec<(NaiveDate, f64)>,
    pub wpm_distribution: Vec<u32>,
    pub sessions: Vec<SessionSummary>,
    pub total_typing_minutes: f64,
    pub longest_session: Option<SessionSummary>,
    pub hourly_activity: [[u64; 24]; 7],
    pub shortcuts: Vec<(String, u64)>,
    pub prev_week_shortcuts: Vec<(String, u64)>,
    pub all_time_keystrokes: u64,
    pub finger_travel_mm: f64,
    pub backspace_ratio: f64,
    pub fastest_day: Option<String>,
    pub peak_hour: Option<u8>,
    pub night_owl_pct: f64,
    pub shortcut_insight: Option<ShortcutInsight>,
}
