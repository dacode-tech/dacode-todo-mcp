//! The shared text/mapping primitives every dialect's parser and
//! mapping reach for (the U31 lift): line and cell splitting, the
//! known-header table walk, the noise strips, the id/date word
//! shapes, and the mapping's position/date/reference construction.
//! The plan types live beside this module in [`crate::import::plan`];
//! everything else a dialect needs stays in the dialect's module.
//!
//! Judgment calls bound here, measured in both dialects so far:
//!
//! - (b) escaped pipes are absent from the dialects while pipes do
//!   occur inside backtick code spans, so the splitter is
//!   code-span aware — a pipe inside an open backtick span does not
//!   split (pinned by
//!   `a_pipe_inside_a_code_span_does_not_split_the_row`). An unpaired
//!   backtick swallows the rest of the line's pipes into one cell,
//!   which then fails its column count and is named.
//! - (c) input splits on `\n` with a trailing `\r` trimmed, so CRLF
//!   input parses identically.
//!
//! Known-header tables tolerate blank lines between rows: a table
//! ends at the first non-blank line that is neither a row nor a row
//! continuation, never at a blank. Wrapped rows: a row line that
//! does not end with `|` continues on the following lines until one
//! ends with `|`, the trimmed pieces joining with a single space —
//! the break replaces a space (pinned by
//! `a_wrapped_table_row_joins_with_a_single_space`). A blank line
//! inside a wrap ends the table and carries the incomplete row, which
//! then fails its column count and is named. A register construct
//! ends the wrap without being absorbed, so the construct stays for
//! its own walker (pinned by
//! `a_truncated_row_does_not_absorb_the_next_register_construct`) —
//! and which lines begin register constructs is the one rule the
//! dialects measure differently, so `collect_rows` takes the check as
//! a parameter and each dialect passes its own.
//!
//! The one rule that keeps this home honest: nothing dialect-specific
//! lives here — an id-shape validator or a grammar rule that differs
//! between dialects stays in the dialect's module even when its name
//! suggests sharing.

use std::num::NonZeroU64;

use todo_domain::{Reference, ReferenceKind, References, Timestamp, ValueError};

/// Splits input into lines: on `\n`, with a trailing `\r` trimmed
/// (judgment call (c)).
pub(crate) fn split_lines(input: &str) -> Vec<&str> {
    input
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
}

/// Splits a table row into its trimmed cells. The leading `|` is
/// consumed; a trailing `|` drops the empty cell it would produce.
/// Pipes inside backtick code spans do not split — the module doc's
/// judgment call (b).
pub(crate) fn cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut split: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_code = false;
    for ch in trimmed.chars() {
        if ch == '`' {
            in_code = !in_code;
            current.push(ch);
        } else if ch == '|' && !in_code {
            split.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    split.push(current);
    if split.first().is_some_and(|cell| cell.trim().is_empty()) {
        split.remove(0);
    }
    if split.last().is_some_and(|cell| cell.trim().is_empty()) {
        split.pop();
    }
    split.iter().map(|cell| cell.trim().to_owned()).collect()
}

/// A markdown separator row: every cell is dashes or empty.
pub(crate) fn is_separator(row_cells: &[String]) -> bool {
    row_cells
        .iter()
        .all(|cell| cell.is_empty() || cell.bytes().all(|b| b == b'-'))
}

/// Whether `text` is a table header row whose cells equal `header` —
/// the known-header recognition (by header, never by position).
pub(crate) fn is_header(text: &str, header: &[&str]) -> bool {
    let row_cells = cells(text);
    row_cells.len() == header.len()
        && row_cells
            .iter()
            .zip(header.iter())
            .all(|(a, b)| a.as_str() == *b)
}

/// The first line whose cells equal `header`, with its index and
/// 1-based number — the known-header recognition (by header,
/// never by position).
pub(crate) fn find_table_header<'a>(
    lines: &'a [(&'a str, usize)],
    header: &[&str],
) -> Option<(usize, &'a str, usize)> {
    for (idx, &(text, number)) in lines.iter().enumerate() {
        if is_header(text, header) {
            return Some((idx, text, number));
        }
    }
    None
}

