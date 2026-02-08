use crate::error::Error;
use crate::keycode::{EventType, KeyEvent, ModifierState};
use crate::keymap_linux::{map_evdev, update_modifier_state};
use evdev::{Device, InputEventKind, Key};
use std::fs;
use std::sync::mpsc::Sender;

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

pub fn run_capture(devices: Vec<Device>, sender: Sender<KeyEvent>) -> Result<(), Error> {
    let mut device = devices.into_iter().next().ok_or(Error::NoKeyboards)?;
    let mut modifiers = ModifierState::empty();

    loop {
        for event in device.fetch_events()? {
            if let InputEventKind::Key(key) = event.kind() {
                let key_code = map_evdev(key);
                let value = event.value();

                let event_type = match value {
                    1 => EventType::KeyDown,
                    0 => EventType::KeyUp,
                    2 => EventType::Repeat,
                    _ => continue,
                };

                // update modifier state for key down/up
                if key_code.is_modifier() {
                    update_modifier_state(&mut modifiers, key_code, value == 1);
                }

                let key_event = KeyEvent::new(key_code, event_type, modifiers);

                if sender.send(key_event).is_err() {
                    // receiver dropped, exit cleanly
                    return Ok(());
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
