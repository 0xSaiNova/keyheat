#![allow(clippy::uninlined_format_args)]

use super::layout::qwerty_ansi;
use super::ReportData;
use std::collections::HashMap;

pub fn render(data: &ReportData) -> String {
    let mut html = String::with_capacity(96 * 1024);

    html.push_str(&render_head(&data.week.label));
    html.push_str("<body>\n<div class=\"page\">\n");

    html.push_str(&render_hero(data));
    html.push_str(&render_heatmap(data));
    html.push_str(&render_speed(data));
    html.push_str(&render_sessions(data));
    html.push_str(&render_shortcuts(data));
    html.push_str(&render_fun_stats(data));
    html.push_str(&render_footer());

    html.push_str("</div>\n");
    html.push_str(&render_scripts(data));
    html.push_str("</body>\n</html>");

    html
}

fn render_head(title: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>KeyHeat — {title}</title>
<style>
@import url('https://fonts.googleapis.com/css2?family=Anybody:wght@400;500;600;700;800;900&family=DM+Sans:ital,wght@0,400;0,500;0,600;0,700;1,400&family=JetBrains+Mono:wght@400;500;600;700;800&display=swap');

:root {{
  --bg-deep: #030712;
  --bg: #0a0f1a;
  --surface: #0d1424;
  --surface-2: #111827;
  --border: rgba(34,211,238,0.08);
  --border-glow: rgba(34,211,238,0.25);
  --text: #c9d1d9;
  --text-dim: #4b5563;
  --text-muted: #374151;
  --white: #f0f6fc;
  --cyan: #22d3ee;
  --cyan-2: #67e8f9;
  --cyan-3: #a5f3fc;
  --green: #10b981;
  --green-2: #34d399;
  --green-bg: rgba(16,185,129,0.1);
  --amber: #f59e0b;
  --amber-2: #fbbf24;
  --amber-bg: rgba(245,158,11,0.1);
  --red: #ef4444;
  --red-2: #f87171;
  --red-bg: rgba(239,68,68,0.1);
  --gradient-hot: linear-gradient(135deg, #22d3ee, #10b981);
  --gradient-fire: linear-gradient(135deg, #f59e0b, #ef4444);
  --mono: 'JetBrains Mono', monospace;
  --display: 'Anybody', sans-serif;
  --body: 'DM Sans', sans-serif;
  --glow-cyan: 0 0 40px rgba(34,211,238,0.15), 0 0 80px rgba(34,211,238,0.05);
  --glow-green: 0 0 40px rgba(16,185,129,0.12);
}}

*, *::before, *::after {{ margin: 0; padding: 0; box-sizing: border-box; }}
html {{ scroll-behavior: smooth; }}

body {{
  background: var(--bg-deep);
  color: var(--text);
  font-family: var(--body);
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
  overflow-x: hidden;
}}

body::before {{
  content: '';
  position: fixed;
  top: -20%; left: -10%;
  width: 120%; height: 60%;
  background: radial-gradient(ellipse at 50% 0%, rgba(34,211,238,0.06) 0%, transparent 60%),
              radial-gradient(ellipse at 80% 40%, rgba(16,185,129,0.03) 0%, transparent 50%);
  pointer-events: none;
  z-index: 0;
}}
body::after {{
  content: '';
  position: fixed;
  bottom: -20%; right: -10%;
  width: 80%; height: 50%;
  background: radial-gradient(ellipse at 60% 100%, rgba(245,158,11,0.02) 0%, transparent 50%);
  pointer-events: none;
  z-index: 0;
}}

.page {{ max-width: 700px; margin: 0 auto; padding: 0 24px; position: relative; z-index: 1; }}

.reveal {{
  opacity: 0;
  transform: translateY(24px);
  transition: opacity 0.65s cubic-bezier(0.22,1,0.36,1), transform 0.65s cubic-bezier(0.22,1,0.36,1);
}}
.reveal.vis {{ opacity: 1; transform: translateY(0); }}

@keyframes shimmer {{ 0% {{ background-position: -200% center; }} 100% {{ background-position: 200% center; }} }}
@keyframes dotPulse {{ 0%,100% {{ transform: scale(1); opacity: 1; }} 50% {{ transform: scale(0.7); opacity: 0.4; }} }}
@keyframes barGrow {{ from {{ transform: scaleX(0); }} to {{ transform: scaleX(1); }} }}
@keyframes scanline {{
  0% {{ transform: translateY(-100%); }}
  100% {{ transform: translateY(100vh); }}
}}

/* HERO */
.hero {{ padding: 90px 0 56px; text-align: center; }}

.hero-chip {{
  display: inline-flex; align-items: center; gap: 7px;
  font-family: var(--mono); font-size: 10px; font-weight: 600;
  letter-spacing: 0.12em; text-transform: uppercase;
  color: var(--cyan);
  background: rgba(34,211,238,0.08);
  border: 1px solid rgba(34,211,238,0.2);
  padding: 5px 14px; border-radius: 100px;
}}
.hero-chip::before {{
  content: ''; width: 5px; height: 5px;
  background: var(--cyan); border-radius: 50%;
  animation: dotPulse 2s ease-in-out infinite;
  box-shadow: 0 0 8px var(--cyan);
}}
.hero-week {{
  margin-top: 20px;
  font-size: 13px; font-weight: 500; color: var(--text-dim);
  letter-spacing: 0.02em;
}}

.hero-number-wrap {{ margin: 20px 0 12px; position: relative; }}
.hero-number {{
  font-family: var(--display);
  font-size: clamp(72px, 16vw, 140px);
  font-weight: 900;
  line-height: 0.95;
  letter-spacing: -0.04em;
  background: linear-gradient(135deg, #fff 10%, var(--cyan-2) 30%, var(--green-2) 60%, var(--amber) 100%);
  background-size: 200% auto;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: shimmer 6s ease-in-out infinite;
}}
.hero-number::after {{
  content: '';
  position: absolute;
  bottom: -4px; left: 50%; transform: translateX(-50%);
  width: 60%; height: 30px;
  background: radial-gradient(ellipse, rgba(34,211,238,0.2) 0%, transparent 70%);
  filter: blur(8px);
}}
.hero-unit {{
  font-family: var(--mono);
  font-size: 11px; font-weight: 600;
  letter-spacing: 0.18em; text-transform: uppercase;
  color: var(--text-dim);
  margin-top: 6px;
}}
.hero-meta {{
  margin-top: 20px;
  font-size: 15px; color: var(--text-dim); line-height: 1.8;
}}
.hero-meta strong {{ color: var(--white); font-weight: 600; }}
.hero-tag {{
  display: inline-flex; align-items: center; gap: 4px;
  font-family: var(--mono); font-size: 11px; font-weight: 600;
  padding: 3px 9px; border-radius: 6px; margin-left: 4px;
}}
.tag-up {{ color: var(--green); background: var(--green-bg); }}
.tag-down {{ color: var(--red); background: var(--red-bg); }}

/* SECTIONS */
section {{ padding: 52px 0; }}
section + section {{ border-top: 1px solid var(--border); }}

.sec-eyebrow {{
  font-family: var(--mono);
  font-size: 9px; font-weight: 700;
  letter-spacing: 0.16em; text-transform: uppercase;
  color: var(--cyan);
  margin-bottom: 6px;
}}
.sec-title {{
  font-family: var(--display);
  font-size: 26px; font-weight: 800; color: var(--white);
  letter-spacing: -0.03em; margin-bottom: 4px;
}}
.sec-desc {{
  font-size: 13.5px; color: var(--text-dim); margin-bottom: 28px;
  max-width: 460px;
}}

/* HEATMAP */
.heatmap-wrap {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 18px;
  padding: 28px 20px 20px;
  position: relative;
  overflow: hidden;
}}
.heatmap-wrap::before {{
  content: '';
  position: absolute; top: 0; left: 50%; transform: translateX(-50%);
  width: 40%; height: 1px;
  background: linear-gradient(90deg, transparent, var(--cyan), transparent);
}}
.heatmap-wrap::after {{
  content: '';
  position: absolute; top: 0; left: 50%; transform: translateX(-50%);
  width: 30%; height: 40px;
  background: radial-gradient(ellipse, rgba(34,211,238,0.08), transparent);
  pointer-events: none;
}}

.kb {{ width: 100%; }}
.kb-row {{ display: flex; gap: 3px; margin-bottom: 3px; justify-content: center; }}
.kb-key {{
  height: 36px;
  border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
  font-family: var(--mono); font-size: 9px; font-weight: 500;
  color: rgba(255,255,255,0.45);
  position: relative;
  transition: all 0.2s ease;
  cursor: default;
  border: 1px solid rgba(255,255,255,0.02);
  flex-shrink: 0;
}}
.kb-key:hover {{
  border-color: var(--cyan);
  transform: translateY(-1px);
  z-index: 2;
}}
.kb-key:hover .kb-tooltip {{ opacity: 1; transform: translateX(-50%) translateY(0); }}
.kb-tooltip {{
  position: absolute;
  bottom: calc(100% + 8px); left: 50%;
  transform: translateX(-50%) translateY(4px);
  background: var(--surface-2);
  border: 1px solid var(--border-glow);
  border-radius: 8px;
  padding: 6px 10px;
  font-family: var(--mono); font-size: 10px;
  color: var(--white);
  white-space: nowrap;
  opacity: 0;
  transition: all 0.2s ease;
  pointer-events: none;
  box-shadow: var(--glow-cyan);
  z-index: 10;
}}
.kb-tooltip span {{ color: var(--cyan); }}

.heatmap-legend {{
  display: flex; align-items: center; justify-content: center;
  gap: 10px; margin-top: 16px;
  font-family: var(--mono); font-size: 9px; color: var(--text-muted);
}}
.legend-gradient {{
  width: 100px; height: 6px; border-radius: 3px;
  background: linear-gradient(90deg, var(--surface-2), #134e4a, #0d9488, #22d3ee, #f59e0b);
}}

.top-keys-row {{
  display: flex; gap: 10px; margin-top: 18px; justify-content: center; flex-wrap: wrap;
}}
.top-key-pill {{
  display: flex; align-items: center; gap: 8px;
  background: var(--bg); border: 1px solid var(--border);
  border-radius: 10px; padding: 7px 12px;
  transition: border-color 0.2s;
}}
.top-key-pill:hover {{ border-color: var(--border-glow); }}
.tkp-rank {{
  font-family: var(--mono); font-size: 9px; font-weight: 700;
  color: var(--text-muted);
}}
.tkp-key {{
  font-family: var(--mono); font-size: 12px; font-weight: 700;
  color: var(--white);
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 5px; padding: 1px 8px; min-width: 28px; text-align: center;
}}
.tkp-count {{
  font-family: var(--mono); font-size: 11px; color: var(--text-dim);
}}

/* CARDS */
.cards {{ display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }}
.card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 20px;
  position: relative;
  overflow: hidden;
  transition: border-color 0.25s, box-shadow 0.25s;
}}
.card:hover {{ border-color: rgba(34,211,238,0.15); box-shadow: 0 0 30px rgba(34,211,238,0.04); }}
.card.wide {{ grid-column: 1 / -1; }}
.card.glow-border {{ border-color: var(--border-glow); }}
.card.glow-border::before {{
  content: '';
  position: absolute; inset: -1px;
  border-radius: 15px;
  background: linear-gradient(135deg, rgba(34,211,238,0.08), transparent 50%);
  pointer-events: none;
}}
.card-val {{
  font-family: var(--mono);
  font-size: 34px; font-weight: 800;
  color: var(--white);
  line-height: 1;
  margin-bottom: 6px;
}}
.card-val .sm {{ font-size: 18px; font-weight: 500; color: var(--text-dim); }}
.card-label {{ font-size: 13px; color: var(--text-dim); font-weight: 500; }}
.card-sub {{ font-size: 11px; color: var(--text-muted); margin-top: 2px; font-style: italic; }}
.card-tag {{
  position: absolute; top: 14px; right: 14px;
  font-family: var(--mono); font-size: 10px; font-weight: 600;
  padding: 2px 8px; border-radius: 5px;
}}

/* CHART BOX */
.chart-box {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 22px;
  margin-top: 10px;
}}
.chart-box-title {{
  font-family: var(--mono);
  font-size: 10px; font-weight: 600;
  letter-spacing: 0.08em; text-transform: uppercase;
  color: var(--text-dim);
  margin-bottom: 16px;
}}
.chart-svg {{ width: 100%; overflow: visible; }}

/* ACTIVITY GRID */
.grid-row {{ display: flex; align-items: center; gap: 2px; margin-bottom: 2px; }}
.grid-label {{
  width: 32px; flex-shrink: 0; text-align: right;
  font-family: var(--mono); font-size: 9px; color: var(--text-muted);
  padding-right: 6px;
}}
.grid-cell {{
  flex: 1; aspect-ratio: 1;
  border-radius: 3px;
  background: var(--surface);
  transition: all 0.15s;
  cursor: default;
  min-width: 0;
}}
.grid-cell:hover {{ border-radius: 4px; transform: scale(1.3); z-index: 2; }}
.grid-cell[data-level="1"] {{ background: #134e4a; }}
.grid-cell[data-level="2"] {{ background: #0f766e; }}
.grid-cell[data-level="3"] {{ background: #0d9488; }}
.grid-cell[data-level="4"] {{ background: #14b8a6; }}
.grid-cell[data-level="5"] {{ background: #22d3ee; box-shadow: 0 0 6px rgba(34,211,238,0.4); }}
.hour-labels {{ display: flex; gap: 2px; margin-left: 32px; margin-top: 4px; }}
.hour-label {{
  flex: 1; text-align: center;
  font-family: var(--mono); font-size: 8px; color: var(--text-muted);
}}

/* SHORTCUT BARS */
.sc-row {{
  display: flex; align-items: center; gap: 12px;
  padding: 10px 0;
}}
.sc-row + .sc-row {{ border-top: 1px solid rgba(255,255,255,0.02); }}
.sc-name {{
  width: 100px; flex-shrink: 0;
  font-family: var(--mono); font-size: 12px; font-weight: 600;
  color: var(--text);
}}
.sc-bar-track {{ flex: 1; height: 24px; background: rgba(34,211,238,0.06); border-radius: 6px; overflow: hidden; }}
.sc-bar-fill {{
  height: 100%; border-radius: 6px;
  background: linear-gradient(90deg, var(--cyan), var(--green));
  transform-origin: left;
  animation: barGrow 0.8s cubic-bezier(0.22,1,0.36,1) forwards;
}}
.sc-count {{
  width: 40px; text-align: right; flex-shrink: 0;
  font-family: var(--mono); font-size: 12px; font-weight: 600;
  color: var(--text-dim);
}}

.insight-card {{
  margin-top: 16px;
  background: rgba(34,211,238,0.04);
  border: 1px solid rgba(34,211,238,0.12);
  border-radius: 12px;
  padding: 14px 18px;
  display: flex; align-items: flex-start; gap: 10px;
}}
.insight-icon {{ font-size: 18px; flex-shrink: 0; line-height: 1.4; }}
.insight-text {{ font-size: 13px; color: var(--text); line-height: 1.6; }}
.insight-text strong {{ color: var(--white); }}

/* FUN STATS */
.fun-cards {{ display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }}
.fun-card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 22px 18px;
  text-align: center;
  transition: border-color 0.25s, transform 0.25s;
}}
.fun-card:hover {{ border-color: var(--border-glow); transform: translateY(-2px); }}
.fun-icon {{ font-size: 22px; margin-bottom: 10px; }}
.fun-val {{
  font-family: var(--mono); font-size: 28px; font-weight: 800;
  background: var(--gradient-hot);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
  background-clip: text;
  line-height: 1; margin-bottom: 6px;
}}
.fun-label {{ font-size: 12px; color: var(--text-dim); font-weight: 600; }}
.fun-sub {{ font-size: 10.5px; color: var(--text-muted); margin-top: 3px; font-style: italic; }}

/* FOOTER */
.footer {{
  padding: 48px 0 60px;
  text-align: center;
  border-top: 1px solid var(--border);
}}
.footer p {{
  font-size: 12px; color: var(--text-muted); line-height: 2;
}}
.footer a {{ color: var(--cyan); text-decoration: none; }}
.footer code {{
  font-family: var(--mono); font-size: 11px;
  background: var(--surface); padding: 2px 8px; border-radius: 4px;
  border: 1px solid var(--border);
}}

@media (max-width: 500px) {{
  .cards, .fun-cards {{ grid-template-columns: 1fr; }}
  .top-keys-row {{ flex-direction: column; align-items: center; }}
  .hero {{ padding: 60px 0 40px; }}
  .kb-key {{ height: 28px; font-size: 7px; }}
}}
</style>
</head>
"##,
        title = title
    )
}

fn render_hero(data: &ReportData) -> String {
    let delta_html = if let Some(prev) = data.prev_week_keystrokes {
        if prev > 0 {
            let pct = ((data.total_keystrokes as f64 - prev as f64) / prev as f64) * 100.0;
            let (class, arrow) = if pct >= 0.0 {
                ("tag-up", "&#9650;")
            } else {
                ("tag-down", "&#9660;")
            };
            format!(
                r#"<span class="hero-tag {class}">{arrow} {pct:.1}%</span>"#,
                class = class,
                arrow = arrow,
                pct = pct.abs()
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let hours = (data.total_typing_minutes / 60.0).floor() as u32;
    let mins = (data.total_typing_minutes % 60.0).round() as u32;

    format!(
        r#"<div class="hero reveal">
  <div class="hero-chip">Weekly Report</div>
  <p class="hero-week">{week_label}</p>
  <div class="hero-number-wrap">
    <div class="hero-number" id="heroNum">{keystrokes_fmt}</div>
  </div>
  <p class="hero-unit">keystrokes</p>
  <p class="hero-meta">
    <strong>{sessions} sessions</strong> across 7 days · <strong>{hours}h {mins}m</strong> total typing time
    {delta_html}
  </p>
</div>
"#,
        week_label = data.week.label,
        keystrokes_fmt = format_number(data.total_keystrokes),
        sessions = data.sessions.len(),
        hours = hours,
        mins = mins,
        delta_html = delta_html,
    )
}

fn render_heatmap(data: &ReportData) -> String {
    let top_keys: Vec<String> = data
        .key_frequencies
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, (k, c))| {
            format!(
                r#"<div class="top-key-pill">
  <span class="tkp-rank">#{rank}</span>
  <span class="tkp-key">{key}</span>
  <span class="tkp-count">{count}</span>
</div>"#,
                rank = i + 1,
                key = format_key_display(k),
                count = format_number(*c)
            )
        })
        .collect();

    format!(
        r##"<section class="reveal">
  <p class="sec-eyebrow">01 — Heatmap</p>
  <h2 class="sec-title">Your Keyboard</h2>
  <p class="sec-desc">Where your fingers spent the week. Brighter keys were hit more often.</p>

  <div class="heatmap-wrap">
    <div class="kb" id="keyboard"></div>
    <div class="heatmap-legend">
      <span>less</span>
      <div class="legend-gradient"></div>
      <span>more</span>
    </div>
  </div>

  <div class="top-keys-row">
    {top_keys}
  </div>
</section>
"##,
        top_keys = top_keys.join("\n    ")
    )
}

fn render_speed(data: &ReportData) -> String {
    let peak_time = data
        .peak_wpm_time
        .map(|t| t.format("%a %l:%M %p").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let wpm_delta = data
        .prev_week_avg_wpm
        .map(|prev| {
            if prev > 0.0 {
                let pct = ((data.avg_wpm - prev) / prev) * 100.0;
                let (class, arrow) = if pct >= 0.0 {
                    ("tag-up", "&#9650;")
                } else {
                    ("tag-down", "&#9660;")
                };
                format!(
                    r#"<div class="card-tag {class}">{arrow} {pct:.0}%</div>"#,
                    class = class,
                    arrow = arrow,
                    pct = pct.abs()
                )
            } else {
                String::new()
            }
        })
        .unwrap_or_default();

    let trend_svg = render_wpm_trend(&data.daily_wpm);
    let dist_svg = render_wpm_distribution(&data.wpm_distribution);

    format!(
        r##"<section class="reveal">
  <p class="sec-eyebrow">02 — Speed</p>
  <h2 class="sec-title">Speed Story</h2>
  <p class="sec-desc">How fast you typed and when you hit your stride.</p>

  <div class="cards">
    <div class="card glow-border">
      <div class="card-val">{avg_wpm:.0}<span class="sm"> wpm</span></div>
      <div class="card-label">Average Speed</div>
      {wpm_delta}
    </div>
    <div class="card">
      <div class="card-val">{peak_wpm:.0}<span class="sm"> wpm</span></div>
      <div class="card-label">Peak Speed</div>
      <div class="card-sub">{peak_time}</div>
    </div>
  </div>

  <div class="chart-box">
    <div class="chart-box-title">Daily WPM Trend</div>
    {trend_svg}
  </div>

  <div class="chart-box">
    <div class="chart-box-title">Speed Distribution</div>
    {dist_svg}
  </div>
</section>
"##,
        avg_wpm = data.avg_wpm,
        peak_wpm = data.peak_wpm,
        peak_time = peak_time,
        wpm_delta = wpm_delta,
        trend_svg = trend_svg,
        dist_svg = dist_svg,
    )
}

fn render_wpm_trend(daily_wpm: &[(chrono::NaiveDate, f64)]) -> String {
    if daily_wpm.is_empty() {
        return r##"<svg class="chart-svg" viewBox="0 0 600 140"><text x="300" y="70" fill="#374151" font-family="JetBrains Mono" font-size="12" text-anchor="middle">No data yet</text></svg>"##.to_string();
    }

    let max_wpm = daily_wpm
        .iter()
        .map(|(_, w)| *w)
        .fold(0.0f64, |a, b| a.max(b))
        .max(1.0);
    let min_wpm = daily_wpm
        .iter()
        .map(|(_, w)| *w)
        .fold(f64::MAX, |a, b| a.min(b));
    let range = (max_wpm - min_wpm).max(10.0);

    let x_positions: Vec<f64> = (0..7).map(|i| 80.0 + i as f64 * 75.0).collect();

    let points: Vec<String> = daily_wpm
        .iter()
        .enumerate()
        .map(|(i, (_, wpm))| {
            let x = x_positions.get(i).copied().unwrap_or(80.0);
            let y = 100.0 - ((wpm - min_wpm) / range) * 70.0;
            format!("{x:.0},{y:.0}")
        })
        .collect();

    let area_points = {
        let mut pts = points.clone();
        if let Some(last_x) = x_positions.get(daily_wpm.len().saturating_sub(1)) {
            pts.push(format!("{:.0},100", last_x));
        }
        pts.push(format!("{:.0},100", x_positions[0]));
        pts.join(" ")
    };

    let dots: Vec<String> = daily_wpm
        .iter()
        .enumerate()
        .map(|(i, (_, wpm))| {
            let x = x_positions.get(i).copied().unwrap_or(80.0);
            let y = 100.0 - ((wpm - min_wpm) / range) * 70.0;
            format!(
                r##"<circle cx="{x:.0}" cy="{y:.0}" r="4" fill="#22d3ee" stroke="#0a0f1a" stroke-width="2"/>"##
            )
        })
        .collect();

    let labels: Vec<String> = daily_wpm
        .iter()
        .enumerate()
        .map(|(i, (date, _))| {
            let x = x_positions.get(i).copied().unwrap_or(80.0);
            let day = date.format("%a").to_string();
            format!(
                r##"<text x="{x:.0}" y="125" fill="#374151" font-family="JetBrains Mono" font-size="9" text-anchor="middle">{day}</text>"##
            )
        })
        .collect();

    format!(
        r##"<svg class="chart-svg" viewBox="0 0 600 140" preserveAspectRatio="none">
  <defs>
    <linearGradient id="lineGrad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#22d3ee"/>
      <stop offset="100%" stop-color="#10b981"/>
    </linearGradient>
    <linearGradient id="areaGrad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="rgba(34,211,238,0.15)"/>
      <stop offset="100%" stop-color="rgba(34,211,238,0)"/>
    </linearGradient>
  </defs>
  <line x1="60" y1="20" x2="560" y2="20" stroke="rgba(255,255,255,0.03)" stroke-width="1"/>
  <line x1="60" y1="55" x2="560" y2="55" stroke="rgba(255,255,255,0.03)" stroke-width="1"/>
  <line x1="60" y1="90" x2="560" y2="90" stroke="rgba(255,255,255,0.03)" stroke-width="1"/>
  <polygon points="{area_points}" fill="url(#areaGrad)"/>
  <polyline points="{points}" fill="none" stroke="url(#lineGrad)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  {dots}
  {labels}
</svg>"##,
        area_points = area_points,
        points = points.join(" "),
        dots = dots.join("\n  "),
        labels = labels.join("\n  ")
    )
}

fn render_wpm_distribution(distribution: &[u32]) -> String {
    let labels = [
        "0-20", "20-40", "40-60", "60-80", "80-100", "100-120", "120+",
    ];
    let max_count = distribution.iter().copied().max().unwrap_or(1).max(1);

    let bars: Vec<String> = distribution
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let x = 50.0 + i as f64 * 78.0;
            let height = (count as f64 / max_count as f64) * 60.0;
            let y = 70.0 - height;
            let colors = ["#134e4a", "#0f766e", "#0d9488", "#14b8a6", "#22d3ee", "#67e8f9", "#f59e0b"];
            let color = colors.get(i).unwrap_or(&"#22d3ee");
            format!(
                r##"<rect x="{x:.0}" y="{y:.0}" width="65" height="{height:.0}" rx="4" fill="{color}"/>
<text x="{tx:.0}" y="88" fill="#374151" font-family="JetBrains Mono" font-size="8" text-anchor="middle">{label}</text>"##,
                x = x,
                y = y,
                height = height,
                color = color,
                tx = x + 32.5,
                label = labels[i]
            )
        })
        .collect();

    format!(
        r#"<svg class="chart-svg" viewBox="0 0 600 100">
{bars}
</svg>"#,
        bars = bars.join("\n")
    )
}

fn render_sessions(data: &ReportData) -> String {
    let busiest_hour = data
        .peak_hour
        .map(|h| {
            let period = if h < 12 { "AM" } else { "PM" };
            let hour_12 = if h == 0 {
                12
            } else if h > 12 {
                h - 12
            } else {
                h
            };
            format!("{}<span class=\"sm\"> {}</span>", hour_12, period)
        })
        .unwrap_or_else(|| "-".to_string());

    let longest = data
        .longest_session
        .as_ref()
        .map(|s| {
            let hours = (s.duration_minutes / 60.0).floor() as u32;
            let mins = (s.duration_minutes % 60.0).round() as u32;
            if hours > 0 {
                format!("{}h {}<span class=\"sm\">m</span>", hours, mins)
            } else {
                format!("{}<span class=\"sm\">m</span>", mins)
            }
        })
        .unwrap_or_else(|| "-".to_string());

    format!(
        r##"<section class="reveal">
  <p class="sec-eyebrow">03 — Rhythm</p>
  <h2 class="sec-title">Session Timeline</h2>
  <p class="sec-desc">When you were at the keys. Each cell is one hour.</p>

  <div class="chart-box">
    <div class="chart-box-title">Activity by Hour</div>
    <div id="activityGrid"></div>
  </div>

  <div class="cards" style="margin-top:10px">
    <div class="card">
      <div class="card-val">{busiest_hour}</div>
      <div class="card-label">Peak Hour</div>
      <div class="card-sub">Your most productive time</div>
    </div>
    <div class="card">
      <div class="card-val">{longest}</div>
      <div class="card-label">Longest Session</div>
    </div>
  </div>
</section>
"##,
        busiest_hour = busiest_hour,
        longest = longest,
    )
}

fn render_shortcuts(data: &ReportData) -> String {
    if data.shortcuts.is_empty() {
        return String::new();
    }

    let max_count = data.shortcuts.first().map(|(_, c)| *c).unwrap_or(1) as f64;

    let bars: Vec<String> = data
        .shortcuts
        .iter()
        .take(6)
        .enumerate()
        .map(|(i, (combo, count))| {
            let pct = (*count as f64 / max_count) * 100.0;
            let delay = i as f64 * 0.05;
            format!(
                r#"<div class="sc-row">
  <span class="sc-name">{combo}</span>
  <div class="sc-bar-track"><div class="sc-bar-fill" style="width:{pct:.0}%;animation-delay:{delay}s"></div></div>
  <span class="sc-count">{count}</span>
</div>"#,
                combo = format_shortcut_display(combo),
                pct = pct,
                delay = delay,
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
  <div class="insight-icon">&#128161;</div>
  <div class="insight-text">{}</div>
</div>"#,
                i.message
            )
        })
        .unwrap_or_default();

    format!(
        r##"<section class="reveal">
  <p class="sec-eyebrow">04 — Commands</p>
  <h2 class="sec-title">Shortcut Game</h2>
  <p class="sec-desc">Your most used keyboard combos this week.</p>

  <div class="chart-box">
    {bars}
  </div>

  {insight_html}
</section>
"##,
        bars = bars.join("\n    "),
        insight_html = insight_html,
    )
}

fn render_fun_stats(data: &ReportData) -> String {
    let finger_km = data.finger_travel_mm / 1_000_000.0;
    let finger_m = data.finger_travel_mm / 1000.0;

    let (travel_value, travel_note) = if finger_km >= 1.0 {
        (
            format!("{:.1} km", finger_km),
            format!("{:.0} laps around a track", finger_km * 2.5),
        )
    } else {
        (format!("{:.0} m", finger_m), "Keep typing!".to_string())
    };

    let backspace_pct = data.backspace_ratio * 100.0;

    let all_time = if data.all_time_keystrokes >= 1_000_000 {
        format!("{:.1}M", data.all_time_keystrokes as f64 / 1_000_000.0)
    } else if data.all_time_keystrokes >= 1_000 {
        format!("{:.0}K", data.all_time_keystrokes as f64 / 1_000.0)
    } else {
        format_number(data.all_time_keystrokes)
    };

    format!(
        r##"<section class="reveal">
  <p class="sec-eyebrow">05 — For Fun</p>
  <h2 class="sec-title">Fun Stats</h2>
  <p class="sec-desc">The numbers nobody asked for but everyone wants to see.</p>

  <div class="fun-cards">
    <div class="fun-card">
      <div class="fun-icon">&#127939;</div>
      <div class="fun-val">{travel_value}</div>
      <div class="fun-label">Finger Travel</div>
      <div class="fun-sub">{travel_note}</div>
    </div>
    <div class="fun-card">
      <div class="fun-icon">&#9003;</div>
      <div class="fun-val">{backspace_pct:.1}%</div>
      <div class="fun-label">Backspace Ratio</div>
      <div class="fun-sub">Everyone makes mistakes</div>
    </div>
    <div class="fun-card">
      <div class="fun-icon">&#128290;</div>
      <div class="fun-val">{all_time}</div>
      <div class="fun-label">All Time Keys</div>
      <div class="fun-sub">Your odometer keeps rolling</div>
    </div>
    <div class="fun-card">
      <div class="fun-icon">&#127769;</div>
      <div class="fun-val">{night_owl:.0}%</div>
      <div class="fun-label">Night Owl</div>
      <div class="fun-sub">Typing after 6 PM</div>
    </div>
  </div>
</section>
"##,
        travel_value = travel_value,
        travel_note = travel_note,
        backspace_pct = backspace_pct,
        all_time = all_time,
        night_owl = data.night_owl_pct,
    )
}

fn render_footer() -> String {
    let now = chrono::Utc::now();
    format!(
        r#"<div class="footer">
  <p>Generated by <a href="https://github.com/0xSaiNova/keyheat">KeyHeat</a> on {date}</p>
  <p>Run <code>keyheat report</code> to regenerate</p>
</div>
"#,
        date = now.format("%b %d, %Y")
    )
}

fn render_scripts(data: &ReportData) -> String {
    let key_counts: HashMap<&str, u64> = data
        .key_frequencies
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();

    let layout = qwerty_ansi();

    let rows_js = build_keyboard_rows_js(&layout);
    let counts_js = build_counts_js(&key_counts);
    let activity_js = build_activity_js(&data.hourly_activity);
    let total_keystrokes = data.total_keystrokes;

    format!(
        r##"<script>
// Scroll reveal
const obs = new IntersectionObserver((entries) => {{
  entries.forEach(e => {{ if (e.isIntersecting) {{ e.target.classList.add('vis'); obs.unobserve(e.target); }} }});
}}, {{ threshold: 0.1 }});
document.querySelectorAll('.reveal').forEach(el => obs.observe(el));

// Keyboard heatmap
(function() {{
  const rows = {rows_js};
  const counts = {counts_js};
  const max = Math.max(...Object.values(counts), 1);

  function heatColor(count) {{
    if (!count) return '#0d1424';
    const t = Math.pow(count / max, 0.45);
    const colors = [
      [13,20,36],
      [19,78,74],
      [13,148,136],
      [34,211,238],
      [245,158,11]
    ];
    const idx = t * (colors.length - 1);
    const lo = Math.floor(idx);
    const hi = Math.min(lo + 1, colors.length - 1);
    const f = idx - lo;
    const r = Math.round(colors[lo][0] + (colors[hi][0] - colors[lo][0]) * f);
    const g = Math.round(colors[lo][1] + (colors[hi][1] - colors[lo][1]) * f);
    const b = Math.round(colors[lo][2] + (colors[hi][2] - colors[lo][2]) * f);
    return `rgb(${{r}},${{g}},${{b}})`;
  }}

  const kb = document.getElementById('keyboard');
  const unit = 42, gap = 3;

  rows.forEach(row => {{
    const rowDiv = document.createElement('div');
    rowDiv.className = 'kb-row';
    row.forEach(([label, width, code]) => {{
      const key = document.createElement('div');
      key.className = 'kb-key';
      key.style.width = (width * unit + (width - 1) * gap) + 'px';
      const count = counts[code] || 0;
      key.style.background = heatColor(count);
      key.textContent = label;
      if (count) {{
        const tt = document.createElement('div');
        tt.className = 'kb-tooltip';
        tt.innerHTML = `${{label}} <span>${{count.toLocaleString()}}</span>`;
        key.appendChild(tt);
      }}
      rowDiv.appendChild(key);
    }});
    kb.appendChild(rowDiv);
  }});
}})();

// Activity grid
(function() {{
  const days = ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'];
  const data = {activity_js};
  const grid = document.getElementById('activityGrid');
  const maxVal = Math.max(...data.flat(), 1);

  days.forEach((day, di) => {{
    const row = document.createElement('div');
    row.className = 'grid-row';
    const lbl = document.createElement('div');
    lbl.className = 'grid-label';
    lbl.textContent = day;
    row.appendChild(lbl);
    for (let h = 0; h < 24; h++) {{
      const cell = document.createElement('div');
      cell.className = 'grid-cell';
      const val = data[di][h];
      const level = val === 0 ? 0 : Math.min(5, Math.ceil((val / maxVal) * 5));
      cell.dataset.level = level;
      row.appendChild(cell);
    }}
    grid.appendChild(row);
  }});

  const hourRow = document.createElement('div');
  hourRow.className = 'hour-labels';
  for (let h = 0; h < 24; h++) {{
    const lbl = document.createElement('div');
    lbl.className = 'hour-label';
    lbl.textContent = h % 6 === 0 ? h : '';
    hourRow.appendChild(lbl);
  }}
  grid.appendChild(hourRow);
}})();

// Count up hero
(function() {{
  const el = document.getElementById('heroNum');
  const target = {total_keystrokes};
  const dur = 1200;
  const start = performance.now();
  function tick(now) {{
    const t = Math.min((now - start) / dur, 1);
    const ease = 1 - Math.pow(1 - t, 3);
    el.textContent = Math.floor(target * ease).toLocaleString();
    if (t < 1) requestAnimationFrame(tick);
  }}
  requestAnimationFrame(tick);
}})();
</script>
"##,
        rows_js = rows_js,
        counts_js = counts_js,
        activity_js = activity_js,
        total_keystrokes = total_keystrokes,
    )
}

fn build_keyboard_rows_js(layout: &[super::layout::KeyPosition]) -> String {
    let mut rows: Vec<Vec<(&str, f64, &str)>> = vec![vec![]; 5];

    for key in layout {
        let row_idx = key.y as usize;
        if row_idx < 5 {
            rows[row_idx].push((key.label, key.width, key.key_code));
        }
    }

    let rows_str: Vec<String> = rows
        .iter()
        .map(|row| {
            let items: Vec<String> = row
                .iter()
                .map(|(label, width, code)| {
                    let display_label = if *label == "\\" { "\\\\" } else { label };
                    format!(r#"["{}",{},"{}"]"#, display_label, width, code)
                })
                .collect();
            format!("[{}]", items.join(","))
        })
        .collect();

    format!("[{}]", rows_str.join(","))
}

fn build_counts_js(counts: &HashMap<&str, u64>) -> String {
    let items: Vec<String> = counts
        .iter()
        .map(|(k, v)| format!(r#""{}": {}"#, k, v))
        .collect();
    format!("{{{}}}", items.join(", "))
}

fn build_activity_js(grid: &[[u64; 24]; 7]) -> String {
    let rows: Vec<String> = grid
        .iter()
        .map(|row| {
            let vals: Vec<String> = row.iter().map(|v| v.to_string()).collect();
            format!("[{}]", vals.join(","))
        })
        .collect();
    format!("[{}]", rows.join(","))
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
        "backspace" => "Back".to_string(),
        "tab" => "Tab".to_string(),
        "lshift" | "rshift" => "Shift".to_string(),
        "lctrl" | "rctrl" => "Ctrl".to_string(),
        "lalt" | "ralt" => "Alt".to_string(),
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