/// A known-header table's rows: the `|`-lines following the header at
/// `header_idx`, blank-line tolerant, ending at the first non-blank
/// line that is not a `|` row. The separator row directly after the
/// header is skipped. The dialect's register-construct check arrives
/// as `is_construct` — the one rule the dialects measure differently,
/// so the walk does not pick a side.
pub(crate) fn collect_rows(
    lines: &[(&str, usize)],
    header_idx: usize,
    is_construct: impl Fn(&str) -> bool,
) -> (Vec<(String, usize)>, usize) {
    let mut rows: Vec<(String, usize)> = Vec::new();
    let mut last = header_idx;
    let mut expect_separator = true;
    let mut wrap: Option<(String, usize)> = None;
    let mut idx = header_idx + 1;
    while let Some(&(text, number)) = lines.get(idx) {
        if text.trim().is_empty() {
            if wrap.is_some() {
                break;
            }
            idx += 1;
            continue;
        }
        if wrap.is_some() && is_construct(text) {
            // A register construct is never absorbed into a cell: the
            // wrap ends, the partial row is carried to face its own
            // column count, and the construct stays for the walker.
            break;
        }
        if wrap.is_none() && !text.starts_with('|') {
            break;
        }
        let ends_pipe = text.trim_end().ends_with('|');
        if let Some(mut partial) = wrap.take() {
            partial.0.push(' ');
            partial.0.push_str(text.trim());
            if ends_pipe {
                rows.push(partial);
            } else {
                wrap = Some(partial);
            }
            last = idx;
        } else if ends_pipe {
            let row_cells = cells(text);
            if expect_separator && is_separator(&row_cells) {
                expect_separator = false;
            } else {
                rows.push((text.to_owned(), number));
            }
            last = idx;
        } else {
            wrap = Some((text.trim().to_owned(), number));
            last = idx;
        }
        idx += 1;
    }
    if let Some(partial) = wrap {
        rows.push(partial);
    }
    (rows, last)
}

/// Removes strike markers, keeping the struck text: `~~B1~~` → `B1`.
pub(crate) fn strip_strikes(text: &str) -> String {
    text.replace("~~", "")
}

/// Removes struck spans with their content: `~~x~~ y` → ` y`. An
/// unclosed `~~` strikes to the end of the text.
pub(crate) fn remove_struck_spans(text: &str) -> String {
    let mut out = String::new();
    let mut struck = false;
    for part in text.split("~~") {
        if !struck {
            out.push_str(part);
        }
        struck = !struck;
    }
    out
}

/// Removes bold markers and backticks: `**01**` → `01`,
/// `` `README` `` → `README`.
pub(crate) fn strip_emphasis(text: &str) -> String {
    text.replace("**", "").replace('`', "")
}

/// Strips the noise around a table id cell — strike markers, bold,
/// backticks, whitespace — keeping the content.
pub(crate) fn strip_id_noise(text: &str) -> String {
    strip_emphasis(&strip_strikes(text)).trim().to_owned()
}

/// `token` is an id of shape `<prefix><digits>[a-z]?` — `B1a` is
/// legal, two trailing letters are not.
pub(crate) fn is_prefixed_id(token: &str, prefix: char) -> bool {
    // A byte-slice walk, not a `take_while` chain: `take_while` consumes
    // the first non-digit, which silently admitted two trailing
    // letters (`B1ab`). The digits are ascii, so byte indices are
    // char boundaries by construction.
    let rest = token.strip_prefix(prefix).unwrap_or("");
    let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    let suffix = rest.get(digits..).unwrap_or("");
    digits > 0
        && (suffix.is_empty()
            || (suffix.len() == 1
                && suffix
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_lowercase())))
}

