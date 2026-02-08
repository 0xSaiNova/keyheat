use std::fmt;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Number row
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    // Modifiers (left/right tracked separately)
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LSuper,
    RSuper,

    // Common keys
    Space,
    Tab,
    Enter,
    Escape,
    Backspace,
    Delete,
    CapsLock,
    NumLock,
    ScrollLock,
    Insert,
    PrintScreen,
    Pause,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Punctuation and symbols
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Semicolon,
    Apostrophe,
    Grave,
    Backslash,
    Comma,
    Period,
    Slash,

    // Numpad
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadEnter,
    NumpadDecimal,

    // Misc
    Menu,

    // Fallback for unmapped keys
    Unknown(u32),
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            KeyCode::A => "a",
            KeyCode::B => "b",
            KeyCode::C => "c",
            KeyCode::D => "d",
            KeyCode::E => "e",
            KeyCode::F => "f",
            KeyCode::G => "g",
            KeyCode::H => "h",
            KeyCode::I => "i",
            KeyCode::J => "j",
            KeyCode::K => "k",
            KeyCode::L => "l",
            KeyCode::M => "m",
            KeyCode::N => "n",
            KeyCode::O => "o",
            KeyCode::P => "p",
            KeyCode::Q => "q",
            KeyCode::R => "r",
            KeyCode::S => "s",
            KeyCode::T => "t",
            KeyCode::U => "u",
            KeyCode::V => "v",
            KeyCode::W => "w",
            KeyCode::X => "x",
            KeyCode::Y => "y",
            KeyCode::Z => "z",

            KeyCode::Num0 => "0",
            KeyCode::Num1 => "1",
            KeyCode::Num2 => "2",
            KeyCode::Num3 => "3",
            KeyCode::Num4 => "4",
            KeyCode::Num5 => "5",
            KeyCode::Num6 => "6",
            KeyCode::Num7 => "7",
            KeyCode::Num8 => "8",
            KeyCode::Num9 => "9",

            KeyCode::F1 => "f1",
            KeyCode::F2 => "f2",
            KeyCode::F3 => "f3",
            KeyCode::F4 => "f4",
            KeyCode::F5 => "f5",
            KeyCode::F6 => "f6",
            KeyCode::F7 => "f7",
            KeyCode::F8 => "f8",
            KeyCode::F9 => "f9",
            KeyCode::F10 => "f10",
            KeyCode::F11 => "f11",
            KeyCode::F12 => "f12",
            KeyCode::F13 => "f13",
            KeyCode::F14 => "f14",
            KeyCode::F15 => "f15",
            KeyCode::F16 => "f16",
            KeyCode::F17 => "f17",
            KeyCode::F18 => "f18",
            KeyCode::F19 => "f19",
            KeyCode::F20 => "f20",
            KeyCode::F21 => "f21",
            KeyCode::F22 => "f22",
            KeyCode::F23 => "f23",
            KeyCode::F24 => "f24",

            KeyCode::LShift => "lshift",
            KeyCode::RShift => "rshift",
            KeyCode::LCtrl => "lctrl",
            KeyCode::RCtrl => "rctrl",
            KeyCode::LAlt => "lalt",
            KeyCode::RAlt => "ralt",
            KeyCode::LSuper => "lsuper",
            KeyCode::RSuper => "rsuper",

            KeyCode::Space => "space",
            KeyCode::Tab => "tab",
            KeyCode::Enter => "enter",
            KeyCode::Escape => "escape",
            KeyCode::Backspace => "backspace",
            KeyCode::Delete => "delete",
            KeyCode::CapsLock => "capslock",
            KeyCode::NumLock => "numlock",
            KeyCode::ScrollLock => "scrolllock",
            KeyCode::Insert => "insert",
            KeyCode::PrintScreen => "printscreen",
            KeyCode::Pause => "pause",

            KeyCode::Up => "up",
            KeyCode::Down => "down",
            KeyCode::Left => "left",
            KeyCode::Right => "right",
            KeyCode::Home => "home",
            KeyCode::End => "end",
            KeyCode::PageUp => "pageup",
            KeyCode::PageDown => "pagedown",

            KeyCode::Minus => "minus",
            KeyCode::Equal => "equal",
            KeyCode::LeftBracket => "leftbracket",
            KeyCode::RightBracket => "rightbracket",
            KeyCode::Semicolon => "semicolon",
            KeyCode::Apostrophe => "apostrophe",
            KeyCode::Grave => "grave",
            KeyCode::Backslash => "backslash",
            KeyCode::Comma => "comma",
            KeyCode::Period => "period",
            KeyCode::Slash => "slash",

            KeyCode::Numpad0 => "numpad0",
            KeyCode::Numpad1 => "numpad1",
            KeyCode::Numpad2 => "numpad2",
            KeyCode::Numpad3 => "numpad3",
            KeyCode::Numpad4 => "numpad4",
            KeyCode::Numpad5 => "numpad5",
            KeyCode::Numpad6 => "numpad6",
            KeyCode::Numpad7 => "numpad7",
            KeyCode::Numpad8 => "numpad8",
            KeyCode::Numpad9 => "numpad9",
            KeyCode::NumpadAdd => "numpadadd",
            KeyCode::NumpadSubtract => "numpadsubtract",
            KeyCode::NumpadMultiply => "numpadmultiply",
            KeyCode::NumpadDivide => "numpaddivide",
            KeyCode::NumpadEnter => "numpadenter",
            KeyCode::NumpadDecimal => "numpaddecimal",

            KeyCode::Menu => "menu",

            KeyCode::Unknown(code) => return write!(f, "unknown_{code}"),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub struct ParseKeyCodeError;

