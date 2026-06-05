use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

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
