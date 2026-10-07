//! The own dialect's own text shapes (t54): the checkbox-line and
//! register-construct rules and the id shapes — camera's exact
//! grammar, per the measured study (§4/§5's items are the same
//! checkbox + bold-id + ` · ` shapes, and the id vocabulary is the
//! same `U<digits>` family). The line/cell splitting, the
//! known-header table walk and the noise strips live in the shared
//! `crate::import::helpers`; what stays here is what this dialect
//! measures for itself. The em-dash separator is this file's because
//! the State cell's prose and the Status cell's note both use it.

use crate::import::helpers::is_prefixed_id;

/// The milestone State cell's prose separator and the Status cell's
/// note separator (em-dash), spaced.
pub(super) const EM_DASH_SEP: &str = " — ";

/// A checkbox line at any indentation: `- [ ]` or `- [x]` after
/// leading spaces. Camera's rule — every checkbox in §4/§5 is a
/// register item, and the measured board's items sit at column zero
/// with indented continuation prose.
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
/// The §2 Milestone cell is bare `M<n>` — a cell carrying the ` — `
/// separator (camera's shape) fails this check and is named.
pub(super) fn is_m_id(token: &str) -> bool {
    is_prefixed_id(token, 'M')
}

/// A D id: `D<digits>` with one optional trailing lowercase letter.
pub(super) fn is_d_id(token: &str) -> bool {
    is_prefixed_id(token, 'D')
}
