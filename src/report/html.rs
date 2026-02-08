#![allow(clippy::uninlined_format_args)]

use super::layout::qwerty_ansi;
use super::ReportData;
use std::collections::HashMap;

const BG_COLOR: &str = "#0a0a0f";
const SURFACE_COLOR: &str = "#12121a";
const BORDER_COLOR: &str = "#1e1e2e";
const TEXT_PRIMARY: &str = "#e0e0e8";
const TEXT_SECONDARY: &str = "#6b6b80";
const ACCENT: &str = "#6366f1";
const ACCENT_LIGHT: &str = "#818cf8";
const HOT: &str = "#f472b6";
const COLD: &str = "#1e1b4b";

pub fn render(data: &ReportData) -> String {
    let mut html = String::with_capacity(64 * 1024);

    html.push_str(&render_head(&data.week.label));
    html.push_str("<body>\n");
    html.push_str("<div class=\"container\">\n");

    html.push_str(&render_hero(data));
    html.push_str(&render_heatmap(data));
    html.push_str(&render_speed_story(data));
    html.push_str(&render_session_timeline(data));
    html.push_str(&render_shortcuts(data));
    html.push_str(&render_fun_stats(data));
    html.push_str(&render_footer(data));

    html.push_str("</div>\n");
    html.push_str(&render_scripts());
    html.push_str("</body>\n</html>");

    html
}

fn render_head(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>KeyHeat Report - {title}</title>
<style>
{css}
</style>
</head>
"#,
        title = title,
        css = render_css()
    )
}

