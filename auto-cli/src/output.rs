//! Printing CLI results: JSON for scripts and agents, tables for people.
//!
//! Tables reuse the TUI's column definitions (`view`) and theme. They are
//! drawn with ratatui into an off-screen `Buffer` and then printed as styled
//! lines, so no terminal setup (raw mode, alternate screen) is involved and
//! the output stays plain text in the scrollback.

use std::io::{self, IsTerminal, Write};

use clap::ValueEnum;
use ratatui::backend::IntoCrossterm;
use ratatui::buffer::{Buffer, Cell};
use ratatui::crossterm::queue;
use ratatui::crossterm::style::{Print, ResetColor, SetAttribute, SetStyle};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Row, Table, Widget};
use serde_json::Value;

use crate::api::EntityKind;
use crate::openapi::{EntitySchema, Field};
use crate::ui::theme;
use crate::view;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Coloured tables (default on a terminal).
    Table,
    /// Tables without colour.
    Plain,
    /// JSON (default when stdout is not a terminal).
    Json,
}

impl Format {
    pub fn resolve(flag: Option<Format>, json: bool) -> Self {
        match (json, flag) {
            (true, _) => Format::Json,
            (false, Some(format)) => format,
            (false, None) if !io::stdout().is_terminal() => Format::Json,
            (false, None) if std::env::var_os("NO_COLOR").is_some() => Format::Plain,
            (false, None) => Format::Table,
        }
    }

    fn color(self) -> bool {
        self == Format::Table
    }
}

pub fn json(value: &(impl serde::Serialize + ?Sized)) -> io::Result<()> {
    let text = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    writeln!(io::stdout().lock(), "{text}")
}

/// A status line for people, always on stderr so stdout stays parseable.
pub fn note(message: &str) {
    eprintln!("{message}");
}

pub fn list(kind: EntityKind, response: &Value, format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(response);
    }
    let rows = response
        .get("data")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if rows.is_empty() {
        note("no results");
        return Ok(());
    }
    let columns = view::columns(kind);
    let headers: Vec<&str> = std::iter::once("ID")
        .chain(columns.iter().map(|(label, _)| *label))
        .collect();
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            std::iter::once(short_id(row))
                .chain(columns.iter().map(|(_, key)| view::cell_text(row, key)))
                .collect()
        })
        .collect();
    table(&headers, &cells, format)?;

    let number = |key| response.get(key).and_then(Value::as_u64).unwrap_or(0);
    if number("total_pages") > 1 {
        note(&format!(
            "page {}/{} · {} total (use --page or --all)",
            number("page"),
            number("total_pages"),
            number("total")
        ));
    }
    Ok(())
}

pub fn detail(value: &Value, format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(value);
    }
    let fields: Vec<Vec<String>> = view::scalar_fields(value)
        .into_iter()
        .map(|(key, text)| vec![key, text])
        .collect();
    key_values(&fields, format)?;

    let items = view::relation_items(value);
    items
        .chunk_by(|a, b| a.section == b.section)
        .try_for_each(|group| {
            println!();
            heading(&format!("{} ({})", group[0].section, group.len()), format)?;
            let rows: Vec<Vec<String>> = group
                .iter()
                .map(|item| {
                    let id = item.id.as_deref().map(short).unwrap_or_default();
                    vec![id.to_string(), item.label.clone()]
                })
                .collect();
            table(&["ID", "Name"], &rows, format)
        })
}

pub fn search(results: &Value, format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(results);
    }
    let rows: Vec<Vec<String>> = results
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(section, hits)| hits.as_array().map(|hits| (section, hits)))
        .flat_map(|(section, hits)| {
            hits.iter().map(move |hit| {
                let text = |key| {
                    hit.get(key)
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                };
                vec![
                    section.clone(),
                    short_id(hit),
                    text("name"),
                    text("description"),
                ]
            })
        })
        .collect();
    match rows.is_empty() {
        true => {
            note("no results");
            Ok(())
        }
        false => table(&["Type", "ID", "Name", "Description"], &rows, format),
    }
}

/// Healthcheck states from `GET /api/healthchecks/status` (already sorted
/// worst first), with a count per state on stderr.
pub fn status(rows: &Value, format: Format) -> io::Result<()> {
    let rows = rows.as_array().map(Vec::as_slice).unwrap_or_default();
    let state_of = |row: &Value| {
        row.get("state")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string()
    };
    let mut counts: Vec<(String, usize)> = Vec::new();
    rows.iter().map(state_of).for_each(|state| {
        match counts.iter_mut().find(|(known, _)| *known == state) {
            Some((_, count)) => *count += 1,
            None => counts.push((state, 1)),
        }
    });
    let summary: Vec<String> = counts
        .iter()
        .map(|(state, count)| format!("{count} {state}"))
        .collect();
    note(&match summary.is_empty() {
        true => "no healthchecks match".to_string(),
        false => summary.join(", "),
    });
    if format == Format::Json {
        return json(rows);
    }
    if rows.is_empty() {
        return Ok(());
    }
    let text = |row: &Value, key: &str| {
        row.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let owner = ["application_name", "service_name"]
                .iter()
                .find_map(|key| row.get(key).and_then(Value::as_str))
                .unwrap_or("")
                .to_string();
            let since: String = text(row, "since").chars().take(16).collect();
            vec![
                state_of(row),
                text(row, "name"),
                owner,
                since,
                text(row, "message"),
                text(row, "url"),
            ]
        })
        .collect();
    table(
        &["State", "Name", "For", "Since", "Message", "URL"],
        &cells,
        format,
    )
}

