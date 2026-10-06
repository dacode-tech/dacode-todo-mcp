//! The camera dialect's shared text helpers (t46): line splitting,
//! cell splitting, the known-header table walk, the register-construct
//! and checkbox line shapes, and the id/date shapes the registers
//! share — the dacode `table.rs` precedent, split out of `board.rs`
//! at the file budget (the dacode module's own judgment call (e)):
//! the helpers are every camera parser's, not the TODO.md walker's
//! alone.
//!
//! Judgment calls bound here, the dacode `table.rs` ones mirrored:
//! escaped pipes are absent from the dialect while pipes do occur
//! inside backtick code spans, so the splitter is code-span aware;
//! input splits on `\n` with a trailing `\r` trimmed, so CRLF input
//! parses identically; known-header tables tolerate blank lines
//! between rows and end at the first non-blank line that is neither a
//! row nor a row continuation; a wrapped row joins its pieces with a
//! single space — the break replaces a space — and a register
//! construct is never absorbed into a cell, so the construct stays
//! for its own walker.

/// The milestone cell's and the resolved status's separator (em-dash),
/// spaced.
pub(super) const EM_DASH_SEP: &str = " — ";

/// Splits input into lines: on `\n`, with a trailing `\r` trimmed.
pub(super) fn split_lines(input: &str) -> Vec<&str> {
    input
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
}

/// Splits a table row into its trimmed cells. The leading `|` is
/// consumed; a trailing `|` drops the empty cell it would produce.
/// Pipes inside backtick code spans do not split.
pub(super) fn cells(line: &str) -> Vec<String> {
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
fn is_separator(row_cells: &[String]) -> bool {
    row_cells
        .iter()
        .all(|cell| cell.is_empty() || cell.bytes().all(|b| b == b'-'))
}

/// Whether `text` is a table header row whose cells equal `header` —
/// the known-header recognition (by header, never by position).
fn is_header(text: &str, header: &[&str]) -> bool {
    let row_cells = cells(text);
    row_cells.len() == header.len()
        && row_cells
            .iter()
            .zip(header.iter())
            .all(|(a, b)| a.as_str() == *b)
}

/// The first line whose cells equal `header`, with its index and
/// 1-based number.
pub(super) fn find_table_header<'a>(
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
/// header is skipped. Wrapped rows join with a single space.
pub(super) fn collect_rows(
    lines: &[(&str, usize)],
    header_idx: usize,
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
        if wrap.is_some() && is_register_construct(text) {
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

/// A checkbox line at any indentation: `- [ ]` or `- [x]` after
/// leading spaces. Indentation is not body text in this dialect —
/// every checkbox in §4/§5 is a register item.
pub(super) fn is_checkbox_line(text: &str) -> bool {
    let rest = text.trim_start_matches(' ');
    rest.starts_with("- [ ]") || rest.starts_with("- [x]")
}

/// A line that begins a register construct: a checkbox item at any
/// indentation, a subsection header or a section header. A wrapped
/// bold run ends at one so the construct is never absorbed.
pub(super) fn is_register_construct(text: &str) -> bool {
    is_checkbox_line(text) || text.starts_with("### ") || text.starts_with("## ")
}

/// Strips the noise around a table id cell — strike markers, bold,
/// backticks, whitespace — keeping the content.
pub(super) fn strip_id_noise(text: &str) -> String {
    text.replace("~~", "")
        .replace("**", "")
        .replace('`', "")
        .trim()
        .to_owned()
}

/// A register id as written: `U<digits>`, `U<digits><letter>`, or
/// `U<digits> (original)`.
pub(super) fn is_u_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('U') else {
        return false;
    };
    let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 {
        return false;
    }
    let suffix = rest.get(digits..).unwrap_or("");
    suffix.is_empty()
        || (suffix.len() == 1
            && suffix
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase()))
        || suffix == " (original)"
}

/// An M id: `M<digits>` with one optional trailing lowercase letter.
pub(super) fn is_m_id(token: &str) -> bool {
    is_prefixed_id(token, 'M')
}

/// A D id: `D<digits>` with one optional trailing lowercase letter.
pub(super) fn is_d_id(token: &str) -> bool {
    is_prefixed_id(token, 'D')
}

/// The dependency-id shape (the dialect's): `[UDM]<digits>[a-z]?`.
pub(super) fn is_id_shaped(token: &str) -> bool {
    ['U', 'D', 'M'].iter().any(|p| is_prefixed_id(token, *p))
}

/// `token` is an id of shape `<prefix><digits>[a-z]?`.
fn is_prefixed_id(token: &str, prefix: char) -> bool {
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
pub(super) fn is_day_date(text: &str) -> bool {
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
/// word, leaving the id candidate: `U53.` → `U53`, `U128**:` →
/// `U128`.
pub(super) fn clean_id_word(word: &str) -> &str {
    let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
    trimmed.strip_suffix("'s").unwrap_or(trimmed)
}