fn render_css() -> String {
    format!(
        r#"
* {{ margin: 0; padding: 0; box-sizing: border-box; }}

body {{
    background: {bg};
    color: {text_primary};
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    min-height: 100vh;
}}

.container {{
    max-width: 720px;
    margin: 0 auto;
    padding: 2rem 1rem;
}}

section {{
    margin-bottom: 3rem;
    opacity: 0;
    transform: translateY(20px);
    animation: fadeIn 0.5s ease forwards;
}}

section:nth-child(1) {{ animation-delay: 0s; }}
section:nth-child(2) {{ animation-delay: 0.1s; }}
section:nth-child(3) {{ animation-delay: 0.2s; }}
section:nth-child(4) {{ animation-delay: 0.3s; }}
section:nth-child(5) {{ animation-delay: 0.4s; }}
section:nth-child(6) {{ animation-delay: 0.5s; }}

@keyframes fadeIn {{
    to {{ opacity: 1; transform: translateY(0); }}
}}

.hero {{
    text-align: center;
    padding: 2rem 0;
}}

.hero h1 {{
    font-size: 1.2rem;
    color: {text_secondary};
    font-weight: 400;
    margin-bottom: 1rem;
}}

.hero-number {{
    font-size: 4rem;
    font-weight: 700;
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    color: {text_primary};
    letter-spacing: -0.02em;
}}

.hero-label {{
    color: {text_secondary};
    font-size: 1rem;
    margin-top: 0.5rem;
}}

.hero-delta {{
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
    font-size: 0.875rem;
    margin-top: 1rem;
}}

.hero-delta.positive {{
    background: rgba(99, 102, 241, 0.2);
    color: {accent_light};
}}

.hero-delta.negative {{
    background: rgba(244, 114, 182, 0.2);
    color: {hot};
}}

.section-title {{
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 1.5rem;
    color: {text_primary};
}}

.card {{
    background: {surface};
    border: 1px solid {border};
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1rem;
}}

.heatmap-container {{
    overflow-x: auto;
    padding: 1rem 0;
}}

.heatmap-svg {{
    display: block;
    margin: 0 auto;
}}

.stats-grid {{
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
}}

@media (max-width: 500px) {{
    .stats-grid {{
        grid-template-columns: 1fr;
    }}
}}

.stat-card {{
    background: {surface};
    border: 1px solid {border};
    border-radius: 12px;
    padding: 1.25rem;
}}

.stat-value {{
    font-size: 2rem;
    font-weight: 700;
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    color: {text_primary};
}}

.stat-label {{
    color: {text_secondary};
    font-size: 0.875rem;
    margin-top: 0.25rem;
}}

.stat-sublabel {{
    color: {text_secondary};
    font-size: 0.75rem;
    margin-top: 0.5rem;
}}

.activity-grid {{
    display: block;
    margin: 0 auto;
}}

.shortcut-list {{
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}}

.shortcut-item {{
    display: flex;
    align-items: center;
    gap: 1rem;
}}

.shortcut-name {{
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    font-size: 0.875rem;
    min-width: 100px;
    color: {text_primary};
}}

.shortcut-bar {{
    flex: 1;
    height: 24px;
    background: {border};
    border-radius: 4px;
    overflow: hidden;
}}

.shortcut-bar-fill {{
    height: 100%;
    background: linear-gradient(90deg, {accent}, {accent_light});
    border-radius: 4px;
    transition: width 0.5s ease;
}}

.shortcut-count {{
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    font-size: 0.875rem;
    color: {text_secondary};
    min-width: 50px;
    text-align: right;
}}

.insight-card {{
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(244, 114, 182, 0.1));
    border: 1px solid {accent};
    border-radius: 12px;
    padding: 1.25rem;
    margin-top: 1rem;
}}

.insight-text {{
    color: {text_primary};
    font-size: 1rem;
}}

.fun-cards {{
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
}}

@media (max-width: 500px) {{
    .fun-cards {{
        grid-template-columns: 1fr;
    }}
}}

.fun-card {{
    background: {surface};
    border: 1px solid {border};
    border-radius: 12px;
    padding: 1.25rem;
    text-align: center;
}}

.fun-card-value {{
    font-size: 1.75rem;
    font-weight: 700;
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    color: {accent_light};
}}

.fun-card-label {{
    color: {text_secondary};
    font-size: 0.875rem;
    margin-top: 0.25rem;
}}

.fun-card-note {{
    color: {text_secondary};
    font-size: 0.75rem;
    margin-top: 0.5rem;
    font-style: italic;
}}

footer {{
    text-align: center;
    padding: 2rem 0;
    color: {text_secondary};
    font-size: 0.875rem;
}}

footer a {{
    color: {accent};
    text-decoration: none;
}}

footer a:hover {{
    text-decoration: underline;
}}

.tooltip {{
    position: absolute;
    background: {surface};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    font-size: 0.75rem;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.2s;
    z-index: 100;
}}

.key-rect {{
    cursor: pointer;
    transition: filter 0.2s;
}}

.key-rect:hover {{
    filter: brightness(1.2);
}}
"#,
        bg = BG_COLOR,
        surface = SURFACE_COLOR,
        border = BORDER_COLOR,
        text_primary = TEXT_PRIMARY,
        text_secondary = TEXT_SECONDARY,
        accent = ACCENT,
        accent_light = ACCENT_LIGHT,
        hot = HOT,
    )
}

