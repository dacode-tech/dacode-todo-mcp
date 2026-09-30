//! The `repair` command: printing a [`RepairOutcome`], §4.3's
//! confirmation prompt, and the post-repair re-open. The decisions all
//! came from the store — [`DataRoot::repair`] classifies, validates and
//! truncates; this module formats, prompts and picks the exit code
//! (spec §2: the binary translates, it does not decide).
//!
//! # Stream discipline
//!
//! `repair` is the fault-handling command, so its fault reports — the
//! torn line's facts, refusals, the prompt itself, the aborted line — go
//! to **stderr**, and stdout carries only the two no-fault statements:
//! `nothing to repair` and `the store now opens clean`. An operator
//! redirecting stderr away sees good news only; the prompt follows the
//! same convention as its subject (and GNU practice: a prompt on stderr
//! survives `> log` redirection).
//!
//! # The double call on the consent path
//!
//! A real run without `--yes` calls `repair` twice: `dry_run = true` to
//! get the facts the prompt shows, then — only on consent — the real
//! `repair(slug, false)`, which is the act. The double call is correct,
//! not a race being lost: t22's dry run validates the candidate prefix
//! with the log's own replay, so the truncation it promises is the one
//! the real run keeps, and the first call wrote nothing. Calling
//! `repair(slug, false)` *first* and prompting afterwards would ask
//! consent for an act already committed; classifying again after it
//! would report `Clean` and hide what happened. `--yes` and `--dry-run`
//! each make the single call with the caller's own `dry_run` value.

use std::io::BufRead;
use std::path::PathBuf;
use std::process::ExitCode;

use todo_domain::RepoSlug;
use todo_store::{DataRoot, RepairOutcome, TornTail};

use crate::args::Invocation;
use crate::{EXIT_FAULT, EXIT_OK, fault_lines};

/// The report lines of a [`RepairOutcome`] — the one spelling of the
/// store's classification, shared by this command's own output and by
/// `verify`'s corrupt-store report, so the same state reads the same
/// wherever the cli shows it. Every fact is the store's; the labels are
/// this crate's, as is the one remedy line the store documents but does
/// not print (`Diverged`'s re-base). A refusal's detail is printed
/// **verbatim**: it is cli-facing contract text (f33) and already names
/// §4.3's backup.
pub(crate) fn outcome_report(outcome: &RepairOutcome) -> String {
    match outcome {
        RepairOutcome::Clean => "classification: clean\n".to_owned(),
        RepairOutcome::TornTail { torn, .. } => format!(
            "classification: torn_tail\ntruncate_at: {}\nbytes_removed: {}\ntorn_line: {}\n",
            torn.offset, torn.removed, torn.line
        ),
        RepairOutcome::Unrepairable { detail, .. } => {
            format!("classification: unrepairable\nrefusal: {detail}\n")
        }
        RepairOutcome::Diverged {
            log_offset,
            log_len,
        } => format!(
            // The remedy wording is this crate's rendering of the store's
            // documented remedy — the outcome itself carries only its two
            // numbers, and a refusal's detail is the store's own text.
            "classification: diverged\nsnapshot_log_offset: {log_offset}\nlog_bytes: {log_len}\n\
             remedy: re-base the snapshot onto the log\n"
        ),
    }
}

