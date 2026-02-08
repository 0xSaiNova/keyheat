use crate::aggregator::{record_key, record_shortcut, SharedAggregator};
use crate::error::Error;
use evdev::{Device, InputEventKind, Key};
use std::collections::HashSet;
use std::fs;

const MODIFIERS: &[Key] = &[
    Key::KEY_LEFTCTRL,
    Key::KEY_RIGHTCTRL,
    Key::KEY_LEFTSHIFT,
    Key::KEY_RIGHTSHIFT,
    Key::KEY_LEFTALT,
    Key::KEY_RIGHTALT,
    Key::KEY_LEFTMETA,
    Key::KEY_RIGHTMETA,
];

pub fn find_keyboards() -> Result<Vec<Device>, Error> {
    let mut keyboards = Vec::new();

    for entry in fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if !path.to_string_lossy().contains("event") {
            continue;
        }

        if let Ok(device) = Device::open(&path) {
            if is_keyboard(&device) {
                keyboards.push(device);
            }
        }
    }

    if keyboards.is_empty() {
        return Err(Error::NoKeyboards);
    }

    Ok(keyboards)
}

fn is_keyboard(device: &Device) -> bool {
    let Some(keys) = device.supported_keys() else {
        return false;
    };

    keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z) && keys.contains(Key::KEY_SPACE)
}

fn is_modifier(key: Key) -> bool {
    MODIFIERS.contains(&key)
}

fn normalize_combo(modifiers: &HashSet<Key>, key: Key) -> String {
    let mut parts: Vec<&str> = Vec::new();

    // sort modifiers alphabetically: alt, ctrl, shift, super
    if modifiers.contains(&Key::KEY_LEFTALT) || modifiers.contains(&Key::KEY_RIGHTALT) {
        parts.push("alt");
    }
    if modifiers.contains(&Key::KEY_LEFTCTRL) || modifiers.contains(&Key::KEY_RIGHTCTRL) {
        parts.push("ctrl");
    }
    if modifiers.contains(&Key::KEY_LEFTSHIFT) || modifiers.contains(&Key::KEY_RIGHTSHIFT) {
        parts.push("shift");
    }
    if modifiers.contains(&Key::KEY_LEFTMETA) || modifiers.contains(&Key::KEY_RIGHTMETA) {
        parts.push("super");
    }

    parts.push(key_name(key));
    parts.join("+")
}

fn key_name(key: Key) -> &'static str {
    // evdev key names are like KEY_A, strip the prefix and lowercase
    let name = format!("{key:?}");
    let stripped = name.strip_prefix("KEY_").unwrap_or(&name);
    // leak a lowercase version for the static lifetime
    // this is fine, we have a bounded set of keys
    Box::leak(stripped.to_lowercase().into_boxed_str())
}

pub fn run_capture(devices: Vec<Device>, agg: SharedAggregator) -> Result<(), Error> {
    let mut device = devices.into_iter().next().ok_or(Error::NoKeyboards)?;
    let mut held_modifiers: HashSet<Key> = HashSet::new();

    loop {
        for event in device.fetch_events()? {
            if let InputEventKind::Key(key) = event.kind() {
                let value = event.value();

                if is_modifier(key) {
                    match value {
                        1 => {
                            held_modifiers.insert(key);
                        }
                        0 => {
                            held_modifiers.remove(&key);
                        }
                        _ => {}
                    }
                } else if value == 1 {
                    // non-modifier key press
                    record_key(&agg, key.code());

                    if !held_modifiers.is_empty() {
                        let combo = normalize_combo(&held_modifiers, key);
                        record_shortcut(&agg, combo);
                    }
                }
            }
        }
    }
}

pub fn device_names(devices: &[Device]) -> Vec<String> {
    devices
        .iter()
        .map(|d| d.name().unwrap_or("unknown").to_string())
        .collect()
}
