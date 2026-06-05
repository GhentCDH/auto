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
    // Cards render 3 text lines inside two layers of padding (row + card).
    // The health section is flexible and scrolls internally; collapsed it
    // shrinks to its header line.
    let health_constraint = if app.health_collapsed {
        Constraint::Length(2)
    } else {
        Constraint::Min(6)
    };
    let [counts_area, health_area, lower_area] = Layout::vertical([
        Constraint::Length(7),
        health_constraint,
        Constraint::Length(10),
    ])
    .areas(area);

    draw_counts(frame, stats, counts_area);
    draw_health(frame, app, health_area);

    let [activity_area, domains_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(lower_area);
    draw_activity(frame, stats, activity_area);
    draw_expiring(frame, stats, domains_area);
}

/// Live health grid, mirroring the web dashboard: status dot, check name and
/// a heartbeat sparkline per healthcheck. Scrollable (`j/k`) and collapsible
/// (`h`).
fn draw_health(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::new().style(theme::panel()), area);
    let inner = pad(area);

    let rows = app.list(crate::api::EntityKind::Healthchecks).rows();
    let header_hint = if app.health_collapsed {
        format!("Health ({})  ·  h expand", rows.len())
    } else {
        "Health  ·  j/k scroll · h collapse".to_string()
    };
    let mut lines = vec![Line::from(Span::styled(
        header_hint,
        theme::title().bg(theme::BG_PANEL),
    ))];
    if app.health_collapsed {
        frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
        return;
    }

    if rows.is_empty() {
        lines.push(Line::from(Span::styled(
            "loading healthchecks…",
            theme::dim(),
        )));
        frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
        return;
    }

    // Sort like the web alerts: down first, then pending, unknown, up.
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

    const NAME_WIDTH: usize = 28;
    let bar_capacity = ((inner.width as usize).saturating_sub(NAME_WIDTH + 4) / 2).min(48);
    for (row, status) in &entries {
        let name = crate::app::list::row_label(row);
        let name = if name.chars().count() > NAME_WIDTH {
            let truncated: String = name.chars().take(NAME_WIDTH - 1).collect();
            format!("{truncated}…")
        } else {
            format!("{name:<NAME_WIDTH$}")
        };
        let mut spans = vec![
            widgets::status_dot(*status),
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
            spans.extend(widgets::heartbeat_spans(
                beats,
                bar_capacity,
                theme::BG_PANEL,
            ));
        }
        lines.push(Line::from(spans));
    }

    // Clamp the scroll so the last line stays visible.
    let visible = inner.height.saturating_sub(1) as usize;
    let max_scroll = lines.len().saturating_sub(visible) as u16;
    let scroll = app.health_scroll.min(max_scroll);
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme::panel())
            .scroll((scroll, 0)),
        inner,
    );
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
