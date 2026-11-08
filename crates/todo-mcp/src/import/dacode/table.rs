//! The dacode tables' dialect-specific shapes (t42): the separator
//! and dossier-marker constants, the register-construct rule, the id
//! shapes the registers share, the Depends-on id scan, and the
//! annotation scan. The line/cell splitting, the known-header table
//! walk and the noise strips moved to the shared
//! `crate::import::helpers` (the U31 lift) — the judgment calls that
//! narrate them moved with the code. The register-construct rule
//! stays here because the dialects measure it differently: this
//! dialect's checkboxes are register constructs only at column zero
//! (indentation is body text), camera's at any indentation.

use crate::import::helpers::is_prefixed_id;

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

/// A line that begins a register construct: a column-0 checkbox item,
/// a subsection header or a section header. A wrapped row ends at one
/// so the construct is never absorbed into a cell.
pub(crate) fn is_register_construct(text: &str) -> bool {
    text.starts_with("- [ ]")
        || text.starts_with("- [x]")
        || text.starts_with("### ")
        || text.starts_with("## ")
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
