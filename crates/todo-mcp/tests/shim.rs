//! The shim suite (U66, D25): every test drives the real binary with
//! no subcommand — the shape every existing client config names —
//! over raw stdio, so the full shim→daemon path is what runs: the
//! connect phase, the preamble, the byte proxy, and (for one test)
//! the degraded failure mode. The suite's pins are the task's
//! acceptance: a session answers through the spawned daemon with the
//! `--repo` flag reaching it as the per-connection default; the
//! daemon absent → the retry window → the degraded handshake
//! (byte-compared against a daemon-backed session's, so a client
//! cannot tell the modes apart) and every `tools/call` answering
//! §6.1's `io` carrier; the daemon spawned when absent and attached
//! to (never duplicated) when present; and the abnormal end — the
//! daemon lost mid-session — exiting 1 without waiting for the
//! client's still-open stdin (the binary's bounded teardown, pinned
//! by a test that hung on the pre-fix code).
//!
//! The window test waits out the real [`RETRY_WINDOW`] — the honest
//! cost, exactly one such test. The fixture is the daemon suite's
//! shape: bare registered repos (the `repo_show` answers need only
//! the repo node), two distinct slugs so the per-connection default
//! has a real choice, every store handle dropped before the binary
//! spawns (the advisory lock is per process). The daemon cleanup is
//! the common drive's own rule; the attach test drives manually so
//! the mid-session daemon census can run while the session is live.

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

mod shim {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::path::Path;
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::str::FromStr;
    use std::thread;
    use std::time::{Duration, Instant};

    use serde_json::Value;
    use tempfile::TempDir;
    use todo_domain::{Repo, RepoSlug, Timestamp};
    use todo_mcp::shim::RETRY_WINDOW;
    use todo_store::DataRoot;

    use super::common::{assert_carrier, call, daemon_pids, drive_with, stop_daemon};

    /// The fixture repo's slug.
    const FIXTURE: &str = "fixture";
    /// The second repo's slug — a different default for a different
    /// connection.
    const ALPHA: &str = "alpha";
    /// How long a manual session's reads wait before failing the
    /// assertion instead of hanging the suite.
    const READ_TIMEOUT: Duration = Duration::from_secs(20);
    /// How long the lost-daemon test waits for the shim's exit: the
    /// bounded teardown answers within milliseconds of the loss, so
    /// the bound exists to catch the unbounded wait (the pre-fix
    /// hang), not to tolerate one.
    const EXIT_BOUND: Duration = Duration::from_secs(10);

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
    /// slugs so the per-connection default has a real choice. The
    /// registered paths are placeholders the test's cwd never
    /// matches.
    fn shim_root() -> TempDir {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        register_repo(
            &data_root,
            FIXTURE,
            "/repos/fixture",
            "2026-10-13T08:00:00Z",
        );
        register_repo(&data_root, ALPHA, "/repos/alpha", "2026-10-13T09:00:00Z");
        root
    }

    /// The fixture slug, parsed.
    fn fixture_slug() -> RepoSlug {
        RepoSlug::from_str(FIXTURE).expect("a valid slug")
    }

    /// The repo slug of one `repo_show` response's structured
    /// payload.
    fn slug_of(response: &str) -> String {
        let value: Value = serde_json::from_str(response).expect("the response is JSON");
        value["result"]["structuredContent"]["slug"]
            .as_str()
            .expect("the repo slug")
            .to_owned()
    }

    /// One response line by its JSON-RPC id — never by position: rmcp
    /// dispatches each request in its own task (§6.3), so arrivals
    /// can reorder under a write-everything drive (the wire suite's
    /// rapid-fire rule).
    fn response_of(id: u64, lines: &[String]) -> &str {
        let prefix = format!(r#"{{"jsonrpc":"2.0","id":{id},""#);
        lines
            .iter()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("the id-{id} response is missing: {lines:?}"))
    }

