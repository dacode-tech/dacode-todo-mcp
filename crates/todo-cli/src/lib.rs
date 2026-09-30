//! The store inspector (D5, spec §2): read, verify and repair a
//! repository's store without an MCP client, so the storage engine can be
//! inspected and put right before M3 builds a server on top of it.
//!
//! §2's rule for a binary binds this crate: **it translates, it does not
//! decide.** Every classification and every byte offset comes from
//! `todo-store` — [`todo_store::DataRoot::repair`] decides what a fault
//! is and whether a truncation may fix it, the `StoreError` codes are
//! §6.1's, the file layout is the D4 one — and this crate formats,
//! prompts and picks an exit code. (One remedy *wording* is this crate's:
//! `Diverged` carries its two numbers only, and its re-base remedy lives
//! in the store's documentation, which `repair`'s report gives a line
//! of.) §2 also makes
//! `todo-cli` the one crate *permitted* to name the storage engine's
//! types directly; it deliberately does not exercise the permission
//! beyond the store's public surface (U14, f25): no `dacode-json`
//! dependency, no raw log parsing, no second notion of a line — the
//! store's re-exports ([`todo_store::to_bytes`]/[`todo_store::from_bytes`],
//! [`todo_store::DataRoot`], [`todo_store::JsonlStore`],
//! [`todo_store::RepairOutcome`]) are enough, and keeping the engine
//! sealed in one crate is worth more than the option §2 grants.
//!
//! The shape is a lib target plus a thin binary: `main.rs` takes the
//! argument vector and returns an [`std::process::ExitCode`] and holds no
//! logic of its own, while everything testable — the parse
//! ([`args::parse`]), the dump document ([`dump::Dump`]) and the report
//! spelling — lives here, so the integration tests drive it without
//! spawning a process except where the process *is* the subject.
//!
//! # The exit-code table
//!
//! | Code | Meaning |
//! |---|---|
//! | [`EXIT_OK`] | the command did its job — for `verify` that includes a clean store, for `repair` "repaired" and "nothing to repair" alike |
//! | [`EXIT_FAULT`] | a fault reported that the command will not act on: `verify` on a faulty store; `repair` refused or aborted at the prompt |
//! | [`EXIT_USAGE`] | usage: a bad flag, a missing required option, an unknown subcommand, a malformed slug |
//! | [`EXIT_STORE`] | the store refused the operation: any `Err(StoreError)` — `io`, `locked`, `version_mismatch`, a corrupt store document |
//!
//! The machine-readable identity of a fault is §6.1's **code**, printed
//! on its own line; the exit code only separates worked /
//! found-a-fault / you-used-it-wrong / the-store-refused. There is no
//! fourth meaning.

pub mod args;
pub mod dump;
pub mod repair;
pub mod verify;

use std::process::ExitCode;

use todo_store::StoreError;

/// The command did its job (exit-code table, crate docs).
pub const EXIT_OK: u8 = 0;
/// A fault reported that the command will not act on.
pub const EXIT_FAULT: u8 = 1;
/// A usage fault: bad flag, missing required option, unknown subcommand,
/// malformed slug.
pub const EXIT_USAGE: u8 = 2;
/// The store refused the operation: any `Err(StoreError)`.
pub const EXIT_STORE: u8 = 3;

/// Runs the subcommand an [`args::Invocation`] names — the binary's whole
/// dispatch, so `main.rs` stays a translation of the argument vector into
/// an exit code.
pub fn dispatch(invocation: &args::Invocation) -> ExitCode {
    match invocation.command {
        args::Command::Dump => dump::run(invocation),
        args::Command::Verify => verify::run(invocation),
        args::Command::Repair => repair::run(invocation),
    }
}

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
