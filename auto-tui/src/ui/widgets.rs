use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::api::models::HeartbeatEntry;

use super::theme;

const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

pub fn spinner(ticks: usize) -> &'static str {
    SPINNER_FRAMES[ticks % SPINNER_FRAMES.len()]
}

/// Centered "loading" line for a panel that has no data yet.
pub fn draw_loading(frame: &mut Frame, area: Rect, ticks: usize) {
    let line = Line::from(Span::styled(
        format!("{} loading…", spinner(ticks)),
        theme::dim(),
    ));
    frame.render_widget(Paragraph::new(line).centered(), area);
}

/// Heartbeat bar spans: one ping-scaled glyph per beat with a gap between
/// cells, newest on the right. `capacity` is the max number of beats
/// (each costs 2 columns).
pub fn heartbeat_spans(
    heartbeats: &[HeartbeatEntry],
    capacity: usize,
    bg: Color,
) -> Vec<Span<'static>> {
    let visible = &heartbeats[heartbeats.len().saturating_sub(capacity)..];
    let max_ping = visible
        .iter()
        .filter_map(|beat| beat.ping)
        .max()
        .unwrap_or(0)
        .max(1);
    let mut spans = Vec::with_capacity(visible.len() * 2);
    for (index, beat) in visible.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" ", Style::new().bg(bg)));
        }
        spans.push(Span::styled(
            heartbeat_glyph(beat, max_ping),
            Style::new().fg(theme::heartbeat_color(beat.status)).bg(bg),
        ));
    }
    spans
}

/// Pick a bottom-aligned block glyph for one heartbeat: height tracks the
/// ping relative to the window's slowest response; down beats are always a
/// full block so they stand out.
pub fn heartbeat_glyph(beat: &HeartbeatEntry, max_ping: i32) -> &'static str {
    const LEVELS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    if beat.status == 0 {
        return "█";
    }
    let Some(ping) = beat.ping else {
        return "▁";
    };
    let level = (ping.max(0) as usize * (LEVELS.len() - 1)) / max_ping.max(1) as usize;
    LEVELS[level.min(LEVELS.len() - 1)]
}

/// Marquee a string into a fixed-width window. Short names render padded and
/// static; long names scroll left one character at a time at a constant
/// speed (so longer names take proportionally longer), with a pause at the
/// initial position and another at the end before snapping back. `phase`
/// staggers the start so neighboring marquees don't move in lockstep.
pub fn marquee(text: &str, width: usize, ticks: usize, phase: usize) -> String {
    const START_PAUSE: usize = 16; // ticks (250 ms each) ≈ 4 s
    const END_PAUSE: usize = 8; //                       ≈ 2 s
    const TICKS_PER_CHAR: usize = 2; //                  ≈ 2 chars/s

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return format!("{text:<width$}");
    }

    let overflow = chars.len() - width;
    let cycle = START_PAUSE + overflow * TICKS_PER_CHAR + END_PAUSE;
    let t = (ticks + phase) % cycle;
    let offset = if t < START_PAUSE {
        0
    } else {
        ((t - START_PAUSE) / TICKS_PER_CHAR).min(overflow)
    };
    chars[offset..offset + width].iter().collect()
}

/// Status dot color for the latest heartbeat status (None = no data yet).
pub fn status_dot(status: Option<i32>) -> Span<'static> {
    let color = status.map(theme::heartbeat_color).unwrap_or(theme::FG_DIM);
    Span::styled("●", Style::new().fg(color))
}

/// Centered floating rect for popups (no border — contrasting bg instead).
pub fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// Inline failure banner with a retry hint.
pub fn draw_failed(frame: &mut Frame, area: Rect, message: &str) {
    let lines = vec![
        Line::from(Span::styled(format!(" {message} "), theme::error())),
        Line::from(Span::styled("press r to retry", theme::dim())),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}
