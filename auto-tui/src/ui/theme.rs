//! Border-less color theme: sections are distinguished by background color
//! rather than border lines.

use ratatui::style::{Color, Modifier, Style};

/// Base background for the main content area.
pub const BG: Color = Color::Rgb(24, 26, 32);
/// Slightly raised background for side panels / secondary sections.
pub const BG_PANEL: Color = Color::Rgb(34, 37, 46);
/// Background for the tab bar strip.
pub const BG_TABS: Color = Color::Rgb(44, 48, 60);
/// Background for the footer/help strip.
pub const BG_FOOTER: Color = Color::Rgb(38, 41, 51);
/// Background for the selected row / active element.
pub const BG_SELECTED: Color = Color::Rgb(62, 68, 86);

pub const FG: Color = Color::Rgb(205, 209, 219);
pub const FG_DIM: Color = Color::Rgb(120, 126, 140);
pub const ACCENT: Color = Color::Rgb(122, 162, 247);

/// Kuma heartbeat status colors: 1=up, 0=down, 2=pending, 3=maintenance.
pub const STATUS_UP: Color = Color::Rgb(115, 218, 145);
pub const STATUS_DOWN: Color = Color::Rgb(237, 110, 121);
pub const STATUS_PENDING: Color = Color::Rgb(229, 192, 123);
pub const STATUS_MAINTENANCE: Color = Color::Rgb(97, 175, 239);

pub fn base() -> Style {
    Style::new().fg(FG).bg(BG)
}

pub fn panel() -> Style {
    Style::new().fg(FG).bg(BG_PANEL)
}

pub fn tabs() -> Style {
    Style::new().fg(FG_DIM).bg(BG_TABS)
}

pub fn tab_active() -> Style {
    Style::new().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn footer() -> Style {
    Style::new().fg(FG_DIM).bg(BG_FOOTER)
}

pub fn selected() -> Style {
    Style::new()
        .fg(FG)
        .bg(BG_SELECTED)
        .add_modifier(Modifier::BOLD)
}

pub fn title() -> Style {
    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::new().fg(FG_DIM)
}

pub fn error() -> Style {
    Style::new()
        .fg(BG)
        .bg(STATUS_DOWN)
        .add_modifier(Modifier::BOLD)
}

pub fn success() -> Style {
    Style::new()
        .fg(BG)
        .bg(STATUS_UP)
        .add_modifier(Modifier::BOLD)
}

pub fn heartbeat_color(status: i32) -> Color {
    match status {
        1 => STATUS_UP,
        0 => STATUS_DOWN,
        2 => STATUS_PENDING,
        3 => STATUS_MAINTENANCE,
        _ => FG_DIM,
    }
}
