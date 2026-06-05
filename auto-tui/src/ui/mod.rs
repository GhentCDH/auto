pub mod dashboard;
pub mod detail;
pub mod dns;
pub mod healthchecks;
pub mod list;
pub mod search;
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
    draw_exec_popup(frame, app, content_area);
    if app.dns.is_some() {
        dns::draw(frame, app, content_area);
    }
    if app.search.open {
        search::draw(frame, app, content_area);
    }
    if app.show_help {
        draw_help(frame, content_area);
    }
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = widgets::popup_area(area, 64, 20);
    frame.render_widget(ratatui::widgets::Clear, popup);
    frame.render_widget(Block::new().style(theme::panel()), popup);

    let entries: [(&str, &str); 16] = [
        ("Tab / Shift-Tab", "next / previous tab"),
        ("1-9", "jump to tab"),
        ("/", "global search"),
        ("r", "refresh current view"),
        ("?", "this help"),
        ("q / Ctrl-C", "quit"),
        ("j/k or arrows", "move selection"),
        ("g / G", "first / last row"),
        ("n / p", "next / previous page"),
        ("f", "filter list (Enter apply, Esc cancel)"),
        ("Enter", "open detail / drill into relation"),
        ("Esc / Backspace", "back / close overlay"),
        ("x", "execute healthcheck"),
        ("s / S", "sync one / all (healthchecks → Kuma, infra → IPs)"),
        ("d", "DNS records (domains)"),
        ("h", "collapse health section (dashboard)"),
    ];
    let mut lines = vec![
        Line::from(Span::styled("Keys", theme::title().bg(theme::BG_PANEL))),
        Line::default(),
    ];
    lines.extend(entries.iter().map(|(key, action)| {
        Line::from(vec![
            Span::styled(format!("  {key:<16}"), theme::selected()),
            Span::styled(format!("  {action}"), theme::panel()),
        ])
    }));
    frame.render_widget(Paragraph::new(lines).style(theme::panel()), pad(popup));
}

/// Floating result popup for a healthcheck execution.
fn draw_exec_popup(frame: &mut Frame, app: &App, area: Rect) {
    let Some(result) = &app.exec_result else {
        return;
    };
    let popup = widgets::popup_area(area, 60, 9);
    frame.render_widget(ratatui::widgets::Clear, popup);
    frame.render_widget(Block::new().style(theme::panel()), popup);
    let inner = pad(popup);

    let lines: Vec<Line> = match result {
        crate::app::Loadable::Loading => vec![Line::from(Span::styled(
            format!("{} executing healthcheck…", widgets::spinner(app.ticks)),
            theme::dim(),
        ))],
        crate::app::Loadable::Failed(message) => vec![
            Line::from(Span::styled(" execute failed ", theme::error())),
            Line::default(),
            Line::from(Span::raw(message.clone())),
            Line::default(),
            Line::from(Span::styled("any key to close", theme::dim())),
        ],
        crate::app::Loadable::Ready(result) => {
            let (label, style) = if result.success {
                (" SUCCESS ", theme::success())
            } else {
                (" FAILED ", theme::error())
            };
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(label, style),
                    Span::styled(
                        format!(
                            "  {}  {} ms",
                            result
                                .status_code
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "—".into()),
                            result.response_time_ms
                        ),
                        theme::panel(),
                    ),
                ]),
                Line::default(),
                Line::from(Span::styled(result.url.clone(), theme::dim())),
            ];
            if let Some(error) = &result.error {
                lines.push(Line::from(Span::styled(
                    error.clone(),
                    ratatui::style::Style::new()
                        .fg(theme::STATUS_DOWN)
                        .bg(theme::BG_PANEL),
                )));
            }
            if let Some(body_match) = result.body_match {
                lines.push(Line::from(Span::styled(
                    format!("body match: {}", if body_match { "yes" } else { "no" }),
                    theme::dim(),
                )));
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::styled("any key to close", theme::dim())));
            lines
        }
        crate::app::Loadable::Idle => return,
    };
    frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
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
        Tab::Entity(crate::api::EntityKind::Healthchecks) => healthchecks::draw(frame, app, area),
        Tab::Entity(kind) => list::draw(frame, app, kind, area),
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let line = match (&app.error, &app.toast) {
        (Some(error), _) => Line::from(Span::styled(format!(" {error} "), theme::error())),
        (None, Some((message, _))) => {
            Line::from(Span::styled(format!(" ✓ {message} "), theme::success()))
        }
        (None, None) => {
            let hints = if !app.detail_stack.is_empty() {
                " j/k move · Enter drill · Esc back · r refresh · q quit "
            } else {
                match app.tab {
                    Tab::Dashboard => {
                        " Tab/1-9 switch · j/k scroll health · h collapse · r refresh · q quit "
                    }
                    Tab::Entity(crate::api::EntityKind::Healthchecks) => {
                        " j/k move · Enter detail · x execute · s/S kuma sync · f filter · q quit "
                    }
                    Tab::Entity(crate::api::EntityKind::Infra) => {
                        " j/k move · Enter detail · s/S sync IPs · n/p page · f filter · q quit "
                    }
                    Tab::Entity(crate::api::EntityKind::Domains) => {
                        " j/k move · Enter detail · d dns · / search · n/p page · f filter · q quit "
                    }
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
