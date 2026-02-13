use crate::error::Error;
use crate::keycode::{EventType, KeyEvent, ModifierState};
use crate::keymap_windows::{map_vk_extended, update_modifier_state};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

// Use Mutex<Option<T>> so we can reset state even after OnceLock initialization
static SENDER: OnceLock<Mutex<Option<Sender<KeyEvent>>>> = OnceLock::new();
static MODIFIERS: OnceLock<Mutex<ModifierState>> = OnceLock::new();

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;
        let is_extended = (kb.flags & LLKHF_EXTENDED) != 0;

        let event_type = match wparam as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => EventType::KeyDown,
            WM_KEYUP | WM_SYSKEYUP => EventType::KeyUp,
            _ => {
                return CallNextHookEx(0, code, wparam, lparam);
            }
        };

        let key_code = map_vk_extended(vk, is_extended);

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

                if let Some(sender_mutex) = SENDER.get() {
                    if let Ok(sender_opt) = sender_mutex.lock() {
                        if let Some(sender) = sender_opt.as_ref() {
                            let _ = sender.send(event);
                        }
                    }
                }
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}

pub fn run_capture(sender: Sender<KeyEvent>) -> Result<(), Error> {
    // Set up the hook first
    let hook: HHOOK = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), 0, 0) };

    if hook == 0 {
        return Err(Error::Hook("SetWindowsHookExW failed".into()));
    }

    // Initialize or reuse OnceLock containers, then set the inner values
    let sender_mutex = SENDER.get_or_init(|| Mutex::new(None));
    let modifiers_mutex = MODIFIERS.get_or_init(|| Mutex::new(ModifierState::empty()));

    // Try to set the sender
    {
        let mut sender_guard = sender_mutex.lock().map_err(|_| {
            unsafe { UnhookWindowsHookEx(hook); }
            Error::Hook("failed to lock sender mutex".into())
        })?;

        if sender_guard.is_some() {
            unsafe { UnhookWindowsHookEx(hook); }
            return Err(Error::Hook("hook already running".into()));
        }

        *sender_guard = Some(sender);
    }

    // Reset modifiers to empty state
    {
        let mut modifiers_guard = modifiers_mutex.lock().map_err(|_| {
            unsafe { UnhookWindowsHookEx(hook); }
            Error::Hook("failed to lock modifiers mutex".into())
        })?;
        *modifiers_guard = ModifierState::empty();
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

    // Clean up: unhook and clear sender
    unsafe {
        UnhookWindowsHookEx(hook);
    }

    if let Ok(mut sender_guard) = sender_mutex.lock() {
        *sender_guard = None;
    }

    Ok(())
}
