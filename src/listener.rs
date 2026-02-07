use crate::aggregator::{record_key, SharedCounts};
use crate::error::Error;
use evdev::{Device, InputEventKind, Key};
use std::fs;

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

    // real keyboards support the alphabet
    keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z) && keys.contains(Key::KEY_SPACE)
}

pub fn run_capture(devices: Vec<Device>, counts: SharedCounts) -> Result<(), Error> {
    // for slice 1, just grab the first keyboard
    // multi-device polling comes later
    let mut device = devices.into_iter().next().ok_or(Error::NoKeyboards)?;

    loop {
        for event in device.fetch_events()? {
            if let InputEventKind::Key(key) = event.kind() {
                // value 1 = key down, 0 = key up, 2 = repeat
                if event.value() == 1 {
                    record_key(&counts, key.code());
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
