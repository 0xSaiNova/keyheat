use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const IDLE_THRESHOLD: Duration = Duration::from_secs(30);

pub struct ActiveSession {
    pub db_id: i64,
    pub keystroke_count: u64,
    pub last_event_at: Instant,
}

pub struct Aggregator {
    key_counts: HashMap<u16, u64>,
    shortcut_counts: HashMap<String, u64>,
    session: Option<ActiveSession>,
    pending_session_start: bool,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            key_counts: HashMap::new(),
            shortcut_counts: HashMap::new(),
            session: None,
            pending_session_start: false,
        }
    }

    pub fn record_key(&mut self, key_code: u16) {
        *self.key_counts.entry(key_code).or_insert(0) += 1;

        let now = Instant::now();

        match &mut self.session {
            Some(session) => {
                session.keystroke_count += 1;
                session.last_event_at = now;
            }
            None => {
                self.pending_session_start = true;
            }
        }
    }

    pub fn record_shortcut(&mut self, combo: String) {
        *self.shortcut_counts.entry(combo).or_insert(0) += 1;
    }

    pub fn needs_session_start(&self) -> bool {
        self.pending_session_start && self.session.is_none()
    }

    pub fn start_session(&mut self, db_id: i64) {
        self.session = Some(ActiveSession {
            db_id,
            keystroke_count: self.key_counts.values().sum(),
            last_event_at: Instant::now(),
        });
        self.pending_session_start = false;
    }

    pub fn check_idle(&self) -> Option<(i64, u64)> {
        let session = self.session.as_ref()?;
        if session.last_event_at.elapsed() > IDLE_THRESHOLD {
            Some((session.db_id, session.keystroke_count))
        } else {
            None
        }
    }

    pub fn end_session(&mut self) {
        self.session = None;
    }

    pub fn current_session(&self) -> Option<(i64, u64)> {
        self.session.as_ref().map(|s| (s.db_id, s.keystroke_count))
    }

    pub fn take_counts(&mut self) -> HashMap<u16, u64> {
        std::mem::take(&mut self.key_counts)
    }

    pub fn take_shortcuts(&mut self) -> HashMap<String, u64> {
        std::mem::take(&mut self.shortcut_counts)
    }
}

pub type SharedAggregator = Arc<Mutex<Aggregator>>;

pub fn new_aggregator() -> SharedAggregator {
    Arc::new(Mutex::new(Aggregator::new()))
}

pub fn record_key(agg: &SharedAggregator, key_code: u16) {
    agg.lock().unwrap().record_key(key_code);
}

pub fn record_shortcut(agg: &SharedAggregator, combo: String) {
    agg.lock().unwrap().record_shortcut(combo);
}
