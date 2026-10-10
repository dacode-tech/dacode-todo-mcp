//! The serve subcommand's suite (t71, D20's third slice): the live
//! listener driven through the real binary — spawned via
//! `CARGO_BIN_EXE_todo-mcp` against a temp root with `--port-file`
//! (the hermetic seam; the per-user default is never touched) — the
//! integration pins of the task spec, mirroring the Python
//! driver `tools/progress-serve-check.py` whose checks they inherit:
//! the static invocation publishes no port; `serve` publishes and
//! answers every route; a second invocation attaches; a stale
//! publication is taken over (SIGTERM via `/bin/kill`, unix-gated —
//! `Child::kill` sends SIGKILL, so the signals need the real tool);
//! SIGINT is the clean shutdown (the t55 driver's note: the pin runs
//! from the test's child, which gets the default disposition); the WS
//! push channel end-to-end (the RFC 6455 example key, a store write
//! through the `Store` trait, the push frame within a bounded wait);
//! the WS control frames (a masked ping answered with a pong, a
//! masked close echoed and the connection ended); and a corrupt-store
//! root where `/health` stays 200 while `/`
//! answers 500 and the listener stays up.
//!
//! The unit pins (SHA-1's vectors, the frame codec, the handshake
//! accept, the poller, the HTTP parser) live in-file in the serve
//! module per the house style; the dispatch rows live in main.rs's
//! tests. The fixture is minimal — one epic and one task per repo,
//! two repos so the `?repo=` parameter and the first-slug fallback
//! both have a real choice — and every store handle is dropped before
//! the binary spawns (the advisory lock is per process).

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
    use std::process::{Child, Command, Stdio};
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

    /// Spawns the binary serving `root` with the port published to
    /// `port_file` — stdout piped so the serving line can be read.
    fn spawn_serve(root: &Path, port_file: &Path) -> Child {
        Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root)
            .arg("serve")
            .arg("--port-file")
            .arg(port_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the listener spawns")
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
        let mut child = spawn_serve(root.path(), &port_file);
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
        let mut first = spawn_serve(root.path(), &port_file);
        let port = wait_for_listener(&port_file, &mut first).expect("the first listener answers");

        let mut second = Command::new(env!("CARGO_BIN_EXE_todo-mcp"))
            .arg("--root")
            .arg(root.path())
            .arg("serve")
            .arg("--port-file")
            .arg(&port_file)
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
        let mut first = spawn_serve(root.path(), &port_file);
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
        let mut third = spawn_serve(root.path(), &port_file);
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
        let mut child = spawn_serve(root.path(), &port_file);
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
        let mut child = spawn_serve(root.path(), &port_file);
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
        let mut child = spawn_serve(root.path(), &port_file);
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
    ///    answers 500, the listener stays up.
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
        let mut child = spawn_serve(root.path(), &port_file);
        let port = wait_for_listener(&port_file, &mut child).expect("the listener starts");

        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));
        let (status, _) = http_get(port, "/");
        assert_eq!(status, Some(500), "the corrupt store's render fails");
        // The listener stays up after the failure.
        let (status, body) = http_get(port, "/health");
        assert_eq!(status, Some(200));
        assert!(body.contains(HEALTH_MARKER));

        shutdown(&mut child);
    }
}
