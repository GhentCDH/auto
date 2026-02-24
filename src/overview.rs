/*!
* Generate a markdown overview of applications and services,
* with zero-width Unicode markers for idempotent Outline sync.
*/

use crate::AppState;
use crate::models::{ApplicationWithRelations, ServiceWithRelations};
use itertools::Itertools;
use std::fmt::Write;
use std::sync::LazyLock;

pub trait Overview {
    fn to_md(&self, state: &AppState) -> String;
}

/// Helper: append a row to the markdown table only if the value is present
fn row(md: &mut String, label: &str, value: &str) {
    writeln!(md, "| {label} | {value} |").unwrap();
}

/// Helper: append a row to the markdown table only if the value is present
fn header(md: &mut String, label: &str, value: &str) {
    writeln!(md, "| {label} | {value} |").unwrap();
    writeln!(md, "|---------|---------|").unwrap();
}

// ---------------------------------------------------------------------------
// Zero-width marker encoding
// ---------------------------------------------------------------------------

/// Encode an ASCII string as a sequence of zero-width characters.
/// Each bit of each byte is encoded as either:
/// - `\u{200B}` (Zero-Width Space)  for bit `1`
/// - `\u{200D}` (Zero-Width Joiner) for bit `0`
fn encode_zwc(tag: &str) -> String {
    let mut out = String::new();
    for byte in tag.bytes() {
        for bit in (0..8).rev() {
            if (byte >> bit) & 1 == 1 {
                out.push('\u{200B}');
            } else {
                out.push('\u{200D}');
            }
        }
    }
    out
}

static MARKER_START: LazyLock<String> = LazyLock::new(|| encode_zwc("<auto>"));
static MARKER_END: LazyLock<String> = LazyLock::new(|| encode_zwc("</auto>"));

/// Wrap `new_overview` in start/end markers and splice it into `existing_text`.
///
/// - If both markers are found, the block between them (inclusive) is replaced.
/// - Otherwise the marked block is prepended to the existing text.
pub fn splice_overview(existing_text: &str, new_overview: &str) -> String {
    let marked = format!("{}{}{}", *MARKER_START, new_overview, *MARKER_END);

    if let Some(start_pos) = existing_text.find(&*MARKER_START)
        && let Some(end_offset) = existing_text[start_pos..].find(&*MARKER_END)
    {
        let end_pos = start_pos + end_offset + MARKER_END.len();
        // Consume a trailing newline if present
        let end_pos = if existing_text[end_pos..].starts_with('\n') {
            end_pos + 1
        } else {
            end_pos
        };
        let mut result = String::with_capacity(existing_text.len());
        result.push_str(&existing_text[..start_pos]);
        result.push_str(&marked);
        result.push_str(&existing_text[end_pos..]);
        return result;
    }

    // No existing markers found — prepend
    format!("{marked}\n\n{existing_text}")
}

// ---------------------------------------------------------------------------
// Overview implementations
// ---------------------------------------------------------------------------

impl Overview for ApplicationWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.application.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        header(
            &mut md,
            "Auto",
            &format!("{}/{}", state.config.domain, &self.application.id[..8]),
        );

        if let Some(url) = &self.application.url {
            row(&mut md, "URL", &format!("[{url}]({url})"));
        }

        if let Some(repo) = &self.application.repository_url {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.stacks.is_empty() {
            row(
                &mut md,
                "Stack",
                &self.stacks.iter().map(|s| &*s.name).join(", "),
            );
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self.infra.iter().map(|i| &*i.name).join(", "),
            );
        }

        if !self.network_shares.is_empty() {
            row(
                &mut md,
                "Storage",
                &self.network_shares.iter().map(|s| &*s.name).join(", "),
            );
        }

        if !self.people.is_empty() {
            let people = self
                .people
                .iter()
                .map(|p| format!("{} ({})", p.name, p.contribution_type))
                .join(", ");
            row(&mut md, "People", &people);
        }

        md
    }
}

impl Overview for ServiceWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.service.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        header(
            &mut md,
            "Auto",
            &format!("{}/{}", state.config.domain, &self.service.id[..8]),
        );

        if let Some(repo) = &self.service.repository_url {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self.infra.iter().map(|i| &*i.name).join(", "),
            );
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_distinct() {
        assert_ne!(*MARKER_START, *MARKER_END);
    }

    #[test]
    fn splice_prepends_when_no_markers() {
        let existing = "# Hello\n\nSome content";
        let overview = "| Auto | test |\n|---------|---------|";
        let result = splice_overview(existing, overview);
        assert!(result.starts_with(&*MARKER_START));
        assert!(result.contains(overview));
        assert!(result.ends_with(existing));
    }

    #[test]
    fn splice_replaces_existing_block() {
        let overview_old = "old overview";
        let overview_new = "new overview";
        let existing = format!(
            "{}\n{}\n{}\n\n# Rest of doc",
            *MARKER_START, overview_old, *MARKER_END
        );
        let result = splice_overview(&existing, overview_new);
        assert!(result.contains(overview_new));
        assert!(!result.contains(overview_old));
        assert!(result.contains("# Rest of doc"));
        // Should only have one pair of markers
        assert_eq!(result.matches(&*MARKER_START).count(), 1);
        assert_eq!(result.matches(&*MARKER_END).count(), 1);
    }
}
