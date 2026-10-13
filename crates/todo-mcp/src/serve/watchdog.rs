//! The D13 ensure-listener (t85): the no-subcommand server's live
//! view. One std thread spawned beside the tokio runtime — the first
//! tick is the ensure (probe the per-root publication; a live
//! listener means attach, dead/stale/absent means bind, publish and
//! run the accept thread through the same [`serve`] entry the
//! subcommand uses), and the loop is the watchdog (re-probe at the
//! poll interval; a dead publication means take over, so the view is
//! up while at least one session server — or a manual `serve` — is
//! alive).
//!
//! The tick is a factored method ([`Watchdog::tick`]) so the tests
//! drive it deterministically; the thread ([`spawn_live_view`]) is a
//! thin loop. The startup never blocks on the thread — the MCP
//! handshake cannot wait on a listener — and the thread is detached:
//! a session server dies with its process (D5, no clean shutdown on
//! this path), the publication goes stale, and the survivors'
//! watchdogs take over within the interval. Only the `serve`
//! subcommand keeps its SIGINT clean shutdown.
//!
//! The URL line (D3) is stderr-only: stdout is the JSON-RPC channel,
//! and the protocol suite's exact-byte pins are what enforce that.
//! An ensure failure (D4) is one stderr line and the watch ends —
//! the live view is an add-on, never a gate on the wire.

use std::path::Path;
use std::thread;
use std::time::Duration;

use todo_store::DataRoot;

use super::Listener;
use super::POLL_INTERVAL;
use super::{ServeOutcome, serve};

/// One tick of the live-view watch: what the tick did, and the URL
/// when there is one to report.
#[derive(Debug, PartialEq, Eq)]
pub enum TickOutcome {
    /// The tick found a live listener owned by another process and
    /// attached to it — never a second listener.
    Attached {
        /// The answering listener's URL.
        url: String,
    },
    /// The tick bound and published a listener this process now owns:
    /// the first tick's ensure, or a takeover after the owner died.
    Started {
        /// The new listener's URL.
        url: String,
    },
    /// This process already owns the listener — the tick is a no-op.
    /// An owned listener lives exactly as long as the process, so
    /// there is nothing to re-probe.
    Owned,
    /// The ensure failed (the registry will not load, the root holds
    /// no repository): the live view is off for this session, the
    /// message is for the one stderr line D4 allows.
    Failed {
        /// The ensure's failure message.
        message: String,
    },
}

/// The live-view watch's state between ticks (D13, D2): the root and
/// its per-root publication, the values the [`serve`] entry needs,
/// and the listener this process owns — `None` while another process
/// owns the view, which is exactly when the watchdog has work to do.
pub struct Watchdog {
    /// The root whose live view is watched.
    root: DataRoot,
    /// The per-root publication ([`super::port_file`]).
    port_file: std::path::PathBuf,
    /// The startup-read working directory (the serve entry's default
    /// tier).
    cwd: Option<std::path::PathBuf>,
    /// The WS poller's interval, shared with the re-probe cadence.
    poll_interval: Duration,
    /// The listener this process owns, held (never shut down — D5:
    /// the process exit reaps it) so the accept thread outlives the
    /// tick that started it.
    owner: Option<Listener>,
}

impl Watchdog {
    /// A watch over `root`'s live view, deriving the per-root
    /// publication and carrying the cwd and poll interval the serve
    /// entry wants.
    pub fn new(root: &DataRoot, cwd: Option<&Path>, poll_interval: Duration) -> Self {
        Self {
            root: root.clone(),
            port_file: super::port_file(root),
            cwd: cwd.map(std::path::PathBuf::from),
            poll_interval,
            owner: None,
        }
    }

    /// One tick (D2): owning means no-op — this process's listener
    /// cannot die before it does; otherwise the same [`serve`] entry
    /// the subcommand uses, which probes the publication first: a
    /// live listener attaches (the owner still lives, or a simultaneous
    /// takeover won the publish race and this one attaches to the
    /// winner), a dead, stale or absent publication binds and
    /// publishes — the takeover.
    pub fn tick(&mut self) -> TickOutcome {
        if self.owner.is_some() {
            return TickOutcome::Owned;
        }
        match serve(
            &self.root,
            &self.port_file,
            self.cwd.as_deref(),
            self.poll_interval,
        ) {
            Ok(ServeOutcome::Attached { url }) => TickOutcome::Attached { url },
            Ok(ServeOutcome::Started { url, listener }) => {
                self.owner = Some(listener);
                TickOutcome::Started { url }
            }
            Err(message) => TickOutcome::Failed { message },
        }
    }

    /// The first tick's reporting (D3, stderr only — stdout is the
    /// JSON-RPC channel): the URL line on start or attach, nothing on
    /// the owner no-op, one line and `false` on failure. The thread
    /// body's first half and the daemon's one-shot ensure (U65) are
    /// this one method — the loop is the only part the daemon does
    /// not take.
    pub(crate) fn ensure(&mut self) -> bool {
        match self.tick() {
            TickOutcome::Started { url } | TickOutcome::Attached { url } => {
                eprintln!("todo-mcp: live view at {url}");
                true
            }
            TickOutcome::Owned => true,
            TickOutcome::Failed { message } => {
                eprintln!("todo-mcp: the live view is unavailable: {message}");
                false
            }
        }
    }
}