fn render_hero(data: &ReportData) -> String {
    let delta_html = if let Some(prev) = data.prev_week_keystrokes {
        if prev > 0 {
            let pct = ((data.total_keystrokes as f64 - prev as f64) / prev as f64) * 100.0;
            let class = if pct >= 0.0 { "positive" } else { "negative" };
            let sign = if pct >= 0.0 { "+" } else { "" };
            format!(r#"<div class="hero-delta {class}">{sign}{pct:.0}% from last week</div>"#)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let hours = (data.total_typing_minutes / 60.0).floor() as u32;
    let mins = (data.total_typing_minutes % 60.0).round() as u32;

    format!(
        r#"<section class="hero">
<h1>{week_label}</h1>
<div class="hero-number" data-count="{keystrokes}">{keystrokes_fmt}</div>
<div class="hero-label">keystrokes across {sessions} sessions</div>
<div class="hero-label">{hours}h {mins}m total typing time</div>
{delta_html}
</section>
"#,
        week_label = data.week.label,
        keystrokes = data.total_keystrokes,
        keystrokes_fmt = format_number(data.total_keystrokes),
        sessions = data.sessions.len(),
        hours = hours,
        mins = mins,
        delta_html = delta_html,
    )
}

fn render_heatmap(data: &ReportData) -> String {
    let key_counts: HashMap<&str, u64> = data
        .key_frequencies
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();

    let max_count = key_counts.values().copied().max().unwrap_or(1) as f64;
    let layout = qwerty_ansi();

    let key_unit = 44.0;
    let padding = 4.0;
    let svg_width = 15.0 * key_unit + padding * 2.0;
    let svg_height = 5.0 * key_unit + padding * 2.0;

    let mut keys_svg = String::new();

    for key in &layout {
        let count = key_counts.get(key.key_code).copied().unwrap_or(0);
        let intensity = if max_count > 0.0 {
            (count as f64 / max_count).powf(0.5)
        } else {
            0.0
        };

        let color = interpolate_color(COLD, ACCENT, HOT, intensity);

        let x = padding + key.x * key_unit;
        let y = padding + key.y * key_unit;
        let w = key.width * key_unit - 4.0;
        let h = key_unit - 4.0;

        keys_svg.push_str(&format!(
            r#"<rect class="key-rect" x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{color}" data-key="{key_code}" data-count="{count}"/>
<text x="{tx}" y="{ty}" fill="{text_color}" font-size="11" text-anchor="middle" dominant-baseline="middle" pointer-events="none">{label}</text>
"#,
            x = x,
            y = y,
            w = w,
            h = h,
            color = color,
            key_code = key.key_code,
            count = count,
            tx = x + w / 2.0,
            ty = y + h / 2.0,
            text_color = if intensity > 0.5 { TEXT_PRIMARY } else { TEXT_SECONDARY },
            label = key.label,
        ));
    }

    let top_keys: Vec<String> = data
        .key_frequencies
        .iter()
        .take(3)
        .map(|(k, c)| {
            format!(
                "<strong>{}</strong>: {}",
                format_key_display(k),
                format_number(*c)
            )
        })
        .collect();

    format!(
        r#"<section>
<h2 class="section-title">Keyboard Heatmap</h2>
<div class="card">
<div class="heatmap-container">
<svg class="heatmap-svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {svg_width} {svg_height}">
{keys_svg}
</svg>
</div>
<p style="text-align: center; color: {text_secondary}; margin-top: 1rem; font-size: 0.875rem;">
Top keys: {top_keys}
</p>
</div>
<div id="tooltip" class="tooltip"></div>
</section>
"#,
        svg_width = svg_width,
        svg_height = svg_height,
        keys_svg = keys_svg,
        text_secondary = TEXT_SECONDARY,
        top_keys = top_keys.join(" | "),
    )
}

fn render_speed_story(data: &ReportData) -> String {
    let peak_time = data
        .peak_wpm_time
        .map(|t| t.format("%a %l:%M %p").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let trend_svg = render_wpm_trend(&data.daily_wpm);

    let distribution_svg = render_wpm_distribution(&data.wpm_distribution);

    format!(
        r#"<section>
<h2 class="section-title">Speed Story</h2>
<div class="stats-grid">
<div class="stat-card">
<div class="stat-value">{avg_wpm:.0}</div>
<div class="stat-label">Average WPM</div>
</div>
<div class="stat-card">
<div class="stat-value">{peak_wpm:.0}</div>
<div class="stat-label">Peak WPM</div>
<div class="stat-sublabel">{peak_time}</div>
</div>
</div>
<div class="card" style="margin-top: 1rem;">
<p style="color: {text_secondary}; font-size: 0.875rem; margin-bottom: 1rem;">Daily WPM Trend</p>
{trend_svg}
</div>
<div class="card">
<p style="color: {text_secondary}; font-size: 0.875rem; margin-bottom: 1rem;">Speed Distribution</p>
{distribution_svg}
</div>
</section>
"#,
        avg_wpm = data.avg_wpm,
        peak_wpm = data.peak_wpm,
        peak_time = peak_time,
        text_secondary = TEXT_SECONDARY,
        trend_svg = trend_svg,
        distribution_svg = distribution_svg,
    )
}

fn render_wpm_trend(daily_wpm: &[(chrono::NaiveDate, f64)]) -> String {
    let width = 660.0;
    let height = 120.0;
    let padding = 30.0;

    let max_wpm = daily_wpm
        .iter()
        .map(|(_, w)| *w)
        .fold(0.0f64, |a, b| a.max(b))
        .max(1.0);

    let points: Vec<String> = daily_wpm
        .iter()
        .enumerate()
        .map(|(i, (_, wpm))| {
            let x = padding + (i as f64 / 6.0) * (width - padding * 2.0);
            let y = height - padding - (wpm / max_wpm) * (height - padding * 2.0);
            format!("{x},{y}")
        })
        .collect();

    let day_labels: Vec<String> = daily_wpm
        .iter()
        .enumerate()
        .map(|(i, (date, _))| {
            let x = padding + (i as f64 / 6.0) * (width - padding * 2.0);
            let label = date.format("%a").to_string();
            format!(
                r#"<text x="{x}" y="{y}" fill="{color}" font-size="10" text-anchor="middle">{label}</text>"#,
                x = x,
                y = height - 8.0,
                color = TEXT_SECONDARY,
                label = label,
            )
        })
        .collect();

    format!(
        r#"<svg width="100%" height="{height}" viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMid meet">
<polyline fill="none" stroke="{accent}" stroke-width="2" points="{points}"/>
{day_labels}
</svg>"#,
        height = height,
        width = width,
        accent = ACCENT,
        points = points.join(" "),
        day_labels = day_labels.join("\n"),
    )
}

fn render_wpm_distribution(distribution: &[u32]) -> String {
    let width = 660.0;
    let height = 80.0;
    let bar_width = 80.0;
    let gap = 10.0;
    let labels = [
        "0-20", "20-40", "40-60", "60-80", "80-100", "100-120", "120+",
    ];

    let max_count = distribution.iter().copied().max().unwrap_or(1).max(1);

    let bars: Vec<String> = distribution
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let x = 20.0 + i as f64 * (bar_width + gap);
            let bar_height = (count as f64 / max_count as f64) * 50.0;
            let y = 50.0 - bar_height;
            format!(
                r#"<rect x="{x}" y="{y}" width="{bar_width}" height="{bar_height}" fill="{accent}" rx="3"/>
<text x="{tx}" y="70" fill="{text_color}" font-size="9" text-anchor="middle">{label}</text>"#,
                x = x,
                y = y,
                bar_width = bar_width,
                bar_height = bar_height,
                accent = ACCENT,
                tx = x + bar_width / 2.0,
                text_color = TEXT_SECONDARY,
                label = labels[i],
            )
        })
        .collect();

    format!(
        r#"<svg width="100%" height="{height}" viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMid meet">
{bars}
</svg>"#,
        height = height,
        width = width,
        bars = bars.join("\n"),
    )
}

fn render_session_timeline(data: &ReportData) -> String {
    let grid_svg = render_activity_grid(&data.hourly_activity);

    let busiest_hour = data.peak_hour.map(|h| {
        let period = if h < 12 { "AM" } else { "PM" };
        let hour_12 = if h == 0 {
            12
        } else if h > 12 {
            h - 12
        } else {
            h
        };
        format!("{}:00 {}", hour_12, period)
    });

    let longest = data.longest_session.as_ref().map(|s| {
        let hours = (s.duration_minutes / 60.0).floor() as u32;
        let mins = (s.duration_minutes % 60.0).round() as u32;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    });

    format!(
        r#"<section>
<h2 class="section-title">Session Timeline</h2>
<div class="card">
<p style="color: {text_secondary}; font-size: 0.875rem; margin-bottom: 1rem;">Activity by hour</p>
{grid_svg}
</div>
<div class="stats-grid" style="margin-top: 1rem;">
<div class="stat-card">
<div class="stat-value">{busiest}</div>
<div class="stat-label">Peak Hour</div>
</div>
<div class="stat-card">
<div class="stat-value">{longest}</div>
<div class="stat-label">Longest Session</div>
</div>
</div>
</section>
"#,
        text_secondary = TEXT_SECONDARY,
        grid_svg = grid_svg,
        busiest = busiest_hour.unwrap_or_else(|| "-".to_string()),
        longest = longest.unwrap_or_else(|| "-".to_string()),
    )
}

fn render_activity_grid(grid: &[[u64; 24]; 7]) -> String {
    let cell_size = 24.0;
    let gap = 3.0;
    let label_width = 40.0;
    let width = label_width + 24.0 * (cell_size + gap);
    let height = 7.0 * (cell_size + gap) + 20.0;

    let max_val = grid
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let mut cells = String::new();

    for (day_idx, row) in grid.iter().enumerate() {
        let y = day_idx as f64 * (cell_size + gap);

        cells.push_str(&format!(
            r#"<text x="0" y="{ty}" fill="{color}" font-size="10" dominant-baseline="middle">{label}</text>"#,
            ty = y + cell_size / 2.0,
            color = TEXT_SECONDARY,
            label = days[day_idx],
        ));

        for (hour_idx, &count) in row.iter().enumerate() {
            let x = label_width + hour_idx as f64 * (cell_size + gap);
            let intensity = (count as f64 / max_val).powf(0.5);
            let color = interpolate_color(BORDER_COLOR, ACCENT, ACCENT_LIGHT, intensity);

            cells.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{size}" height="{size}" rx="4" fill="{color}"/>"#,
                x = x,
                y = y,
                size = cell_size,
                color = color,
            ));
        }
    }

    // hour labels
    for hour in (0..24).step_by(6) {
        let x = label_width + hour as f64 * (cell_size + gap) + cell_size / 2.0;
        cells.push_str(&format!(
            r#"<text x="{x}" y="{y}" fill="{color}" font-size="9" text-anchor="middle">{hour}</text>"#,
            x = x,
            y = height - 5.0,
            color = TEXT_SECONDARY,
            hour = hour,
        ));
    }

    format!(
        r#"<svg class="activity-grid" width="100%" height="{height}" viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMid meet">
{cells}
</svg>"#,
        height = height,
        width = width,
        cells = cells,
    )
}

