//! The update-cycle suite (U67, D25): the end-to-end proof of the
//! arc's last move — the node's verify, verbatim: "two shims on one
//! daemon, the daemon killed and restarted under them, the next
//! query reconnecting to the new process without a session
//! restart". Two shim processes over one root and one daemon answer
//! concurrent queries; the daemon is SIGTERM'd mid-session; an
//! in-flight query from each shim is answered during the drain (the
//! daemon serves until the bound); the connections break at the
//! drain's END; the shims reconnect — respawning the daemon — and
//! each shim's NEXT query answers through the new daemon with no
//! session restart: the clients initialized exactly once and never
//! re-send `initialize` (the shim's handshake-on-behalf re-arms the
//! daemon-side session invisibly), yet still get tool answers.
//!
//! The new-inode shape the task body names is the test's own setup:
//! the binary is copied to a scratch path, the shims are spawned
//! from THAT path, and the file is replaced with a fresh copy (cp to
//! a temp name, then `mv` — the install ritual in miniature) before
//! the SIGTERM, so the respawn — `current_exe` resolving the scratch
//! path — necessarily execs the replaced inode. The proof is direct:
//! the path's inode changes across the replace (asserted), and the
//! respawned daemon's executing inode (lsof's `txt` entry) is the
//! replaced one, not the one the shims themselves still execute.
//!
//! The timing: the drain bound is the honest cost — the connections
//! break at the drain's END, so no poll for the respawn can complete
//! before [`DRAIN_TIMEOUT`] has elapsed; that poll is the suite's
//! one such wait (absorbed, not slept). The reconnect itself is
//! observed as the gap between the old daemon's exit and the new
//! one's appearance. The fixture is the shim suite's shape: bare
//! registered repos (the `repo_show` answers need only the repo
//! node), two distinct slugs so the per-connection defaults have a
//! real choice, every store handle dropped before the binary spawns
//! (the advisory lock is per process). The suite is unix-gated with
//! the daemon and the inode reads it drives.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

mod common;