pub fn here(here: &crate::here::Here, format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(here);
    }
    let repo = &here.repo;
    println!(
        "repo {} ({})",
        repo.name,
        repo.remote.as_deref().unwrap_or("no remote")
    );
    match here.matches.is_empty() {
        true => note("no matching application or service in auto"),
        false => {
            let rows: Vec<Vec<String>> = here
                .matches
                .iter()
                .map(|m| {
                    vec![
                        m.entity.clone(),
                        m.id.get(..8).unwrap_or(&m.id).to_string(),
                        m.name.clone(),
                        m.environment.clone().unwrap_or_default(),
                        m.matched_by.clone(),
                    ]
                })
                .collect();
            table(&["Type", "ID", "Name", "Env", "Matched by"], &rows, format)?;
        }
    }
    match here.deploy_repos.is_empty() {
        true => note("no deploy repos configured (deploy_repos in ~/.config/auto/config.toml)"),
        false => {
            println!();
            deploy_repos(&here.deploy_repos, format)?;
        }
    }
    Ok(())
}

pub fn deploy_repos(repos: &[crate::here::DeployRepo], format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(repos);
    }
    let rows: Vec<Vec<String>> = repos
        .iter()
        .map(|repo| {
            let state = match (repo.exists, repo.behind, repo.fetched) {
                (false, _, _) => "missing".to_string(),
                (true, _, Some(false)) => "fetch failed".to_string(),
                (true, Some(0), _) => "up to date".to_string(),
                (true, Some(n), _) => format!("{n} behind"),
                (true, None, _) => "no upstream".to_string(),
            };
            let dirty = if repo.dirty == Some(true) {
                ", local changes"
            } else {
                ""
            };
            vec![
                repo.path.display().to_string(),
                repo.kinds.join(","),
                repo.branch.clone().unwrap_or_default(),
                repo.last_commit.clone().unwrap_or_default(),
                format!("{state}{dirty}"),
            ]
        })
        .collect();
    table(
        &["Deploy repo", "Holds", "Branch", "Last commit", "State"],
        &rows,
        format,
    )
}

pub fn schema(schema: &EntitySchema, format: Format) -> io::Result<()> {
    if format == Format::Json {
        return json(schema);
    }
    let field_rows = |fields: &[Field]| -> Vec<Vec<String>> {
        fields
            .iter()
            .map(|field| {
                let default = field.default.as_ref().map(|value| match value {
                    Value::String(text) => format!("default {text}"),
                    other => format!("default {other}"),
                });
                let values = (!field.enum_values.is_empty())
                    .then(|| format!("one of: {}", field.enum_values.join(", ")));
                let description = [field.description.clone(), default, values]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("; ");
                let required = if field.required { "yes" } else { "" };
                vec![
                    field.name.clone(),
                    field.ty.clone(),
                    required.into(),
                    description,
                ]
            })
            .collect()
    };
    let headers = ["Field", "Type", "Required", "Description"];

    heading(&format!("create {}", schema.entity), format)?;
    table(&headers, &field_rows(&schema.create), format)?;
    println!();
    heading(&format!("update {} (all optional)", schema.entity), format)?;
    table(&headers, &field_rows(&schema.update), format)?;
    if !schema.list_filters.is_empty() {
        println!();
        heading("list filters", format)?;
        table(&headers, &field_rows(&schema.list_filters), format)?;
    }
    schema.relations.iter().try_for_each(|relation| {
        println!();
        heading(&format!("link {} {}", schema.entity, relation.name), format)?;
        match relation.fields.is_empty() {
            true => {
                println!("(no fields)");
                Ok(())
            }
            false => table(&headers, &field_rows(&relation.fields), format),
        }
    })
}

/// Print any JSON result: strings as-is, everything else as a detail view.
pub fn value(value: &Value, format: Format) -> io::Result<()> {
    match (value, format) {
        (_, Format::Json) => json(value),
        (Value::String(text), _) => writeln!(io::stdout().lock(), "{text}"),
        (Value::Null, _) => Ok(()),
        (Value::Object(_), _) => detail(value, format),
        _ => json(value),
    }
}

