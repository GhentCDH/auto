/*!
* Generate a markdown overview of applications and services,
* with zero-width Unicode markers for idempotent Outline sync.
*/

use crate::AppState;
use crate::models::{ApplicationWithRelations, ServiceWithRelations};
use itertools::Itertools;
use std::fmt::Write;
use std::sync::LazyLock;

/// The set of zero-width characters used for encoding.
/// Adding more characters reduces the number of characters needed per byte.
/// Must have a power-of-2 length (2, 4, 8, or 16).
const ZWC_ALPHABET: &[char] = &[
    '\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{2061}', '\u{2062}', '\u{2063}', '\u{2064}',
    '\u{206A}', '\u{206B}', '\u{206C}', '\u{206D}', '\u{206E}', '\u{206F}', '\u{FEFF}', '\u{FFA0}',
];

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

/// Encode an ASCII string as a sequence of zero-width characters.
/// Uses `ZWC_ALPHABET.len()` as the base, encoding each byte in
/// `ceil(8 / log2(alphabet_len))` characters.
///
/// # Panics
/// Panics if `ZWC_ALPHABET` length is not a power of 2 between 2 and 256.
fn encode_zwc(tag: &str) -> String {
    let base = ZWC_ALPHABET.len();
    assert!(base.is_power_of_two() && (2..=256).contains(&base));

    let bits_per_char = base.ilog2() as usize;
    let chars_per_byte = 8usize.div_ceil(bits_per_char);
    let mask = base - 1;

    let mut out = String::new();
    for byte in tag.bytes() {
        // Iterate over chunks from most-significant to least-significant
        for i in (0..chars_per_byte).rev() {
            let index = (byte as usize >> (i * bits_per_char)) & mask;
            out.push(ZWC_ALPHABET[index]);
        }
    }
    out
}

/// Number of times each marker is repeated for redundancy.
/// Detection succeeds if *any single* copy survives editing.
const MARKER_REPETITIONS: usize = 3;

/// A thin spacer placed between repetitions so that a single localised edit
/// is less likely to destroy adjacent copies. Uses a Unicode character that is
/// unlikely to appear in the tag encoding itself.
const MARKER_SEP: char = '\u{2028}'; // Line Separator — invisible in most renderers

static MARKER_START_SINGLE: LazyLock<String> = LazyLock::new(|| encode_zwc("<auto>"));
static MARKER_END_SINGLE: LazyLock<String> = LazyLock::new(|| encode_zwc("</auto>"));

/// Build the full (repeated) marker string.
fn repeated_marker(single: &str) -> String {
    let mut out = String::new();
    for i in 0..MARKER_REPETITIONS {
        if i > 0 {
            out.push(MARKER_SEP);
        }
        out.push_str(single);
    }
    out
}

static MARKER_START: LazyLock<String> = LazyLock::new(|| repeated_marker(&MARKER_START_SINGLE));
static MARKER_END: LazyLock<String> = LazyLock::new(|| repeated_marker(&MARKER_END_SINGLE));

/// Find the *earliest* surviving copy of a marker in `text`.
/// Returns `Some((byte_start, byte_end))` of the single copy that matched.
fn find_marker(text: &str, single: &str) -> Option<(usize, usize)> {
    text.find(single).map(|pos| (pos, pos + single.len()))
}

/// Find the *last* surviving copy of a marker in `text`, searching from `after`.
/// Returns `Some((byte_start, byte_end))` of the single copy that matched.
fn rfind_marker(text: &str, after: usize, single: &str) -> Option<(usize, usize)> {
    text[after..].rfind(single).map(|off| {
        let pos = after + off;
        (pos, pos + single.len())
    })
}

/// Expand a matched marker span outward to consume any adjacent repetitions
/// and separators that are part of the same marker block.
fn expand_marker_span(text: &str, matched: (usize, usize), single: &str) -> (usize, usize) {
    let sep_len = MARKER_SEP.len_utf8();
    let single_len = single.len();
    let (mut start, mut end) = matched;

    // Expand backwards: consume (separator? + single) before current start
    loop {
        let mut candidate = start;
        if candidate >= sep_len
            && text.is_char_boundary(candidate - sep_len)
            && text[(candidate - sep_len)..candidate].starts_with(MARKER_SEP)
        {
            candidate -= sep_len;
        }
        if candidate >= single_len
            && text.is_char_boundary(candidate - single_len)
            && text[(candidate - single_len)..candidate] == *single
        {
            start = candidate - single_len;
            continue;
        }
        break;
    }

    // Expand forwards: consume (single + separator?) after current end
    loop {
        let mut candidate = end;
        if text[candidate..].starts_with(MARKER_SEP) {
            candidate += sep_len;
        }
        if text[candidate..].starts_with(single) {
            end = candidate + single_len;
            continue;
        }
        break;
    }

    (start, end)
}