#[cfg(unix)]
mod update {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, Command, Stdio};
    use std::str::FromStr;
    use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
    use std::thread;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;
    use todo_domain::{Repo, RepoSlug, Timestamp};
    use todo_mcp::daemon::DRAIN_TIMEOUT;
    use todo_mcp::shim::RETRY_WINDOW;
    use todo_store::DataRoot;

    use super::common::{call, daemon_pids, stop_daemon};

    /// The fixture repo's slug.
    const FIXTURE: &str = "fixture";
    /// The second repo's slug — a different default for a different
    /// shim.
    const ALPHA: &str = "alpha";
    /// How long a manual session's reads wait before failing the
    /// assertion instead of hanging the suite.
    const READ_TIMEOUT: Duration = Duration::from_secs(20);
    /// The bound for the old daemon's exit: the drain bound plus
    /// slack — the connections are live at SIGTERM, so the daemon
    /// waits the full bound before cutting them.
    const EXIT_BOUND: Duration = Duration::from_secs(DRAIN_TIMEOUT.as_secs() + 5);
    /// The bound for the respawn: the reconnect window plus slack —
    /// the shims observe the loss at the drain's end and retry the
    /// connect within the window.
    const RESPAWN_BOUND: Duration = Duration::from_secs(RETRY_WINDOW.as_secs() + 5);

    /// A fixed canonical timestamp.
    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).expect("a fixed canonical timestamp")
    }

    /// Registers one bare repo — the repo node is all a session's
    /// `repo_show` needs — dropping the store handle before the
    /// binary spawns (the advisory lock is per process).
    fn register_repo(root: &DataRoot, slug: &str, path: &str, instant: &str) {
        let store = root
            .register(Repo {
                slug: RepoSlug::from_str(slug).expect("a valid slug"),
                name: slug.to_owned(),
                path: path.to_owned(),
                registered_at: ts(instant),
            })
            .expect("the repo registers");
        drop(store);
    }

    /// A temp root with two bare repos registered — two distinct
    /// slugs so the two shims' defaults have a real choice. The
    /// registered paths are placeholders the test's cwd never
    /// matches.
    fn update_root() -> TempDir {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        register_repo(
            &data_root,
            FIXTURE,
            "/repos/fixture",
            "2026-10-14T08:00:00Z",
        );
        register_repo(&data_root, ALPHA, "/repos/alpha", "2026-10-14T09:00:00Z");
        root
    }

    /// Installs the binary to a scratch path — the path the shims
    /// are spawned from and the daemon is respawned through.
    fn install_scratch(dir: &Path) -> PathBuf {
        let binary = dir.join("todo-mcp");
        std::fs::copy(env!("CARGO_BIN_EXE_todo-mcp"), &binary).expect("the scratch copy lands");
        binary
    }

    /// Replaces the scratch binary with a fresh copy — cp to a temp
    /// name, then `mv` over — the install ritual in miniature: the
    /// path names a new inode, the running shims keep executing the
    /// old one.
    fn replace_scratch(binary: &Path) {
        let staged = binary.with_extension("next");
        std::fs::copy(env!("CARGO_BIN_EXE_todo-mcp"), &staged).expect("the fresh copy lands");
        std::fs::rename(&staged, binary).expect("the replace lands");
    }

    /// A path's inode — the fresh-inode ritual's observable.
    fn inode_of(path: &Path) -> u64 {
        std::fs::metadata(path).expect("the path stats").ino()
    }

    /// The executing inode of `pid`'s binary — lsof's `txt` entry for
    /// the scratch path: the direct proof that the respawned daemon
    /// runs the replaced inode rather than the one the shims
    /// themselves still execute.
    fn executing_inode(pid: u32, binary: &Path) -> Option<u64> {
        let output = Command::new("lsof")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let path = binary.as_os_str().to_string_lossy();
        text.lines()
            .filter(|line| line.contains(" txt "))
            .find(|line| line.contains(path.as_ref()))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(7)
                    .and_then(|node| node.parse().ok())
            })
    }

    /// The initialize request every session opens with — protocol
    /// 2025-11-25, the version this server declares. Sent exactly
    /// once per session: the no-restart assertion is that no second
    /// initialize is ever sent.
    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"u67","version":"0"}}}"#;

    /// The handshake-completion notification — no response.
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

    /// One live shim session over stdio, spawned from the scratch
    /// binary: the shim's stdin held open, its stdout drained by a
    /// dedicated reader thread (a pipe read cannot be timed out in
    /// place, so the bound lives on the channel receive), the
    /// handshake driven interactively, `send` and `receive` split so
    /// a query can be written while the shim is mid-reconnect (the
    /// bytes then ride the stdin pipe — the design's backpressure
    /// bound).
    struct Session {
        stdin: Option<ChildStdin>,
        lines: Receiver<String>,
        child: Child,
    }

    impl Session {
        /// Spawns the shim from `binary` (no subcommand, `--repo` as
        /// the default) and completes the handshake — the one
        /// initialize this session will ever see.
        fn open(binary: &Path, root: &Path, repo: &str) -> Session {
            let mut child = Command::new(binary)
                .arg("--root")
                .arg(root)
                .arg("--repo")
                .arg(repo)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("the shim spawns");
            let stdin = child.stdin.take().expect("the shim's stdin");
            let mut stdout = BufReader::new(child.stdout.take().expect("the shim's stdout"));
            let (sender, lines) = channel();
            // The reader thread: one line per send; EOF or a read
            // fault drops the sender, which the receiver reads as
            // Disconnected. The thread ends with the shim's stdout.
            thread::spawn(move || {
                let mut line = String::new();
                loop {
                    match stdout.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if sender.send(std::mem::take(&mut line)).is_err() {
                                break;
                            }
                        }
                    }
                }
            });
            let mut session = Session {
                stdin: Some(stdin),
                lines,
                child,
            };
            session.send(INITIALIZE);
            session.send(INITIALIZED);
            let response = session.receive();
            assert!(
                response.starts_with(
                    r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25""#
                ),
                "the handshake answered: {response}"
            );
            session
        }

        /// Sends one `tools/call` and reads its response line.
        fn call(&mut self, id: u64, name: &str, arguments: &str) -> String {
            self.send(&format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{arguments}}}}}"#
            ));
            self.receive()
        }

        /// Writes one line without reading — the shape a query sent
        /// during the reconnect window needs.
        fn send(&mut self, line: &str) {
            let stdin = self.stdin.as_mut().expect("the session's stdin");
            stdin.write_all(line.as_bytes()).expect("write the line");
            stdin.write_all(b"\n").expect("terminate the line");
        }

        /// Reads one response line with a real bound: the blocking
        /// read runs on the session's reader thread and the wait is a
        /// bounded channel receive — a silently-hung shim fails the
        /// assertion instead of hanging the suite (a plain blocking
        /// `read_line` cannot be bounded on a pipe).
        fn receive(&mut self) -> String {
            match self.lines.recv_timeout(READ_TIMEOUT) {
                Ok(line) => line,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("the response never arrived within {READ_TIMEOUT:?}")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("the shim closed its stdout before answering")
                }
            }
        }
    }

    impl Drop for Session {
        /// Closes the session's stdin — the shim's natural end — and
        /// waits for its exit: a completed session ends 0, with the
        /// shim's stderr empty — the reconnect is silent, the client
        /// never knew.
        fn drop(&mut self) {
            drop(self.stdin.take());
            let deadline = Instant::now() + READ_TIMEOUT;
            let status = loop {
                if let Some(status) = self.child.try_wait().expect("the shim is waitable") {
                    break status;
                }
                assert!(Instant::now() < deadline, "the shim exits after the close");
                thread::sleep(Duration::from_millis(25));
            };
            assert_eq!(status.code(), Some(0), "a completed session ends 0");
            let mut stderr = String::new();
            if let Some(mut pipe) = self.child.stderr.take() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            assert!(stderr.is_empty(), "the shim's stderr stays empty: {stderr}");
        }
    }

    /// The repo slug of one `repo_show` response's structured
    /// payload.
    fn slug_of(response: &str) -> String {
        let value: serde_json::Value =
            serde_json::from_str(response).expect("the response is JSON");
        value["result"]["structuredContent"]["slug"]
            .as_str()
            .expect("the repo slug")
            .to_owned()
    }

    /// Polls until the daemon census satisfies `want`, bounded — the
    /// poll that cannot complete before the drain bound is the
    /// suite's one honest wait.
    fn wait_for_daemons(root: &Path, want: impl Fn(&[u32]) -> bool) -> Vec<u32> {
        let deadline = Instant::now() + EXIT_BOUND + RESPAWN_BOUND;
        loop {
            let pids = daemon_pids(root);
            if want(&pids) {
                return pids;
            }
            assert!(
                Instant::now() < deadline,
                "the daemon census never satisfied the wait: {pids:?}"
            );
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// The update cycle, end to end: two shims on one daemon, the
    /// daemon killed and restarted under them — the replaced binary
    /// is what restarts — and the next query reconnecting to the new
    /// process without a session restart.
    #[test]
    fn the_update_cycle_reconnects_both_shims_without_a_session_restart() {
        let root = update_root();
        let scratch = TempDir::new().expect("a fresh scratch dir");
        let binary = install_scratch(scratch.path());
        let inode_before = inode_of(&binary);

        // Two shims from the scratch path over one daemon: both
        // queries in flight before either answer is read — genuinely
        // concurrent — and exactly one daemon serves them.
        let mut first = Session::open(&binary, root.path(), FIXTURE);
        let mut second = Session::open(&binary, root.path(), ALPHA);
        first.send(&call(2, "repo_show", "{}"));
        second.send(&call(2, "repo_show", "{}"));
        assert_eq!(slug_of(&first.receive()), FIXTURE);
        assert_eq!(slug_of(&second.receive()), ALPHA);
        let daemon = wait_for_daemons(root.path(), |pids| pids.len() == 1);
        let old_daemon = daemon[0];

        // The fresh-inode replace: the path now names a new inode,
        // the running shims keep executing the old one.
        replace_scratch(&binary);
        let inode_after = inode_of(&binary);
        assert_ne!(
            inode_before, inode_after,
            "the replace is a fresh inode, not an in-place rewrite"
        );

        // The cycle: SIGTERM the daemon mid-session. An in-flight
        // query from each shim is answered during the drain — the
        // daemon serves until the bound.
        let signaled = Instant::now();
        let signal = Command::new("/bin/kill")
            .arg("-TERM")
            .arg(old_daemon.to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGTERM is delivered");
        assert_eq!(
            slug_of(&first.call(3, "repo_show", "{}")),
            FIXTURE,
            "the in-flight query is answered during the drain"
        );
        assert_eq!(
            slug_of(&second.call(3, "repo_show", "{}")),
            ALPHA,
            "the in-flight query is answered during the drain"
        );

        // The connections break at the drain's END: the daemon waits
        // the bound with the shims' connections live, then exits.
        // This poll is the suite's one honest wait — it cannot
        // complete before the bound.
        wait_for_daemons(root.path(), |pids| !pids.contains(&old_daemon));
        let drained = signaled.elapsed();
        println!(
            "update cycle: the drain held the connections for {drained:?} \
             (the bound is {DRAIN_TIMEOUT:?})"
        );
        assert!(
            drained >= DRAIN_TIMEOUT,
            "the connections broke at the drain's end, not before: {drained:?}"
        );

        // The next queries ride the reconnect: written now — the
        // connection is broken, the shims are mid-reconnect, the
        // bytes sit in the stdin pipe — and answered once the new
        // sessions are live. No initialize is re-sent: the
        // no-session-restart assertion.
        first.send(&call(4, "repo_show", "{}"));
        second.send(&call(4, "repo_show", "{}"));

        // The shims reconnect, respawning the daemon from the scratch
        // path — the replaced inode.
        let respawned =
            wait_for_daemons(root.path(), |pids| pids.len() == 1 && pids[0] != old_daemon);
        let new_daemon = respawned[0];
        let reconnected = signaled.elapsed();
        println!(
            "update cycle: the respawn landed {reconnected:?} after the signal \
             ({:?} after the drain's end)",
            reconnected - drained
        );
        assert!(
            reconnected - drained <= RETRY_WINDOW,
            "the reconnect landed within the window: {:?}",
            reconnected - drained
        );
        assert_eq!(
            executing_inode(new_daemon, &binary),
            Some(inode_after),
            "the respawned daemon executes the replaced inode"
        );

        // The answers arrive through the new daemon — the clients
        // initialized exactly once, never re-sent, still answered.
        assert_eq!(
            slug_of(&first.receive()),
            FIXTURE,
            "the next query answers through the new daemon, no session restart"
        );
        assert_eq!(
            slug_of(&second.receive()),
            ALPHA,
            "the next query answers through the new daemon, no session restart"
        );
        println!("update cycle: the next queries answered {reconnected:?} after the signal");

        // The sessions end naturally: exit 0, stderr empty — the
        // reconnect was silent, the clients never knew.
        drop(first);
        drop(second);
        stop_daemon(root.path());
    }
}
