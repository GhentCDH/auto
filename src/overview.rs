/*!
* Generate a markdown overview of applications and services,
* with zero-width Unicode markers for idempotent Outline sync.
*/

use crate::AppState;
use crate::models::{ApplicationWithRelations, ServiceWithRelations};
use itertools::Itertools;
use std::fmt::Write;

/// The set of zero-width characters used for encoding.
/// Adding more characters reduces the number of characters needed per byte.
/// Must have a power-of-2 length (2, 4, 8, or 16, 32, 64 <- current).
const ZWC_ALPHABET: &[char] = &[
    '\u{E0020}',
    '\u{E0021}',
    '\u{E0022}',
    '\u{E0023}',
    '\u{E0024}',
    '\u{E0025}',
    '\u{E0026}',
    '\u{E0027}',
    '\u{E0028}',
    '\u{E0029}',
    '\u{E002A}',
    '\u{E002B}',
    '\u{E002C}',
    '\u{E002D}',
    '\u{E002E}',
    '\u{E002F}',
    '\u{E0030}',
    '\u{E0031}',
    '\u{E0032}',
    '\u{E0033}',
    '\u{E0034}',
    '\u{E0035}',
    '\u{E0036}',
    '\u{E0037}',
    '\u{E0038}',
    '\u{E0039}',
    '\u{E003A}',
    '\u{E003B}',
    '\u{E003C}',
    '\u{E003D}',
    '\u{E003E}',
    '\u{E003F}',
    '\u{E0040}',
    '\u{E0041}',
    '\u{E0042}',
    '\u{E0043}',
    '\u{E0044}',
    '\u{E0045}',
    '\u{E0046}',
    '\u{E0047}',
    '\u{E0048}',
    '\u{E0049}',
    '\u{E004A}',
    '\u{E004B}',
    '\u{E004C}',
    '\u{E004D}',
    '\u{E004E}',
    '\u{E004F}',
    '\u{E0050}',
    '\u{E0051}',
    '\u{E0052}',
    '\u{E0053}',
    '\u{E0054}',
    '\u{E0055}',
    '\u{E0056}',
    '\u{E0057}',
    '\u{E0058}',
    '\u{E0059}',
    '\u{E005A}',
    '\u{E005B}',
    '\u{E005C}',
    '\u{E005D}',
    '\u{E005E}',
    '\u{E005F}',
];

pub trait Overview {
    fn to_md(&self, state: &AppState) -> String;
    fn marker_single_start(&self) -> String;
    fn marker_single_end(&self) -> String;
    fn marker_start(&self) -> String {
        repeated_marker(&self.marker_single_start())
    }
    fn marker_end(&self) -> String {
        repeated_marker(&self.marker_single_end())
    }
    /// Wrap the markdown overview in start/end markers and splice it into `existing_text`.
    ///
    /// Detection is resilient: if even a single repetition of each marker survives
    /// editing, the block is located and replaced. On replacement the full set of
    /// repetitions is written back, restoring redundancy.
    ///
    /// - If both markers are found, the block between them (inclusive) is replaced.
    /// - Otherwise the marked block is prepended to the existing text.
    fn splice_overview(&self, state: &AppState, existing_text: &str) -> String {
        let new_overview = self.to_md(state);
        let marked = format!(
            "{}{}{}",
            self.marker_start(),
            new_overview,
            self.marker_end()
        );

        let start_single = &self.marker_single_start();
        let end_single = &self.marker_single_end();

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
    let mask = (base - 1) as u64;

    // Accumulate all bits into a buffer, then drain in chunks of bits_per_char
    let mut bit_buf: u64 = 0;
    let mut bits_in_buf = 0usize;
    let mut out = String::new();

    for byte in tag.bytes() {
        bit_buf = (bit_buf << 8) | byte as u64;
        bits_in_buf += 8;

        while bits_in_buf >= bits_per_char {
            bits_in_buf -= bits_per_char;
            let index = ((bit_buf >> bits_in_buf) & mask) as usize;
            out.push(ZWC_ALPHABET[index]);
        }
    }

    // Flush remaining bits, left-aligned (pad with zeros on the right)
    if bits_in_buf > 0 {
        let index = ((bit_buf << (bits_per_char - bits_in_buf)) & mask) as usize;
        out.push(ZWC_ALPHABET[index]);
    }

    out
}

#[allow(dead_code)]
fn decode_zwc(encoded: &str) -> Option<String> {
    let base = ZWC_ALPHABET.len();
    let bits_per_char = base.ilog2() as usize;

    let mut bit_buf: u64 = 0;
    let mut bits_in_buf = 0usize;
    let mut out = Vec::new();

    for ch in encoded.chars() {
        let index = ZWC_ALPHABET.iter().position(|&c| c == ch)?;
        bit_buf = (bit_buf << bits_per_char) | index as u64;
        bits_in_buf += bits_per_char;

        while bits_in_buf >= 8 {
            bits_in_buf -= 8;
            out.push(((bit_buf >> bits_in_buf) & 0xFF) as u8);
        }
    }

    // Ignore leftover bits (they're just padding zeros)
    String::from_utf8(out).ok()
}

/// Number of times each marker is repeated for redundancy.
/// Detection succeeds if *any single* copy survives editing.
const MARKER_REPETITIONS: usize = 3;

/// A thin spacer placed between repetitions so that a single localised edit
/// is less likely to destroy adjacent copies. Uses a Unicode character that is
/// unlikely to appear in the tag encoding itself.
const MARKER_SEP: char = '\u{2028}'; // Line Separator — invisible in most renderers

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

impl Overview for ApplicationWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        writeln!(
            md,
            "\n# Auto Information ({})\n",
            self.application.environment.to_uppercase()
        )
        .unwrap();

