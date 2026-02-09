use crate::keycode::{EventType, KeyCode, KeyEvent};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const IDLE_THRESHOLD: Duration = Duration::from_secs(30);
const WPM_WINDOW: Duration = Duration::from_secs(30);
const WPM_SAMPLE_INTERVAL: Duration = Duration::from_secs(10);
const WPM_MIN_KEYSTROKES: usize = 10;
const WPM_MIN_WINDOW: Duration = Duration::from_secs(3);

pub struct ActiveSession {
    pub db_id: i64,
    pub keystroke_count: u64,
    pub last_event_at: Instant,
}

#[derive(Debug, Clone)]
pub struct WpmSample {
    pub timestamp: DateTime<Utc>,
    pub session_id: i64,
    pub wpm: f64,
    pub keystrokes_in_window: u32,
}

pub struct WpmTracker {
    recent_timestamps: VecDeque<Instant>,
    current_wpm: f64,
    session_keystrokes: u32,
    session_start: Option<Instant>,
    peak_wpm: f64,
    samples: Vec<WpmSample>,
    last_sample_at: Option<Instant>,
    last_event_utc: Option<DateTime<Utc>>,
    had_keystrokes_since_sample: bool,
}

impl WpmTracker {
    pub fn new() -> Self {
        Self {
            recent_timestamps: VecDeque::new(),
            current_wpm: 0.0,
            session_keystrokes: 0,
            session_start: None,
            peak_wpm: 0.0,
            samples: Vec::new(),
            last_sample_at: None,
            last_event_utc: None,
            had_keystrokes_since_sample: false,
        }
    }

    pub fn reset(&mut self) {
        self.recent_timestamps.clear();
        self.current_wpm = 0.0;
        self.session_keystrokes = 0;
        self.session_start = None;
        self.peak_wpm = 0.0;
        self.last_sample_at = None;
        self.last_event_utc = None;
        self.had_keystrokes_since_sample = false;
    }

    pub fn start_session(&mut self) {
        self.reset();
        self.session_start = Some(Instant::now());
    }

    pub fn record_keystroke(&mut self, timestamp: Instant, utc_time: DateTime<Utc>, session_id: i64) {
        self.recent_timestamps.push_back(timestamp);
        self.last_event_utc = Some(utc_time);
        self.session_keystrokes += 1;
        self.had_keystrokes_since_sample = true;

        // evict timestamps outside the window
        while let Some(&front) = self.recent_timestamps.front() {
            if timestamp.duration_since(front) > WPM_WINDOW {
                self.recent_timestamps.pop_front();
            } else {
                break;
            }
        }

        self.recompute_wpm();
        self.maybe_emit_sample(timestamp, session_id);
    }

    fn recompute_wpm(&mut self) {
        let count = self.recent_timestamps.len();
        if count < WPM_MIN_KEYSTROKES {
            self.current_wpm = 0.0;
            return;
        }

        let oldest = self.recent_timestamps.front().copied();
        let newest = self.recent_timestamps.back().copied();

        if let (Some(oldest), Some(newest)) = (oldest, newest) {
            let elapsed = newest.duration_since(oldest);
            if elapsed < WPM_MIN_WINDOW {
                self.current_wpm = 0.0;
                return;
            }

            let minutes = elapsed.as_secs_f64() / 60.0;
            let words = count as f64 / 5.0;
            self.current_wpm = words / minutes;

            if self.current_wpm > self.peak_wpm {
                self.peak_wpm = self.current_wpm;
            }
        }
    }

    fn maybe_emit_sample(&mut self, now: Instant, session_id: i64) {
        let should_sample = match self.last_sample_at {
            None => true,
            Some(last) => now.duration_since(last) >= WPM_SAMPLE_INTERVAL,
        };

        if !should_sample {
            return;
        }

        // skip if WPM not ready or no keystrokes since last sample
        if self.current_wpm == 0.0 || !self.had_keystrokes_since_sample {
            return;
        }

        // Use the actual event timestamp instead of current time
        let timestamp = self.last_event_utc.unwrap_or_else(Utc::now);

        self.samples.push(WpmSample {
            timestamp,
            session_id,
            wpm: self.current_wpm,
            keystrokes_in_window: self.recent_timestamps.len() as u32,
        });

        self.last_sample_at = Some(now);
        self.had_keystrokes_since_sample = false;
    }

    pub fn take_samples(&mut self) -> Vec<WpmSample> {
        std::mem::take(&mut self.samples)
    }

    pub fn session_stats(&self) -> Option<(f64, f64)> {
        let start = self.session_start?;
        let elapsed = start.elapsed();
        if elapsed.is_zero() || self.session_keystrokes == 0 {
            return Some((0.0, self.peak_wpm));
        }

        let minutes = elapsed.as_secs_f64() / 60.0;
        let words = self.session_keystrokes as f64 / 5.0;
        let avg_wpm = words / minutes;

        Some((avg_wpm, self.peak_wpm))
    }

    pub fn current_wpm(&self) -> f64 {
        self.current_wpm
    }
}

