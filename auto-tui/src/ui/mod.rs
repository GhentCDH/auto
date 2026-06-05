pub mod dashboard;
pub mod detail;
pub mod list;
pub mod theme;
pub mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Tab};

pub fn draw(frame: &mut Frame, app: &App) {
    let [tabs_area, content_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // Base background under everything.
    frame.render_widget(Block::new().style(theme::base()), frame.area());

    draw_tabs(frame, app, tabs_area);
    draw_content(frame, app, content_area);
    draw_footer(frame, app, footer_area);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![Span::styled(" auto ", theme::title().bg(theme::BG_TABS))];
    for (index, tab) in Tab::ALL.iter().enumerate() {
        let label = format!(" {} {} ", index + 1, tab.label());
        let style = if *tab == app.tab {
            theme::tab_active()
        } else {
            theme::tabs()
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::styled(" ", theme::tabs()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).style(theme::tabs()), area);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    if !app.detail_stack.is_empty() {
        detail::draw(frame, app, area);
        return;
    }
    match app.tab {
        Tab::Dashboard => dashboard::draw(frame, app, area),
        Tab::Entity(kind) => list::draw(frame, app, kind, area),
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let line = match &app.error {
        Some(error) => Line::from(Span::styled(format!(" {error} "), theme::error())),
        None => {
            let hints = if !app.detail_stack.is_empty() {
                " j/k move · Enter drill · Esc back · r refresh · q quit "
            } else {
                match app.tab {
                    Tab::Dashboard => " Tab/1-9 switch · r refresh · q quit ",
                    Tab::Entity(_) => {
                        " Tab/1-9 switch · j/k move · Enter detail · n/p page · f filter · r refresh · q quit "
                    }
                }
            };
            Line::from(Span::styled(hints, theme::footer()))
        }
    };
    frame.render_widget(Paragraph::new(line).style(theme::footer()), area);
}

/// One cell of padding on every side — visual breathing room instead of borders.
pub fn pad(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}
