//! The D13 ensure-listener (t85; retired to its first tick by D25,
//! U66): the daemon's live view. One call — [`Watchdog::ensure`] —
//! probes the per-root publication; a live listener means attach,
//! dead/stale/absent means bind, publish and run the accept thread
//! through the same [`serve`] entry the subcommand uses. The
//! no-subcommand server's watch loop retired with the stdio path
//! (U66): the daemon owns the listener outright, so there is no
//! survivor to take over — the listener lives exactly as long as the
//! daemon, and the next owner's ensure handles the stale
//! publication. Only the `serve` subcommand keeps its SIGINT clean
//! shutdown; the daemon ends by SIGTERM's drain.
//!
//! The URL line (D3) is stderr-only: stdout is the JSON-RPC channel,
//! and the protocol suite's exact-byte pins are what enforce that.
//! An ensure failure (D4) is one stderr line and the answer is
//! `false` — the live view is an add-on, never a gate on the wire.

use std::path::Path;
use std::time::Duration;

use todo_store::DataRoot;

use super::{ServeOutcome, serve};

/// The live-view ensure (D13, D2): the root and its per-root
/// publication, the values the [`serve`] entry needs. The daemon
/// constructs one at startup and calls [`Watchdog::ensure`] once —
/// the started listener's thread is detached (the struct holds no
/// handle; the process exit reaps it, the serve module's own Drop
/// discipline), and the publication is left for the next owner.
pub struct Watchdog {
    /// The root whose live view is ensured.
    root: DataRoot,
    /// The per-root publication ([`super::port_file`]).
    port_file: std::path::PathBuf,
    /// The startup-read working directory (the serve entry's default
    /// tier).
    cwd: Option<std::path::PathBuf>,
    /// The WS poller's interval.
    poll_interval: Duration,
}

impl Watchdog {
    /// An ensure over `root`'s live view, deriving the per-root
    /// publication and carrying the cwd and poll interval the serve
    /// entry wants.
    pub fn new(root: &DataRoot, cwd: Option<&Path>, poll_interval: Duration) -> Self {
        Self {
            root: root.clone(),
            port_file: super::port_file(root),
            cwd: cwd.map(std::path::PathBuf::from),
            poll_interval,
        }
    }

    /// The one ensure (D2): the same [`serve`] entry the subcommand
    /// uses, which probes the publication first — a live listener
    /// attaches (the owner still lives, or a simultaneous invocation
    /// won the publish race and this one attaches to the winner), a
    /// dead, stale or absent publication binds and publishes. The
    /// URL line is stderr's (D3); a failure is one line and `false`
    /// (D4) — the wire is unaffected either way.
    pub(crate) fn ensure(&mut self) -> bool {
        match serve(
            &self.root,
            &self.port_file,
            self.cwd.as_deref(),
            self.poll_interval,
        ) {
            Ok(ServeOutcome::Attached { url }) | Ok(ServeOutcome::Started { url, .. }) => {
                eprintln!("todo-mcp: live view at {url}");
                true
            }
            Err(message) => {
                eprintln!("todo-mcp: the live view is unavailable: {message}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::thread;

    use todo_domain::{Repo, RepoSlug, Timestamp};

    use super::super::POLL_INTERVAL;

    /// The per-root derivation (D1, M4's pin): the publication is
    /// `<root>/serve.port`, and two roots derive two paths — the
    /// keying that gives multi-root coexistence by construction.
    #[test]
    fn the_port_file_is_derived_per_root() {
        let first = tempfile::tempdir().expect("a temp root");
        let second = tempfile::tempdir().expect("another temp root");
        let first_root = DataRoot::new(first.path());
        let second_root = DataRoot::new(second.path());
        assert_eq!(
            super::super::port_file(&first_root),
            first.path().join("serve.port")
        );
        assert_ne!(
            super::super::port_file(&first_root),
            super::super::port_file(&second_root)
        );
    }

    /// A root with one registered repo — the serve entry's startup
    /// check demands at least one.
    fn registered_root() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("a temp root");
        let root = DataRoot::new(dir.path());
        root.register(Repo {
            slug: RepoSlug::from_str("fixture").expect("a valid slug"),
            name: "fixture".to_owned(),
            path: "/repos/fixture".to_owned(),
            registered_at: Timestamp::parse("2026-10-02T08:00:00Z").expect("a timestamp"),
        })
        .expect("the repo registers");
        dir
    }

    /// The entry called twice on one root (test group 2): the first
    /// starts, the second attaches — the same port, never a second
    /// listener.
    #[test]
    fn the_entry_called_twice_starts_then_attaches() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let port_file = super::super::port_file(&root);
        let first = serve(&root, &port_file, None, POLL_INTERVAL).expect("the first starts");
        let ServeOutcome::Started {
            url: first_url,
            listener: _owner,
        } = first
        else {
            panic!("the first call starts");
        };
        let second = serve(&root, &port_file, None, POLL_INTERVAL).expect("the second attaches");
        let ServeOutcome::Attached { url } = second else {
            panic!("the second call attaches");
        };
        assert_eq!(url, first_url);
        let port = super::super::read_published_port(&port_file).expect("the publication");
        assert!(super::super::listener_answers(port));
    }

    /// The double-bind race (test group 3, rewritten off the retired
    /// tick): two concurrent serve entries over one root with no
    /// publication — whichever way the interleaving goes, the end
    /// state is one advertised live listener and the loser (if the
    /// race produced one) attached to it. Two entries exactly: a
    /// third could supersede the publication after the loser's
    /// attach, which the four-party shape allowed and this pin does
    /// not. Two `Started` outcomes are possible (the
    /// double-overwrite interleaving leaves one orphaned listener on
    /// an unadvertised port — benign, spec-recorded); the publication
    /// is the truth, and it names a live port.
    #[test]
    fn racing_serve_entries_leave_one_advertised_listener() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let port_file = super::super::port_file(&root);
        let mut entries = Vec::new();
        for _ in 0..2 {
            let root = root.clone();
            entries.push(thread::spawn(move || {
                serve(&root, &super::super::port_file(&root), None, POLL_INTERVAL)
            }));
        }
        let mut started = 0;
        let mut attached_urls = Vec::new();
        for entry in entries {
            match entry.join().expect("the entry completes") {
                Ok(ServeOutcome::Started { .. }) => started += 1,
                Ok(ServeOutcome::Attached { url }) => attached_urls.push(url),
                Err(message) => panic!("an entry attaches or starts: {message}"),
            }
        }
        assert!(started >= 1, "at least one entry owns the listener");
        let port = super::super::read_published_port(&port_file).expect("the publication");
        assert!(
            super::super::listener_answers(port),
            "the publication names a live listener (port {port}, started {started}, attached \
             {attached_urls:?})"
        );
        for url in attached_urls {
            assert_eq!(url, format!("http://127.0.0.1:{port}/"));
        }
    }

    /// The failure arm (D4): a root with no repository — the ensure's
    /// startup check — fails without touching the wire.
    #[test]
    fn the_ensure_fails_on_a_root_with_nothing_to_serve() {
        let dir = tempfile::tempdir().expect("a temp root");
        let root = DataRoot::new(dir.path());
        let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
        assert!(!watchdog.ensure(), "an empty root fails the ensure");
    }
}