    /// The initialize request every session opens with — protocol
    /// 2025-11-25, the version this server declares.
    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"u66","version":"0"}}}"#;

    /// The handshake-completion notification — no response.
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

    /// A `tools/list` request with a fixed id.
    const TOOLS_LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

    /// 1. A shim's session answers through the daemon: the relaxed
    ///    `repo_show` (absent repo parameter) answers the `--repo`
    ///    flag's default — the per-connection default the preamble
    ///    carried — and an explicit parameter still wins (D8's
    ///    order). The session cannot answer otherwise: D25's
    ///    no-fallback means every byte came through the daemon the
    ///    shim spawned.
    #[test]
    fn a_shims_session_answers_through_the_daemon() {
        let root = shim_root();
        let driven = drive_with(
            root.path(),
            Some(&fixture_slug()),
            &[
                call(2, "repo_show", "{}"),
                call(3, "repo_show", &format!(r#"{{"repo":"{ALPHA}"}}"#)),
            ],
        );
        assert!(
            driven.status.success(),
            "a completed session ends 0; stderr: {}",
            driven.stderr
        );
        assert_eq!(slug_of(&driven.responses[0]), FIXTURE);
        assert_eq!(slug_of(&driven.responses[1]), ALPHA);
    }

    /// 2. The daemon absent → the retry window → the degraded
    ///    failure mode. The root's socket path is occupied by a
    ///    directory — the connect fails, the spawned daemon's ensure
    ///    cannot remove it and exits — so the shim waits out the real
    ///    window (the honest cost; exactly one such test) and then
    ///    serves degraded: the initialize handshake and `tools/list`
    ///    byte-identical to a daemon-backed session's (a client
    ///    cannot tell the modes apart), every `tools/call` answering
    ///    §6.1's `io` carrier naming the daemon unreachable and the
    ///    window expired, and the client's close ending the process
    ///    0 — the session completed, degraded or not.
    #[test]
    fn the_daemon_absent_answers_the_io_carrier_after_the_window() {
        // The healthy root: a daemon-backed session, for the
        // byte-comparison.
        let healthy = shim_root();
        let healthy_lines = drive_lines(
            healthy.path(),
            &[
                INITIALIZE.to_owned(),
                INITIALIZED.to_owned(),
                TOOLS_LIST.to_owned(),
            ],
        );
        assert_eq!(healthy_lines.len(), 2, "the handshake and the list");

        // The broken root: the socket path occupied by a directory —
        // neither connectable nor removable, so the spawned daemon's
        // ensure fails and it exits, and no daemon can ever bind.
        let broken = shim_root();
        std::fs::create_dir(broken.path().join("serve.sock")).expect("the socket path is occupied");
        let started = Instant::now();
        let broken_lines = drive_lines(
            broken.path(),
            &[
                INITIALIZE.to_owned(),
                INITIALIZED.to_owned(),
                TOOLS_LIST.to_owned(),
                call(3, "repo_show", "{}"),
            ],
        );
        assert!(
            started.elapsed() >= RETRY_WINDOW,
            "the failure mode answered after the real window: {:?}",
            started.elapsed()
        );
        assert_eq!(
            broken_lines.len(),
            3,
            "the handshake, the list, the carrier"
        );

        // The handshake and the tool list are byte-identical between
        // the modes — the degraded server is the daemon's shape. The
        // responses are matched by JSON-RPC id (the module-level
        // helper's rule), never by position.
        assert_eq!(
            response_of(1, &broken_lines),
            response_of(1, &healthy_lines),
            "the handshake is byte-identical to the daemon's"
        );
        assert_eq!(
            response_of(2, &broken_lines),
            response_of(2, &healthy_lines),
            "tools/list is byte-identical to the daemon's"
        );
        // Every tool call answers the io carrier.
        let message = format!(
            "the daemon is unreachable: no connection to {} within the {}s retry window — \
             the tool did not run; retry the call once the daemon is back",
            broken.path().join("serve.sock").display(),
            RETRY_WINDOW.as_secs()
        );
        assert_carrier(response_of(3, &broken_lines), "io", &message);
    }

    /// 3. The daemon spawned when absent, attached to when present:
    ///    (a) no daemon pre-existing — the session works, which is
    ///    the proof of the spawn (the shim cannot answer otherwise);
    ///    (b) a daemon pre-existing — the shim attaches, and exactly
    ///    one daemon serves the root while the session is live: the
    ///    pre-existing pid, never a second one.
    #[test]
    fn the_daemon_is_spawned_when_absent_and_attached_when_present() {
        // (a) No daemon pre-existing: the session answers through the
        // daemon the shim spawned (the common drive stops it after).
        let root = shim_root();
        let driven = drive_with(
            root.path(),
            Some(&fixture_slug()),
            &[call(2, "repo_show", "{}")],
        );
        assert!(
            driven.status.success(),
            "a completed session ends 0; stderr: {}",
            driven.stderr
        );
        assert_eq!(slug_of(&driven.responses[0]), FIXTURE);

        // (b) A daemon pre-existing: spawn it, wait for the socket,
        // then drive a shim session manually so the daemon census can
        // run while the session is live.
        let present = shim_root();
        let daemon = spawn_daemon(present.path());
        wait_for_socket(present.path());
        let mut session = Session::open(present.path(), Some(&fixture_slug()));
        session.finish_handshake();
        let response = session.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        // Exactly one daemon serves this root while the session is
        // live: the pre-existing pid, never a second one.
        let pids = daemon_pids(present.path());
        assert_eq!(
            pids,
            vec![daemon.id()],
            "the shim attached to the pre-existing daemon, no second one appeared"
        );
        drop(session);
        // The common rule, applied by hand for the manual drive: stop
        // the daemon, then reap the test's own child.
        stop_daemon(present.path());
        let output = daemon.wait_with_output().expect("the daemon is reaped");
        assert_eq!(output.status.code(), Some(0));
    }

    /// 4. The abnormal end does not hang the exit: a live session
    ///    whose daemon is SIGKILLed — the connection lost while the
    ///    client's stdin is still open — answers one stderr line and
    ///    exit 1 within the bound, never waiting for the client. The
    ///    pre-fix shape hung exactly here: the runtime's implicit
    ///    drop waited out the client task's in-flight blocking-pool
    ///    stdin read, so the exit code arrived only when the client
    ///    closed — a supervisor waiting on the exit (U67's restart
    ///    flow) would deadlock. The fix is the binary's bounded
    ///    teardown; this test pins it.
    #[test]
    fn a_lost_daemon_exits_one_without_waiting_for_the_client() {
        let root = shim_root();
        let mut child = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("--repo")
            .arg(FIXTURE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the shim spawns");
        let mut stdin = child.stdin.take().expect("the shim's stdin");
        let mut stdout = BufReader::new(child.stdout.take().expect("the shim's stdout"));
        // The handshake completes through the daemon the shim spawned.
        stdin
            .write_all(format!("{INITIALIZE}\n{INITIALIZED}\n").as_bytes())
            .expect("the handshake is sent");
        let mut handshake = String::new();
        stdout
            .read_line(&mut handshake)
            .expect("the handshake arrives");
        assert!(
            handshake.starts_with(r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"#),
            "{handshake}"
        );
        // The daemon dies hard — SIGKILL, no drain — while the
        // client's stdin stays open.
        let pids = daemon_pids(root.path());
        assert_eq!(pids.len(), 1, "one daemon serves the root");
        let signal = Command::new("/bin/kill")
            .arg("-9")
            .arg(pids[0].to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGKILL is delivered");
        // The shim answers the loss without waiting for the client:
        // exit 1 within the bound.
        let deadline = Instant::now() + EXIT_BOUND;
        let status = loop {
            if let Some(status) = child.try_wait().expect("the shim is waitable") {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "the shim exits within the bound after the loss — the exit must not \
                 wait on the client's open stdin"
            );
            thread::sleep(Duration::from_millis(25));
        };
        assert_eq!(status.code(), Some(1), "the lost connection is exit 1");
        drop(stdin);
        let mut stderr = String::new();
        child
            .stderr
            .take()
            .expect("the shim's stderr")
            .read_to_string(&mut stderr)
            .expect("read the shim's stderr");
        assert!(
            stderr.contains("the daemon connection was lost mid-session"),
            "one stderr line names the loss: {stderr}"
        );
    }

    /// Drives the shim in the protocol suite's shape — every request
    /// line written, stdin closed, exit waited out, lines collected —
    /// stopping the daemon the shim leaves behind (the common
    /// drive's own rule, U66).
    fn drive_lines(root: &Path, requests: &[String]) -> Vec<String> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn todo-mcp");
        {
            let mut stdin = child.stdin.take().expect("the child's stdin");
            let mut text = String::new();
            for request in requests {
                text.push_str(request);
                text.push('\n');
            }
            stdin
                .write_all(text.as_bytes())
                .expect("write the requests");
        }
        let output = child.wait_with_output().expect("wait for the shim");
        stop_daemon(root);
        let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
        let lines: Vec<String> = stdout.lines().map(str::to_owned).collect();
        let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
        assert!(
            output.status.success(),
            "a completed session ends 0; stderr: {stderr}"
        );
        lines
    }

    /// Spawns the daemon binary over `root` — the daemon suite's
    /// shape, stdout piped so it stays assertably empty.
    fn spawn_daemon(root: &Path) -> Child {
        Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root)
            .arg("--daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the daemon spawns")
    }

    /// Waits for the root's socket to answer a connect — the
    /// daemon's readiness (the silent probe shape: nothing sent,
    /// nothing logged).
    fn wait_for_socket(root: &Path) {
        let path = root.join("serve.sock");
        let deadline = Instant::now() + READ_TIMEOUT;
        loop {
            match std::os::unix::net::UnixStream::connect(&path) {
                Ok(_) => return,
                Err(_) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => panic!("the daemon's socket never answered: {error}"),
            }
        }
    }

    /// One manual shim session over stdio: the shim's stdin and
    /// stdout held open, the handshake driven interactively — the
    /// shape the daemon census needs (the session live while the
    /// process table is read).
    struct Session {
        stdin: Option<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        child: Child,
    }

    impl Session {
        /// Spawns the shim (no subcommand, `--repo` as the default)
        /// and sends the initialize request.
        fn open(root: &Path, repo: Option<&RepoSlug>) -> Session {
            let mut command = Command::new(env!("CARGO_BIN_EXE_todo-mcp"));
            command.arg("--root").arg(root);
            if let Some(slug) = repo {
                command.arg("--repo").arg(slug.as_str());
            }
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("the shim spawns");
            let stdin = child.stdin.take().expect("the shim's stdin");
            let stdout = BufReader::new(child.stdout.take().expect("the shim's stdout"));
            let mut session = Session {
                stdin: Some(stdin),
                stdout,
                child,
            };
            session.send(INITIALIZE);
            session
        }

        /// Reads the initialize response and completes the handshake.
        fn finish_handshake(&mut self) {
            let response = self.receive();
            assert!(
                response.starts_with(
                    r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25""#
                ),
                "the handshake answered: {response}"
            );
            self.send(INITIALIZED);
        }

        /// Sends one `tools/call` and reads its response line.
        fn call(&mut self, id: u64, name: &str, arguments: &str) -> String {
            self.send(&format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{arguments}}}}}"#
            ));
            self.receive()
        }

        /// Writes one line.
        fn send(&mut self, line: &str) {
            let stdin = self.stdin.as_mut().expect("the session's stdin");
            stdin.write_all(line.as_bytes()).expect("write the line");
            stdin.write_all(b"\n").expect("terminate the line");
        }

        /// Reads one response line with a bound — a missing line
        /// fails the assertion instead of hanging the suite.
        fn receive(&mut self) -> String {
            let mut line = String::new();
            match self.stdout.read_line(&mut line) {
                Ok(0) => panic!("the shim closed its stdout before answering"),
                Ok(_) => line,
                Err(error) => panic!("the response never arrived: {error}"),
            }
        }
    }

    impl Drop for Session {
        /// Closes the session's stdin — the shim's natural end — and
        /// waits for its exit: a completed session ends 0, with the
        /// shim's stderr empty on the green path.
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
}
