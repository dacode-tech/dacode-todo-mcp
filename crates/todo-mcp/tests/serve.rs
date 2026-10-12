//! The serve subcommand's suite (t71, D20's third slice) and the D13
//! default-server live view (t85): the listener driven through the
//! real binary — spawned via `CARGO_BIN_EXE_todo-mcp` against a temp
//! root, the per-root default publication (`<root>/serve.port`,
//! D13's flip) making every spawn hermetic by construction;
//! `--port-file` remains the override, pinned by its own test — the
//! integration pins of the task spec, mirroring the Python driver
//! `tools/progress-serve-check.py` whose checks they inherit: the
//! static invocation publishes no port; `serve` publishes and answers
//! every route — the epic page's `/repo/<slug>/epic/<number>` among
//! them (E12, U53: the page answered, the named 404s for an unknown
//! repo or epic, the malformed shapes over the wire; U54: the repo
//! page's epic links are epic-page URLs and both pages carry the lazy
//! navigation script); a second invocation attaches; a stale
//! publication is
//! taken over (SIGTERM via `/bin/kill`, unix-gated — `Child::kill`
//! sends SIGKILL, so the signals need the real tool); SIGINT is the
//! clean shutdown (the t55 driver's note: the pin runs from the
//! test's child, which gets the default disposition); the WS push
//! channel end-to-end (the RFC 6455 example key, a store write
//! through the `Store` trait, the push frame within a bounded wait);
//! the WS control frames (a masked ping answered with a pong, a
//! masked close echoed and the connection ended); and a corrupt-store
//! root where `/health` stays 200 while `/` and the epic route answer
//! 500 and the listener stays up.
//!
//! The D13 pins: two roots key two listeners (the per-root keying);
//! the no-subcommand server — the shape every opencode session spawns
//! through the global config — ensures the listener while its stdio
//! protocol stays pure (stdout carries only protocol bytes, the URL
//! line is stderr's), a second spawn attaches to the same port, and
//! the survivor's watchdog takes over when the owner dies (bounded
//! waits, the t62 precedent — no flaky sleeps).
//!
//! The unit pins (SHA-1's vectors, the frame codec, the handshake
//! accept, the poller, the HTTP parser, the watchdog tick) live
//! in-file in the serve module per the house style; the dispatch rows
//! live in main.rs's tests. The fixture is minimal — one epic and one
//! task per repo, two repos so the `?repo=` parameter and the
//! first-slug fallback both have a real choice — and every store
//! handle is dropped before the binary spawns (the advisory lock is
//! per process).

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

mod serve {

    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    use std::path::Path;
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::str::FromStr;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;
    use todo_domain::{
        EpicDraft, References, Repo, RepoSlug, Status, TaskDraft, TaskKind, Timestamp,
    };
    use todo_store::{DataRoot, Store};

    /// The fixture repo's slug.
    const FIXTURE: &str = "fixture";
    /// The second repo's slug — it sorts before `fixture`, so `/`'s
    /// first-slug fallback answers it (the registered checkouts are
    /// placeholders the test's cwd never matches).
    const ALPHA: &str = "alpha";
    /// The `/health` marker (byte-identical to the Python's).
    const HEALTH_MARKER: &str = "dacode-todo-mcp progress listener";
    /// The RFC 6455 example handshake pair.
    const WS_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
    const WS_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
    /// How long the listener wait polls before giving up.
    const LISTEN_TIMEOUT: Duration = Duration::from_secs(20);

    /// A fixed canonical timestamp.
    fn ts(value: &str) -> Timestamp {
        Timestamp::parse(value).expect("a fixed canonical timestamp")
    }

    /// The fixture repo's slug, parsed.
    fn fixture_slug() -> RepoSlug {
        RepoSlug::from_str(FIXTURE).expect("a valid slug")
    }

