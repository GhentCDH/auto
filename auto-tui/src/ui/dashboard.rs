use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::api::models::DashboardStats;
use crate::app::{App, Loadable};

use super::{pad, theme, widgets};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    match &app.dashboard {
        Loadable::Idle | Loadable::Loading => widgets::draw_loading(frame, area, app.ticks),
        Loadable::Failed(message) => widgets::draw_failed(frame, pad(area), message),
        Loadable::Ready(stats) => draw_stats(frame, app, stats, area),
    }
}

fn draw_stats(frame: &mut Frame, app: &App, stats: &DashboardStats, area: Rect) {
    let entries = health_entries(app);
    let down: Vec<&str> = alert_names(&entries, 0);
    let pending: Vec<&str> = alert_names(&entries, 2);
    let alert_height = (!down.is_empty() as u16) + (!pending.is_empty() as u16);

    // Cards render 3 text lines inside two layers of padding (row + card).
    // The health section is flexible and scrolls internally; collapsed it
    // shrinks to its header line. spacing(1) leaves base-bg gap rows between
    // the panel backgrounds so the sections stay visually separated.
    let health_constraint = if app.health_collapsed {
        Constraint::Length(2)
    } else {
        Constraint::Min(6)
    };
    let [alerts_area, counts_area, health_area, lower_area] = Layout::vertical([
        Constraint::Length(alert_height),
        Constraint::Length(7),
        health_constraint,
        Constraint::Length(10),
    ])
    .spacing(1)
    .areas(area);

    draw_alerts(frame, &down, &pending, alerts_area);
    draw_counts(frame, stats, counts_area);
    draw_health(frame, app, &entries, health_area);

    let [activity_area, domains_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .spacing(1)
            .areas(lower_area);
    draw_activity(frame, stats, activity_area);
    draw_expiring(frame, stats, domains_area);
}

/// Enabled healthchecks with their live status, sorted by severity
/// (down, pending, no-data/maintenance, up) like the web dashboard.
fn health_entries(app: &App) -> Vec<(&serde_json::Value, Option<i32>)> {
    let rows = app.list(crate::api::EntityKind::Healthchecks).rows();
    let mut entries: Vec<(&serde_json::Value, Option<i32>)> = rows
        .iter()
        .filter(|row| row.get("is_enabled") != Some(&serde_json::Value::Bool(false)))
        .map(|row| {
            let status = row
                .get("kuma_id")
                .and_then(serde_json::Value::as_i64)
                .and_then(|id| app.uptime.status(id as i32));
            (row, status)
        })
        .collect();
    entries.sort_by_key(|(_, status)| match status {
        Some(0) => 0,
        Some(2) => 1,
        None | Some(3) => 2,
        _ => 3,
    });
    entries
}

fn alert_names<'a>(entries: &[(&'a serde_json::Value, Option<i32>)], status: i32) -> Vec<&'a str> {
    entries
        .iter()
        .filter(|(_, s)| *s == Some(status))
        .map(|(row, _)| crate::app::list::row_label(row))
        .collect()
}

/// Down/pending banner strips at the top, like the web dashboard alerts.
fn draw_alerts(frame: &mut Frame, down: &[&str], pending: &[&str], area: Rect) {
    if area.height == 0 {
        return;
    }
    let mut lines = Vec::new();
    if !down.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" ▼ {} down: {} ", down.len(), down.join(", ")),
            theme::error(),
        )));
    }
    if !pending.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" ▲ {} pending: {} ", pending.len(), pending.join(", ")),
            ratatui::style::Style::new()
                .fg(theme::BG)
                .bg(theme::STATUS_PENDING)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

/// Live health grid, mirroring the web dashboard: status dot, check name and
/// a heartbeat sparkline per healthcheck. Lays out 1-3 columns depending on
/// width. Scrollable (`j/k`) and collapsible (`h`).
fn draw_health(
    frame: &mut Frame,
    app: &App,
    entries: &[(&serde_json::Value, Option<i32>)],
    area: Rect,
) {
    frame.render_widget(Block::new().style(theme::panel()), area);
    let inner = pad(area);

    let header_hint = if app.health_collapsed {
        format!("Health ({})  ·  h expand", entries.len())
    } else {
        "Health  ·  j/k scroll · h collapse".to_string()
    };
    let header = Line::from(Span::styled(
        header_hint,
        theme::title().bg(theme::BG_PANEL),
    ));
    let [header_area, grid_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    frame.render_widget(Paragraph::new(header).style(theme::panel()), header_area);
    if app.health_collapsed {
        return;
    }

    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("loading healthchecks…", theme::dim()))
                .style(theme::panel()),
            grid_area,
        );
        return;
    }

    // 1-3 columns, ~55 cells each like the web's responsive grid.
    let column_count = ((grid_area.width / 55).clamp(1, 3)) as usize;
    let columns = Layout::horizontal(vec![Constraint::Fill(1); column_count])
        .spacing(2)
        .split(grid_area);

    // Column-major fill: entries flow down the first column, then the next.
    let rows_per_column = entries.len().div_ceil(column_count);
    let max_scroll = rows_per_column.saturating_sub(grid_area.height as usize) as u16;
    let scroll = app.health_scroll.min(max_scroll);

    for (column_index, column) in columns.iter().enumerate() {
        let start = column_index * rows_per_column;
        let chunk =
            &entries[start.min(entries.len())..(start + rows_per_column).min(entries.len())];
        let lines: Vec<Line> = chunk
            .iter()
            .map(|(row, status)| health_line(app, row, *status, column.width))
            .collect();
        frame.render_widget(
            Paragraph::new(lines)
                .style(theme::panel())
                .scroll((scroll, 0)),
            *column,
        );
    }
}

