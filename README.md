# KeyHeat

Lightweight daemon that tracks keyboard usage and generates visual reports. WPM tracking, key frequency heatmaps, and shortcut analytics. All data stays local.

![KeyHeat Weekly Report](docs/screenshot.png)

## Install

```bash
git clone https://github.com/0xSaiNova/keyheat.git
cd keyheat
cargo build --release
```

**Linux:** Add user to `input` group: `sudo usermod -a -G input $USER` (then log out/in)
**Windows:** No setup needed (cannot capture elevated windows)
**macOS:** Not yet supported

## Usage

```bash
keyheat start              # Start daemon
keyheat status             # Check status
keyheat report             # Generate report
keyheat stop               # Stop daemon
```

Reports open in your browser. Customize via `~/.config/keyheat/config.toml`

## What It Tracks

- Real-time and historical WPM
- Key frequency heatmaps
- Keyboard shortcuts
- Session patterns and activity

## Privacy

No network code. Only key codes tracked, never actual text. Data stored locally in SQLite.

**Linux/macOS:** `~/.local/share/keyheat/keyheat.db`
**Windows:** `%LOCALAPPDATA%\keyheat\keyheat.db`

## v1.0 Highlights

Production-ready with atomic database transactions, Windows stability fixes, multi-keyboard support, and configuration file support. See [releases](https://github.com/0xSaiNova/keyheat/releases) for full changelog.

## License

MIT