    /// Builds one minimal renderable repo — an epic and a task —
    /// registered at a placeholder path the test's cwd never matches.
    fn build_repo(root: &DataRoot, slug: &str, instant: &str) {
        let mut store = root
            .register(Repo {
                slug: RepoSlug::from_str(slug).expect("a valid slug"),
                name: slug.to_owned(),
                path: format!("/repos/{slug}"),
                registered_at: ts(instant),
            })
            .expect("the repo registers");
        let epic = store
            .add_epic(
                EpicDraft {
                    title: format!("The {slug} epic"),
                    description: "An epic.".to_owned(),
                    status: Status::Open,
                    references: References::new(),
                },
                ts("2026-10-02T08:01:00Z"),
            )
            .expect("the epic lands");
        store
            .add_task(
                TaskDraft {
                    epic: epic.id,
                    kind: TaskKind::Task,
                    title: format!("A {slug} task"),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test -p fixture".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                ts("2026-10-02T08:02:00Z"),
            )
            .expect("the task lands");
    }

    /// A temp root with both repos registered.
    fn serve_fixture() -> TempDir {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        build_repo(&data_root, FIXTURE, "2026-10-02T08:00:00Z");
        build_repo(&data_root, ALPHA, "2026-10-02T09:00:00Z");
        root
    }

    /// Spawns the binary serving `root`, the port published to the
    /// per-root default (`<root>/serve.port`) — stdout piped so the
    /// serving line can be read.
    fn spawn_serve(root: &Path) -> Child {
        Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root)
            .arg("serve")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the listener spawns")
    }

    /// Spawns the binary as the MCP server (no subcommand) over
    /// `root` — the shape every opencode session spawns through the
    /// global config, and the shape D13's ensure-listener rides.
    fn spawn_server(root: &Path) -> Child {
        Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the server spawns")
    }

    /// The initialize request and its completion notification — the
    /// handshake every server spawn is driven through.
    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t85","version":"0"}}}"#;
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

    /// Writes the handshake to a server child's stdin and puts the
    /// handle back — dropping it would close the pipe, which is a
    /// stdio server's natural end; the test closes it deliberately
    /// when its assertions are done.
    fn handshake(child: &mut Child) {
        let mut stdin = child.stdin.take().expect("the piped stdin");
        stdin
            .write_all(format!("{INITIALIZE}\n{INITIALIZED}\n").as_bytes())
            .expect("the handshake is sent");
        child.stdin = Some(stdin);
    }

