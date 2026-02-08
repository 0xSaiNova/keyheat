use crate::error::Error;
use crate::keycode::{EventType, KeyEvent, ModifierState};
use crate::keymap_windows::{map_vk, update_modifier_state};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static SENDER: OnceLock<Sender<KeyEvent>> = OnceLock::new();
static MODIFIERS: OnceLock<std::sync::Mutex<ModifierState>> = OnceLock::new();

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;

        let event_type = match wparam as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => EventType::KeyDown,
            WM_KEYUP | WM_SYSKEYUP => EventType::KeyUp,
            _ => {
                return CallNextHookEx(0, code, wparam, lparam);
            }
        };

        let key_code = map_vk(vk);

        if let Some(mutex) = MODIFIERS.get() {
            if let Ok(mut modifiers) = mutex.lock() {
                if key_code.is_modifier() {
                    update_modifier_state(
                        &mut modifiers,
                        key_code,
                        event_type == EventType::KeyDown,
                    );
                }

                let event = KeyEvent::new(key_code, event_type, *modifiers);

                if let Some(sender) = SENDER.get() {
                    let _ = sender.send(event);
                }
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}

pub fn run_capture(sender: Sender<KeyEvent>) -> Result<(), Error> {
    SENDER
        .set(sender)
        .map_err(|_| Error::Hook("sender already initialized".into()))?;

    MODIFIERS
        .set(std::sync::Mutex::new(ModifierState::empty()))
        .map_err(|_| Error::Hook("modifiers already initialized".into()))?;

    let hook: HHOOK = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), 0, 0) };

    if hook == 0 {
        return Err(Error::Hook("SetWindowsHookExW failed".into()));
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };

    loop {
        let ret = unsafe { GetMessageW(&mut msg, 0, 0, 0) };

        if ret == 0 || ret == -1 {
            break;
        }

        unsafe {
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        UnhookWindowsHookEx(hook);
    }

    Ok(())
}
