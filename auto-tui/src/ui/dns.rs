use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Row, Table};

use crate::api::models::DnsLookup;
use crate::app::{App, Loadable};

use super::{pad, theme, widgets};

/// DNS records overlay for a domain (`d` on the Domains tab).
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let Some((label, lookup)) = &app.dns else {
        return;
    };

    let popup = widgets::popup_area(area, area.width.saturating_sub(8).min(100), 20);
    frame.render_widget(Clear, popup);
    frame.render_widget(Block::new().style(theme::panel()), popup);
    let inner = pad(popup);

    let [title_area, body_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(inner);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("DNS ", theme::dim()),
            Span::styled(label.clone(), theme::title().bg(theme::BG_PANEL)),
            Span::styled("   Esc close", theme::dim()),
        ]))
        .style(theme::panel()),
        title_area,
    );

    match lookup {
        Loadable::Idle | Loadable::Loading => widgets::draw_loading(frame, body_area, app.ticks),
        Loadable::Failed(message) => widgets::draw_failed(frame, body_area, message),
        Loadable::Ready(lookup) => draw_records(frame, lookup, body_area),
    }
}

fn draw_records(frame: &mut Frame, lookup: &DnsLookup, area: Rect) {
    if let Some(error) = &lookup.error {
        widgets::draw_failed(frame, area, error);
        return;
    }
    if lookup.records.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("no records", theme::dim())).style(theme::panel()),
            area,
        );
        return;
    }

    let header = Row::new(["Type", "Value", "TTL", "Prio", "Infra"]).style(theme::dim());
    let rows = lookup.records.iter().map(|record| {
        Row::new([
            record.record_type.clone(),
            record.value.clone(),
            record.ttl.to_string(),
            record.priority.map(|p| p.to_string()).unwrap_or_default(),
            record
                .infra
                .as_ref()
                .map(|infra| format!("→ {}", infra.name))
                .unwrap_or_default(),
        ])
        .style(theme::panel())
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Fill(3),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .style(theme::panel());
    frame.render_widget(table, area);
}
