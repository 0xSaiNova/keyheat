use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("evdev error: {0}")]
    Evdev(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("no keyboard devices found")]
    NoKeyboards,

    #[cfg(target_os = "windows")]
    #[error("hook error: {0}")]
    Hook(String),
}
