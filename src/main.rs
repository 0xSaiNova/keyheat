mod aggregator;
mod error;
mod listener;
mod storage;

use anyhow::{Context, Result};
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

    let counts = aggregator::new_shared_counts();
    let counts_for_listener = counts.clone();

    thread::spawn(move || {
        if let Err(e) = listener::run_capture(devices, counts_for_listener) {
            eprintln!("listener error: {e}");
        }
    });

    eprintln!("capturing keystrokes, ctrl+c to stop");

    loop {
        thread::sleep(Duration::from_secs(5));

        let snapshot = aggregator::take_counts(&counts);
        if snapshot.is_empty() {
            continue;
        }

        let today = today_str();
        if let Err(e) = storage.flush_counts(&snapshot, &today) {
            eprintln!("flush error: {e}");
        } else {
            eprintln!("flushed {} key types", snapshot.len());
        }
    }
}

fn today_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // days since epoch, then format as YYYY-MM-DD
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining = days % 365;
    let month = remaining / 30 + 1;
    let day = remaining % 30 + 1;

    format!("{years:04}-{month:02}-{day:02}")
}
