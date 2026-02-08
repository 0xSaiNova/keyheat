mod aggregator;
mod error;
mod keycode;

#[cfg(target_os = "linux")]
mod keymap_linux;
#[cfg(target_os = "linux")]
mod listener_linux;

mod listener_mock;
mod storage;

use aggregator::Aggregator;
use anyhow::{Context, Result};
use chrono::{Local, Utc};
use clap::{Parser, Subcommand};
use keycode::KeyEvent;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "keyheat")]
#[command(about = "Keyboard analytics daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the capture loop in the foreground
    Run {
        /// Use mock listener for testing (generates synthetic events)
        #[arg(long)]
        mock: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { mock } => run_foreground(mock),
    }
}

fn run_foreground(use_mock: bool) -> Result<()> {
    let mut storage = storage::Storage::open().context("failed to open database")?;
    eprintln!("database ready");

    let (sender, receiver) = mpsc::channel::<KeyEvent>();

    if use_mock {
        eprintln!("starting mock listener");
        thread::spawn(move || {
            if let Err(e) = listener_mock::run_mock(sender) {
                eprintln!("mock listener error: {e}");
            }
        });
    } else {
        #[cfg(target_os = "linux")]
        {
            let devices =
                listener_linux::find_keyboards().context("failed to find keyboard devices")?;

            eprintln!(
                "found {} keyboard(s): {:?}",
                devices.len(),
                listener_linux::device_names(&devices)
            );

            thread::spawn(move || {
                if let Err(e) = listener_linux::run_capture(devices, sender) {
                    eprintln!("listener error: {e}");
                }
            });
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!(
                "native keyboard capture not yet supported on this platform. \
                 Use --mock for testing or wait for platform support."
            );
        }
    }

    eprintln!("capturing keystrokes, ctrl+c to stop");

    let mut aggregator = Aggregator::new();

    loop {
        // drain all available events (non-blocking after timeout)
        let deadline = std::time::Instant::now() + Duration::from_secs(5);

        loop {
            let timeout = deadline.saturating_duration_since(std::time::Instant::now());
            match receiver.recv_timeout(timeout) {
                Ok(event) => {
                    aggregator.process_event(event);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("listener disconnected, exiting");
                    return Ok(());
                }
            }
        }

        // session management
        if aggregator.needs_session_start() {
            let now = Utc::now();
            match storage.start_session(now) {
                Ok(session_id) => {
                    aggregator.start_session(session_id);
                    eprintln!("session {session_id} started");
                }
                Err(e) => eprintln!("failed to start session: {e}"),
            }
        }

        if let Some((session_id, keystroke_count)) = aggregator.check_idle() {
            let now = Utc::now();
            let wpm_stats = aggregator.end_session();
            let (avg_wpm, peak_wpm) = wpm_stats.unwrap_or((0.0, 0.0));

            if let Err(e) = storage.end_session(
                session_id,
                now,
                keystroke_count,
                Some(avg_wpm),
                Some(peak_wpm),
            ) {
                eprintln!("failed to end session: {e}");
            } else {
                eprintln!(
                    "session {session_id} ended ({keystroke_count} keystrokes, avg {avg_wpm:.1} WPM, peak {peak_wpm:.1} WPM)"
                );
            }
        } else if let Some((session_id, keystroke_count)) = aggregator.current_session() {
            if let Err(e) = storage.update_session_keystrokes(session_id, keystroke_count) {
                eprintln!("failed to update session: {e}");
            }
        }

        // flush counts
        let key_snapshot = aggregator.take_counts();
        let shortcut_snapshot = aggregator.take_shortcuts();
        let wpm_samples = aggregator.take_wpm_samples();

        let today = Local::now().format("%Y-%m-%d").to_string();

        if !key_snapshot.is_empty() {
            if let Err(e) = storage.flush_counts(&key_snapshot, &today) {
                eprintln!("flush error: {e}");
            } else {
                let wpm = aggregator.current_wpm();
                if wpm > 0.0 {
                    eprintln!(
                        "flushed {} key types (current: {wpm:.0} WPM)",
                        key_snapshot.len()
                    );
                } else {
                    eprintln!("flushed {} key types", key_snapshot.len());
                }
            }
        }

        if !shortcut_snapshot.is_empty() {
            if let Err(e) = storage.flush_shortcuts(&shortcut_snapshot, &today) {
                eprintln!("shortcut flush error: {e}");
            } else {
                eprintln!("flushed {} shortcuts", shortcut_snapshot.len());
            }
        }

        if !wpm_samples.is_empty() {
            if let Err(e) = storage.flush_wpm_samples(&wpm_samples) {
                eprintln!("wpm sample flush error: {e}");
            } else {
                eprintln!("flushed {} WPM samples", wpm_samples.len());
            }
        }
    }
}