/// A day-precision board date: `YYYY-MM-DD`.
pub(crate) fn is_day_date(text: &str) -> bool {
    let mut parts = text.split('-');
    let year = parts.next().unwrap_or_default();
    let month = parts.next().unwrap_or_default();
    let day = parts.next().unwrap_or_default();
    parts.next().is_none()
        && year.len() == 4
        && year.bytes().all(|b| b.is_ascii_digit())
        && month.len() == 2
        && month.bytes().all(|b| b.is_ascii_digit())
        && day.len() == 2
        && day.bytes().all(|b| b.is_ascii_digit())
}

/// Strips surrounding punctuation and a trailing `'s` from a prose
/// word, leaving the id candidate: `U53.` → `U53`, `B19's` → `B19`.
pub(crate) fn clean_id_word(word: &str) -> &str {
    let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
    trimmed.strip_suffix("'s").unwrap_or(trimmed)
}

/// The 1-based id number for a 0-based position: the plan numbers
/// every kind from 1, mirroring the store's allocation, so a position
/// cannot encode zero. `wrapping_add` with the `MAX` fallback keeps
/// the helper total without a panic path; the fallback is unreachable
/// at any board's scale.
pub(crate) fn position(index: usize) -> NonZeroU64 {
    NonZeroU64::new((index as u64).wrapping_add(1)).unwrap_or(NonZeroU64::MAX)
}

/// The numeric sort key of a board id: the number after the prefix,
/// with the id itself as the tiebreaker (a letter-suffixed id orders
/// after its bare base). The parser validates id shapes; a malformed
/// id sorts first rather than panicking.
pub(crate) fn id_sort_key(id: &str) -> (u64, &str) {
    let digits: String = id
        .chars()
        .skip(1)
        .take_while(|c| c.is_ascii_digit())
        .collect();
    (digits.parse().unwrap_or(0), id)
}

/// The error seam of the shared mapping helpers: the error-typed
/// primitives (`midnight`, `references`) fail through this neutral
/// form, and each dialect converts it into its own error
/// variant-for-variant, so every message and field stays the
/// dialect's own.
pub(crate) enum HelperError {
    /// A board date that is shape-valid but not a real calendar date.
    BadDate {
        /// The date as written.
        date: String,
    },
    /// The domain rejected a reference target.
    BadReference {
        /// The rejected target.
        target: String,
        /// Why the domain rejected it.
        detail: String,
    },
}

/// A board date as the midnight UTC timestamp the conventions pin:
/// `YYYY-MM-DD` → `T00:00:00.000000000Z`. The parser validates the
/// date's shape; the calendar check happens here, so an impossible
/// day is a named error.
pub(crate) fn midnight(date: &str) -> Result<Timestamp, HelperError> {
    Timestamp::parse(&format!("{date}T00:00:00Z")).map_err(|_| HelperError::BadDate {
        date: date.to_owned(),
    })
}

/// The references for one draft, propagating the domain's rejection
/// of a target. The mapping's targets are parser-validated ids and
/// doc keys (non-empty, anchor-free) and the A-delta section strings —
/// a pathological span could carry an anchor, which the domain
/// rejects for a non-`doc` kind.
pub(crate) fn references(entries: Vec<(ReferenceKind, String)>) -> Result<References, HelperError> {
    let mut references = References::new();
    for (kind, target) in entries {
        let reference = Reference::new(kind, target.clone(), None)
            .map_err(|error| bad_reference(&target, &error))?;
        references
            .insert(reference)
            .map_err(|error| bad_reference(&target, &error))?;
    }
    Ok(references)
}

/// The named form of the domain rejecting a reference target.
fn bad_reference(target: &str, error: &ValueError) -> HelperError {
    HelperError::BadReference {
        target: target.to_owned(),
        detail: error.to_string(),
    }
}