/// Wrap `new_overview` in start/end markers and splice it into `existing_text`.
///
/// Detection is resilient: if even a single repetition of each marker survives
/// editing, the block is located and replaced. On replacement the full set of
/// repetitions is written back, restoring redundancy.
///
/// - If both markers are found, the block between them (inclusive) is replaced.
/// - Otherwise the marked block is prepended to the existing text.
pub fn splice_overview(existing_text: &str, new_overview: &str) -> String {
    let marked = format!("{}{}{}", *MARKER_START, new_overview, *MARKER_END);

    let start_single = &*MARKER_START_SINGLE;
    let end_single = &*MARKER_END_SINGLE;

    // Find the earliest start marker and the latest end marker
    if let Some(start_match) = find_marker(existing_text, start_single) {
        let search_after = start_match.1;
        if let Some(end_match) = rfind_marker(existing_text, search_after, end_single) {
            // Expand both spans to cover all adjacent repetitions
            let (start_pos, _) = expand_marker_span(existing_text, start_match, start_single);
            let (_, end_pos) = expand_marker_span(existing_text, end_match, end_single);

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
    }

    // No existing markers found — prepend
    format!("{marked}\n\n{existing_text}")
}

impl Overview for ApplicationWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        if let Some(description) = &self.application.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        let auto_url = format!("{}/{}", state.config.domain, &self.application.id[..8]);
        header(
            &mut md,
            "Auto",
            &format!("[{}]({})", self.application.name, auto_url),
        );

        if let Some(url) = &self.application.url
            && !url.is_empty()
        {
            row(&mut md, "URL", &format!("[{url}]({url})"));
        }

        if let Some(repo) = &self.application.repository_url
            && !repo.is_empty()
        {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.stacks.is_empty() {
            row(
                &mut md,
                "Stack",
                &self
                    .stacks
                    .iter()
                    .map(|s| &*s.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self
                    .infra
                    .iter()
                    .map(|i| &*i.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        if !self.network_shares.is_empty() {
            row(
                &mut md,
                "Storage",
                &self
                    .network_shares
                    .iter()
                    .map(|s| &*s.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        if !self.people.is_empty() {
            let people = self
                .people
                .iter()
                .map(|p| format!("{} ({})", p.name, p.contribution_type))
                .filter(|i| !i.is_empty())
                .join(", ");
            row(&mut md, "People", &people);
        }

        writeln!(md, "\n").unwrap();

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

        let auto_url = format!("{}/{}", state.config.domain, &self.service.id[..8]);
        header(
            &mut md,
            "Auto",
            &format!("[{}]({})", self.service.name, auto_url),
        );

        if let Some(repo) = &self.service.repository_url
            && !repo.is_empty()
        {
            row(&mut md, "Repo", &format!("[{repo}]({repo})"));
        }

        if !self.infra.is_empty() {
            row(
                &mut md,
                "Infra",
                &self
                    .infra
                    .iter()
                    .map(|i| &*i.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        writeln!(md, "\n").unwrap();

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_distinct() {
        assert_ne!(*MARKER_START, *MARKER_END);
        assert_ne!(*MARKER_START_SINGLE, *MARKER_END_SINGLE);
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
        // Should only have one set of markers
        assert_eq!(
            result.matches(&*MARKER_START_SINGLE).count(),
            MARKER_REPETITIONS
        );
    }

    #[test]
    fn splice_survives_partial_marker_corruption() {
        let overview_old = "old overview";
        let overview_new = "new overview";

        // Build a document with full markers, then corrupt some repetitions
        let full = format!(
            "{}{}{}# Rest of doc",
            *MARKER_START, overview_old, *MARKER_END
        );

        // Remove the first repetition of the start marker (keep 2nd and 3rd)
        let start_single = &*MARKER_START_SINGLE;
        let corrupted = {
            // Find the first occurrence and blank it out
            let pos = full.find(start_single.as_str()).unwrap();
            let mut s = full.clone();
            s.replace_range(pos..pos + start_single.len(), "");
            s
        };

        let result = splice_overview(&corrupted, overview_new);
        assert!(result.contains(overview_new));
        assert!(!result.contains(overview_old));
        assert!(result.contains("# Rest of doc"));
    }

    #[test]
    fn repeated_marker_has_correct_structure() {
        let single = &*MARKER_START_SINGLE;
        let full = &*MARKER_START;

        // Should contain exactly MARKER_REPETITIONS copies
        assert_eq!(full.matches(single.as_str()).count(), MARKER_REPETITIONS);

        // Should contain MARKER_REPETITIONS - 1 separators
        assert_eq!(full.matches(MARKER_SEP).count(), MARKER_REPETITIONS - 1);
    }
}