impl Default for WpmTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Aggregator {
    key_counts: HashMap<KeyCode, u64>,
    shortcut_counts: HashMap<String, u64>,
    session: Option<ActiveSession>,
    pending_session_start: bool,
    wpm_tracker: WpmTracker,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            key_counts: HashMap::new(),
            shortcut_counts: HashMap::new(),
            session: None,
            pending_session_start: false,
            wpm_tracker: WpmTracker::new(),
        }
    }

    pub fn process_event(&mut self, event: KeyEvent) {
        // only count key down events, not releases or repeats
        if event.event_type != EventType::KeyDown {
            return;
        }

        // count the key press
        *self.key_counts.entry(event.key_code).or_insert(0) += 1;

        // update session state
        match &mut self.session {
            Some(session) => {
                session.keystroke_count += 1;
                session.last_event_at = Instant::now();

                // WPM tracking for typing-eligible keys without modifiers held
                if event.key_code.is_typing_key() && event.modifiers.is_empty() {
                    self.wpm_tracker
                        .record_keystroke(event.timestamp, Utc::now(), session.db_id);
                }
            }
            None => {
                self.pending_session_start = true;
            }
        }

        // detect shortcuts: non-modifier key pressed with modifiers held
        if !event.key_code.is_modifier() && !event.modifiers.is_empty() {
            let combo = format!("{}{}", event.modifiers.combo_prefix(), event.key_code);
            *self.shortcut_counts.entry(combo).or_insert(0) += 1;
        }
    }

    pub fn needs_session_start(&self) -> bool {
        self.pending_session_start && self.session.is_none()
    }

    pub fn start_session(&mut self, db_id: i64) {
        self.session = Some(ActiveSession {
            db_id,
            keystroke_count: 0,
            last_event_at: Instant::now(),
        });
        self.pending_session_start = false;
        self.wpm_tracker.start_session();
    }

    pub fn check_idle(&self) -> Option<(i64, u64)> {
        let session = self.session.as_ref()?;
        if session.last_event_at.elapsed() > IDLE_THRESHOLD {
            Some((session.db_id, session.keystroke_count))
        } else {
            None
        }
    }

    pub fn end_session(&mut self) -> Option<(f64, f64)> {
        let stats = self.wpm_tracker.session_stats();
        self.session = None;
        self.wpm_tracker.reset();
        stats
    }

    pub fn current_session(&self) -> Option<(i64, u64)> {
        self.session.as_ref().map(|s| (s.db_id, s.keystroke_count))
    }

    pub fn take_counts(&mut self) -> HashMap<KeyCode, u64> {
        std::mem::take(&mut self.key_counts)
    }

    pub fn take_shortcuts(&mut self) -> HashMap<String, u64> {
        std::mem::take(&mut self.shortcut_counts)
    }

    pub fn take_wpm_samples(&mut self) -> Vec<WpmSample> {
        self.wpm_tracker.take_samples()
    }

    pub fn current_wpm(&self) -> f64 {
        self.wpm_tracker.current_wpm()
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keycode::ModifierState;

    fn make_event(key: KeyCode, timestamp: Instant) -> KeyEvent {
        KeyEvent {
            key_code: key,
            event_type: EventType::KeyDown,
            timestamp,
            modifiers: ModifierState::empty(),
        }
    }

    #[test]
    fn wpm_tracker_needs_minimum_keystrokes() {
        let mut tracker = WpmTracker::new();
        tracker.start_session();

        let start = Instant::now();
        let utc_start = Utc::now();
        for i in 0..5 {
            tracker.record_keystroke(start + Duration::from_millis(i * 100), utc_start, 1);
        }

        assert_eq!(tracker.current_wpm(), 0.0);
    }

    #[test]
    fn wpm_tracker_computes_after_threshold() {
        let mut tracker = WpmTracker::new();
        tracker.start_session();

        let start = Instant::now();
        let utc_start = Utc::now();
        // 15 keystrokes over 5 seconds = 15/5 * 60 / 5 = 36 WPM
        for i in 0..15 {
            let ts = start + Duration::from_millis(i * 333);
            tracker.record_keystroke(ts, utc_start, 1);
        }

        assert!(tracker.current_wpm() > 30.0);
        assert!(tracker.current_wpm() < 40.0);
    }

    #[test]
    fn shortcuts_not_counted_for_wpm() {
        let mut agg = Aggregator::new();
        agg.start_session(1);

        let start = Instant::now();
        let mut ctrl = ModifierState::empty();
        ctrl.set_ctrl(true);

        // 20 ctrl+c presses should not affect WPM
        for i in 0..20 {
            let event = KeyEvent {
                key_code: KeyCode::C,
                event_type: EventType::KeyDown,
                timestamp: start + Duration::from_millis(i * 100),
                modifiers: ctrl,
            };
            agg.process_event(event);
        }

        assert_eq!(agg.current_wpm(), 0.0);
    }

    #[test]
    fn navigation_keys_not_counted_for_wpm() {
        let mut agg = Aggregator::new();
        agg.start_session(1);

        let start = Instant::now();
        for i in 0..20 {
            let event = make_event(KeyCode::Down, start + Duration::from_millis(i * 100));
            agg.process_event(event);
        }

        assert_eq!(agg.current_wpm(), 0.0);
    }
}
