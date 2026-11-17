use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Row, Table, TableState};

use crate::api::EntityKind;
use crate::app::list::{cell_text, columns};
use crate::app::{App, Loadable};

use super::{pad, theme, widgets};

pub fn draw(frame: &mut Frame, app: &App, kind: EntityKind, area: Rect) {
    let list = app.list(kind);
    let inner = pad(area);

    let [header_area, table_area, pager_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(inner);

    draw_header(frame, kind, list, header_area);

    match &list.data {
        Loadable::Idle | Loadable::Loading => widgets::draw_loading(frame, table_area, app.ticks),
        Loadable::Failed(message) => widgets::draw_failed(frame, table_area, message),
        Loadable::Ready(resp) if resp.data.is_empty() => {
            frame.render_widget(
                Paragraph::new(Span::styled("no results", theme::dim())).centered(),
                table_area,
            );
        }
        Loadable::Ready(resp) => {
            draw_table(frame, kind, list.selected, &resp.data, table_area);
            draw_pager(frame, resp.page, resp.total_pages, resp.total, pager_area);
        }
    }
}

fn draw_header(
    frame: &mut Frame,
    kind: EntityKind,
    list: &crate::app::list::EntityList,
    area: Rect,
) {
    let line = match &list.filter_input {
        // Active filter input: editable line with a block cursor.
        Some(input) => Line::from(vec![
            Span::styled("filter ", theme::dim()),
            Span::styled(input.clone(), theme::selected()),
            Span::styled("█", theme::selected()),
            Span::styled("  Enter apply · Esc cancel", theme::dim()),
        ]),
        None if !list.filter.is_empty() => Line::from(vec![
            Span::styled(kind.label(), theme::title()),
            Span::styled(format!("  filtered: \"{}\"", list.filter), theme::dim()),
        ]),
        None => Line::from(Span::styled(kind.label(), theme::title())),
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_table(
    frame: &mut Frame,
    kind: EntityKind,
    selected: usize,
    rows: &[serde_json::Value],
    area: Rect,
) {
    let cols = columns(kind);
    let header = Row::new(cols.iter().map(|(label, _)| *label)).style(theme::dim());
    let table_rows = rows.iter().map(|row| {
        Row::new(
            cols.iter()
                .map(|(_, key)| cell_text(row, key))
                .collect::<Vec<_>>(),
        )
    });
    // First column (name) gets more room than the rest.
    let mut constraints = vec![Constraint::Fill(2)];
    constraints.extend(std::iter::repeat_n(Constraint::Fill(1), cols.len() - 1));

    let table = Table::new(table_rows, constraints)
        .header(header)
        .row_highlight_style(theme::selected());
    let mut state = TableState::default().with_selected(Some(selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_pager(frame: &mut Frame, page: u32, total_pages: u32, total: i64, area: Rect) {
    let line = Line::from(Span::styled(
        format!("page {page}/{} · {total} total", total_pages.max(1)),
        theme::dim(),
    ));
    frame.render_widget(Paragraph::new(line).right_aligned(), area);
}
