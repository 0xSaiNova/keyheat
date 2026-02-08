use crate::keycode::KeyCode;
use evdev::Key;

pub fn map_evdev(key: Key) -> KeyCode {
    match key {
        // Letters
        Key::KEY_A => KeyCode::A,
        Key::KEY_B => KeyCode::B,
        Key::KEY_C => KeyCode::C,
        Key::KEY_D => KeyCode::D,
        Key::KEY_E => KeyCode::E,
        Key::KEY_F => KeyCode::F,
        Key::KEY_G => KeyCode::G,
        Key::KEY_H => KeyCode::H,
        Key::KEY_I => KeyCode::I,
        Key::KEY_J => KeyCode::J,
        Key::KEY_K => KeyCode::K,
        Key::KEY_L => KeyCode::L,
        Key::KEY_M => KeyCode::M,
        Key::KEY_N => KeyCode::N,
        Key::KEY_O => KeyCode::O,
        Key::KEY_P => KeyCode::P,
        Key::KEY_Q => KeyCode::Q,
        Key::KEY_R => KeyCode::R,
        Key::KEY_S => KeyCode::S,
        Key::KEY_T => KeyCode::T,
        Key::KEY_U => KeyCode::U,
        Key::KEY_V => KeyCode::V,
        Key::KEY_W => KeyCode::W,
        Key::KEY_X => KeyCode::X,
        Key::KEY_Y => KeyCode::Y,
        Key::KEY_Z => KeyCode::Z,

        // Number row
        Key::KEY_0 => KeyCode::Num0,
        Key::KEY_1 => KeyCode::Num1,
        Key::KEY_2 => KeyCode::Num2,
        Key::KEY_3 => KeyCode::Num3,
        Key::KEY_4 => KeyCode::Num4,
        Key::KEY_5 => KeyCode::Num5,
        Key::KEY_6 => KeyCode::Num6,
        Key::KEY_7 => KeyCode::Num7,
        Key::KEY_8 => KeyCode::Num8,
        Key::KEY_9 => KeyCode::Num9,

        // Function keys
        Key::KEY_F1 => KeyCode::F1,
        Key::KEY_F2 => KeyCode::F2,
        Key::KEY_F3 => KeyCode::F3,
        Key::KEY_F4 => KeyCode::F4,
        Key::KEY_F5 => KeyCode::F5,
        Key::KEY_F6 => KeyCode::F6,
        Key::KEY_F7 => KeyCode::F7,
        Key::KEY_F8 => KeyCode::F8,
        Key::KEY_F9 => KeyCode::F9,
        Key::KEY_F10 => KeyCode::F10,
        Key::KEY_F11 => KeyCode::F11,
        Key::KEY_F12 => KeyCode::F12,
        Key::KEY_F13 => KeyCode::F13,
        Key::KEY_F14 => KeyCode::F14,
        Key::KEY_F15 => KeyCode::F15,
        Key::KEY_F16 => KeyCode::F16,
        Key::KEY_F17 => KeyCode::F17,
        Key::KEY_F18 => KeyCode::F18,
        Key::KEY_F19 => KeyCode::F19,
        Key::KEY_F20 => KeyCode::F20,
        Key::KEY_F21 => KeyCode::F21,
        Key::KEY_F22 => KeyCode::F22,
        Key::KEY_F23 => KeyCode::F23,
        Key::KEY_F24 => KeyCode::F24,

        // Modifiers
        Key::KEY_LEFTSHIFT => KeyCode::LShift,
        Key::KEY_RIGHTSHIFT => KeyCode::RShift,
        Key::KEY_LEFTCTRL => KeyCode::LCtrl,
        Key::KEY_RIGHTCTRL => KeyCode::RCtrl,
        Key::KEY_LEFTALT => KeyCode::LAlt,
        Key::KEY_RIGHTALT => KeyCode::RAlt,
        Key::KEY_LEFTMETA => KeyCode::LSuper,
        Key::KEY_RIGHTMETA => KeyCode::RSuper,

        // Common keys
        Key::KEY_SPACE => KeyCode::Space,
        Key::KEY_TAB => KeyCode::Tab,
        Key::KEY_ENTER => KeyCode::Enter,
        Key::KEY_ESC => KeyCode::Escape,
        Key::KEY_BACKSPACE => KeyCode::Backspace,
        Key::KEY_DELETE => KeyCode::Delete,
        Key::KEY_CAPSLOCK => KeyCode::CapsLock,
        Key::KEY_NUMLOCK => KeyCode::NumLock,
        Key::KEY_SCROLLLOCK => KeyCode::ScrollLock,
        Key::KEY_INSERT => KeyCode::Insert,
        Key::KEY_SYSRQ => KeyCode::PrintScreen,
        Key::KEY_PAUSE => KeyCode::Pause,

        // Navigation
        Key::KEY_UP => KeyCode::Up,
        Key::KEY_DOWN => KeyCode::Down,
        Key::KEY_LEFT => KeyCode::Left,
        Key::KEY_RIGHT => KeyCode::Right,
        Key::KEY_HOME => KeyCode::Home,
        Key::KEY_END => KeyCode::End,
        Key::KEY_PAGEUP => KeyCode::PageUp,
        Key::KEY_PAGEDOWN => KeyCode::PageDown,

        // Punctuation and symbols
        Key::KEY_MINUS => KeyCode::Minus,
        Key::KEY_EQUAL => KeyCode::Equal,
        Key::KEY_LEFTBRACE => KeyCode::LeftBracket,
        Key::KEY_RIGHTBRACE => KeyCode::RightBracket,
        Key::KEY_SEMICOLON => KeyCode::Semicolon,
        Key::KEY_APOSTROPHE => KeyCode::Apostrophe,
        Key::KEY_GRAVE => KeyCode::Grave,
        Key::KEY_BACKSLASH => KeyCode::Backslash,
        Key::KEY_COMMA => KeyCode::Comma,
        Key::KEY_DOT => KeyCode::Period,
        Key::KEY_SLASH => KeyCode::Slash,

        // Numpad
        Key::KEY_KP0 => KeyCode::Numpad0,
        Key::KEY_KP1 => KeyCode::Numpad1,
        Key::KEY_KP2 => KeyCode::Numpad2,
        Key::KEY_KP3 => KeyCode::Numpad3,
        Key::KEY_KP4 => KeyCode::Numpad4,
        Key::KEY_KP5 => KeyCode::Numpad5,
        Key::KEY_KP6 => KeyCode::Numpad6,
        Key::KEY_KP7 => KeyCode::Numpad7,
        Key::KEY_KP8 => KeyCode::Numpad8,
        Key::KEY_KP9 => KeyCode::Numpad9,
        Key::KEY_KPPLUS => KeyCode::NumpadAdd,
        Key::KEY_KPMINUS => KeyCode::NumpadSubtract,
        Key::KEY_KPASTERISK => KeyCode::NumpadMultiply,
        Key::KEY_KPSLASH => KeyCode::NumpadDivide,
        Key::KEY_KPENTER => KeyCode::NumpadEnter,
        Key::KEY_KPDOT => KeyCode::NumpadDecimal,

        // Misc
        Key::KEY_COMPOSE => KeyCode::Menu,

        // Unknown
        _ => KeyCode::Unknown(key.code() as u32),
    }
}

pub fn update_modifier_state(
    modifiers: &mut crate::keycode::ModifierState,
    key: KeyCode,
    pressed: bool,
) {
    match key {
        KeyCode::LShift | KeyCode::RShift => modifiers.set_shift(pressed),
        KeyCode::LCtrl | KeyCode::RCtrl => modifiers.set_ctrl(pressed),
        KeyCode::LAlt | KeyCode::RAlt => modifiers.set_alt(pressed),
        KeyCode::LSuper | KeyCode::RSuper => modifiers.set_super(pressed),
        _ => {}
    }
}
