# KeyHeat

Spotify Wrapped for your keyboard. Tracks typing speed, key frequency, and shortcuts, then generates a weekly report with a heatmap. Everything stays local.

![KeyHeat Weekly Report](docs/screenshot.png)

## Install

```bash
git clone https://github.com/0xSaiNova/keyheat.git
cd keyheat
cargo build --release
```

## Usage

```bash
# try it without root (fake keystrokes)
keyheat run --mock

# run for real on linux
sudo keyheat run

# generate report
keyheat report --format html
```

Reports save to `~/.local/share/keyheat/reports/`

## What the report shows

Keyboard heatmap, WPM stats, activity grid, top shortcuts, finger travel distance, backspace ratio.

Single HTML file with no dependencies. Just open it in a browser.

## Privacy

No network code. Grep the codebase, you wont find any HTTP clients or sockets. Data stays in a local SQLite file. You build it yourself from source.

## Roadmap

macOS and Windows support in progress. Daemon mode, config file, and TUI dashboard planned.

## License

MIT