/// Whether one line of prompt input consents to §4.3's truncation:
/// `y`/`yes`, case-insensitive, trimmed. Everything else refuses — the
/// confirmation is an explicit act, so silence, EOF and anything
/// unrecognizable abort rather than truncate.
pub fn consent(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// The confirmation prompt: the offset, the byte count and the bounded
/// preview line — the store's own facts — then the question. The preview
/// is already cut at the store's bound, so a tear of any size cannot
/// flood the terminal.
fn prompt_report(torn: &TornTail) -> String {
    format!(
        "torn line at offset {} ({} bytes):\n{}\n\
         truncate it, losing exactly this one in-flight event? [y/N]\n",
        torn.offset, torn.removed, torn.line
    )
}

/// Reads the one line of consent from stdin. EOF — an empty or closed
/// stdin — aborts, as does a stdin that cannot be read: a prompt nobody
/// answered is not a confirmation, and the default is the safe arm.
fn read_consent() -> bool {
    let stdin = std::io::stdin();
    let mut answer = String::new();
    match stdin.lock().read_line(&mut answer) {
        Ok(0) => false,
        Ok(_) => consent(&answer),
        Err(_) => false,
    }
}

/// Runs `repair` (spec §4.3's explicit, confirmed act; module docs for
/// the call discipline per flag combination). `--dry-run` wins when both
/// flags are given: a run that classifies and writes nothing is the safe
/// arm, and `--yes` only ever skips a prompt for the act.
pub fn run(invocation: &Invocation) -> ExitCode {
    let root = DataRoot::new(PathBuf::from(&invocation.root));
    let slug = &invocation.slug;
    if invocation.dry_run {
        return classify_only(&root, slug);
    }
    if invocation.yes {
        return act(&root, slug);
    }
    // The consent path: the facts come from a dry run, which writes
    // nothing, and only consent reaches the act (module docs).
    match root.repair(slug, true) {
        Err(error) => crate::store_refused(&error),
        Ok(RepairOutcome::Clean) => nothing_to_repair(),
        Ok(RepairOutcome::TornTail {
            torn,
            truncated: false,
        }) => {
            // stderr is unbuffered, so the prompt is on the terminal
            // before stdin is read — a piped answer cannot race it.
            eprint!("{}", prompt_report(&torn));
            if read_consent() {
                act(&root, slug)
            } else {
                aborted()
            }
        }
        // A refusal — and, impossibly through the store's contract, a
        // dry run claiming it wrote: both are faults this command will
        // not act on, reported and exited.
        Ok(other) => report_only(&other),
    }
}

/// `--dry-run`: classify with the caller's own `dry_run = true` and
/// report; a torn tail adds the plain "no change written" line and stays
/// a success — the command did its job, which was to look.
fn classify_only(root: &DataRoot, slug: &RepoSlug) -> ExitCode {
    match root.repair(slug, true) {
        Err(error) => crate::store_refused(&error),
        Ok(RepairOutcome::Clean) => nothing_to_repair(),
        Ok(outcome @ RepairOutcome::TornTail { .. }) => {
            eprint!("{}", outcome_report(&outcome));
            eprintln!("no change written");
            ExitCode::from(EXIT_OK)
        }
        Ok(other) => report_only(&other),
    }
}

/// The act: one `repair(slug, false)` — classify-and-write in the store's
/// single call — then the post-truncation re-open that proves the promise
/// kept. A [`RepairOutcome::Clean`] from the act is the true statement it
/// reads as (nothing was there to write), whatever a consent path saw a
/// moment earlier.
fn act(root: &DataRoot, slug: &RepoSlug) -> ExitCode {
    match root.repair(slug, false) {
        Err(error) => crate::store_refused(&error),
        Ok(RepairOutcome::Clean) => nothing_to_repair(),
        Ok(
            outcome @ RepairOutcome::TornTail {
                truncated: true, ..
            },
        ) => {
            // What was removed and where — the act's own outcome values,
            // printed even on the consent path, because the confirmation
            // is a report of what the real run did, not an echo of the
            // dry run's promise.
            eprint!("{}", outcome_report(&outcome));
            reopen_report(root, slug)
        }
        Ok(other) => report_only(&other),
    }
}

/// The post-repair re-open: whether the store now opens clean. Success
/// is stdout's second no-fault statement and exit 0; failure after a
/// truncation is exit 1 and a **loud** line — the store was modified and
/// still does not open, which is never silent.
fn reopen_report(root: &DataRoot, slug: &RepoSlug) -> ExitCode {
    match root.open(slug) {
        Ok(_store) => {
            println!("the store now opens clean");
            ExitCode::from(EXIT_OK)
        }
        Err(error) => {
            eprintln!("the store was modified by this repair and still does not open");
            eprint!("{}", fault_lines(&error));
            ExitCode::from(EXIT_FAULT)
        }
    }
}

/// `Clean` → the exit-code table's 0: "nothing to repair" is the command
/// doing its job.
fn nothing_to_repair() -> ExitCode {
    println!("nothing to repair");
    ExitCode::from(EXIT_OK)
}

/// A refusal (`Unrepairable`, `Diverged`): the store's report verbatim on
/// stderr, exit 1, nothing written.
fn report_only(outcome: &RepairOutcome) -> ExitCode {
    eprint!("{}", outcome_report(outcome));
    ExitCode::from(EXIT_FAULT)
}

/// Aborted at the prompt: exit 1, nothing written.
fn aborted() -> ExitCode {
    eprintln!("aborted");
    ExitCode::from(EXIT_FAULT)
}
