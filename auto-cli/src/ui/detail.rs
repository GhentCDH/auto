use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::detail::DetailView;
use crate::app::{App, Loadable};

use super::{pad, theme, widgets};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let Some(view) = app.detail_stack.last() else {
        return;
    };

    match &view.data {
        Loadable::Idle | Loadable::Loading => widgets::draw_loading(frame, area, app.ticks),
        Loadable::Failed(message) => widgets::draw_failed(frame, pad(area), message),
        Loadable::Ready(_) => draw_detail(frame, app, view, area),
    }
}

fn draw_detail(frame: &mut Frame, app: &App, view: &DetailView, area: Rect) {
    let inner = pad(area);
    let [title_area, body_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(inner);

    // Breadcrumb of the drill-down stack.
    let mut crumbs: Vec<Span> = Vec::new();
    for (index, level) in app.detail_stack.iter().enumerate() {
        if index > 0 {
            crumbs.push(Span::styled(" › ", theme::dim()));
        }
        let name = level
            .value()
            .map(crate::app::list::row_label)
            .unwrap_or("…")
            .to_string();
        let style = if index == app.detail_stack.len() - 1 {
            theme::title()
        } else {
            theme::dim()
        };
        crumbs.push(Span::styled(
            format!("{} {name}", level.kind.label()),
            style,
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(crumbs)), title_area);

    let [fields_area, relations_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .areas(body_area);
    draw_fields(frame, view, fields_area);
    draw_relations(frame, view, relations_area);
}

fn draw_fields(frame: &mut Frame, view: &DetailView, area: Rect) {
    let fields = view.scalar_fields();
    let width = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let lines: Vec<Line> = fields
        .iter()
        .map(|(key, value)| {
            Line::from(vec![
                Span::styled(format!("{key:>width$}  "), theme::dim()),
                Span::raw(value.clone()),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_relations(frame: &mut Frame, view: &DetailView, area: Rect) {
    // Panel background separates relations from the scalar column.
    frame.render_widget(Block::new().style(theme::panel()), area);
    let inner = pad(area);

    let items = view.relation_items();
    if items.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("no relations", theme::dim())).style(theme::panel()),
            inner,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let mut last_section = "";
    for (index, item) in items.iter().enumerate() {
        if item.section != last_section {
            if !last_section.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::from(Span::styled(
                item.section.replace('_', " "),
                theme::title().bg(theme::BG_PANEL),
            )));
            last_section = &item.section;
        }
        let style = if index == view.selected {
            theme::selected()
        } else {
            theme::panel()
        };
        let drill = if item.kind.is_some() && item.id.is_some() {
            "↵"
        } else {
            " "
        };
        lines.push(Line::from(Span::styled(
            format!("  {} {drill}", item.label),
            style,
        )));
    }
    frame.render_widget(Paragraph::new(lines).style(theme::panel()), inner);
}
