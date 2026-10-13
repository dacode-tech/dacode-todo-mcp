//! The daemon suite (U65, D25): every test spawns the real binary
//! with `--daemon` and drives it over the unix socket — the wire is
//! the product, and the socket is the daemon's whole surface. The
//! suite's pins are the task's acceptance: concurrent connections
//! each answering their own session (the preamble's per-connection
//! defaults, the shared core invisible), SIGTERM's bounded drain
//! (in-flight responses delivered, the process gone within
//! [`DRAIN_TIMEOUT`] plus slack, new connections refused), the
//! live-view listener owned by and dying with the daemon, the
//! second-daemon ensure, and the preamble fault path (a garbage
//! first line drops the connection with no MCP answer).
//!
//! The args surface (`--daemon` parses, the combination faults,
//! `USAGE` carries the line) is pinned in the lib's `args` tests,
//! where the parse lives. The preamble's accepted shapes are pinned
//! in the daemon module's own unit tests.
//!
//! The fixture is minimal — bare registered repos, no epics or
//! tasks, because `repo_show` reads the repo node and the daemon's
//! sessions never need more; two distinct slugs give the
//! per-connection defaults a real choice, and the registered paths
//! are placeholders the test's cwd never matches. Every store handle
//! is dropped before the daemon spawns (the advisory lock is per
//! process). The suite is unix-gated with the socket it drives.

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

#[cfg(unix)]
mod daemon {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::str::FromStr;
    use std::thread;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;
    use todo_domain::{Repo, RepoSlug, Timestamp};
    use todo_mcp::daemon::DRAIN_TIMEOUT;
    use todo_store::DataRoot;

    /// The fixture repo's slug.
    const FIXTURE: &str = "fixture";
    /// The second repo's slug — a different default for a different
    /// connection.
    const ALPHA: &str = "alpha";
    /// The `/health` marker (the listener's own contract).
    const HEALTH_MARKER: &str = "dacode-todo-mcp progress listener";
    /// How long a socket or listener wait polls before giving up.
    const WAIT_TIMEOUT: Duration = Duration::from_secs(20);