impl FromStr for KeyCode {
    type Err = ParseKeyCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = match s {
            "a" => KeyCode::A,
            "b" => KeyCode::B,
            "c" => KeyCode::C,
            "d" => KeyCode::D,
            "e" => KeyCode::E,
            "f" => KeyCode::F,
            "g" => KeyCode::G,
            "h" => KeyCode::H,
            "i" => KeyCode::I,
            "j" => KeyCode::J,
            "k" => KeyCode::K,
            "l" => KeyCode::L,
            "m" => KeyCode::M,
            "n" => KeyCode::N,
            "o" => KeyCode::O,
            "p" => KeyCode::P,
            "q" => KeyCode::Q,
            "r" => KeyCode::R,
            "s" => KeyCode::S,
            "t" => KeyCode::T,
            "u" => KeyCode::U,
            "v" => KeyCode::V,
            "w" => KeyCode::W,
            "x" => KeyCode::X,
            "y" => KeyCode::Y,
            "z" => KeyCode::Z,

            "0" => KeyCode::Num0,
            "1" => KeyCode::Num1,
            "2" => KeyCode::Num2,
            "3" => KeyCode::Num3,
            "4" => KeyCode::Num4,
            "5" => KeyCode::Num5,
            "6" => KeyCode::Num6,
            "7" => KeyCode::Num7,
            "8" => KeyCode::Num8,
            "9" => KeyCode::Num9,

            "f1" => KeyCode::F1,
            "f2" => KeyCode::F2,
            "f3" => KeyCode::F3,
            "f4" => KeyCode::F4,
            "f5" => KeyCode::F5,
            "f6" => KeyCode::F6,
            "f7" => KeyCode::F7,
            "f8" => KeyCode::F8,
            "f9" => KeyCode::F9,
            "f10" => KeyCode::F10,
            "f11" => KeyCode::F11,
            "f12" => KeyCode::F12,
            "f13" => KeyCode::F13,
            "f14" => KeyCode::F14,
            "f15" => KeyCode::F15,
            "f16" => KeyCode::F16,
            "f17" => KeyCode::F17,
            "f18" => KeyCode::F18,
            "f19" => KeyCode::F19,
            "f20" => KeyCode::F20,
            "f21" => KeyCode::F21,
            "f22" => KeyCode::F22,
            "f23" => KeyCode::F23,
            "f24" => KeyCode::F24,

            "lshift" => KeyCode::LShift,
            "rshift" => KeyCode::RShift,
            "lctrl" => KeyCode::LCtrl,
            "rctrl" => KeyCode::RCtrl,
            "lalt" => KeyCode::LAlt,
            "ralt" => KeyCode::RAlt,
            "lsuper" => KeyCode::LSuper,
            "rsuper" => KeyCode::RSuper,

            "space" => KeyCode::Space,
            "tab" => KeyCode::Tab,
            "enter" => KeyCode::Enter,
            "escape" => KeyCode::Escape,
            "backspace" => KeyCode::Backspace,
            "delete" => KeyCode::Delete,
            "capslock" => KeyCode::CapsLock,
            "numlock" => KeyCode::NumLock,
            "scrolllock" => KeyCode::ScrollLock,
            "insert" => KeyCode::Insert,
            "printscreen" => KeyCode::PrintScreen,
            "pause" => KeyCode::Pause,

            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,

            "minus" => KeyCode::Minus,
            "equal" => KeyCode::Equal,
            "leftbracket" => KeyCode::LeftBracket,
            "rightbracket" => KeyCode::RightBracket,
            "semicolon" => KeyCode::Semicolon,
            "apostrophe" => KeyCode::Apostrophe,
            "grave" => KeyCode::Grave,
            "backslash" => KeyCode::Backslash,
            "comma" => KeyCode::Comma,
            "period" => KeyCode::Period,
            "slash" => KeyCode::Slash,

            "numpad0" => KeyCode::Numpad0,
            "numpad1" => KeyCode::Numpad1,
            "numpad2" => KeyCode::Numpad2,
            "numpad3" => KeyCode::Numpad3,
            "numpad4" => KeyCode::Numpad4,
            "numpad5" => KeyCode::Numpad5,
            "numpad6" => KeyCode::Numpad6,
            "numpad7" => KeyCode::Numpad7,
            "numpad8" => KeyCode::Numpad8,
            "numpad9" => KeyCode::Numpad9,
            "numpadadd" => KeyCode::NumpadAdd,
            "numpadsubtract" => KeyCode::NumpadSubtract,
            "numpadmultiply" => KeyCode::NumpadMultiply,
            "numpaddivide" => KeyCode::NumpadDivide,
            "numpadenter" => KeyCode::NumpadEnter,
            "numpaddecimal" => KeyCode::NumpadDecimal,

            "menu" => KeyCode::Menu,

            _ => {
                if let Some(code_str) = s.strip_prefix("unknown_") {
                    if let Ok(code) = code_str.parse::<u32>() {
                        return Ok(KeyCode::Unknown(code));
                    }
                }
                return Err(ParseKeyCodeError);
            }
        };
        Ok(key)
    }
}

