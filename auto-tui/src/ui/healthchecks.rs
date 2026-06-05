use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use serde_json::Value;

use crate::api::EntityKind;
use crate::app::App;

use super::{list, pad, theme};

/// Healthchecks tab: the generic list on the left, a live uptime panel for
/// the selected check on the right.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let [list_area, uptime_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

    list::draw(frame, app, EntityKind::Healthchecks, list_area);
    draw_uptime_panel(frame, app, uptime_area);
}

fn draw_uptime_panel(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::new().style(theme::panel()), area);
    let inner = pad(area);

    let Some(row) = app.list(EntityKind::Healthchecks).selected_row() else {
        frame.render_widget(
            Paragraph::new(Span::styled("no healthcheck selected", theme::dim()))
                .style(theme::panel()),
            inner,
        );
        return;
    };

    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        crate::app::list::row_label(row).to_string(),
        theme::title().bg(theme::BG_PANEL),
    ))];

    let url = format!(
        "{}://{}{}",
        text(row, "protocol"),
        text(row, "domain_fqdn"),
        text(row, "path"),
    );
    lines.push(Line::from(Span::styled(url, theme::dim())));
    lines.push(Line::default());

    let kuma_id = row
        .get("kuma_id")
        .and_then(Value::as_i64)
        .map(|id| id as i32);
    match kuma_id {
        None => lines.push(Line::from(Span::styled(
            "not linked to a Kuma monitor",
            theme::dim(),
        ))),
        Some(kuma_id) if !app.uptime.connected => lines.push(Line::from(Span::styled(
            format!("monitor #{kuma_id} — waiting for uptime stream…"),
            theme::dim(),
        ))),
        Some(kuma_id) => draw_monitor(app, kuma_id, inner.width, &mut lines),
    }

    if row.get("kuma_dirty") == Some(&Value::Bool(true)) {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "⚠ out of sync with Kuma (s to sync)",
            ratatui::style::Style::new()
                .fg(theme::STATUS_PENDING)
                .bg(theme::BG_PANEL),
        )));
    }

    frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
}

fn draw_monitor(app: &App, kuma_id: i32, width: u16, lines: &mut Vec<Line>) {
    let heartbeats = app.uptime.heartbeats(kuma_id);
    if heartbeats.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("monitor #{kuma_id} — no heartbeats yet"),
            theme::dim(),
        )));
        return;
    }

    let latest = heartbeats.last().expect("non-empty");
    let (status_label, status_color) = match latest.status {
        1 => ("UP", theme::STATUS_UP),
        0 => ("DOWN", theme::STATUS_DOWN),
        2 => ("PENDING", theme::STATUS_PENDING),
        3 => ("MAINTENANCE", theme::STATUS_MAINTENANCE),
        _ => ("UNKNOWN", theme::FG_DIM),
    };
    let mut status_spans = vec![
        Span::styled(
            format!(" {status_label} "),
            ratatui::style::Style::new()
                .fg(theme::BG)
                .bg(status_color)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(format!("  monitor #{kuma_id}"), theme::dim()),
    ];
    if let Some(ping) = latest.ping {
        status_spans.push(Span::styled(format!("  {ping} ms"), theme::dim()));
    }
    lines.push(Line::from(status_spans));
    lines.push(Line::default());

    // Heartbeat bar: one cell per beat with a gap in between so each beat
    // reads as its own element; bar height scales with that beat's ping
    // (slower response = taller bar). Newest on the right.
    let capacity = (width.saturating_sub(2) as usize).div_ceil(2);
    let visible = &heartbeats[heartbeats.len().saturating_sub(capacity)..];
    lines.push(Line::from(super::widgets::heartbeat_spans(
        heartbeats,
        capacity,
        theme::BG_PANEL,
    )));

    // Time axis: oldest timestamp left, beat count centered, newest right.
    let oldest = clock(&visible[0].time);
    let newest = clock(&latest.time);
    let middle = format!(" {} beats ", visible.len());
    let bar_width = visible.len() * 2 - 1;
    let side = bar_width.saturating_sub(middle.len()) / 2;
    lines.push(Line::from(Span::styled(
        format!(
            "{oldest:<side$}{middle}{newest:>rest$}",
            rest = bar_width.saturating_sub(side + middle.len())
        ),
        theme::dim(),
    )));
    lines.push(Line::default());

    lines.push(Line::from(Span::styled(
        format!("last beat {}", latest.time),
        theme::dim(),
    )));
    if let Some(msg) = &latest.msg
        && !msg.is_empty()
    {
        lines.push(Line::from(Span::styled(msg.clone(), theme::dim())));
    }
}

/// `HH:MM` out of a `YYYY-MM-DD HH:MM:SS…` timestamp.
fn clock(time: &str) -> &str {
    time.get(11..16).unwrap_or(time)
}

fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key).and_then(Value::as_str).unwrap_or("")
}