fn render_shortcuts(data: &ReportData) -> String {
    if data.shortcuts.is_empty() {
        return String::new();
    }

    let max_count = data.shortcuts.first().map(|(_, c)| *c).unwrap_or(1) as f64;

    let items: Vec<String> = data
        .shortcuts
        .iter()
        .take(10)
        .map(|(combo, count)| {
            let pct = (*count as f64 / max_count) * 100.0;
            format!(
                r#"<div class="shortcut-item">
<span class="shortcut-name">{combo}</span>
<div class="shortcut-bar"><div class="shortcut-bar-fill" style="width: {pct}%"></div></div>
<span class="shortcut-count">{count}</span>
</div>"#,
                combo = format_shortcut_display(combo),
                pct = pct,
                count = count,
            )
        })
        .collect();

    let insight_html = data
        .shortcut_insight
        .as_ref()
        .map(|i| {
            format!(
                r#"<div class="insight-card">
<p class="insight-text">{}</p>
</div>"#,
                i.message
            )
        })
        .unwrap_or_default();

    format!(
        r#"<section>
<h2 class="section-title">Shortcut Game</h2>
<div class="card">
<div class="shortcut-list">
{items}
</div>
{insight_html}
</div>
</section>
"#,
        items = items.join("\n"),
        insight_html = insight_html,
    )
}

