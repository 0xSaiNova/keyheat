mod aggregator;
mod error;
mod keycode;

#[cfg(target_os = "linux")]
mod keymap_linux;
#[cfg(target_os = "linux")]
mod listener_linux;

mod listener_mock;
mod report;
mod storage;

use aggregator::Aggregator;
use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, Utc};
use clap::{Parser, Subcommand, ValueEnum};
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
    /// Generate a weekly report
    Report {
        /// Week to report on (YYYY-Www format, e.g. 2025-W06)
        #[arg(long)]
        week: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value = "terminal")]
        format: ReportFormat,
    },
}

#[derive(Clone, ValueEnum)]
enum ReportFormat {
    Terminal,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { mock } => run_foreground(mock),
        Commands::Report { week, format } => generate_report(week, format),
    }
}

fn generate_report(week: Option<String>, format: ReportFormat) -> Result<()> {
    let storage = storage::Storage::open().context("failed to open database")?;

    let week_date = match week {
        Some(w) => {
            Some(parse_iso_week(&w).context("invalid week format, use YYYY-Www (e.g. 2025-W06)")?)
        }
        None => None,
    };

    let report_data =
        report::build_report(storage.connection(), week_date).context("failed to build report")?;

    match format {
        ReportFormat::Terminal => print_terminal_report(&report_data),
        ReportFormat::Json => print_json_report(&report_data)?,
    }

    Ok(())
}

fn parse_iso_week(s: &str) -> Result<NaiveDate> {
    let parts: Vec<&str> = s.split("-W").collect();
    if parts.len() != 2 {
        anyhow::bail!("expected YYYY-Www format");
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid year"))?;
    let week: u32 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid week number"))?;

    NaiveDate::from_isoywd_opt(year, week, chrono::Weekday::Mon)
        .ok_or_else(|| anyhow::anyhow!("invalid ISO week"))
}

fn print_terminal_report(data: &report::ReportData) {
    println!();
    println!("KeyHeat \u{2014} {}", data.week.label);
    println!();

    let delta = data
        .prev_week_keystrokes
        .map(|prev| {
            if prev == 0 {
                String::new()
            } else {
                let pct = ((data.total_keystrokes as f64 - prev as f64) / prev as f64) * 100.0;
                if pct >= 0.0 {
                    format!("  (+{pct:.0}% from last week)")
                } else {
                    format!("  ({pct:.0}% from last week)")
                }
            }
        })
        .unwrap_or_default();

    println!(
        "  {:>6} keystrokes{}",
        format_number(data.total_keystrokes),
        delta
    );

    let hours = (data.total_typing_minutes / 60.0).floor() as u32;
    let mins = (data.total_typing_minutes % 60.0).round() as u32;
    println!(
        "  {:>6} sessions, {}h {}m total typing time",
        data.sessions.len(),
        hours,
        mins
    );
    println!();

    let peak_time = data
        .peak_wpm_time
        .map(|t| t.format("%a %l:%M %p").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!(
        "  Speed: {:.0} WPM avg, {:.0} WPM peak ({})",
        data.avg_wpm, data.peak_wpm, peak_time
    );

    let top_keys: Vec<String> = data
        .key_frequencies
        .iter()
        .take(3)
        .map(|(k, c)| format!("{} ({})", format_key_name(k), format_number(*c)))
        .collect();
    println!("  Top keys: {}", top_keys.join(", "));

    let top_shortcuts: Vec<String> = data
        .shortcuts
        .iter()
        .take(3)
        .map(|(k, c)| format!("{} ({})", format_shortcut_name(k), c))
        .collect();
    if !top_shortcuts.is_empty() {
        println!("  Top shortcuts: {}", top_shortcuts.join(", "));
    }

    println!();

    if let Some(insight) = &data.shortcut_insight {
        println!("  {}", insight.message);
        println!();
    }

    let reports_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyheat")
        .join("reports");

    println!(
        "  Full report: {}/week-{}.html",
        reports_dir.display(),
        data.week.start.format("%Y-W%W")
    );
    println!();
}

fn print_json_report(data: &report::ReportData) -> Result<()> {
    // simple manual JSON output to avoid serde dependency for now
    println!("{{");
    println!("  \"week\": {{");
    println!("    \"start\": \"{}\",", data.week.start);
    println!("    \"end\": \"{}\",", data.week.end);
    println!("    \"label\": \"{}\"", data.week.label);
    println!("  }},");
    println!("  \"total_keystrokes\": {},", data.total_keystrokes);
    println!("  \"all_time_keystrokes\": {},", data.all_time_keystrokes);
    println!("  \"avg_wpm\": {:.1},", data.avg_wpm);
    println!("  \"peak_wpm\": {:.1},", data.peak_wpm);
    println!(
        "  \"total_typing_minutes\": {:.1},",
        data.total_typing_minutes
    );
    println!("  \"session_count\": {},", data.sessions.len());
    println!("  \"backspace_ratio\": {:.3},", data.backspace_ratio);
    println!("  \"finger_travel_mm\": {:.1},", data.finger_travel_mm);
    println!("  \"night_owl_pct\": {:.1}", data.night_owl_pct);
    println!("}}");
    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

fn format_key_name(key: &str) -> String {
    match key {
        "space" => "Space".to_string(),
        "enter" => "Enter".to_string(),
        "backspace" => "Backspace".to_string(),
        "tab" => "Tab".to_string(),
        k if k.len() == 1 => k.to_uppercase(),
        k => k.to_string(),
    }
}

fn format_shortcut_name(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("+")
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