    /// A fixed canonical timestamp.
    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).expect("a fixed canonical timestamp")
    }

    /// Registers one bare repo — the repo node is all a daemon
    /// session's `repo_show` needs — dropping the store handle before
    /// the daemon spawns (the advisory lock is per process).
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
    /// slugs so the per-connection defaults have a real choice. The
    /// registered paths are placeholders the test's cwd never
    /// matches.
    fn daemon_root() -> TempDir {
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

    /// Spawns the daemon binary over `root` — stdout piped so the
    /// suite can assert it stays empty (the daemon's diagnostics are
    /// stderr's, the stdio discipline carried over).
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

    /// Connects to the daemon's socket, waiting for the bind — the
    /// socket answering a connect is the daemon's readiness. A
    /// failed connect before the bind is retried; the deadline turns
    /// "never" into a failing assertion instead of a hang.
    fn connect_when_ready(root: &Path) -> UnixStream {
        let path = root.join("serve.sock");
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            match UnixStream::connect(&path) {
                Ok(stream) => return stream,
                Err(_) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => panic!("the daemon's socket never answered: {error}"),
            }
        }
    }

    /// The initialize request every session opens with — protocol
    /// 2025-11-25, the version this server declares.
    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"u65","version":"0"}}}"#;

    /// The handshake-completion notification — no response.
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

    /// One live daemon session: the connected socket split into a
    /// reader and a writer, the preamble sent, the handshake driven
    /// interactively — one request written, its response line read,
    /// repeat (the protocol suite's discipline, so the response order
    /// is the request order by construction).
    struct Session {
        reader: BufReader<UnixStream>,
        writer: UnixStream,
    }

    impl Session {
        /// Connects (waiting for the bind), sends the preamble and
        /// the initialize request — both in flight before the
        /// response is read, the concurrency proof's shape (§6.4's
        /// probe held two initialize requests in flight the same
        /// way).
        fn open(root: &Path, preamble: &str) -> Session {
            let stream = connect_when_ready(root);
            let mut session = Session {
                reader: BufReader::new(stream.try_clone().expect("clone the socket")),
                writer: stream,
            };
            session.send(preamble);
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
            self.writer
                .write_all(line.as_bytes())
                .expect("write the line");
            self.writer.write_all(b"\n").expect("terminate the line");
        }

        /// Reads one response line with a bound — a missing line
        /// fails the assertion instead of hanging the suite.
        fn receive(&mut self) -> String {
            self.reader
                .get_mut()
                .set_read_timeout(Some(Duration::from_secs(10)))
                .expect("the read timeout");
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => panic!("the daemon closed the connection before answering"),
                Ok(_) => line,
                Err(error) => panic!("the response never arrived: {error}"),
            }
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

    /// Sends SIGTERM (the drain signal — `Child::kill` sends SIGKILL,
    /// so the signal needs the real tool, the serve suite's note) and
    /// collects the exit within a bound: a daemon that hangs past the
    /// bound fails the assertion instead of the suite.
    fn terminate(child: Child, bound: Duration) -> std::process::Output {
        let mut child = child;
        let signal = Command::new("/bin/kill")
            .arg("-TERM")
            .arg(child.id().to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGTERM is delivered");
        let deadline = Instant::now() + bound;
        loop {
            if child.try_wait().expect("the child is waitable").is_some() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "the daemon exits within the bound"
            );
            thread::sleep(Duration::from_millis(25));
        }
        child
            .wait_with_output()
            .expect("the daemon's output is collected")
    }

    /// Reads the published port: digits only, else `None`.
    fn read_port(port_file: &Path) -> Option<u16> {
        let text = std::fs::read_to_string(port_file).ok()?;
        let text = text.trim();
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        text.parse().ok()
    }

    /// One plain GET: `(status, body)` — a transport failure is
    /// `(None, "")`, never a panic.
    fn http_get(port: u16, path: &str) -> (Option<u16>, String) {
        let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
            return (None, String::new());
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        if stream.write_all(request.as_bytes()).is_err() {
            return (None, String::new());
        }
        let mut response = Vec::new();
        if stream.read_to_end(&mut response).is_err() {
            return (None, String::new());
        }
        let text = String::from_utf8_lossy(&response).into_owned();
        let status = text
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse().ok());
        let body = text
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_owned())
            .unwrap_or_default();
        (status, body)
    }

    /// Polls until the publication names a port that answers
    /// `/health` — probing, not just reading the file, so a stale
    /// publication never counts as up.
    fn wait_for_health(port_file: &Path, child: &mut Child) -> Option<u16> {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        while Instant::now() < deadline {
            if child.try_wait().expect("the child is waitable").is_some() {
                return None;
            }
            if let Some(port) = read_port(port_file)
                && let (Some(status), body) = http_get(port, "/health")
                && status == 200
                && body.contains(HEALTH_MARKER)
            {
                return Some(port);
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }

    /// 1. Multiple concurrent connections each answer their own
    ///    session: two connections, different preambles, both
    ///    initialize requests in flight before either response is
    ///    read; a relaxed `repo_show` (absent repo parameter) answers
    ///    each connection's own default, repeatedly — the memo is
    ///    per-connection, no cross-talk — and an explicit parameter
    ///    still wins (D8's order).
    #[test]
    fn concurrent_connections_each_answer_their_own_session() {
        let root = daemon_root();
        let child = spawn_daemon(root.path());

        let mut first = Session::open(root.path(), r#"{"repo":"fixture"}"#);
        let mut second = Session::open(root.path(), r#"{"repo":"alpha"}"#);
        first.finish_handshake();
        second.finish_handshake();

        let response = first.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        let response = second.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), ALPHA, "{response}");
        // Again, after the other connection's call: the derived
        // default is per-connection, never moved by a neighbor.
        let response = first.call(3, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        // An explicit parameter still wins.
        let response = first.call(4, "repo_show", r#"{"repo":"alpha"}"#);
        assert_eq!(slug_of(&response), ALPHA, "{response}");

        drop(first);
        drop(second);
        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        assert!(
            output.stdout.is_empty(),
            "the daemon's stdout stays empty: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    /// 2. SIGTERM drains in-flight queries and exits within the
    ///    bound: a request written and unanswered when SIGTERM lands
    ///    still gets its response delivered, the still-open
    ///    connection is *served* through the window (a new request
    ///    after SIGTERM is answered — the services are not cancelled
    ///    mid-window), the daemon stays up while the connection is
    ///    live (the drain waits, it does not cut), the exit lands
    ///    within `DRAIN_TIMEOUT` plus slack once the connection
    ///    closes, and a new connection after SIGTERM is refused.
    #[test]
    fn sigterm_drains_in_flight_queries_and_exits_within_the_bound() {
        let root = daemon_root();
        let mut child = spawn_daemon(root.path());
        let mut session = Session::open(root.path(), r#"{"repo":"fixture"}"#);
        session.finish_handshake();

        // The in-flight request: written, its response not yet read.
        session.send(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"repo_show","arguments":{}}}"#,
        );
        let signaled = Instant::now();
        let signal = Command::new("/bin/kill")
            .arg("-TERM")
            .arg(child.id().to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGTERM is delivered");

        // The in-flight response is delivered after SIGTERM.
        let response = session.receive();
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        // The window serves: a request written after SIGTERM is still
        // answered — the connection's service was not cancelled.
        let response = session.call(3, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        // The drain waits: with the connection still live, the daemon
        // has not exited a second into the window.
        assert!(
            child.try_wait().expect("the child is waitable").is_none(),
            "the daemon waits for the live connection, it does not cut it"
        );

        // The connection ends; the exit lands within the bound.
        drop(session);
        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        assert!(
            signaled.elapsed() < DRAIN_TIMEOUT + Duration::from_secs(5),
            "the exit landed within the bound plus slack: {:?}",
            signaled.elapsed()
        );
        // A new connection after SIGTERM is refused — the listener is
        // gone, the socket file left in place for the next owner.
        assert!(
            UnixStream::connect(root.path().join("serve.sock")).is_err(),
            "a new connection after SIGTERM is refused"
        );
        assert!(
            root.path().join("serve.sock").exists(),
            "the socket file is left for the next owner's ensure"
        );
    }

    /// 3. The live-view listener is owned by and moves with the
    ///    daemon: after startup the publication exists and `/health`
    ///    answers; after the process exits the listener is gone (the
    ///    publication file itself is left in place — the next owner's
    ///    ensure handles staleness).
    #[test]
    fn the_live_view_listener_is_owned_by_the_daemon() {
        let root = daemon_root();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_daemon(root.path());

        let port = wait_for_health(&port_file, &mut child).expect("the live view is up");
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert_eq!(body, format!("{HEALTH_MARKER}\n"));

        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        assert!(
            port_file.exists(),
            "the publication is left for the next owner"
        );
        let (status, _) = http_get(port, "/health");
        assert_eq!(status, None, "the listener died with the daemon");
    }

    /// 4. The second-daemon ensure: invoking again on the same root
    ///    exits 0 with one stderr line, without disturbing the first
    ///    — its live session still answers afterward.
    #[test]
    fn a_second_daemon_invocation_exits_zero_without_disturbing_the_first() {
        let root = daemon_root();
        let first = spawn_daemon(root.path());
        let mut session = Session::open(root.path(), r#"{"repo":"fixture"}"#);
        session.finish_handshake();

        let second = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("--daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("the second invocation runs");
        assert_eq!(second.status.code(), Some(0));
        assert!(
            second.stdout.is_empty(),
            "the second invocation prints nothing to stdout"
        );
        let stderr = String::from_utf8_lossy(&second.stderr);
        assert!(
            stderr.contains("already serves"),
            "one stderr line names the live daemon: {stderr}"
        );

        // The first daemon is undisturbed: its session still answers.
        let response = session.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        drop(session);
        let output = terminate(first, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
    }

    /// 5. The preamble fault path: a garbage first line drops the
    ///    connection with one stderr line and no MCP answer —
    ///    pre-protocol, nothing is answered.
    #[test]
    fn a_garbage_preamble_drops_the_connection_with_no_mcp_answer() {
        let root = daemon_root();
        let child = spawn_daemon(root.path());

        let stream = connect_when_ready(root.path());
        let mut writer = stream.try_clone().expect("clone the socket");
        let mut reader = BufReader::new(stream);
        writer
            .write_all(b"not json\n")
            .expect("the garbage line is sent");
        reader
            .get_mut()
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("the read timeout");
        let mut received = Vec::new();
        reader
            .read_to_end(&mut received)
            .expect("the connection closes");
        assert!(
            received.is_empty(),
            "no MCP answer: {:?}",
            String::from_utf8_lossy(&received)
        );
        drop(writer);

        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("the socket preamble is not valid"),
            "one stderr line names the fault: {stderr}"
        );
        assert!(output.stdout.is_empty(), "the daemon's stdout stays empty");
    }

    /// 5b. The non-UTF8 preamble: the read itself fails (a `String`
    ///     cannot carry invalid UTF-8), the connection is dropped
    ///     with one stderr line and no MCP answer — the same
    ///     pre-protocol contract as the garbage line, one fault
    ///     earlier.
    #[test]
    fn a_non_utf8_preamble_drops_the_connection_with_no_mcp_answer() {
        let root = daemon_root();
        let child = spawn_daemon(root.path());

        let stream = connect_when_ready(root.path());
        let mut writer = stream.try_clone().expect("clone the socket");
        let mut reader = BufReader::new(stream);
        writer
            .write_all(&[0xff, 0xfe, b'\n'])
            .expect("the non-utf8 line is sent");
        reader
            .get_mut()
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("the read timeout");
        let mut received = Vec::new();
        reader
            .read_to_end(&mut received)
            .expect("the connection closes");
        assert!(
            received.is_empty(),
            "no MCP answer: {:?}",
            String::from_utf8_lossy(&received)
        );
        drop(writer);

        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("the preamble read failed"),
            "one stderr line names the read fault: {stderr}"
        );
        assert!(output.stdout.is_empty(), "the daemon's stdout stays empty");
    }

    /// 6. The preamble's cwd: a connection whose preamble carries the
    ///    shim's working directory derives its default through the
    ///    cwd tier (§7's longest-ancestor match against a real
    ///    checkout), while a connection with an empty preamble
    ///    derives nothing — the tiers are per-connection, and the
    ///    two never cross.
    #[test]
    fn a_preamble_cwd_derives_the_connection_default() {
        let checkout = TempDir::new().expect("a fresh checkout");
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        register_repo(
            &data_root,
            "checkout",
            checkout.path().to_str().expect("a utf-8 checkout path"),
            "2026-10-13T08:00:00Z",
        );
        let child = spawn_daemon(root.path());

        let cwd_preamble = format!(
            r#"{{"cwd":{}}}"#,
            serde_json::json!(checkout.path().to_str().expect("a utf-8 checkout path"))
        );
        let mut located = Session::open(root.path(), &cwd_preamble);
        located.finish_handshake();
        let response = located.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), "checkout", "{response}");

        // An empty preamble: no repo, no cwd — the relaxed call has
        // no default to resolve and answers §6.1's `invalid`.
        let mut bare = Session::open(root.path(), "{}");
        bare.finish_handshake();
        let response = bare.call(2, "repo_show", "{}");
        assert!(
            response.contains(r#""isError":true"#),
            "the bare session has no default: {response}"
        );
        assert!(
            response.contains(r#""code":"invalid""#),
            "the refusal is the invalid carrier: {response}"
        );

        drop(located);
        drop(bare);
        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
    }

    /// 7. No connections at SIGTERM: a prompt exit, not the drain
    ///    bound — the bound exists for live sessions, and an empty
    ///    daemon has nothing to wait for. The readiness probe's
    ///    connect-and-drop is the EOF-silent contract's pin: the
    ///    daemon's stderr carries exactly the live-view URL line and
    ///    nothing for the dropped connection.
    #[test]
    fn sigterm_with_no_connections_exits_promptly() {
        let root = daemon_root();
        let child = spawn_daemon(root.path());
        // Readiness: the socket answers a connect, dropped at once —
        // the silent probe shape (nothing sent, nothing logged).
        drop(connect_when_ready(root.path()));

        let started = Instant::now();
        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        assert!(
            started.elapsed() < DRAIN_TIMEOUT - Duration::from_secs(2),
            "a prompt exit, not the drain bound: {:?}",
            started.elapsed()
        );
        // The probe's EOF stayed silent: exactly one stderr line, the
        // live view's — a regression that logs on the dropped probe
        // fails here.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let lines: Vec<&str> = stderr.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "the probe's EOF is silent, one stderr line only: {stderr}"
        );
        assert!(
            lines[0].contains("live view at"),
            "the one line is the URL line: {stderr}"
        );
    }

    /// 8. The drain bound with a connection that never ends: an idle
    ///    session (connected, handshaken, never closed) holds the
    ///    drain open, the daemon waits the full bound rather than
    ///    cutting early, exits 0 at it, and says so on stderr — the
    ///    bound is the documented contract, the line is the cut's
    ///    cause. This is also the drain's observed timing for the
    ///    shim (U66): a held connection means the daemon is gone at
    ///    the bound, not before.
    #[test]
    fn the_drain_bound_cuts_a_connection_that_never_ends() {
        let root = daemon_root();
        let child = spawn_daemon(root.path());
        let mut session = Session::open(root.path(), r#"{"repo":"fixture"}"#);
        session.finish_handshake();
        // The session is deliberately never closed: the idle client
        // the bound exists for.

        let signaled = Instant::now();
        let output = terminate(child, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        let elapsed = signaled.elapsed();
        assert!(
            elapsed >= DRAIN_TIMEOUT,
            "the daemon waits the bound before cutting: {elapsed:?}"
        );
        assert!(
            elapsed < DRAIN_TIMEOUT + Duration::from_secs(5),
            "the exit lands at the bound, not after it: {elapsed:?}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("the drain bound was reached"),
            "one stderr line names the cut: {stderr}"
        );
    }

    /// 9. The racing double-spawn (the ensure's TOCTOU, reproduced
    ///    twice in review): two daemons spawned concurrently on one
    ///    root. The serialized ensure makes the outcome
    ///    deterministic — exactly one serves (its session answers),
    ///    the other exits 0 having found it live; the loser's remove
    ///    can never land after the winner's bind, so the winner's
    ///    socket is never unlinked. The discriminator is the
    ///    already-serves line, on exactly one stderr.
    #[test]
    fn racing_daemon_spawns_leave_exactly_one_serving() {
        let root = daemon_root();
        let first = spawn_daemon(root.path());
        let second = spawn_daemon(root.path());

        // Someone serves: a session over the socket answers.
        let mut session = Session::open(root.path(), r#"{"repo":"fixture"}"#);
        session.finish_handshake();
        let response = session.call(2, "repo_show", "{}");
        assert_eq!(slug_of(&response), FIXTURE, "{response}");
        drop(session);

        // Both exit 0 — the loser already did (it found the winner
        // live under the ensure lock); the winner drains.
        let first_output = terminate(first, DRAIN_TIMEOUT + Duration::from_secs(5));
        let second_output = terminate(second, DRAIN_TIMEOUT + Duration::from_secs(5));
        assert_eq!(first_output.status.code(), Some(0));
        assert_eq!(second_output.status.code(), Some(0));
        let already = [
            String::from_utf8_lossy(&first_output.stderr),
            String::from_utf8_lossy(&second_output.stderr),
        ]
        .iter()
        .filter(|stderr| stderr.contains("already serves"))
        .count();
        assert_eq!(
            already,
            1,
            "exactly one daemon found the other live: {:?} / {:?}",
            String::from_utf8_lossy(&first_output.stderr),
            String::from_utf8_lossy(&second_output.stderr)
        );
    }
}
