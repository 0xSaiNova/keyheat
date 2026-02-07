mod aggregator;
mod error;
mod listener;
mod storage;

use anyhow::{Context, Result};
use chrono::{Local, Utc};
use clap::{Parser, Subcommand};
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
    Run,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run => run_foreground(),
    }
}

fn run_foreground() -> Result<()> {
    let devices = listener::find_keyboards().context("failed to find keyboard devices")?;

    eprintln!(
        "found {} keyboard(s): {:?}",
        devices.len(),
        listener::device_names(&devices)
    );

    let mut storage = storage::Storage::open().context("failed to open database")?;
    eprintln!("database ready");

    let agg = aggregator::new_aggregator();
    let agg_for_listener = agg.clone();

    thread::spawn(move || {
        if let Err(e) = listener::run_capture(devices, agg_for_listener) {
            eprintln!("listener error: {e}");
        }
    });

    eprintln!("capturing keystrokes, ctrl+c to stop");

    loop {
        thread::sleep(Duration::from_secs(5));

        let mut guard = agg.lock().unwrap();

        if guard.needs_session_start() {
            let now = Utc::now();
            match storage.start_session(now) {
                Ok(session_id) => {
                    guard.start_session(session_id);
                    eprintln!("session {session_id} started");
                }
                Err(e) => eprintln!("failed to start session: {e}"),
            }
        }

        if let Some((session_id, keystroke_count)) = guard.check_idle() {
            let now = Utc::now();
            if let Err(e) = storage.end_session(session_id, now, keystroke_count) {
                eprintln!("failed to end session: {e}");
            } else {
                eprintln!("session {session_id} ended ({keystroke_count} keystrokes)");
            }
            guard.end_session();
        } else if let Some((session_id, keystroke_count)) = guard.current_session() {
            if let Err(e) = storage.update_session_keystrokes(session_id, keystroke_count) {
                eprintln!("failed to update session: {e}");
            }
        }

        let snapshot = guard.take_counts();
        drop(guard);

        if snapshot.is_empty() {
            continue;
        }

        let today = Local::now().format("%Y-%m-%d").to_string();
        if let Err(e) = storage.flush_counts(&snapshot, &today) {
            eprintln!("flush error: {e}");
        } else {
            eprintln!("flushed {} key types", snapshot.len());
        }
    }
}
