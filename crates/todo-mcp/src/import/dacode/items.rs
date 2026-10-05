//! The checkbox-item grammar and marker extraction (t42): the shapes
//! §5's milestone items and §6's U items share, split out of
//! `board.rs` by responsibility (the seam after `doc_register.rs`).
//!
//! The grammar: `- [ ]`/`- [x]` at column zero (indentation is body
//! text), then a bold run `**<id><separator><title>**` — the U
//! separator ` · `, the M separator ` — `. The bold run may wrap: it
//! closes at the first `**` on the item line or a following line, the
//! trimmed pieces joining with a single space — the break replaces a
//! space (pinned by `a_wrapped_u_item_bold_run_joins_with_a_single_space`);
//! the item's raw body span is unaffected — the walker collects every
//! line verbatim, and markers are extracted from that same text and
//! never removed from it. The scan never crosses a register construct
//! or a table row: an unclosed run is a named error, not a title
//! polluted by whatever follows it (pinned by
//! `an_unclosed_bold_run_before_the_next_item_is_a_named_error` and
//! `an_unclosed_bold_run_before_a_table_is_a_named_error`).
//!
//! Markers: one per kind per line, the first occurrence winning;
//! the recognized dependency phrases are the ruled spellings,
//! case-sensitive, and must start at a word boundary — `unblocked on`
//! and `overdue by` do not match. `before M<n>` is not recognized —
//! vague temporal prose stays body text.

use super::ast::Marker;
use super::error::ImportError;
use super::table::{
    EM_DASH_SEP, MIDDLE_DOT_SEP, clean_id_word, is_id_shaped, is_m_id, is_register_construct,
    is_u_id,
};

/// The ✅ discharge emoji.
const CHECK_EMOJI: &str = "✅";
/// The ▶️ started emoji.
const IN_WORK_EMOJI: &str = "▶️";
/// The ⚠️ caveat emoji.
const WARNING_EMOJI: &str = "⚠️";
/// The 🧠 brainstorm emoji.
const BRAINSTORM_EMOJI: &str = "🧠";
/// The recognized blocked-on phrase.
const BLOCKED_ON_PHRASE: &str = "blocked on ";
/// The recognized milestone-assertion phrases, in try order.
const MILESTONE_PHRASES: [&str; 3] = ["gates ", "Blocking ", "due by "];

/// A top-level checkbox line: `- [ ]` or `- [x]` at column zero.
/// Indented checkboxes are body text.
pub(crate) fn is_checkbox_line(text: &str) -> bool {
    text.starts_with("- [ ]") || text.starts_with("- [x]")
}

/// Splits the checkbox marker, returning the state and the rest of
/// the line. A marker with nothing after it is a malformed register
/// line, not body text.
pub(crate) fn split_marker(text: &str, line: usize) -> Result<(bool, &str), ImportError> {
    if let Some(rest) = text.strip_prefix("- [ ] ") {
        return Ok((false, rest));
    }
    if let Some(rest) = text.strip_prefix("- [x] ") {
        return Ok((true, rest));
    }
    Err(ImportError::MalformedLine {
        line,
        detail: "a checkbox marker with no bold run".to_owned(),
    })
}

/// The checkbox item kinds: the U grammar (`**U<n> · <title>**`) and
/// the M grammar (`**M<n> — <title>**`).
pub(crate) enum ItemKind {
    /// §6's U item.
    U,
    /// §5's milestone item.
    M,
}

impl ItemKind {
    /// The kind's name, for error details.
    fn name(&self) -> &'static str {
        match self {
            ItemKind::U => "U",
            ItemKind::M => "M",
        }
    }
    /// The id/title separator inside the bold run.
    fn separator(&self) -> &'static str {
        match self {
            ItemKind::U => MIDDLE_DOT_SEP,
            ItemKind::M => EM_DASH_SEP,
        }
    }
    /// The id shape the kind accepts.
    fn accepts_id(&self, token: &str) -> bool {
        match self {
            ItemKind::U => is_u_id(token),
            ItemKind::M => is_m_id(token),
        }
    }
}

/// A checkbox item under construction: its span accumulates the raw
/// lines from the item line (after the checkbox marker) through its
/// continuations.
pub(crate) struct PartialItem {
    checked: bool,
    /// The item's id, readable by the walker for its duplicate check.
    pub(crate) id: String,
    title: String,
    span: Vec<String>,
}

impl PartialItem {
    /// Accumulates one raw span line — the walker's continuation
    /// collection.
    pub(crate) fn push_line(&mut self, line: &str) {
        self.span.push(line.to_owned());
    }
}

