//! The store inspector (D5, spec §2; absorbed from the retired
//! `todo-cli` crate at t72, D20): read, verify and repair a
//! repository's store without an MCP client.
//!
//! §2's rule for a binary binds this module: **it translates, it does
//! not decide.** Every classification and every byte offset comes from
//! `todo-store` — [`todo_store::DataRoot::repair`] decides what a
//! fault is and whether a truncation may fix it, the `StoreError`
//! codes are §6.1's, the file layout is the D4 one — and this module
//! formats, prompts and picks an exit code. (One remedy *wording* is
//! this module's: `Diverged` carries its two numbers only, and its
//! re-base remedy lives in the store's documentation, which `repair`'s
//! report gives a line of.) The engine stays sealed as everywhere in
//! this crate (§2): no `dacode-json` dependency, no raw log parsing,
//! no second notion of a line — the store's re-exports
//! ([`todo_store::to_bytes`], [`todo_store::DataRoot`],
//! [`todo_store::RepairOutcome`]) are enough.
//!
//! The shape: the host binary's `main.rs` takes the argument vector,
//! resolves the repo through the subcommand tier (§13: the flag over
//! the cwd match) and dispatches; everything testable — the dump
//! document ([`dump::Dump`]) and the report spelling — lives here, so
//! the integration tests drive it without spawning a process except
//! where the process *is* the subject.
//!
//! # The exit-code table
//!
//! | Code | Meaning |
//! |---|---|
//! | [`EXIT_OK`] | the command did its job — for `verify` that includes a clean store, for `repair` "repaired" and "nothing to repair" alike |
//! | [`EXIT_FAULT`] | a fault reported that the command will not act on: `verify` on a faulty store; `repair` refused or aborted at the prompt |
//! | 2 | usage: the host parse's own fault — a bad flag, a missing required option, an unknown subcommand, a malformed slug (the binary's `EXIT_USAGE`) |
//! | [`EXIT_STORE`] | the store refused the operation: any `Err(StoreError)` — `io`, `locked`, `version_mismatch`, a corrupt store document |
//!
//! The machine-readable identity of a fault is §6.1's **code**, printed
//! on its own line; the exit code only separates worked /
//! found-a-fault / you-used-it-wrong / the-store-refused. There is no
//! fourth meaning.

pub mod dump;
pub mod repair;
pub mod verify;

use std::process::ExitCode;

use todo_store::StoreError;

/// The command did its job (exit-code table, module docs).
pub const EXIT_OK: u8 = 0;
/// A fault reported that the command will not act on.
pub const EXIT_FAULT: u8 = 1;
/// The store refused the operation: any `Err(StoreError)`.
pub const EXIT_STORE: u8 = 3;

/// The fault's §6.1 code on its own line, then the store's own report —
/// the one spelling every command's stderr fault lines use, so a code is
/// always the first line of a fault and `head -1` is a machine read.
pub(crate) fn fault_lines(error: &StoreError) -> String {
    format!("{}\n{error}\n", error.code())
}

/// The exit-code table's [`EXIT_STORE`] arm: the store refused, so the
/// refusal is reported through [`fault_lines`] on stderr and the process
/// exits 3.
pub(crate) fn store_refused(error: &StoreError) -> ExitCode {
    eprint!("{}", fault_lines(error));
    ExitCode::from(EXIT_STORE)
}