impl KeyCode {
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            KeyCode::LShift
                | KeyCode::RShift
                | KeyCode::LCtrl
                | KeyCode::RCtrl
                | KeyCode::LAlt
                | KeyCode::RAlt
                | KeyCode::LSuper
                | KeyCode::RSuper
        )
    }

    pub fn is_typing_key(&self) -> bool {
        matches!(
            self,
            // Letters
            KeyCode::A | KeyCode::B | KeyCode::C | KeyCode::D | KeyCode::E |
            KeyCode::F | KeyCode::G | KeyCode::H | KeyCode::I | KeyCode::J |
            KeyCode::K | KeyCode::L | KeyCode::M | KeyCode::N | KeyCode::O |
            KeyCode::P | KeyCode::Q | KeyCode::R | KeyCode::S | KeyCode::T |
            KeyCode::U | KeyCode::V | KeyCode::W | KeyCode::X | KeyCode::Y |
            KeyCode::Z |
            // Numbers
            KeyCode::Num0 | KeyCode::Num1 | KeyCode::Num2 | KeyCode::Num3 |
            KeyCode::Num4 | KeyCode::Num5 | KeyCode::Num6 | KeyCode::Num7 |
            KeyCode::Num8 | KeyCode::Num9 |
            // Whitespace and corrections
            KeyCode::Space | KeyCode::Tab | KeyCode::Enter |
            KeyCode::Backspace | KeyCode::Delete |
            // Punctuation
            KeyCode::Minus | KeyCode::Equal |
            KeyCode::LeftBracket | KeyCode::RightBracket |
            KeyCode::Semicolon | KeyCode::Apostrophe |
            KeyCode::Grave | KeyCode::Backslash |
            KeyCode::Comma | KeyCode::Period | KeyCode::Slash |
            // Numpad (typing, not navigation)
            KeyCode::Numpad0 | KeyCode::Numpad1 | KeyCode::Numpad2 |
            KeyCode::Numpad3 | KeyCode::Numpad4 | KeyCode::Numpad5 |
            KeyCode::Numpad6 | KeyCode::Numpad7 | KeyCode::Numpad8 |
            KeyCode::Numpad9 | KeyCode::NumpadAdd | KeyCode::NumpadSubtract |
            KeyCode::NumpadMultiply | KeyCode::NumpadDivide |
            KeyCode::NumpadEnter | KeyCode::NumpadDecimal
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    KeyDown,
    KeyUp,
    Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState(u8);

impl ModifierState {
    pub const SHIFT: u8 = 0b0001;
    pub const CTRL: u8 = 0b0010;
    pub const ALT: u8 = 0b0100;
    pub const SUPER: u8 = 0b1000;

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn set_shift(&mut self, pressed: bool) {
        if pressed {
            self.0 |= Self::SHIFT;
        } else {
            self.0 &= !Self::SHIFT;
        }
    }

    pub fn set_ctrl(&mut self, pressed: bool) {
        if pressed {
            self.0 |= Self::CTRL;
        } else {
            self.0 &= !Self::CTRL;
        }
    }

    pub fn set_alt(&mut self, pressed: bool) {
        if pressed {
            self.0 |= Self::ALT;
        } else {
            self.0 &= !Self::ALT;
        }
    }

    pub fn set_super(&mut self, pressed: bool) {
        if pressed {
            self.0 |= Self::SUPER;
        } else {
            self.0 &= !Self::SUPER;
        }
    }

    pub fn has_shift(&self) -> bool {
        self.0 & Self::SHIFT != 0
    }
    pub fn has_ctrl(&self) -> bool {
        self.0 & Self::CTRL != 0
    }
    pub fn has_alt(&self) -> bool {
        self.0 & Self::ALT != 0
    }
    pub fn has_super(&self) -> bool {
        self.0 & Self::SUPER != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn combo_prefix(self) -> String {
        let mut parts = Vec::new();
        if self.has_alt() {
            parts.push("alt");
        }
        if self.has_ctrl() {
            parts.push("ctrl");
        }
        if self.has_shift() {
            parts.push("shift");
        }
        if self.has_super() {
            parts.push("super");
        }
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("+") + "+"
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub event_type: EventType,
    pub timestamp: Instant,
    pub modifiers: ModifierState,
}

impl KeyEvent {
    pub fn new(key_code: KeyCode, event_type: EventType, modifiers: ModifierState) -> Self {
        Self {
            key_code,
            event_type,
            timestamp: Instant::now(),
            modifiers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_display_roundtrip() {
        let keys = [
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Num0,
            KeyCode::F12,
            KeyCode::LCtrl,
            KeyCode::Space,
            KeyCode::Numpad5,
            KeyCode::Unknown(999),
        ];

        for key in keys {
            let s = key.to_string();
            let parsed: KeyCode = s.parse().unwrap();
            assert_eq!(key, parsed);
        }
    }

    #[test]
    fn modifier_state_combo_prefix() {
        let mut mods = ModifierState::empty();
        assert_eq!(mods.combo_prefix(), "");

        mods.set_ctrl(true);
        assert_eq!(mods.combo_prefix(), "ctrl+");

        mods.set_shift(true);
        assert_eq!(mods.combo_prefix(), "ctrl+shift+");

        mods.set_alt(true);
        assert_eq!(mods.combo_prefix(), "alt+ctrl+shift+");
    }
}
