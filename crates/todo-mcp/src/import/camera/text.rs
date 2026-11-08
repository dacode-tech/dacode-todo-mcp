//! The camera dialect's own text shapes (t46): the checkbox-line and
//! register-construct rules — indentation is not body text in this
//! dialect, every checkbox is a register item — the id shapes with
//! their `(original)` suffix arm, and the em-dash separator constant.
//! The line/cell splitting, the known-header table walk and the noise
//! strips moved to the shared `crate::import::helpers` (the U31
//! lift) — the judgment calls that narrate them moved with the code.
//! The register-construct rule stays here because the dialects
//! measure it differently: this dialect's checkboxes are register
//! constructs at any indentation, dacode's only at column zero.

use crate::import::helpers::is_prefixed_id;

/// The milestone cell's and the resolved status's separator (em-dash),
/// spaced.
pub(super) const EM_DASH_SEP: &str = " — ";

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
