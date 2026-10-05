//! The importer's home (spec §2): the module that turns the sibling
//! boards' markdown dialects into this server's graph, once,
//! read-only (§9). The dacode dialect is first — its board carries
//! the deeper structure and the more complex references (the signed
//! ordering, with the reasoning, is in
//! `docs/implementation/decisions.md`);
//! camera and ikvm-html arrive as sibling modules under this root
//! when their tasks land, sharing the discipline but no code until
//! measurement says otherwise: a dialect parser is a pure function of
//! text — no I/O, no clock, no store — and everything the mapping
//! stage needs arrives structured, so no consumer ever re-scans
//! prose.
//!
//! The dacode parser lives in [`dacode`]; its module doc carries the
//! task's judgment calls and the strictness rule that makes the
//! importer trustworthy: a register line that does not parse is a
//! named error, never a silent skip.

pub mod dacode;
