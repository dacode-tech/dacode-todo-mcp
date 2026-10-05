//! Shared cell helpers for the dacode tables (t42): row splitting,
//! separator detection, the strikethrough and annotation scans, and
//! the id/date shapes the registers share.
//!
//! Judgment call (b): escaped pipes are absent from the dialect, while
//! pipes do occur inside backtick code spans, so the splitter is
//! code-span aware — a pipe inside an open backtick span does not
//! split (pinned by
//! `a_pipe_inside_a_code_span_does_not_split_the_row`). An unpaired
//! backtick swallows the rest of the line's pipes into one cell,
//! which then fails its column count and is named.
//!
//! Known-header tables tolerate blank lines between rows: a table
//! ends at the first non-blank line that is neither a row nor a row
//! continuation, never at a blank.
//!
//! Wrapped rows: a row line that does not end with `|` continues on
//! the following lines until one ends with `|`, the trimmed pieces
//! joining with a single space — the break replaces a space (pinned
//! by `a_wrapped_table_row_joins_with_a_single_space`). A blank line
//! inside a wrap ends the table and carries the incomplete row, which
//! then fails its column count and is named. A register construct — a
//! column-0 checkbox, `### ` or `## ` — ends the wrap without being
//! absorbed, so the construct stays for its own walker (pinned by
//! `a_truncated_row_does_not_absorb_the_next_register_construct`).
//!
//! Judgment call (c) lives here too: input splits on `\n` with a
//! trailing `\r` trimmed, so CRLF input parses identically.

/// The board's em-dash separator (U+2014), as written with a space on
/// each side.
pub(crate) const EM_DASH: &str = "—";
/// The U item's id/title separator (U+00B7 middle dot), spaced.
pub(crate) const MIDDLE_DOT_SEP: &str = " · ";
/// The M item's id/title separator (em-dash), spaced.
pub(crate) const EM_DASH_SEP: &str = " — ";
/// The status-cell and Gates-cell supersession arrow (U+2192).
pub(crate) const ARROW: &str = "→";
/// The dossier status marker's resolved intro.
pub(crate) const DOSSIER_MARKER_OK: &str = " — ✅ **";
/// The dossier status marker's caveat intro.
pub(crate) const DOSSIER_MARKER_WARN: &str = " — ⚠️ **";
/// The dossier status marker's arrow-resolved intro.
pub(crate) const DOSSIER_MARKER_ARROW_OK: &str = " → ✅ **";
/// The dossier status marker's arrow-caveat intro.
pub(crate) const DOSSIER_MARKER_ARROW_WARN: &str = " → ⚠️ **";

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

/// A line that begins a register construct: a column-0 checkbox item,
/// a subsection header or a section header. A wrapped row ends at one
/// so the construct is never absorbed into a cell.
pub(crate) fn is_register_construct(text: &str) -> bool {
    text.starts_with("- [ ]")
        || text.starts_with("- [x]")
        || text.starts_with("### ")
        || text.starts_with("## ")
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
/// header is skipped.
pub(crate) fn collect_rows(
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

/// Removes annotation spans — `*(…)*` — with their content. The span
/// ends at the first `)*` — the annotations' nested parentheses are
/// never directly followed by `*`. An
/// unclosed `*(` keeps its content rather than dropping it, so a
/// malformed cell fails the entry parse and is named.
pub(crate) fn remove_annotations(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("*(") {
        out.push_str(before);
        match after.split_once(")*") {
            Some((_, tail)) => rest = tail,
            None => {
                out.push_str(after);
                return out;
            }
        }
    }
    out.push_str(rest);
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
fn is_prefixed_id(token: &str, prefix: char) -> bool {
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

/// The dependency-id shape (the dialect's): `[UBAM]<digits>[a-z]?`.
pub(crate) fn is_id_shaped(token: &str) -> bool {
    ['U', 'B', 'A', 'M']
        .iter()
        .any(|p| is_prefixed_id(token, *p))
}

/// A U id: `U<digits>[a-z]?`.
pub(crate) fn is_u_id(token: &str) -> bool {
    is_prefixed_id(token, 'U')
}

/// An M id: `M<digits>` with one optional trailing lowercase letter —
/// the id shape `[UBAM]<digits>[a-z]?`, shared by every id gate.
pub(crate) fn is_m_id(token: &str) -> bool {
    is_prefixed_id(token, 'M')
}

/// A B id: `B<digits>[a-z]?` — `B1a` is legal.
pub(crate) fn is_b_id(token: &str) -> bool {
    is_prefixed_id(token, 'B')
}

/// An A id: `A<digits>[a-z]?`.
pub(crate) fn is_a_id(token: &str) -> bool {
    is_prefixed_id(token, 'A')
}

/// A sub-step id: an M id, a dot, then digits — `M<digits>[a-z]?.<digits>`.
pub(crate) fn is_substep_id(token: &str) -> bool {
    match token.split_once('.') {
        Some((major, minor)) => {
            is_m_id(major) && !minor.is_empty() && minor.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
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

/// Every id-shaped token in `text`, in scan order, duplicates kept —
/// the Depends-on scan. Strike markers are noise: strip them before
/// scanning so a struck id still counts.
pub(crate) fn id_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            if is_id_shaped(&current) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if !current.is_empty() && is_id_shaped(&current) {
        tokens.push(current);
    }
    tokens
}

/// Splits a dossier header at its status marker: the first
/// ` — ✅ **`, ` — ⚠️ **`, ` → ✅ **` or ` → ⚠️ **` — the em-dash forms
/// are the majority, the arrow form is the alternative. Returns the
/// pre-marker text and the post-marker text, or `None` when no marker
/// is present. When several intros occur, the earliest wins — it
/// yields the shortest pre-marker text, so comparing lengths picks
/// the first.
pub(crate) fn split_at_dossier_marker(line: &str) -> Option<(&str, &str)> {
    [
        line.split_once(DOSSIER_MARKER_OK),
        line.split_once(DOSSIER_MARKER_WARN),
        line.split_once(DOSSIER_MARKER_ARROW_OK),
        line.split_once(DOSSIER_MARKER_ARROW_WARN),
    ]
    .into_iter()
    .flatten()
    .min_by_key(|(title, _)| title.len())
}