fn render_fun_stats(data: &ReportData) -> String {
    let finger_km = data.finger_travel_mm / 1_000_000.0;
    let finger_m = data.finger_travel_mm / 1000.0;

    let travel_value = if finger_km >= 1.0 {
        format!("{:.1} km", finger_km)
    } else {
        format!("{:.0} m", finger_m)
    };

    let backspace_pct = data.backspace_ratio * 100.0;

    let cards = vec![
        (travel_value, "Finger Travel", "Distance your fingers moved"),
        (
            format!("{:.1}%", backspace_pct),
            "Backspace Ratio",
            "Corrections are normal",
        ),
        (
            format_number(data.all_time_keystrokes),
            "All-Time Keystrokes",
            "Your odometer",
        ),
        (
            format!("{:.0}%", data.night_owl_pct),
            "Night Owl",
            "Typing after 6 PM",
        ),
    ];

    let cards_html: Vec<String> = cards
        .iter()
        .map(|(value, label, note)| {
            format!(
                r#"<div class="fun-card">
<div class="fun-card-value">{value}</div>
<div class="fun-card-label">{label}</div>
<div class="fun-card-note">{note}</div>
</div>"#,
                value = value,
                label = label,
                note = note,
            )
        })
        .collect();

    format!(
        r#"<section>
<h2 class="section-title">Fun Stats</h2>
<div class="fun-cards">
{cards}
</div>
</section>
"#,
        cards = cards_html.join("\n"),
    )
}

