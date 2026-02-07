use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedCounts = Arc<Mutex<HashMap<u16, u64>>>;

pub fn new_shared_counts() -> SharedCounts {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn record_key(counts: &SharedCounts, key_code: u16) {
    let mut map = counts.lock().unwrap();
    *map.entry(key_code).or_insert(0) += 1;
}

pub fn take_counts(counts: &SharedCounts) -> HashMap<u16, u64> {
    let mut map = counts.lock().unwrap();
    std::mem::take(&mut *map)
}