/// Spawns the live-view thread (D13, D2): one std thread, detached —
/// the startup never blocks on it, and it dies with the process. The
/// binary's McpServer arm calls this beside the stdio serve; a spawn
/// failure is one stderr line (D4), never a dead session.
pub fn spawn_live_view(root: &DataRoot, cwd: Option<&Path>) {
    let watchdog = Watchdog::new(root, cwd, POLL_INTERVAL);
    let spawned = thread::Builder::new()
        .name("live-view".to_owned())
        .spawn(move || run(watchdog));
    if spawned.is_err() {
        eprintln!("todo-mcp: the live view could not start: the thread did not spawn");
    }
}

/// The thread body: the first tick's reporting (the [`Watchdog::ensure`]
/// half), then the loop re-probes at the interval. A failure ends the
/// watch (D4) — the wire is unaffected either way. Later ticks are
/// silent: the URL line is the first tick's discovery, and a
/// takeover's new port is discovered through the publication, which
/// is its job.
fn run(mut watchdog: Watchdog) {
    if !watchdog.ensure() {
        return;
    }
    loop {
        thread::sleep(POLL_INTERVAL);
        if let TickOutcome::Failed { message } = watchdog.tick() {
            eprintln!("todo-mcp: the live view is unavailable: {message}");
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::str::FromStr;
    use std::thread;

    use todo_domain::{Repo, RepoSlug, Timestamp};

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

    /// The ensure called twice on one root (test group 2): the first
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

    /// The tick's owner arm (test group 3): the first tick starts,
    /// and every tick after is the no-op — this process's listener
    /// lives as long as the process.
    #[test]
    fn the_tick_starts_then_noops_while_owning() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
        assert_eq!(
            watchdog.tick(),
            TickOutcome::Started {
                url: format!("http://127.0.0.1:{}/", published(&root))
            }
        );
        assert_eq!(watchdog.tick(), TickOutcome::Owned);
        assert_eq!(watchdog.tick(), TickOutcome::Owned);
    }

    /// The follower's tick while the owner lives (test group 3): a
    /// no-op — attached again, the same port, no second listener.
    #[test]
    fn the_follower_tick_noops_while_the_owner_lives() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let port_file = super::super::port_file(&root);
        let owner = serve(&root, &port_file, None, POLL_INTERVAL).expect("the owner starts");
        let ServeOutcome::Started { url, .. } = owner else {
            panic!("the owner starts");
        };
        let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
        assert_eq!(watchdog.tick(), TickOutcome::Attached { url: url.clone() });
        assert_eq!(watchdog.tick(), TickOutcome::Attached { url });
    }

    /// The takeover (test group 3): a dead owner — its listener gone,
    /// its publication stale — is taken over: the tick binds its own
    /// port and republishes.
    #[test]
    fn the_tick_takes_over_a_dead_publication() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let port_file = super::super::port_file(&root);
        // A stale publication: a port that was once a listener's but
        // is now refused — the owner died without cleanup (D5's norm).
        let dead = TcpListener::bind(("127.0.0.1", 0)).expect("the dead owner binds");
        let dead_port = dead.local_addr().expect("the port").port();
        drop(dead);
        super::super::publish_port(&port_file, dead_port).expect("the stale publication lands");
        let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
        let TickOutcome::Started { url } = watchdog.tick() else {
            panic!("the dead publication is taken over");
        };
        let port = super::super::read_published_port(&port_file).expect("the publication");
        assert_ne!(port, dead_port, "the takeover binds its own port");
        assert_eq!(url, format!("http://127.0.0.1:{port}/"));
        assert!(super::super::listener_answers(port));
        // And the tick after the takeover is the owner's no-op.
        assert_eq!(watchdog.tick(), TickOutcome::Owned);
    }

    /// The double-bind race (test group 3): two first ticks over one
    /// root with no publication — whichever way the interleaving goes,
    /// the end state is one advertised live listener and the loser (if
    /// the race produced one) attached to it. Two ticks exactly: a
    /// third could supersede the publication after the loser's attach,
    /// which the four-party shape allowed and this pin does not. Two
    /// `Started` outcomes are possible (the double-overwrite
    /// interleaving leaves one orphaned listener on an unadvertised
    /// port — benign, spec-recorded); the publication is the truth,
    /// and it names a live port.
    #[test]
    fn racing_first_ticks_leave_one_advertised_listener() {
        let dir = registered_root();
        let root = DataRoot::new(dir.path());
        let port_file = super::super::port_file(&root);
        let mut ticks = Vec::new();
        for _ in 0..2 {
            let root = root.clone();
            ticks.push(thread::spawn(move || {
                let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
                watchdog.tick()
            }));
        }
        let mut started = 0;
        let mut attached_urls = Vec::new();
        for tick in ticks {
            match tick.join().expect("the tick completes") {
                TickOutcome::Started { .. } => started += 1,
                TickOutcome::Attached { url } => attached_urls.push(url),
                other => panic!("a first tick attaches or starts: {other:?}"),
            }
        }
        assert!(started >= 1, "at least one tick owns the listener");
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
    /// startup check — fails the tick without touching the wire.
    #[test]
    fn the_tick_fails_on_a_root_with_nothing_to_serve() {
        let dir = tempfile::tempdir().expect("a temp root");
        let root = DataRoot::new(dir.path());
        let mut watchdog = Watchdog::new(&root, None, POLL_INTERVAL);
        let TickOutcome::Failed { message } = watchdog.tick() else {
            panic!("an empty root fails the ensure");
        };
        assert_eq!(
            message,
            "the data root registers no repositories — nothing to serve"
        );
    }

    /// Reads the root's published port, for the URL assertions.
    fn published(root: &DataRoot) -> u16 {
        super::super::read_published_port(&super::super::port_file(root)).expect("the publication")
    }
}
