use crate::error::Error;
use crate::keycode::{EventType, KeyEvent, ModifierState};
use crate::keymap_linux::{map_evdev, update_modifier_state};
use evdev::{Device, InputEventKind, Key};
use std::fs;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;

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
    if devices.is_empty() {
        return Err(Error::NoKeyboards);
    }

    let mut handles = Vec::new();

    // Shared modifier state across all keyboard devices
    // This ensures shortcuts work correctly when using multiple keyboards
    let modifiers = Arc::new(Mutex::new(ModifierState::empty()));

    // Spawn a thread for each keyboard device
    for mut device in devices {
        let sender = sender.clone();
        let modifiers = Arc::clone(&modifiers);

        let handle = thread::spawn(move || {
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let InputEventKind::Key(key) = event.kind() {
                                let key_code = map_evdev(key);
                                let value = event.value();

                                let event_type = match value {
                                    1 => EventType::KeyDown,
                                    0 => EventType::KeyUp,
                                    2 => EventType::Repeat,
                                    _ => continue,
                                };

                                // Lock modifier state and update it
                                let current_modifiers = if let Ok(mut mods) = modifiers.lock() {
                                    if key_code.is_modifier() {
                                        update_modifier_state(&mut mods, key_code, value == 1);
                                    }
                                    *mods
                                } else {
                                    // Mutex poisoned, continue with empty modifiers
                                    ModifierState::empty()
                                };

                                let key_event = KeyEvent::new(key_code, event_type, current_modifiers);

                                if sender.send(key_event).is_err() {
                                    // receiver dropped, exit cleanly
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("keyboard device error: {e}, thread exiting");
                        return;
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete (they run indefinitely unless there's an error)
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

pub fn device_names(devices: &[Device]) -> Vec<String> {
    devices
        .iter()
        .map(|d| d.name().unwrap_or("unknown").to_string())
        .collect()
}
