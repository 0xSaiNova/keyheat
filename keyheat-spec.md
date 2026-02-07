# KeyHeat — Keyboard Analytics Daemon

> A background terminal daemon that silently tracks your keystrokes, builds heatmaps, measures typing speed, detects shortcut usage, and delivers weekly reports right to your terminal.

---

## Overview

KeyHeat runs as a lightweight background process that captures keyboard input events at the OS level, stores anonymized keystroke metadata locally, and generates rich weekly reports covering typing speed trends, key frequency heatmaps, shortcut usage, and behavioral patterns. Everything stays local — no network, no telemetry, no cloud.

---

## Core Principles

- **Privacy-first**: Raw keystrokes are never stored. Only aggregated metadata (key codes, timestamps, modifier combos) is persisted. No keylogging of actual text content.
- **Zero friction**: Start with one command, forget about it. It just works in the background.
- **Offline & local**: All data lives in a local SQLite database. Nothing leaves your machine.
- **Minimal resource usage**: Target < 5MB RAM, negligible CPU. It should be invisible.

---

## Architecture

### High-Level Components

```
┌──────────────────────────────────────────────────────┐
│                    User's Machine                     │
│                                                       │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────┐ │
│  │  Input       │───▶│  Aggregation │───▶│  SQLite  │ │
│  │  Listener    │    │  Engine      │    │  Store   │ │
│  └─────────────┘    └──────────────┘    └────┬─────┘ │
│                                              │       │
│                                         ┌────▼─────┐ │
│  ┌─────────────┐    ┌──────────────┐    │  Report  │ │
│  │  Terminal    │◀───│  Renderer    │◀───│  Builder │ │
│  │  Output      │    │  (TUI/HTML)  │    │          │ │
│  └─────────────┘    └──────────────┘    └──────────┘ │
└──────────────────────────────────────────────────────┘
```

### 1. Input Listener (Daemon)

The core background process that captures low-level keyboard events.

**Responsibilities:**
- Capture key-down and key-up events at the OS input level
- Track modifier key state (Shift, Ctrl, Alt, Super/Cmd)
- Detect multi-key combos as single "shortcut" events
- Measure inter-key timing for WPM calculations
- Run as a daemonized process (detached from terminal session)

**Platform strategy:**
- **Linux**: Read from `/dev/input/event*` devices or use `libinput`. Requires the user to be in the `input` group or run with appropriate permissions.
- **macOS**: Use the IOKit HID framework or CGEventTap APIs. Requires Accessibility permissions.
- **Windows** (stretch): Low-level keyboard hooks via the Win32 API.

**Key design decisions:**
- The listener emits "event batches" to the aggregation engine every N seconds (e.g., 5s) rather than per-keystroke, to reduce overhead.
- Raw characters are **never** reconstructed into words or sentences. The listener only records key codes (e.g., `KEY_A`, `KEY_ENTER`) and timing deltas.

### 2. Aggregation Engine

Sits between the raw event stream and persistent storage. Responsible for turning raw events into meaningful metrics before they hit the database.

**Responsibilities:**
- Compute rolling WPM from inter-key intervals (using a sliding window, e.g., 30-second buckets)
- Classify events into categories: alphanumeric, navigation, modifier combos, function keys, etc.
- Detect and log shortcut sequences (e.g., `Ctrl+C`, `Ctrl+Shift+P`, `Cmd+K Cmd+S`)
- Identify "typing sessions" — bursts of activity separated by idle gaps (configurable idle threshold, default 30s)
- Compute per-session and per-bucket aggregates before writing to the DB

**Session detection logic:**
- A session starts when a key event arrives after an idle period exceeding the threshold.
- A session ends when no key event arrives for longer than the threshold.
- Sessions are the fundamental unit for WPM and pattern analysis.

### 3. SQLite Store

Local persistence layer. Single-file database, zero config.

**Schema concept — core tables:**

- **`sessions`**: One row per detected typing session. Stores start time, end time, total keystrokes, average WPM, peak WPM.
- **`key_counts`**: Per-day, per-key aggregate counts. Columns for key code, date, press count, and average hold duration.
- **`shortcut_counts`**: Per-day counts for detected modifier combos. Columns for combo string (e.g., `ctrl+shift+t`), date, count.
- **`wpm_samples`**: Time-series of WPM measurements at bucket granularity (e.g., every 30s during active sessions). Used for trend graphs.
- **`daily_summaries`**: Pre-computed daily rollups — total keys, total sessions, daily avg WPM, top 10 keys, top 10 shortcuts. Makes weekly report generation fast.

**Data retention policy:**
- Granular `wpm_samples` are kept for 90 days, then pruned.
- `key_counts` and `shortcut_counts` are kept for 1 year.
- `daily_summaries` are kept indefinitely (tiny rows).
- User-configurable via a config file.

### 4. Report Builder

Runs on a schedule (weekly cron or internal timer) to generate the actual report from stored data.

**Report contents:**

