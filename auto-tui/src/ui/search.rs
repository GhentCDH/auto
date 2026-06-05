use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::{App, Loadable};

use super::{pad, theme, widgets};

/// Global search overlay (`/`): input line plus grouped live results.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let popup = widgets::popup_area(area, area.width.saturating_sub(10).min(90), 24);
    frame.render_widget(Clear, popup);
    frame.render_widget(Block::new().style(theme::panel()), popup);
    let inner = pad(popup);

    let [input_area, results_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("search ", theme::dim()),
            Span::styled(app.search.input.clone(), theme::selected()),
            Span::styled("█", theme::selected()),
        ]))
        .style(theme::panel()),
        input_area,
    );

    match &app.search.results {
        Loadable::Idle => {
            frame.render_widget(
                Paragraph::new(Span::styled("type to search all entities", theme::dim()))
                    .style(theme::panel()),
                results_area,
            );
        }
        Loadable::Loading => widgets::draw_loading(frame, results_area, app.ticks),
        Loadable::Failed(message) => widgets::draw_failed(frame, results_area, message),
        Loadable::Ready(_) => draw_results(frame, app, results_area),
    }
}

fn draw_results(frame: &mut Frame, app: &App, area: Rect) {
    let flat = app.search.flat();
    if flat.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("no matches", theme::dim())).style(theme::panel()),
            area,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let mut last_kind = None;
    for (index, (kind, result)) in flat.iter().enumerate() {
        if last_kind != Some(*kind) {
            if last_kind.is_some() {
                lines.push(Line::default());
            }
            lines.push(Line::from(Span::styled(
                kind.label(),
                theme::title().bg(theme::BG_PANEL),
            )));
            last_kind = Some(*kind);
        }
        let style = if index == app.search.selected {
            theme::selected()
        } else {
            theme::panel()
        };
        let mut spans = vec![Span::styled(format!("  {}", result.name), style)];
        if let Some(description) = &result.description
            && !description.is_empty()
        {
            spans.push(Span::styled(format!("  {description}"), theme::dim()));
        }
        lines.push(Line::from(spans));
    }
    // Keep the selection in view by scrolling past earlier lines if needed.
    let selected_line = lines
        .iter()
        .position(|line| line.spans.first().map(|s| s.style) == Some(theme::selected()))
        .unwrap_or(0);
    let scroll = (selected_line as u16).saturating_sub(area.height.saturating_sub(2));
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme::panel())
            .scroll((scroll, 0)),
        area,
    );
}