/// Parses a checkbox item's first line: the checkbox marker, the bold
/// run, the id, and the separator. The bold run may wrap — it closes
/// at the first `**` on the item line or a following line, the pieces
/// joining with a single space; markers may sit inside it and are
/// carried raw. The item's raw body span is unaffected: the walker
/// collects every line verbatim.
pub(crate) fn parse_item_start(
    text: &str,
    line: usize,
    lines: &[(&str, usize)],
    idx: usize,
    kind: ItemKind,
) -> Result<PartialItem, ImportError> {
    let (checked, rest) = split_marker(text, line)?;
    let bold = rest
        .strip_prefix("**")
        .ok_or_else(|| ImportError::MalformedLine {
            line,
            detail: format!("a {} item's bold run is missing", kind.name()),
        })?;
    // The bold run may wrap: it closes at the first `**` on the item
    // line or a following line, the trimmed pieces joining with a
    // single space (the module doc's wrap rule).
    let mut inner = String::new();
    let mut piece = bold;
    let mut offset = 0;
    loop {
        if let Some((head, _tail)) = piece.split_once("**") {
            inner.push_str(head);
            break;
        }
        inner.push_str(piece.trim_end());
        inner.push(' ');
        let Some(&(next, _)) = lines.get(idx + 1 + offset) else {
            return Err(ImportError::MalformedLine {
                line,
                detail: format!("a {} item's bold run does not close", kind.name()),
            });
        };
        if is_register_construct(next) || next.starts_with('|') {
            // The scan never crosses a register construct or a table
            // row — an unclosed run is a named error, not a title
            // polluted by whatever follows it. (A table row terminates
            // the scan here but NOT a wrapped table row in
            // `collect_rows`: the two contexts absorb differently.)
            return Err(ImportError::MalformedLine {
                line,
                detail: format!("a {} item's bold run does not close", kind.name()),
            });
        }
        piece = next.trim();
        offset += 1;
    }
    let id: String = inner.chars().take_while(|c| c.is_alphanumeric()).collect();
    let tail = inner.strip_prefix(id.as_str()).unwrap_or("");
    let title = tail
        .strip_prefix(kind.separator())
        .ok_or_else(|| ImportError::MalformedLine {
            line,
            detail: format!(
                "the bold run is not {}-shaped: no '{}' separator after the id",
                kind.name(),
                kind.separator()
            ),
        })?;
    if !kind.accepts_id(&id) {
        return Err(ImportError::MalformedLine {
            line,
            detail: format!(
                "the bold run is not {}-shaped: '{id}' is not a {} id",
                kind.name(),
                kind.name()
            ),
        });
    }
    Ok(PartialItem {
        checked,
        id,
        title: title.to_owned(),
        span: vec![rest.to_owned()],
    })
}

/// A finished checkbox item: the fields both kinds share.
pub(crate) struct FinishedItem {
    /// The item's id.
    pub(crate) id: String,
    /// The title from the bold run.
    pub(crate) title: String,
    /// The checkbox state.
    pub(crate) checked: bool,
    /// The full raw span, joined by `\n`.
    pub(crate) body: String,
    /// The markers extracted from the span.
    pub(crate) markers: Vec<Marker>,
}

/// Finishes an item: joins its span into the raw body and extracts
/// the markers from that same text.
pub(crate) fn finish_item(item: PartialItem) -> FinishedItem {
    let body = item.span.join("\n");
    let markers = extract_markers(&item.span);
    FinishedItem {
        id: item.id,
        title: item.title,
        checked: item.checked,
        body,
        markers,
    }
}

/// Extracts the markers from a checkbox item's full span: per line,
/// the emoji kinds in declaration order, then `BlockedOn`, then
/// `MilestoneAssertion` — one marker per kind per line, the first
/// occurrence winning.
pub(crate) fn extract_markers(span: &[String]) -> Vec<Marker> {
    let mut markers = Vec::new();
    for line in span {
        if line.contains(CHECK_EMOJI) {
            markers.push(Marker::Check { line: line.clone() });
        }
        if line.contains(IN_WORK_EMOJI) {
            markers.push(Marker::InWork { line: line.clone() });
        }
        if line.contains(WARNING_EMOJI) {
            markers.push(Marker::Warning { line: line.clone() });
        }
        if line.contains(BRAINSTORM_EMOJI) {
            markers.push(Marker::Brainstorm { line: line.clone() });
        }
        if let Some(id) = phrase_id(line, BLOCKED_ON_PHRASE) {
            markers.push(Marker::BlockedOn {
                id,
                line: line.clone(),
            });
        }
        for phrase in MILESTONE_PHRASES {
            if let Some(token) = phrase_id(line, phrase)
                && is_m_id(&token)
            {
                markers.push(Marker::MilestoneAssertion {
                    milestone: token,
                    line: line.clone(),
                });
                break;
            }
        }
    }
    markers
}

/// The id-shaped token following `phrase` in `line`, when the phrase
/// starts at a word boundary — the preceding character, if any, is
/// not alphanumeric, so `unblocked on` and `overdue by` do not match.
pub(crate) fn phrase_id(line: &str, phrase: &str) -> Option<String> {
    let (before, rest) = line.split_once(phrase)?;
    if before
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric())
    {
        return None;
    }
    let word = rest.split_whitespace().next()?;
    let token = clean_id_word(word);
    if is_id_shaped(token) {
        Some(token.to_owned())
    } else {
        None
    }
}