/// One grid cell: dot, fixed-width name, heartbeat bar sized to the column.
fn health_line<'a>(
    app: &App,
    row: &'a serde_json::Value,
    status: Option<i32>,
    width: u16,
) -> Line<'a> {
    const NAME_WIDTH: usize = 24;
    let name = crate::app::list::row_label(row);
    let name = if name.chars().count() > NAME_WIDTH {
        let truncated: String = name.chars().take(NAME_WIDTH - 1).collect();
        format!("{truncated}…")
    } else {
        format!("{name:<NAME_WIDTH$}")
    };
    let mut spans = vec![
        widgets::status_dot(status),
        Span::styled(format!(" {name}  "), theme::panel()),
    ];
    let beats = row
        .get("kuma_id")
        .and_then(serde_json::Value::as_i64)
        .map(|id| app.uptime.heartbeats(id as i32))
        .unwrap_or(&[]);
    if beats.is_empty() {
        spans.push(Span::styled("no data", theme::dim()));
    } else {
        // Short bar on the dashboard — recent trend, not the full window.
        let bar_capacity = ((width as usize).saturating_sub(NAME_WIDTH + 4) / 2).min(20);
        spans.extend(widgets::heartbeat_spans(
            beats,
            bar_capacity,
            theme::BG_PANEL,
        ));
    }
    Line::from(spans)
}

/// Row of count cards, one per entity type — each its own background block.
fn draw_counts(frame: &mut Frame, stats: &DashboardStats, area: Rect) {
    let cards: [(&str, i64, i64); 6] = [
        (
            "Applications",
            stats.applications.total,
            stats.applications.active,
        ),
        ("Services", stats.services.total, stats.services.active),
        ("Infra", stats.infra.total, stats.infra.active),
        ("Domains", stats.domains.total, stats.domains.active),
        ("People", stats.people.total, stats.people.active),
        (
            "Shares",
            stats.network_shares.total,
            stats.network_shares.active,
        ),
    ];

    let columns = Layout::horizontal([Constraint::Ratio(1, 7); 7])
        .spacing(1)
        .split(pad(area));
    for (index, (label, total, active)) in cards.iter().enumerate() {
        draw_card(
            frame,
            columns[index],
            label,
            *total,
            &format!("{active} active"),
        );
    }
    let healthcheck_card = columns[6];
    draw_card(
        frame,
        healthcheck_card,
        "Healthchecks",
        stats.healthchecks.total,
        &format!(
            "{} enabled · {} dirty",
            stats.healthchecks.enabled, stats.healthchecks.kuma_dirty
        ),
    );
}

fn draw_card(frame: &mut Frame, area: Rect, label: &str, total: i64, sub: &str) {
    // Background panel instead of a border.
    frame.render_widget(Block::new().style(theme::panel()), area);
    let lines = vec![
        Line::from(Span::styled(label.to_string(), theme::dim())),
        Line::from(Span::styled(
            total.to_string(),
            theme::title().bg(theme::BG_PANEL),
        )),
        Line::from(Span::styled(sub.to_string(), theme::dim())),
    ];
    frame.render_widget(Paragraph::new(lines).style(theme::panel()), pad(area));
}

fn draw_activity(frame: &mut Frame, stats: &DashboardStats, area: Rect) {
    let inner = pad(area);
    let mut lines = vec![Line::from(Span::styled("Recent activity", theme::title()))];
    for item in &stats.recent_activity {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<14}", item.entity_type), theme::dim()),
            Span::raw(item.name.clone()),
            Span::styled(format!("  {}", item.updated_at), theme::dim()),
        ]));
    }
    if stats.recent_activity.is_empty() {
        lines.push(Line::from(Span::styled("nothing yet", theme::dim())));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_expiring(frame: &mut Frame, stats: &DashboardStats, area: Rect) {
    frame.render_widget(Block::new().style(theme::panel()), area);
    let inner = pad(area);
    let mut lines = vec![Line::from(Span::styled(
        "Expiring domains",
        theme::title().bg(theme::BG_PANEL),
    ))];
    for domain in &stats.expiring_domains {
        lines.push(Line::from(vec![
            Span::raw(domain.fqdn.clone()),
            Span::styled(
                format!("  {}", domain.expires_at.as_deref().unwrap_or("?")),
                theme::dim(),
            ),
        ]));
    }
    if stats.expiring_domains.is_empty() {
        lines.push(Line::from(Span::styled("none expiring soon", theme::dim())));
    }
    frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
}
