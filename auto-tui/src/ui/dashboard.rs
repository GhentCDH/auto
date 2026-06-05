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
        Loadable::Ready(stats) => draw_stats(frame, stats, area),
    }
}

fn draw_stats(frame: &mut Frame, stats: &DashboardStats, area: Rect) {
    let [counts_area, lower_area] =
        Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).areas(area);

    draw_counts(frame, stats, counts_area);

    let [activity_area, domains_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(lower_area);
    draw_activity(frame, stats, activity_area);
    draw_expiring(frame, stats, domains_area);
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

    let columns = Layout::horizontal([Constraint::Ratio(1, 7); 7]).split(pad(area));
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