fn short(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn short_id(row: &Value) -> String {
    row.get("id")
        .and_then(Value::as_str)
        .map(short)
        .unwrap_or_default()
        .to_string()
}

fn heading(text: &str, format: Format) -> io::Result<()> {
    let style = if format.color() {
        theme::title()
    } else {
        Style::new()
    };
    let line = Span::styled(text.to_string(), style);
    let width = line.width() as u16;
    let mut buffer = Buffer::empty(Rect::new(0, 0, width.max(1), 1));
    line.render(buffer.area, &mut buffer);
    print_buffer(&buffer, format.color())
}

fn key_values(rows: &[Vec<String>], format: Format) -> io::Result<()> {
    render_table(None, rows, format, &[theme::dim(), Style::new()])
}

fn table(headers: &[&str], rows: &[Vec<String>], format: Format) -> io::Result<()> {
    render_table(Some(headers), rows, format, &[theme::dim()])
}

/// Render rows as a table sized to its content, shrunk to the terminal width
/// when needed. `column_styles[i]` styles column `i` (missing = unstyled).
fn render_table(
    headers: Option<&[&str]>,
    rows: &[Vec<String>],
    format: Format,
    column_styles: &[Style],
) -> io::Result<()> {
    let column_count = headers
        .map(<[&str]>::len)
        .or_else(|| rows.first().map(Vec::len))
        .unwrap_or(0);
    if column_count == 0 {
        return Ok(());
    }
    let text_width = |text: &str| Span::raw(text).width();
    let widths: Vec<usize> = (0..column_count)
        .map(|column| {
            let header = headers.map(|h| text_width(h[column])).unwrap_or(0);
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| text_width(cell))
                .chain(std::iter::once(header))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let widths = fit_widths(&widths, terminal_width());

    let color = format.color();
    let style_of = |column: usize| match color {
        true => column_styles.get(column).copied().unwrap_or_default(),
        false => Style::new(),
    };
    let body = rows.iter().map(|row| {
        Row::new(
            row.iter()
                .enumerate()
                .map(|(column, cell)| Span::styled(cell.as_str(), style_of(column))),
        )
    });
    let constraints = widths.iter().map(|w| Constraint::Length(*w as u16));
    let mut widget = Table::new(body, constraints);
    if let Some(headers) = headers {
        let style = if color { theme::title() } else { Style::new() };
        widget = widget.header(Row::new(headers.iter().copied()).style(style));
    }

    let total_width = widths.iter().sum::<usize>() + widths.len().saturating_sub(1);
    let height = rows.len() + usize::from(headers.is_some());
    let area = Rect::new(0, 0, total_width.max(1) as u16, height as u16);
    let mut buffer = Buffer::empty(area);
    Widget::render(widget, area, &mut buffer);
    print_buffer(&buffer, color)
}

/// Shrink the widest columns until the table (plus one space between
/// columns) fits `available`. Columns at or under the cap keep their width.
fn fit_widths(widths: &[usize], available: usize) -> Vec<usize> {
    let gaps = widths.len().saturating_sub(1);
    let fits = |cap: usize| widths.iter().map(|w| (*w).min(cap)).sum::<usize>() + gaps <= available;
    let widest = widths.iter().copied().max().unwrap_or(0);
    let cap = (8..=widest).rev().find(|cap| fits(*cap)).unwrap_or(8);
    widths.iter().map(|w| (*w).min(cap)).collect()
}

fn terminal_width() -> usize {
    // Some ptys (CI, `script`) report a width of 0: treat that as unknown.
    ratatui::crossterm::terminal::size()
        .map(|(width, _)| width as usize)
        .ok()
        .filter(|width| *width >= 20 && io::stdout().is_terminal())
        .unwrap_or(120)
}

/// Print a buffer line by line, trimming trailing blanks. Cells hidden behind
/// a wide character are skipped, the same way ratatui's own diff does.
fn print_buffer(buffer: &Buffer, color: bool) -> io::Result<()> {
    let mut out = io::stdout().lock();
    buffer
        .content
        .chunks(buffer.area.width as usize)
        .try_for_each(|line| {
            let visible: Vec<&Cell> = line
                .iter()
                .scan(0usize, |hidden, cell| {
                    if *hidden > 0 {
                        *hidden -= 1;
                        return Some(None);
                    }
                    *hidden = Span::raw(cell.symbol()).width().saturating_sub(1);
                    Some(Some(cell))
                })
                .flatten()
                .collect();
            let end = visible
                .iter()
                .rposition(|cell| cell.symbol() != " ")
                .map_or(0, |index| index + 1);
            visible[..end]
                .chunk_by(|a, b| a.style() == b.style())
                .try_for_each(|run| {
                    if color {
                        queue!(out, SetStyle(run[0].style().into_crossterm()))?;
                    }
                    run.iter()
                        .try_for_each(|cell| queue!(out, Print(cell.symbol())))?;
                    if color {
                        queue!(
                            out,
                            SetAttribute(ratatui::crossterm::style::Attribute::Reset),
                            ResetColor
                        )?;
                    }
                    Ok::<_, io::Error>(())
                })?;
            queue!(out, Print("\n"))
        })?;
    out.flush()
}