fn render_footer(_data: &ReportData) -> String {
    let now = chrono::Utc::now();
    format!(
        r#"<footer>
<p>Generated by <a href="https://github.com/0xSaiNova/keyheat">KeyHeat</a> on {date}</p>
<p style="margin-top: 0.5rem;">Run <code>keyheat report</code> to regenerate</p>
</footer>
"#,
        date = now.format("%b %d, %Y"),
    )
}

fn render_scripts() -> String {
    r#"<script>
// Tooltip for heatmap
const tooltip = document.getElementById('tooltip');
document.querySelectorAll('.key-rect').forEach(rect => {
    rect.addEventListener('mouseenter', e => {
        const key = e.target.dataset.key;
        const count = e.target.dataset.count;
        tooltip.textContent = `${key}: ${parseInt(count).toLocaleString()} presses`;
        tooltip.style.opacity = '1';
    });
    rect.addEventListener('mousemove', e => {
        tooltip.style.left = (e.pageX + 10) + 'px';
        tooltip.style.top = (e.pageY - 30) + 'px';
    });
    rect.addEventListener('mouseleave', () => {
        tooltip.style.opacity = '0';
    });
});

// Count-up animation for hero number
const heroNum = document.querySelector('.hero-number');
if (heroNum) {
    const target = parseInt(heroNum.dataset.count);
    const duration = 800;
    const start = performance.now();
    const animate = (now) => {
        const elapsed = now - start;
        const progress = Math.min(elapsed / duration, 1);
        const eased = 1 - Math.pow(1 - progress, 3);
        const current = Math.floor(target * eased);
        heroNum.textContent = current.toLocaleString();
        if (progress < 1) requestAnimationFrame(animate);
    };
    requestAnimationFrame(animate);
}
</script>
"#
    .to_string()
}

fn interpolate_color(cold: &str, mid: &str, hot: &str, t: f64) -> String {
    let cold_rgb = hex_to_rgb(cold);
    let mid_rgb = hex_to_rgb(mid);
    let hot_rgb = hex_to_rgb(hot);

    let (r, g, b) = if t < 0.5 {
        let t2 = t * 2.0;
        (
            lerp(cold_rgb.0, mid_rgb.0, t2),
            lerp(cold_rgb.1, mid_rgb.1, t2),
            lerp(cold_rgb.2, mid_rgb.2, t2),
        )
    } else {
        let t2 = (t - 0.5) * 2.0;
        (
            lerp(mid_rgb.0, hot_rgb.0, t2),
            lerp(mid_rgb.1, hot_rgb.1, t2),
            lerp(mid_rgb.2, hot_rgb.2, t2),
        )
    };

    format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8)
}

fn hex_to_rgb(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64;
    (r, g, b)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
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

fn format_key_display(key: &str) -> String {
    match key {
        "space" => "Space".to_string(),
        "enter" => "Enter".to_string(),
        "backspace" => "Backspace".to_string(),
        "tab" => "Tab".to_string(),
        k if k.len() == 1 => k.to_uppercase(),
        k => k.to_string(),
    }
}

fn format_shortcut_display(shortcut: &str) -> String {
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