    /// Closes a server child's stdin — the stdio server's natural end;
    /// the child is expected to exit 0 after a completed handshake.
    fn end_session(child: &mut Child) {
        drop(child.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("the child is waitable") {
                assert_eq!(status.code(), Some(0), "a completed handshake ends 0");
                return;
            }
            assert!(
                Instant::now() < deadline,
                "the server exits after the close"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Reads one line from a reader with a bound (the t62 precedent:
    /// a missing line fails the assertion instead of hanging the
    /// suite). The reader is consumed — one line per stream per test.
    fn read_line_bounded<R: Read + Send + 'static>(reader: R, timeout: Duration) -> Option<String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            let read = reader.read_line(&mut line);
            let _ = sender.send(if read.unwrap_or(0) > 0 {
                Some(line)
            } else {
                None
            });
        });
        receiver.recv_timeout(timeout).ok().flatten()
    }

    /// Watches a server child's stdout for bytes beyond the first
    /// response line: stdout is the JSON-RPC channel and must carry
    /// nothing else (D3's purity — the URL line is stderr's). The
    /// first line is read byte-at-a-time (no read-ahead), then any
    /// further byte within the window is a purity failure.
    fn stdout_watch(
        stdout: ChildStdout,
    ) -> (
        std::sync::mpsc::Receiver<String>,
        std::sync::mpsc::Receiver<usize>,
    ) {
        let (line_sender, line_receiver) = std::sync::mpsc::channel();
        let (extra_sender, extra_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = stdout;
            let mut line = Vec::new();
            let mut byte = [0u8; 1];
            loop {
                match stdout.read(&mut byte) {
                    Ok(0) => break,
                    Ok(_) => {
                        line.push(byte[0]);
                        if byte[0] == b'\n' {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = line_sender.send(String::from_utf8_lossy(&line).into_owned());
            // Any byte after the first line — the purity watch. EOF
            // reads 0 and is reported as such (a live server's stdout
            // must not close either).
            let mut probe = [0u8; 256];
            let seen = stdout.read(&mut probe).unwrap_or(0);
            let _ = extra_sender.send(seen);
        });
        (line_receiver, extra_receiver)
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

    /// Polls until the publication names a port that answers
    /// `/health` — or the process dies, or the timeout passes.
    /// Probing (not just reading the file) is what makes the
    /// stale-publication takeover honest: a stale port in the file
    /// never counts as published.
    fn wait_for_listener(port_file: &Path, child: &mut Child) -> Option<u16> {
        let deadline = Instant::now() + LISTEN_TIMEOUT;
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
            std::thread::sleep(Duration::from_millis(100));
        }
        None
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

    /// The clean shutdown: SIGINT where the signal exists (the
    /// listener's own contract), SIGKILL elsewhere — the temp root's
    /// cleanup carries the publication either way.
    fn shutdown(child: &mut Child) {
        #[cfg(unix)]
        {
            let _ = Command::new("/bin/kill")
                .arg("-INT")
                .arg(child.id().to_string())
                .status();
        }
        #[cfg(not(unix))]
        {
            let _ = child.kill();
        }
        let _ = child.wait();
    }

    /// 1. The default `progress` invocation stays static: exit 0, no
    ///    port published — a listener would hang. `--repo` names the
    ///    repo because the test's cwd matches no registered checkout
    ///    (the placeholder paths), which the subcommand would refuse.
    #[test]
    fn the_progress_invocation_publishes_no_port() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let output = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("progress")
            .arg("--repo")
            .arg(FIXTURE)
            .arg("--no-write")
            .output()
            .expect("the render spawns");
        assert_eq!(output.status.code(), Some(0));
        assert!(
            !port_file.exists(),
            "the static invocation publishes no port"
        );
    }

    /// 2. `serve` publishes and answers: the port file appears, the
    ///    serving line names the URL, `/` answers the first-slug
    ///    fallback's page, `/?repo=` answers THAT repo's page, `/json`
    ///    parses, `/health` answers the marker, an unknown path is 404.
    #[test]
    fn serve_publishes_and_answers() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let port =
            wait_for_listener(&port_file, &mut child).expect("the listener publishes and answers");

        // The serving line names the published URL (stdout is piped —
        // the line must still arrive, flushed before the block).
        let mut stdout = BufReader::new(child.stdout.take().expect("the piped stdout"));
        let mut line = String::new();
        stdout.read_line(&mut line).expect("the serving line");
        assert!(
            line.contains(&format!("serving http://127.0.0.1:{port}/")),
            "the serving line: {line}"
        );

        // `/` answers the first slug's page (alpha sorts first; the
        // cwd matches no registered checkout).
        let (status, body) = http_get(port, "/");
        assert_eq!(status, Some(200));
        assert!(
            body.contains("<title>alpha · progress</title>"),
            "the fallback page: {}",
            &body[..body.len().min(200)]
        );
        // `/?repo=fixture` answers THAT repo's page.
        let (status, body) = http_get(port, "/?repo=fixture");
        assert_eq!(status, Some(200));
        assert!(body.contains("<title>fixture · progress</title>"));
        // `/json` parses.
        let (status, body) = http_get(port, "/json");
        assert_eq!(status, Some(200));
        serde_json::from_str::<serde_json::Value>(&body).expect("the JSON twin parses");
        // `/health` answers the marker.
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert_eq!(body, format!("{HEALTH_MARKER}\n"));
        // An unknown path is 404.
        let (status, _) = http_get(port, "/nope");
        assert_eq!(status, Some(404));

        shutdown(&mut child);
        assert_eq!(child.wait().expect("the child is reaped").code(), Some(0));
    }

    /// 2b. The epic page route (E12, U53): `/repo/<slug>/epic/<number>`
    ///     answers the epic page — the title names the epic, the
    ///     sidebar marks it current — for both repos; `?repo=` never
    ///     overrides the path; an unknown repo and an unknown epic are
    ///     named 404s; the malformed number shapes 404 with the generic
    ///     unknown-path body (the classifier's full matrix lives in
    ///     http.rs's unit tests — these rows pin that its 404s reach
    ///     the socket). U54's rows: the served repo page's epic links
    ///     are `/repo/<slug>/epic/<n>` URLs (same-repo and cross-repo
    ///     alike) and both pages carry the lazy navigation script.
    #[test]
    fn the_epic_page_route_answers() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let port = wait_for_listener(&port_file, &mut child).expect("the listener answers");

        // The epic page at its path: the title names the epic, the
        // sidebar's current-epic marker is present, and the lazy
        // navigation script rides the page.
        let (status, body) = http_get(port, "/repo/fixture/epic/1");
        assert_eq!(status, Some(200));
        assert!(
            body.contains("<title>fixture · E1 — The fixture epic</title>"),
            "the epic page's title: {}",
            &body[..body.len().min(200)]
        );
        assert!(
            body.contains(r#"<span class="scur">E1 · The fixture epic</span>"#),
            "the sidebar marks the epic current"
        );
        assert!(
            body.contains("async function fetchView(url, push)"),
            "the epic page carries the lazy navigation script"
        );
        // The second repo's route answers its own page.
        let (status, body) = http_get(port, "/repo/alpha/epic/1");
        assert_eq!(status, Some(200));
        assert!(body.contains("<title>alpha · E1 — The alpha epic</title>"));
        // The path is the repo's only source: a disagreeing `?repo=`
        // never overrides it.
        let (status, body) = http_get(port, "/repo/fixture/epic/1?repo=alpha");
        assert_eq!(status, Some(200));
        assert!(body.contains("<title>fixture · E1 — The fixture epic</title>"));
        // The repo page's epic links are epic-page URLs — the
        // same-repo rows and the cross-repo rows alike — and the lazy
        // navigation script rides the page too.
        let (status, body) = http_get(port, "/?repo=fixture");
        assert_eq!(status, Some(200));
        assert!(
            body.contains(r#"<a href="/repo/fixture/epic/1">"#),
            "the same-repo epic row links the epic page"
        );
        assert!(
            body.contains(r#"<a href="/repo/alpha/epic/1">"#),
            "the cross-repo epic row links the epic page"
        );
        assert!(
            body.contains("async function fetchView(url, push)"),
            "the repo page carries the lazy navigation script"
        );
        // An unknown repo is a named 404.
        let (status, body) = http_get(port, "/repo/elsewhere/epic/1");
        assert_eq!(status, Some(404));
        assert_eq!(body, "no repo elsewhere under this root\n");
        // An unknown epic is a named 404.
        let (status, body) = http_get(port, "/repo/fixture/epic/99");
        assert_eq!(status, Some(404));
        assert_eq!(body, "fixture holds no epic 99\n");
        // The malformed number shapes 404 over the wire, with the
        // generic unknown-path body.
        for path in ["/repo/fixture/epic/0", "/repo/fixture/epic/abc"] {
            let (status, body) = http_get(port, path);
            assert_eq!(status, Some(404), "path: {path}");
            assert_eq!(body, "not found\n", "the unknown-path body: {path}");
        }

        shutdown(&mut child);
    }

    /// 3. A second invocation attaches: exit 0, the same URL on
    ///    stdout, the publication unchanged, the first listener still
    ///    answering — never a second listener. The second invocation
    ///    is waited on with a bound: an attach exits promptly, and a
    ///    regression that starts a second listener would block
    ///    forever instead of failing.
    #[test]
    fn a_second_invocation_attaches() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut first = spawn_serve(root.path());
        let port = wait_for_listener(&port_file, &mut first).expect("the first listener answers");

        let mut second = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("serve")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the second invocation spawns");
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = second.try_wait().expect("the child is waitable") {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "the second invocation did not attach — it is serving instead"
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(status.code(), Some(0));
        let mut stdout = String::new();
        second
            .stdout
            .take()
            .expect("the piped stdout")
            .read_to_string(&mut stdout)
            .expect("the attach's stdout");
        assert!(
            stdout.contains(&format!("http://127.0.0.1:{port}/")),
            "the attach names the same URL: {stdout}"
        );
        assert_eq!(
            read_port(&port_file),
            Some(port),
            "the publication is unchanged"
        );

        let (status, body) = http_get(port, "/");
        assert_eq!(status, Some(200));
        assert!(body.contains("<title>alpha · progress</title>"));

        shutdown(&mut first);
    }

    /// 4. A stale publication is taken over: SIGTERM (the default
    ///    disposition — no cleanup) leaves the publication; the next
    ///    `serve` binds its own port and answers.
    #[cfg(unix)]
    #[test]
    fn a_stale_publication_is_taken_over() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut first = spawn_serve(root.path());
        let _port = wait_for_listener(&port_file, &mut first).expect("the first listener answers");

        // SIGTERM: the process dies, the publication survives as stale.
        let signal = Command::new("/bin/kill")
            .arg("-TERM")
            .arg(first.id().to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGTERM is delivered");
        first.wait().expect("the first listener dies");
        assert!(port_file.exists(), "SIGTERM leaves the stale publication");

        // The next invocation takes over: its own port, answering.
        let mut third = spawn_serve(root.path());
        let new_port = wait_for_listener(&port_file, &mut third).expect("the takeover binds");
        let (status, body) = http_get(new_port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));

        shutdown(&mut third);
    }

    /// 5. SIGINT is the clean shutdown: exit 0, the publication
    ///    removed. The signal runs from the test's child via `/bin/kill`
    ///    — the child gets the default disposition, like a real
    ///    terminal's Ctrl-C (the t55 driver's note).
    #[cfg(unix)]
    #[test]
    fn sigint_is_the_clean_shutdown() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let _port = wait_for_listener(&port_file, &mut child).expect("the listener answers");

        let signal = Command::new("/bin/kill")
            .arg("-INT")
            .arg(child.id().to_string())
            .status()
            .expect("the signal tool runs");
        assert!(signal.success(), "the SIGINT is delivered");
        // Poll the exit: a broken shutdown must fail, not hang the
        // suite.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("the child is waitable") {
                assert_eq!(status.code(), Some(0), "SIGINT exits 0");
                break;
            }
            assert!(
                Instant::now() < deadline,
                "the listener shut down within the bound"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(
            !port_file.exists(),
            "the clean shutdown removes the publication"
        );
    }

    /// Reads one response head (through the blank line).
    fn read_head(stream: &mut TcpStream) -> String {
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            assert_eq!(
                stream.read(&mut byte).expect("a head byte"),
                1,
                "the head completes"
            );
            head.push(byte[0]);
            if head.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8_lossy(&head).into_owned()
    }

    /// Reads one server frame: the opcode and the payload.
    fn read_frame(stream: &mut TcpStream) -> (u8, Vec<u8>) {
        let mut head = [0u8; 2];
        stream.read_exact(&mut head).expect("the frame head");
        let length = match u16::from(head[1] & 0x7f) {
            126 => {
                let mut bytes = [0u8; 2];
                stream.read_exact(&mut bytes).expect("the 16-bit length");
                u64::from(u16::from_be_bytes(bytes))
            }
            127 => {
                let mut bytes = [0u8; 8];
                stream.read_exact(&mut bytes).expect("the 64-bit length");
                u64::from_be_bytes(bytes)
            }
            small => u64::from(small),
        };
        let mut payload = vec![0u8; length as usize];
        stream.read_exact(&mut payload).expect("the payload");
        (head[0] & 0x0f, payload)
    }

    /// Writes one masked client frame (RFC 6455: client frames MUST
    /// be masked — the server refuses unmasked ones).
    fn write_client_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) {
        let mask = [0x37u8, 0xfa, 0x21, 0x3d];
        let mut frame = Vec::with_capacity(payload.len() + 10);
        frame.push(0x80 | opcode);
        let length = payload.len();
        if length < 126 {
            frame.push(0x80 | length as u8);
        } else if length <= usize::from(u16::MAX) {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(length as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(length as u64).to_be_bytes());
        }
        frame.extend_from_slice(&mask);
        frame.extend(
            payload
                .iter()
                .zip(mask.iter().cycle())
                .map(|(byte, key)| byte ^ key),
        );
        stream.write_all(&frame).expect("the client frame is sent");
    }

    /// 6. The WS push channel end-to-end: the RFC 6455 handshake over
    ///    a real socket (the known key → the published accept), no write
    ///    → no frame within several poll intervals, a store write through
    ///    the `Store` trait → the push frame within a bounded wait.
    #[test]
    fn the_ws_push_channel_delivers_on_change() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let port = wait_for_listener(&port_file, &mut child).expect("the listener answers");

        // The handshake: the known key computes the published accept.
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("the WS socket connects");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("the read timeout");
        let request = format!(
            "GET /ws?repo={FIXTURE} HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\n\
             Connection: Upgrade\r\nSec-WebSocket-Key: {WS_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .expect("the handshake is sent");
        let head = read_head(&mut stream);
        assert!(
            head.starts_with("HTTP/1.1 101"),
            "the upgrade answers 101: {head}"
        );
        assert!(
            head.contains(&format!("Sec-WebSocket-Accept: {WS_ACCEPT}\r\n")),
            "the accept is the RFC's: {head}"
        );

        // No write: no frame within several poll intervals (the
        // binary's interval is 500ms; three intervals plus slack).
        stream
            .set_read_timeout(Some(Duration::from_millis(1800)))
            .expect("the read timeout");
        let mut probe = [0u8; 1];
        let quiet = stream.read(&mut probe);
        assert!(
            quiet.is_err(),
            "no frame without a change (read: {quiet:?}, byte: {:?})",
            probe.first()
        );

        // A store write through the Store trait: the event log moves,
        // the poller sees it, the push frame arrives.
        let data_root = DataRoot::new(root.path());
        let mut store = data_root.open(&fixture_slug()).expect("the store opens");
        let epic = store
            .graph()
            .epics()
            .next()
            .expect("the fixture epic")
            .id
            .clone();
        store
            .add_task(
                TaskDraft {
                    epic,
                    kind: TaskKind::Task,
                    title: "The pushed task".to_owned(),
                    body: String::new(),
                    status: Status::Open,
                    verify: Some("cargo test -p fixture".to_owned()),
                    evidence: None,
                    note: None,
                    references: References::new(),
                    priority: None,
                    until: None,
                },
                ts("2026-10-02T10:00:00Z"),
            )
            .expect("the write lands");
        drop(store);

        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("the read timeout");
        let (opcode, payload) = read_frame(&mut stream);
        assert_eq!(opcode, 0x1, "the push is a text frame");
        assert_eq!(
            payload,
            format!("{{\"repo\":\"{FIXTURE}\",\"changed\":true}}").as_bytes()
        );

        shutdown(&mut child);
    }

    /// 6b. The WS control frames: a masked ping is answered with a
    ///     pong carrying the same payload; a masked close is echoed
    ///     and the connection ends (the poller goes down with it).
    #[test]
    fn the_ws_control_frames_are_served() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let port = wait_for_listener(&port_file, &mut child).expect("the listener answers");

        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("the WS socket connects");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("the read timeout");
        let request = format!(
            "GET /ws?repo={FIXTURE} HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\n\
             Connection: Upgrade\r\nSec-WebSocket-Key: {WS_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .expect("the handshake is sent");
        let head = read_head(&mut stream);
        assert!(
            head.starts_with("HTTP/1.1 101"),
            "the upgrade answers 101: {head}"
        );

        // A ping: answered with a pong carrying the same payload.
        write_client_frame(&mut stream, 0x9, b"hi");
        let (opcode, payload) = read_frame(&mut stream);
        assert_eq!(opcode, 0xA, "the ping is answered with a pong");
        assert_eq!(payload, b"hi".to_vec());

        // A close: echoed, then the connection ends — the next read is
        // EOF (Ok(0)) or a reset, never a frame and never a timeout
        // (a timeout would pass a socket that leaks past the echo).
        write_client_frame(&mut stream, 0x8, &[]);
        let (opcode, payload) = read_frame(&mut stream);
        assert_eq!(opcode, 0x8, "the close is echoed");
        assert_eq!(payload, Vec::<u8>::new());
        let mut probe = [0u8; 1];
        let ended = stream.read(&mut probe);
        let ended_honestly = match &ended {
            Ok(0) => true,
            Ok(_) => false,
            Err(error) => !matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ),
        };
        assert!(
            ended_honestly,
            "the connection ends after the close echo (read: {ended:?})"
        );

        shutdown(&mut child);
    }

    /// 7. A corrupt-store root: `/health` still 200 (no render), `/`
    ///    and the epic route answer 500 (the resolve fails before any
    ///    render, the unreadable-store arm of the epic route's
    ///    decision table), the listener stays up.
    #[test]
    fn a_corrupt_store_root_still_serves_health() {
        let root = TempDir::new().expect("a fresh temp root");
        let data_root = DataRoot::new(root.path());
        build_repo(&data_root, FIXTURE, "2026-10-02T08:00:00Z");
        std::fs::write(
            root.path()
                .join("repos")
                .join(FIXTURE)
                .join("event-log.jsonl"),
            "not a log\n",
        )
        .expect("the store is corrupted");

        let port_file = root.path().join("serve.port");
        let mut child = spawn_serve(root.path());
        let port = wait_for_listener(&port_file, &mut child).expect("the listener starts");

        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));
        let (status, _) = http_get(port, "/");
        assert_eq!(status, Some(500), "the corrupt store's render fails");
        let (status, _) = http_get(port, "/repo/fixture/epic/1");
        assert_eq!(
            status,
            Some(500),
            "the corrupt store's epic route resolve fails"
        );
        // The listener stays up after the failure.
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));

        shutdown(&mut child);
    }

    /// 8. The `--port-file` override (D1): an explicit path wins over
    ///    the per-root default — the publication lands there, and the
    ///    default path is never touched.
    #[test]
    fn the_port_file_override_publishes_there() {
        let root = serve_fixture();
        let default = root.path().join("serve.port");
        let override_path = root.path().join("elsewhere.port");
        let mut child = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("serve")
            .arg("--port-file")
            .arg(&override_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the listener spawns");
        let port = wait_for_listener(&override_path, &mut child).expect("the override publishes");
        assert!(
            !default.exists(),
            "the per-root default is not touched when the override is given"
        );
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));

        shutdown(&mut child);
    }

    /// 9. The per-root keying (D13, test group 1): two roots, two
    ///    listeners, two port files — the default publication keys by
    ///    root, so the roots coexist by construction.
    #[test]
    fn two_roots_get_two_listeners() {
        let first = serve_fixture();
        let second = serve_fixture();
        let first_port_file = first.path().join("serve.port");
        let second_port_file = second.path().join("serve.port");
        let mut first_child = spawn_serve(first.path());
        let mut second_child = spawn_serve(second.path());
        let first_port = wait_for_listener(&first_port_file, &mut first_child)
            .expect("the first root's listener answers");
        let second_port = wait_for_listener(&second_port_file, &mut second_child)
            .expect("the second root's listener answers");
        assert_ne!(first_port, second_port, "two live listeners hold two ports");
        for port in [first_port, second_port] {
            let (status, body) = http_get(port, "/health");
            assert_eq!(status, Some(200));
            assert!(body.contains(HEALTH_MARKER));
        }

        shutdown(&mut first_child);
        shutdown(&mut second_child);
        assert!(
            !first_port_file.exists(),
            "the first root's publication is removed"
        );
        assert!(
            !second_port_file.exists(),
            "the second root's publication is removed"
        );
    }

    /// 10. The MCP-path integration (D13, test group 4 — the
    ///     load-bearing test): the no-subcommand server — the shape
    ///     every opencode session spawns — ensures the root's live
    ///     view while its stdio protocol stays pure. The initialize
    ///     response is the only stdout bytes (the URL line is
    ///     stderr's, D3), the port file appears, `/health` answers,
    ///     the stderr carries the URL line, and a second spawn
    ///     attaches — the same port, no second listener.
    #[test]
    fn the_default_server_ensures_the_live_view() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");

        let mut first = spawn_server(root.path());
        handshake(&mut first);
        let (line_receiver, extra_receiver) =
            stdout_watch(first.stdout.take().expect("the piped stdout"));
        let response = line_receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("the initialize response arrives");
        assert!(
            response.starts_with(r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"#),
            "the initialize response: {response}"
        );
        assert!(
            response.contains(r#""serverInfo":{"name":"todo-mcp""#),
            "the initialize response: {response}"
        );
        let port =
            wait_for_listener(&port_file, &mut first).expect("the server's listener answers");
        // The purity watch: after the listener is up (the first tick
        // resolved — any misdirected URL line has landed by now),
        // stdout must carry nothing beyond the response line.
        match extra_receiver.recv_timeout(Duration::from_secs(1)) {
            Err(_) => {}
            Ok(0) => panic!("the server closed its stdout while alive"),
            Ok(seen) => panic!("stdout carries bytes beyond the protocol: {seen} bytes"),
        }
        let stderr_line = read_line_bounded(
            first.stderr.take().expect("the piped stderr"),
            Duration::from_secs(10),
        )
        .expect("the URL line arrives");
        assert_eq!(
            stderr_line,
            format!("todo-mcp: live view at http://127.0.0.1:{port}/\n"),
            "the URL line is stderr's, exactly D3's format"
        );

        // The second spawn attaches: the same port, no second listener.
        let mut second = spawn_server(root.path());
        handshake(&mut second);
        let (second_line, second_extra) =
            stdout_watch(second.stdout.take().expect("the piped stdout"));
        let response = second_line
            .recv_timeout(Duration::from_secs(10))
            .expect("the second initialize response arrives");
        assert!(
            response.starts_with(r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"#),
            "the second initialize response: {response}"
        );
        let stderr_line = read_line_bounded(
            second.stderr.take().expect("the piped stderr"),
            Duration::from_secs(10),
        )
        .expect("the second URL line arrives");
        assert_eq!(
            stderr_line,
            format!("todo-mcp: live view at http://127.0.0.1:{port}/\n"),
            "the second spawn attaches to the same port"
        );
        assert_eq!(
            read_port(&port_file),
            Some(port),
            "the publication is unchanged — no second listener"
        );
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));
        drop(second_extra);

        end_session(&mut first);
        end_session(&mut second);
    }

    /// 11. The watchdog end-to-end (D13, test group 5, the t62
    ///     bounded-wait precedent): two spawned servers over one
    ///     root; the owner dies (SIGKILL — no cleanup, D5's stale
    ///     publication); the survivor's watchdog takes over within a
    ///     generous bounded window — the view is back up.
    #[test]
    fn the_survivor_takes_over_after_the_owner_dies() {
        let root = serve_fixture();
        let port_file = root.path().join("serve.port");

        let mut owner = spawn_server(root.path());
        handshake(&mut owner);
        let port = wait_for_listener(&port_file, &mut owner).expect("the owner's listener answers");
        // The owner's stderr URL line is drained so the pipe cannot
        // fill; its content is test 10's pin.
        drop(read_line_bounded(
            owner.stderr.take().expect("the piped stderr"),
            Duration::from_secs(10),
        ));

        let mut survivor = spawn_server(root.path());
        handshake(&mut survivor);
        let survivor_line = read_line_bounded(
            survivor.stderr.take().expect("the piped stderr"),
            Duration::from_secs(10),
        )
        .expect("the survivor's URL line arrives");
        assert_eq!(
            survivor_line,
            format!("todo-mcp: live view at http://127.0.0.1:{port}/\n"),
            "the survivor attached to the owner's listener"
        );

        // The owner dies with its process — SIGKILL, no cleanup: the
        // publication goes stale (D5's norm).
        owner.kill().expect("the owner dies");
        owner.wait().expect("the owner is reaped");

        // The survivor's watchdog takes over within the window: the
        // publication names a port that answers again.
        let deadline = Instant::now() + Duration::from_secs(10);
        let revived = loop {
            if let Some(revived) = read_port(&port_file)
                && let (Some(status), body) = http_get(revived, "/health")
                && status == 200
                && body.contains(HEALTH_MARKER)
            {
                break revived;
            }
            assert!(
                Instant::now() < deadline,
                "the survivor took over within the window"
            );
            std::thread::sleep(Duration::from_millis(100));
        };
        assert!(
            survivor
                .try_wait()
                .expect("the survivor is waitable")
                .is_none(),
            "the survivor is still serving"
        );
        let _ = revived;

        end_session(&mut survivor);
    }
}
