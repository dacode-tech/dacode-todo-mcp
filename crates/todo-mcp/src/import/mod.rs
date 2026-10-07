//! The importer's home (spec §2): the module that turns the sibling
//! boards' markdown dialects into this server's graph, once,
//! read-only (§9). The dacode dialect is first — its board carries
//! the deeper structure and the more complex references (the signed
//! ordering, with the reasoning, is in
//! `docs/implementation/decisions.md`);
//! camera has landed as the second dialect, and ikvm-html as the
//! third and last. The dialects share the discipline and two code
//! homes — [`plan`], the executable plan's types and executor, and
//! `helpers`, the text/mapping primitives every dialect's parser and
//! mapping reach for — while the measured shapes differ everywhere
//! else: a dialect parser is a pure function of text — no I/O, no
//! clock, no store — and everything the mapping stage needs arrives
//! structured, so no consumer ever re-scans prose.
//!
//! The dacode parser lives in [`dacode`], the camera parser in
//! [`camera`] and the ikvm-html parser in [`ikvm`]; each module doc
//! carries its task's judgment calls and the strictness rule that
//! makes the importer trustworthy: a register line that does not
//! parse is a named error, never a silent skip.

pub mod camera;
pub mod dacode;
pub(crate) mod helpers;
pub mod ikvm;
pub mod plan;