        if let Some(description) = &self.application.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        let auto_url = format!("{}/{}", state.config.base_url, &self.application.id[..8]);
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
                    .map(|s| &s.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        if !self.services.is_empty() {
            row(
                &mut md,
                "Services",
                &self
                    .services
                    .iter()
                    .map(|s| {
                        let s_url = format!("{}/{}", state.config.base_url, &s.id[..8]);
                        format!("[{}]({})", &s.name, s_url)
                    })
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
                    .map(|i| &i.name)
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
                    .map(|s| &s.path)
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

    fn marker_single_start(&self) -> String {
        encode_zwc(&format!("<a{}>", &self.application.id[..8],))
    }

    fn marker_single_end(&self) -> String {
        encode_zwc(&format!("</{}>", &self.application.id[..8],))
    }
}

impl Overview for ServiceWithRelations {
    fn to_md(&self, state: &AppState) -> String {
        let mut md = String::new();

        writeln!(
            md,
            "\n# Auto Information ({})\n",
            self.service.environment.to_uppercase()
        )
        .unwrap();

        if let Some(description) = &self.service.description {
            writeln!(md, "{description}").unwrap();
            writeln!(md).unwrap();
        }

        let auto_url = format!("{}/{}", state.config.base_url, &self.service.id[..8]);
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
                    .map(|i| &i.name)
                    .filter(|i| !i.is_empty())
                    .join(", "),
            );
        }

        writeln!(md, "\n").unwrap();

        md
    }

    fn marker_single_start(&self) -> String {
        encode_zwc(&format!("<{}>", &self.service.id[..8]))
    }

    fn marker_single_end(&self) -> String {
        encode_zwc(&format!("</{}>", &self.service.id[..8]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    struct TestOverview;

    impl Overview for TestOverview {
        fn to_md(&self, _state: &AppState) -> String {
            "content!".to_string()
        }

        fn marker_single_start(&self) -> String {
            "<start>".to_string()
        }

        fn marker_single_end(&self) -> String {
            "</end>".to_string()
        }
    }

    async fn appstate() -> Result<AppState> {
        AppState::new().await
    }

    #[tokio::test]
    async fn splice_prepends_when_no_markers() {
        let state = appstate().await.unwrap();
        let existing = "# Hello\n\nSome content";
        let t = TestOverview;
        let result = t.splice_overview(&state, existing);
        assert!(result.starts_with(&t.marker_start()));
        assert!(result.contains(&t.to_md(&state)));
        assert!(result.ends_with(existing));
    }

    #[tokio::test]
    async fn splice_replaces_existing_block() {
        let state = appstate().await.unwrap();
        let t = TestOverview;
        let overview_old = "old overview";
        let existing = format!(
            "{}\n{}\n{}\n\n# Rest of doc",
            t.marker_start(),
            overview_old,
            t.marker_end()
        );
        let result = t.splice_overview(&state, &existing);
        assert!(result.contains(&t.to_md(&state)));
        assert!(!result.contains(overview_old));
        assert!(result.contains("# Rest of doc"));
        assert_eq!(
            result.matches(&t.marker_single_start()).count(),
            MARKER_REPETITIONS
        );
    }

    #[tokio::test]
    async fn splice_survives_partial_marker_corruption() {
        let state = appstate().await.unwrap();
        let t = TestOverview;
        let overview_old = "old overview";

        let full = format!(
            "{}{}{}# Rest of doc",
            t.marker_start(),
            overview_old,
            t.marker_end()
        );

        let start_single = t.marker_single_start();
        let corrupted = {
            let pos = full.find(start_single.as_str()).unwrap();
            let mut s = full.clone();
            s.replace_range(pos..pos + start_single.len(), "");
            s
        };

        let result = t.splice_overview(&state, &corrupted);
        assert!(result.contains(&t.to_md(&state)));
        assert!(!result.contains(overview_old));
        assert!(result.contains("# Rest of doc"));
    }
}
