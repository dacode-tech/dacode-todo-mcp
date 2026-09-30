//! The inspector's binary: the argument vector in, an [`ExitCode`] out.
//! Nothing else lives here — no logic, no formatting (spec §2: the binary
//! translates, it does not decide); the parse, the commands, the
//! exit-code table and the report spellings are the lib's.

use std::process::ExitCode;

use todo_cli::EXIT_USAGE;
use todo_cli::args::{Request, USAGE, parse};

/// Entry point. `clippy::exit` is denied workspace-wide — the binary
/// decides what is fatal by return value, never by aborting the process
/// behind the runtime's back.
fn main() -> ExitCode {
    // The first element of args_os is the program name; the parse takes
    // the arguments alone.
    match parse(std::env::args_os().skip(1)) {
        Ok(Request::Help) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Request::Version) => {
            println!("todo-cli {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Request::Run(invocation)) => todo_cli::dispatch(&invocation),
        Err(error) => {
            // The message, a blank line, then the usage text — a usage
            // fault always carries the way to use the tool correctly, on
            // stderr where the fault belongs.
            eprintln!("{error}\n");
            eprint!("{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}
