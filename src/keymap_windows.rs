use crate::keycode::KeyCode;

pub fn map_vk(vk: u32) -> KeyCode {
    match vk {
        // Letters (0x41-0x5A)
        0x41 => KeyCode::A,
        0x42 => KeyCode::B,
        0x43 => KeyCode::C,
        0x44 => KeyCode::D,
        0x45 => KeyCode::E,
        0x46 => KeyCode::F,
        0x47 => KeyCode::G,
        0x48 => KeyCode::H,
        0x49 => KeyCode::I,
        0x4A => KeyCode::J,
        0x4B => KeyCode::K,
        0x4C => KeyCode::L,
        0x4D => KeyCode::M,
        0x4E => KeyCode::N,
        0x4F => KeyCode::O,
        0x50 => KeyCode::P,
        0x51 => KeyCode::Q,
        0x52 => KeyCode::R,
        0x53 => KeyCode::S,
        0x54 => KeyCode::T,
        0x55 => KeyCode::U,
        0x56 => KeyCode::V,
        0x57 => KeyCode::W,
        0x58 => KeyCode::X,
        0x59 => KeyCode::Y,
        0x5A => KeyCode::Z,

        // Number row (0x30-0x39)
        0x30 => KeyCode::Num0,
        0x31 => KeyCode::Num1,
        0x32 => KeyCode::Num2,
        0x33 => KeyCode::Num3,
        0x34 => KeyCode::Num4,
        0x35 => KeyCode::Num5,
        0x36 => KeyCode::Num6,
        0x37 => KeyCode::Num7,
        0x38 => KeyCode::Num8,
        0x39 => KeyCode::Num9,

        // Function keys (0x70-0x87)
        0x70 => KeyCode::F1,
        0x71 => KeyCode::F2,
        0x72 => KeyCode::F3,
        0x73 => KeyCode::F4,
        0x74 => KeyCode::F5,
        0x75 => KeyCode::F6,
        0x76 => KeyCode::F7,
        0x77 => KeyCode::F8,
        0x78 => KeyCode::F9,
        0x79 => KeyCode::F10,
        0x7A => KeyCode::F11,
        0x7B => KeyCode::F12,
        0x7C => KeyCode::F13,
        0x7D => KeyCode::F14,
        0x7E => KeyCode::F15,
        0x7F => KeyCode::F16,
        0x80 => KeyCode::F17,
        0x81 => KeyCode::F18,
        0x82 => KeyCode::F19,
        0x83 => KeyCode::F20,
        0x84 => KeyCode::F21,
        0x85 => KeyCode::F22,
        0x86 => KeyCode::F23,
        0x87 => KeyCode::F24,

        // Modifiers
        0xA0 => KeyCode::LShift, // VK_LSHIFT
        0xA1 => KeyCode::RShift, // VK_RSHIFT
        0xA2 => KeyCode::LCtrl,  // VK_LCONTROL
        0xA3 => KeyCode::RCtrl,  // VK_RCONTROL
        0xA4 => KeyCode::LAlt,   // VK_LMENU
        0xA5 => KeyCode::RAlt,   // VK_RMENU
        0x5B => KeyCode::LSuper, // VK_LWIN
        0x5C => KeyCode::RSuper, // VK_RWIN

        // Common keys
        0x20 => KeyCode::Space,       // VK_SPACE
        0x09 => KeyCode::Tab,         // VK_TAB
        0x0D => KeyCode::Enter,       // VK_RETURN
        0x1B => KeyCode::Escape,      // VK_ESCAPE
        0x08 => KeyCode::Backspace,   // VK_BACK
        0x2E => KeyCode::Delete,      // VK_DELETE
        0x14 => KeyCode::CapsLock,    // VK_CAPITAL
        0x90 => KeyCode::NumLock,     // VK_NUMLOCK
        0x91 => KeyCode::ScrollLock,  // VK_SCROLL
        0x2D => KeyCode::Insert,      // VK_INSERT
        0x2C => KeyCode::PrintScreen, // VK_SNAPSHOT
        0x13 => KeyCode::Pause,       // VK_PAUSE

        // Navigation
        0x26 => KeyCode::Up,       // VK_UP
        0x28 => KeyCode::Down,     // VK_DOWN
        0x25 => KeyCode::Left,     // VK_LEFT
        0x27 => KeyCode::Right,    // VK_RIGHT
        0x24 => KeyCode::Home,     // VK_HOME
        0x23 => KeyCode::End,      // VK_END
        0x21 => KeyCode::PageUp,   // VK_PRIOR
        0x22 => KeyCode::PageDown, // VK_NEXT

        // Punctuation and symbols (OEM keys)
        0xBD => KeyCode::Minus,        // VK_OEM_MINUS
        0xBB => KeyCode::Equal,        // VK_OEM_PLUS (the = key)
        0xDB => KeyCode::LeftBracket,  // VK_OEM_4
        0xDD => KeyCode::RightBracket, // VK_OEM_6
        0xBA => KeyCode::Semicolon,    // VK_OEM_1
        0xDE => KeyCode::Apostrophe,   // VK_OEM_7
        0xC0 => KeyCode::Grave,        // VK_OEM_3
        0xDC => KeyCode::Backslash,    // VK_OEM_5
        0xBC => KeyCode::Comma,        // VK_OEM_COMMA
        0xBE => KeyCode::Period,       // VK_OEM_PERIOD
        0xBF => KeyCode::Slash,        // VK_OEM_2

        // Numpad
        0x60 => KeyCode::Numpad0,        // VK_NUMPAD0
        0x61 => KeyCode::Numpad1,        // VK_NUMPAD1
        0x62 => KeyCode::Numpad2,        // VK_NUMPAD2
        0x63 => KeyCode::Numpad3,        // VK_NUMPAD3
        0x64 => KeyCode::Numpad4,        // VK_NUMPAD4
        0x65 => KeyCode::Numpad5,        // VK_NUMPAD5
        0x66 => KeyCode::Numpad6,        // VK_NUMPAD6
        0x67 => KeyCode::Numpad7,        // VK_NUMPAD7
        0x68 => KeyCode::Numpad8,        // VK_NUMPAD8
        0x69 => KeyCode::Numpad9,        // VK_NUMPAD9
        0x6B => KeyCode::NumpadAdd,      // VK_ADD
        0x6D => KeyCode::NumpadSubtract, // VK_SUBTRACT
        0x6A => KeyCode::NumpadMultiply, // VK_MULTIPLY
        0x6F => KeyCode::NumpadDivide,   // VK_DIVIDE
        0x6E => KeyCode::NumpadDecimal,  // VK_DECIMAL

        // Misc
        0x5D => KeyCode::Menu, // VK_APPS

        // Unknown
        _ => KeyCode::Unknown(vk),
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
