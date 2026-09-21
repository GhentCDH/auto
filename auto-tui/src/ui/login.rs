//! Login screen shown before the main UI when the client is unauthenticated.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::login::{Field, LoginView, Mode};
use crate::ui::{pad, theme};

pub fn draw(frame: &mut Frame, view: &LoginView) {
    frame.render_widget(Block::new().style(theme::base()), frame.area());

    let [_, middle, _] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(14),
        Constraint::Min(0),
    ])
    .areas(frame.area());
    let [_, panel, _] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(62),
        Constraint::Min(0),
    ])
    .areas(middle);

    frame.render_widget(Block::new().style(theme::panel()), panel);
    let inner = pad(panel);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " Sign in ",
        theme::title().bg(theme::BG_PANEL),
    )));
    lines.push(Line::from(Span::styled(
        format!(" {}", view.url),
        theme::dim(),
    )));
    if let Some(reason) = &view.reason {
        lines.push(Line::from(Span::styled(
            format!(" {reason}"),
            Style::new().fg(theme::STATUS_PENDING),
        )));
    }
    lines.push(Line::default());

    // Primary: browser loopback login (pre-selected).
    let browser_active = matches!(view.mode, Mode::Browser);
    let marker = if browser_active { ">" } else { " " };
    lines.push(Line::from(Span::styled(
        format!(" {marker} Sign in with browser"),
        if browser_active {
            theme::selected()
        } else {
            theme::panel()
        },
    )));
    if !view.status.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("     {}", view.status),
            theme::dim(),
        )));
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "   or sign in with a password",
        theme::dim(),
    )));

    let in_password = matches!(view.mode, Mode::Password);
    let username_active = in_password && matches!(view.field, Field::Username);
    let password_active = in_password && matches!(view.field, Field::Password);

    let username = if view.username.is_empty() {
        "username".to_string()
    } else {
        view.username.clone()
    };
    let password = if view.password.is_empty() {
        "password".to_string()
    } else {
        "*".repeat(view.password.chars().count())
    };
    lines.push(Line::from(Span::styled(
        format!("   Username: {username}"),
        if username_active {
            theme::selected()
        } else {
            theme::dim()
        },
    )));
    lines.push(Line::from(Span::styled(
        format!("   Password: {password}"),
        if password_active {
            theme::selected()
        } else {
            theme::dim()
        },
    )));

    if let Some(error) = &view.error {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            format!(" {error}"),
            Style::new().fg(theme::STATUS_DOWN),
        )));
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        " Enter select · Tab/arrows move · q quit ",
        theme::footer(),
    )));

    frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
}
