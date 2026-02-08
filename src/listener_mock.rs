use crate::keycode::{EventType, KeyCode, KeyEvent, ModifierState};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

struct MockRng {
    state: u64,
}

impl MockRng {
    fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEADBEEF);
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn range(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next() % (max - min + 1))
    }

    fn choice<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.next() as usize % items.len()]
    }
}

const TYPING_KEYS: &[KeyCode] = &[
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::G,
    KeyCode::H,
    KeyCode::J,
    KeyCode::K,
    KeyCode::L,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::T,
    KeyCode::Y,
    KeyCode::U,
    KeyCode::I,
    KeyCode::O,
    KeyCode::P,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
    KeyCode::B,
    KeyCode::N,
    KeyCode::M,
    KeyCode::Space,
    KeyCode::Enter,
    KeyCode::Backspace,
    KeyCode::Comma,
    KeyCode::Period,
];

const NAV_KEYS: &[KeyCode] = &[
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::PageUp,
    KeyCode::PageDown,
    KeyCode::Home,
    KeyCode::End,
];

struct ShortcutTemplate {
    modifiers: ModifierState,
    key: KeyCode,
}

fn get_shortcuts() -> Vec<ShortcutTemplate> {
    let mut shortcuts = Vec::new();

    let mut ctrl = ModifierState::empty();
    ctrl.set_ctrl(true);
    shortcuts.push(ShortcutTemplate {
        modifiers: ctrl,
        key: KeyCode::C,
    });
    shortcuts.push(ShortcutTemplate {
        modifiers: ctrl,
        key: KeyCode::V,
    });
    shortcuts.push(ShortcutTemplate {
        modifiers: ctrl,
        key: KeyCode::S,
    });
    shortcuts.push(ShortcutTemplate {
        modifiers: ctrl,
        key: KeyCode::Z,
    });

    let mut ctrl_shift = ModifierState::empty();
    ctrl_shift.set_ctrl(true);
    ctrl_shift.set_shift(true);
    shortcuts.push(ShortcutTemplate {
        modifiers: ctrl_shift,
        key: KeyCode::T,
    });

    let mut alt = ModifierState::empty();
    alt.set_alt(true);
    shortcuts.push(ShortcutTemplate {
        modifiers: alt,
        key: KeyCode::Tab,
    });

    shortcuts
}

#[derive(Clone, Copy)]
enum TypingSpeed {
    Fast,   // 50-80ms (~90-120 WPM)
    Normal, // 100-150ms (~60-80 WPM)
    Slow,   // 300-500ms (~20-30 WPM)
}

impl TypingSpeed {
    fn delay_ms(&self, rng: &mut MockRng) -> u64 {
        match self {
            TypingSpeed::Fast => rng.range(50, 80),
            TypingSpeed::Normal => rng.range(100, 150),
            TypingSpeed::Slow => rng.range(300, 500),
        }
    }
}

pub fn run_mock(sender: Sender<KeyEvent>) -> Result<(), crate::error::Error> {
    let mut rng = MockRng::new();
    let shortcuts = get_shortcuts();
    let speeds = [TypingSpeed::Fast, TypingSpeed::Normal, TypingSpeed::Slow];

    eprintln!("mock listener: generating synthetic key events");

    loop {
        // pick a speed for this burst (biased toward normal)
        let speed = match rng.range(0, 9) {
            0..=1 => TypingSpeed::Fast,
            2..=3 => TypingSpeed::Slow,
            _ => TypingSpeed::Normal,
        };

        // burst of 20-50 events
        let burst_size = rng.range(20, 50);
        let speed_name = match speed {
            TypingSpeed::Fast => "fast",
            TypingSpeed::Normal => "normal",
            TypingSpeed::Slow => "slow",
        };
        eprintln!("mock: starting {speed_name} burst of {burst_size} events");

        // occasionally ramp up or down mid-burst
        let mut current_speed = speed;
        let speed_change_at = if rng.range(0, 2) == 0 {
            Some(rng.range(burst_size / 3, burst_size * 2 / 3))
        } else {
            None
        };

        for i in 0..burst_size {
            // speed transition mid-burst
            if let Some(change_point) = speed_change_at {
                if i == change_point {
                    current_speed = rng.choice(&speeds);
                }
            }

            // decide what kind of event
            let event = match rng.range(0, 19) {
                0 => {
                    // 5% shortcut
                    let shortcut = &shortcuts[rng.next() as usize % shortcuts.len()];
                    KeyEvent::new(shortcut.key, EventType::KeyDown, shortcut.modifiers)
                }
                1 => {
                    // 5% navigation (should not affect WPM)
                    let nav = rng.choice(NAV_KEYS);
                    KeyEvent::new(nav, EventType::KeyDown, ModifierState::empty())
                }
                _ => {
                    // 90% regular typing
                    let key = rng.choice(TYPING_KEYS);
                    KeyEvent::new(key, EventType::KeyDown, ModifierState::empty())
                }
            };

            if sender.send(event).is_err() {
                return Ok(());
            }

            let delay = current_speed.delay_ms(&mut rng);
            thread::sleep(Duration::from_millis(delay));
        }

        // pause between bursts
        let pause = if rng.range(0, 4) == 0 {
            // 20% long pause triggers session boundary
            rng.range(35, 45)
        } else {
            rng.range(1, 10)
        };

        eprintln!("mock: pausing for {pause}s");
        thread::sleep(Duration::from_secs(pause));
    }
}