| Section | Description |
|---|---|
| **Heatmap** | Visual keyboard layout with color intensity per key based on press frequency. Standard QWERTY layout, with support for custom layouts (Dvorak, Colemak, etc.). |
| **WPM Trends** | Line chart of average WPM per day over the past week. Overlay of peak WPM per day. Comparison to previous week. |
| **Session Summary** | Total typing time, number of sessions, longest session, most active hour of day, most active day of week. |
| **Shortcut Leaderboard** | Top 20 shortcuts used, with counts and deltas from prior week. Categorized by type (editor, OS, browser, custom). |
| **Typing Patterns** | Heatmap of activity by hour-of-day × day-of-week. Identifies your "peak hours." |
| **Key Pair Analysis** | Most common bigrams (key pairs typed in sequence). Highlights awkward reaches or finger travel inefficiencies. |
| **Fun Stats** | Total keys pressed (odometer-style running total), estimated finger travel distance, "fastest burst" WPM, most-hammered key of the week. |

### 5. Renderer

Takes the report builder's output and presents it.

**Output formats:**

- **Terminal (TUI)**: Default. Uses a library for rich terminal output (box drawing, color, sparklines). Designed for `keyheat report` to look great in any modern terminal.
- **HTML**: Generate a self-contained `.html` file with embedded CSS and inline SVGs for the heatmap and charts. User can open it in a browser or archive it. Great for sharing.
- **JSON**: Machine-readable dump of all report data for users who want to pipe it into other tools or build their own dashboards.

---

## CLI Interface

The entire tool is driven by a single CLI binary.

| Command | Description |
|---|---|
| `keyheat start` | Start the daemon in the background. Detaches from the terminal. Writes a PID file. |
| `keyheat stop` | Gracefully stop the daemon. |
| `keyheat status` | Show if the daemon is running, uptime, keys captured today, current session WPM. |
| `keyheat report` | Generate and display the weekly report in the terminal. |
| `keyheat report --format html` | Generate and open an HTML report. |
| `keyheat report --week 2025-W05` | Generate a report for a specific past week. |
| `keyheat live` | Open a real-time TUI dashboard showing live WPM, key frequency, and session timer. |
| `keyheat config` | Open or print the config file location and current settings. |
| `keyheat export` | Dump all data as JSON or CSV. |
| `keyheat reset` | Wipe the database (with confirmation). |

---

## Configuration

Stored in `~/.config/keyheat/config.toml`.

**Key settings:**
- `idle_threshold_seconds` — Gap before a session is considered ended (default: 30)
- `wpm_bucket_seconds` — Granularity of WPM sampling (default: 30)
- `keyboard_layout` — Layout for heatmap rendering (default: `qwerty-us`)
- `report_schedule` — Cron expression for auto-generating reports (default: `0 9 * * 1` — Monday 9 AM)
- `report_format` — Default output format: `terminal`, `html`, or `json`
- `data_dir` — Where to store the SQLite DB (default: `~/.local/share/keyheat/`)
- `retention_days_granular` — How long to keep per-bucket WPM data (default: 90)
- `ignored_keys` — Keys to exclude from tracking entirely (e.g., if you don't care about modifier-only presses)

---

## Privacy & Security Model

This is a sensitive tool. The architecture must make it **impossible to reconstruct typed text** from stored data.

- The listener records **key codes only**, not characters. Modifier state is tracked for shortcut detection, then discarded as standalone events.
- No sequential key log is ever stored. Only aggregated counts and timing metrics are persisted.
- The SQLite database should live in a user-only-readable directory (`chmod 700`).
- The daemon itself should drop any elevated permissions immediately after acquiring the input device handle.
- On macOS, the binary will need to be registered in System Preferences → Privacy → Accessibility. This should be documented clearly.

---

## Tech Stack Recommendations

| Concern | Recommendation | Rationale |
|---|---|---|
| Language | **Rust** or **Go** | Low overhead, single binary distribution, good OS-level input access. Rust preferred for safety guarantees in a security-sensitive tool. |
| Input capture (Linux) | `evdev` crate / `libevdev` | Direct, low-level, well-supported. |
| Input capture (macOS) | `core-graphics` crate / CGEventTap | Standard macOS approach. |
| Database | SQLite via `rusqlite` / `go-sqlite3` | Zero-config, embedded, fast enough for this workload. |
| TUI rendering | `ratatui` (Rust) / `bubbletea` (Go) | Modern, well-maintained terminal UI libraries. |
| HTML reports | Templating engine + inline SVG generation | Keep reports as single self-contained files. No external dependencies. |
| Daemon management | `systemd` unit file (Linux) / `launchd` plist (macOS) | Proper OS-level daemon lifecycle. Fallback: self-daemonize with PID file. |

---

## Future / Stretch Ideas

- **Per-application tracking**: Detect which application is in focus and break down stats by app (e.g., "you type fastest in Vim, slowest in Slack").
- **Ergonomic scoring**: Analyze key pair sequences against known ergonomic models and suggest layout optimizations.
- **Typing challenges**: Built-in typing test mode that uses your own historical patterns to generate personalized practice drills.
- **Multi-machine sync**: Optional encrypted sync between machines via a shared folder or simple peer protocol.
- **Plugin system**: Let users write custom analyzers that hook into the event stream or report builder.
