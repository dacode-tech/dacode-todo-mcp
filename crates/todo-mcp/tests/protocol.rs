//! The protocol-level suite (spec §10): every test spawns the real
//! binary over stdio, writes newline-delimited JSON-RPC to its stdin
//! and reads stdout byte-for-byte — the §6.3 survey's probe pattern,
//! in Rust. The wire is the product: no test calls a handler function
//! around it.
//!
//! Exact-byte assertions are frozen from a measured run: every field
//! was read and checked against the spec's requirements before
//! freezing — the initialize response against §6.3's measured shapes
//! (our `serverInfo`, the 2025-11-25 handshake, U16a's instructions),
//! the error carriers against §6.1/D15, the success payload against
//! D14's declaration order. The one documented exception is
//! `repo_register`'s clock-stamped `registered_at`: a mutating tool's
//! response carries the clock, so the test extracts the stamped value
//! and pins every other byte around it.

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

mod protocol {
    use std::io::Write as _;
    use std::path::Path;
    use std::process::{Command, ExitStatus, Stdio};

    use tempfile::TempDir;
    use todo_store::DataRoot;

    /// U16a's base instructions as measured on the wire — one frozen
    /// copy, shared by the handshake and discover assertions.
    const INSTRUCTIONS: &str = "You are connected to a deterministic task-graph server. \
        Work is tracked as typed nodes: an epic groups work, a task is one unit \
        (kinds: task, gate, finding, debt), a decision is a dossier that blocks \
        work until the user signs it, and a doc is a registered document cited \
        by stable id. Ids are stable and never reused. Ask `next_task` for what \
        to do now, `next_items` for the batch, `status_summary` for the \
        roll-up. When the user assigns a work stream (one epic), pass its \
        `epic` on every selection call — `next_task`, `next_items`, \
        `task_list` and `status_summary` all take it; sessions scoped to \
        different epics of one repository never cross. Closing a task requires \
        `verify` and `evidence`; `blocked` and \
        `deferred` require a `note`; signing a decision requires a \
        `resolution`. Writes return the new state. Errors carry a stable code \
        in `structuredContent.code` — branch on the code, never parse the \
        message. Tool arguments are strict JSON — every string value quoted, \
        enum spellings exactly as the schema lists them. If a call fails, read \
        the error text — argument faults carry no code, and the text is the \
        remedy: it names the missing or unknown field, and a misspelled \
        value's expected variants; correct the arguments and retry once. The \
        same failure twice means stop and tell the user, quoting the tool, the \
        arguments and the error verbatim — never work around a failing \
        tracker by editing files or guessing state. Tasks carry an agent-set \
        `priority` (high, medium, low) — set \
        it when triaging, prefer higher priority when choosing among \
        actionable rows yourself; the server's order is dependency-driven and \
        ignores it by design.";

    /// The initialize request every handshake test opens with —
    /// protocol 2025-11-25, the version this server declares.
    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t33","version":"0"}}}"#;

    /// The handshake-completion notification.
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

    /// The measured initialize response: our serverInfo (not rmcp's
    /// `rmcp/3.5.0` default), tools-only capabilities, the 2025-11-25
    /// handshake version, and the instructions — field order is
    /// rmcp's declaration order, frozen with the rest.
    fn initialize_response() -> String {
        format!(
            concat!(
                r#"{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":"2025-11-25","#,
                r#""capabilities":{{"tools":{{}}}},"serverInfo":{{"name":"todo-mcp","version":"0.1.0"}},"#,
                r#""instructions":"{}"}}}}"#
            ),
            INSTRUCTIONS
        )
    }

    /// A `repo_register` call for `camera`.
    const REGISTER_CAMERA: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"repo_register","arguments":{"slug":"camera","name":"Camera","path":"/Users/probe/camera"}}}"#;

    /// Spawns the binary with `--root`, feeds `requests`
    /// (newline-delimited JSON-RPC) to its stdin, closes stdin — which
    /// is a stdio server's natural end — waits for exit, and returns
    /// stdout and stderr beside the exit status, which each test
    /// asserts for its own case: a completed handshake ends 0, a peer
    /// that closes before initialize ends 1.
    fn drive(root: &Path, requests: &str) -> (String, String, ExitStatus) {
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
            stdin
                .write_all(requests.as_bytes())
                .expect("write the requests");
        }
        let output = child.wait_with_output().expect("wait for the server");
        (
            String::from_utf8(output.stdout).expect("stdout is utf-8"),
            String::from_utf8(output.stderr).expect("stderr is utf-8"),
            output.status,
        )
    }

    /// The response lines, in arrival order.
    fn responses(stdout: &str) -> Vec<&str> {
        stdout.lines().collect()
    }

    /// Extracts the clock-stamped `registered_at` value from a
    /// response line — the one field a test cannot freeze, because a
    /// mutating tool's response carries the clock. The needle's plain
    /// quotes cannot match the text mirror's escaped spelling, so it
    /// finds the `structuredContent` occurrence.
    fn registered_at_of(response: &str) -> String {
        const NEEDLE: &str = r#""registered_at":""#;
        const TIMESTAMP_LEN: usize = 30;
        let start = response
            .find(NEEDLE)
            .expect("the response carries registered_at")
            + NEEDLE.len();
        let end = start + TIMESTAMP_LEN;
        response[start..end].to_owned()
    }

    /// the initialize handshake answers with our identity, the
    /// 2025-11-25 protocol version, tools-only capabilities and the
    /// instructions — exact bytes.
    #[test]
    fn initialize_carries_our_identity_the_handshake_and_the_instructions() {
        let root = TempDir::new().unwrap();
        let (stdout, stderr, status) =
            drive(root.path(), &format!("{INITIALIZE}\n{INITIALIZED}\n"));
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        assert_eq!(
            responses(&stdout),
            vec![initialize_response().as_str()],
            "stderr: {stderr}"
        );
    }

    /// the `initialized` notification completes the handshake
    /// silently — no response bytes exist beyond the initialize
    /// answer.
    #[test]
    fn the_initialized_notification_answers_nothing() {
        let root = TempDir::new().unwrap();
        let (stdout, stderr, status) =
            drive(root.path(), &format!("{INITIALIZE}\n{INITIALIZED}\n"));
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 1, "stderr: {stderr}");
        assert_eq!(responses[0], initialize_response());
    }

    /// `repo_register`'s exact advertisement within `tools/list` — the
    /// frozen bytes t33 pinned as a whole one-tool response, unwrapped
    /// from that response's shell: the router now composes across
    /// cluster files (t34's `ToolRouter::merge`), so the list holds
    /// every landed tool while this entry stays byte-identical.
    const REPO_REGISTER_ENTRY: &str = concat!(
        r#"{"name":"repo_register","#,
        r#""description":"Register a repository under its stable slug key: the store is created under the data root, the registry lists it, and the response is the new state.","#,
        r#""inputSchema":{"$schema":"https://json-schema.org/draft/2020-12/schema","#,
        r#""required":["slug","name","path"],"additionalProperties":false,"type":"object","#,
        r#""properties":{"slug":{"description":"The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the caller's chosen identity, never reused once registered.","type":"string"},"#,
        r#""name":{"description":"The repository's display name — a mutable attribute, unlike the slug.","type":"string"},"#,
        r#""path":{"description":"The filesystem path of the checkout (§7) — an attribute, not the key; `repo_set_path` moves it when a checkout moves.","type":"string"}}},"#,
        r#""outputSchema":{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"RepoRegistered","#,
        r#""description":"`repo_register`'s success payload: the new state (§6 — writes return the new state), the repo node in declaration order (D14).","#,
        r#""type":"object","properties":{"slug":{"description":"The repository's stable slug key.","type":"string"},"#,
        r#""name":{"description":"The repository's display name.","type":"string"},"#,
        r#""path":{"description":"The filesystem path of the checkout.","type":"string"},"#,
        r#""registered_at":{"description":"When the server registered the repository — RFC 3339 UTC with the nine-digit fraction, the one clock-stamped field.","type":"string"}},"#,
        r#""required":["slug","name","path","registered_at"]}}"#,
    );

    /// `tools/list` advertises every landed tool, sorted by name (§6.3:
    /// the router is a name-keyed map and `list_all` sorts), with
    /// `repo_register`'s own entry — description, params schema
    /// (unknown fields denied: `additionalProperties: false`), declared
    /// output schema, D14's declaration-order `required` lists — pinned
    /// byte-exactly within the response. The name list is the
    /// composition pin: it grows with every cluster file the router
    /// merges (t34 added the query cluster's five reads).
    #[test]
    fn tools_list_advertises_repo_register_with_its_description() {
        let root = TempDir::new().unwrap();
        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let (stdout, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{request}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        let response = responses[1];
        // The envelope: one result, one tools array, nothing else —
        // the shape t33's whole-response pin carried.
        assert!(
            response.starts_with(r#"{"jsonrpc":"2.0","id":2,"result":{"tools":[{"#),
            "{response}"
        );
        assert!(response.ends_with("}]}}"), "{response}");
        let value: serde_json::Value =
            serde_json::from_str(response).expect("tools/list answers JSON");
        let names: Vec<&str> = value["result"]["tools"]
            .as_array()
            .expect("the tool list")
            .iter()
            .map(|tool| tool["name"].as_str().expect("a tool name"))
            .collect();
        assert_eq!(
            names,
            [
                "decision_add",
                "decision_list",
                "decision_show",
                "decision_update",
                "doc_list",
                "doc_register",
                "doc_set_path",
                "doc_update",
                "epic_add",
                "epic_list",
                "epic_update",
                "graph_show",
                "import_md",
                "next_items",
                "next_task",
                "repo_list",
                "repo_register",
                "repo_set_path",
                "repo_show",
                "status_summary",
                "task_add",
                "task_link",
                "task_list",
                "task_show",
                "task_unlink",
                "task_update"
            ],
            "the router composes every landed cluster, sorted by name"
        );
        assert!(
            response.contains(REPO_REGISTER_ENTRY),
            "repo_register's advertisement is byte-exact: {response}"
        );
    }

    /// `repo_register` through the protocol creates a real store
    /// under the temp data root and answers with the new state —
    /// `structuredContent` carrying the repo in declaration order
    /// (D14), asserted byte-exactly around the clock-stamped
    /// `registered_at` — and the store is real on disk: the registry
    /// lists it and the store opens afterward (the server process has
    /// exited, so its locks are released).
    #[test]
    fn repo_register_creates_a_real_store_and_returns_the_new_state() {
        let root = TempDir::new().unwrap();
        let (stdout, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{REGISTER_CAMERA}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        let response = responses[1];
        let stamped = registered_at_of(response);
        // The structured payload in declaration order, and its mirror
        // — the same JSON, quotes escaped, as the text block.
        let structured = format!(
            r#"{{"slug":"camera","name":"Camera","path":"/Users/probe/camera","registered_at":"{stamped}"}}"#
        );
        let mirror = structured.replace('"', "\\\"");
        assert_eq!(
            response,
            format!(
                r#"{{"jsonrpc":"2.0","id":2,"result":{{"content":[{{"type":"text","text":"{mirror}"}}],"structuredContent":{structured},"isError":false}}}}"#
            )
        );
        // The store is real on disk: the registry lists it, the node
        // carries the same fields the wire answered with, and the
        // store opens.
        let data_root = DataRoot::new(root.path());
        let repos = data_root.repo_list().unwrap();
        assert_eq!(repos.len(), 1, "the registry lists exactly camera");
        assert_eq!(repos[0].slug.as_str(), "camera");
        assert_eq!(repos[0].name, "Camera");
        assert_eq!(repos[0].path, "/Users/probe/camera");
        assert_eq!(repos[0].registered_at.as_str(), stamped);
        // The store's acquire carries U27's bounded retry now, so this
        // single open is the detector: the in-process `repo_list()`
        // above opens and closes this store's lock, and the kernel's
        // flock teardown can outlive that close by a millisecond-scale
        // lifetime (U26's board entry, the ledger) — a store-retry
        // regression reintroduces the U26 flake right here, and this
        // open fails with `locked` naming it.
        let opened = data_root.open(&repos[0].slug);
        assert!(
            opened.is_ok(),
            "the store opens after the server exits: {opened:?}"
        );
    }

    /// a duplicate register — a second server process over the
    /// same root, so the first registration is committed — carries
    /// §6.1's `duplicate` via D15's carrier: `isError: true`,
    /// `structuredContent.code`, the text mirror. Exact bytes.
    #[test]
    fn a_duplicate_register_carries_the_duplicate_code() {
        let root = TempDir::new().unwrap();
        let (first, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{REGISTER_CAMERA}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        assert_eq!(responses(&first).len(), 2, "stderr: {stderr}");
        assert!(responses(&first)[1].contains(r#""isError":false"#));
        let (second, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{REGISTER_CAMERA}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&second);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        assert_eq!(
            responses[1],
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","#,
                r#""text":"{\"code\":\"duplicate\",\"message\":\"duplicate id: camera\"}"}],"#,
                r#""structuredContent":{"code":"duplicate","message":"duplicate id: camera"},"#,
                r#""isError":true}}"#
            )
        );
    }

    /// a contended register — the *test process* holds the
    /// registry's advisory lock while the server registers — carries
    /// `locked` via the carrier. Cross-process is the advisory lock's
    /// real job (§8): the lock is per file descriptor, so a second
    /// process is the contention the store's own lock can actually
    /// report.
    #[test]
    fn a_contended_registry_carries_the_locked_code() {
        let root = TempDir::new().unwrap();
        let _registry_lock = DataRoot::new(root.path()).lock_registry().unwrap();
        let (stdout, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{REGISTER_CAMERA}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        assert_eq!(
            responses[1],
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","#,
                r#""text":"{\"code\":\"locked\",\"message\":\"store is locked: the registry\"}"}],"#,
                r#""structuredContent":{"code":"locked","message":"store is locked: the registry"},"#,
                r#""isError":true}}"#
            )
        );
    }

    /// a malformed slug — §6.1's own `invalid` example class —
    /// carries `invalid` via the carrier (U9's mapping of the domain's
    /// `ValueError`). Exact bytes.
    #[test]
    fn a_malformed_slug_carries_the_invalid_code() {
        let root = TempDir::new().unwrap();
        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"repo_register","arguments":{"slug":"Camera!","name":"x","path":"/p"}}}"#;
        let (stdout, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{request}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        assert_eq!(
            responses[1],
            concat!(
                r#"{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","#,
                r#""text":"{\"code\":\"invalid\",\"message\":\"invalid repository slug: \\\"Camera!\\\"\"}"}],"#,
                r#""structuredContent":{"code":"invalid","message":"invalid repository slug: \"Camera!\""},"#,
                r#""isError":true}}"#
            )
        );
    }

    /// an unknown argument field is rmcp's own argument fault —
    /// tool-level `isError`, unstructured text, no code — because
    /// `deny_unknown_fields` turned the silent drop into a rejection
    /// before the tool body ran. The test pins that the rejection
    /// happens at all, and that nothing was created.
    #[test]
    fn an_unknown_argument_field_is_rejected_before_the_tool_body() {
        let root = TempDir::new().unwrap();
        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"repo_register","arguments":{"slug":"camera","name":"Camera","path":"/Users/probe/camera","epik":"camera:E1"}}}"#;
        let (stdout, stderr, status) = drive(
            root.path(),
            &format!("{INITIALIZE}\n{INITIALIZED}\n{request}\n"),
        );
        assert!(
            status.success(),
            "a completed handshake ends 0; stderr: {stderr}"
        );
        let responses = responses(&stdout);
        assert_eq!(responses.len(), 2, "stderr: {stderr}");
        let response = responses[1];
        assert!(response.contains(r#""isError":true"#), "{response}");
        assert!(
            !response.contains("structuredContent"),
            "an argument fault carries no code: {response}"
        );
        assert!(
            response.contains("failed to deserialize parameters"),
            "{response}"
        );
        assert!(response.contains("unknown field `epik`"), "{response}");
        // The rejection happened before the tool body: no store, no
        // registry entry.
        assert!(
            DataRoot::new(root.path()).repo_list().unwrap().is_empty(),
            "nothing was registered"
        );
    }

    /// The 2026-07-28 discover flow — measured as served for free
    /// over the same `serve` call, so it is pinned. With the two
    /// required `_meta` keys, the answer carries the same instructions
    /// and our serverInfo (in `_meta`); without them, `-32602` naming
    /// the missing keys — exact bytes.
    #[test]
    fn the_discover_flow_is_served_with_and_without_the_required_meta() {
        let root = TempDir::new().unwrap();
        let discover = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":"#,
            r#"{"io.modelcontextprotocol/protocolVersion":"2026-07-28","#,
            r#""io.modelcontextprotocol/clientCapabilities":{}}}}"#,
        );
        let (stdout, stderr, status) = drive(root.path(), &format!("{discover}\n"));
        assert!(
            status.success(),
            "a completed discover flow ends 0; stderr: {stderr}"
        );
        let discover_responses = responses(&stdout);
        assert_eq!(discover_responses.len(), 1, "stderr: {stderr}");
        let response = discover_responses[0];
        assert!(
            response.contains(&format!(r#""instructions":"{INSTRUCTIONS}""#)),
            "the discover answer carries the instructions: {response}"
        );
        assert!(
            response.contains(
                r#""_meta":{"io.modelcontextprotocol/serverInfo":{"name":"todo-mcp","version":"0.1.0"}}"#
            ),
            "the discover answer carries our serverInfo: {response}"
        );
        assert!(
            response.contains(r#""capabilities":{"tools":{}}"#),
            "{response}"
        );

        let bare_root = TempDir::new().unwrap();
        let bare = r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{}}"#;
        let (bare_stdout, stderr, status) = drive(bare_root.path(), &format!("{bare}\n"));
        assert_eq!(
            status.code(),
            Some(1),
            "a peer that closes before initialize ends 1; stderr: {stderr}"
        );
        assert_eq!(
            responses(&bare_stdout),
            vec![concat!(
                r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"request _meta is missing "#,
                r#"or has malformed required fields: io.modelcontextprotocol/protocolVersion, "#,
                r#"io.modelcontextprotocol/clientCapabilities"}}"#
            )],
            "stderr: {stderr}"
        );
    }

    /// handshake determinism — two fresh spawns answer
    /// byte-identical initialize responses (§8: two runs over
    /// unchanged state return identical bytes; the handshake carries
    /// no clock).
    #[test]
    fn two_fresh_spawns_answer_byte_identical_initialize_responses() {
        let root_a = TempDir::new().unwrap();
        let root_b = TempDir::new().unwrap();
        let (stdout_a, stderr_a, status_a) =
            drive(root_a.path(), &format!("{INITIALIZE}\n{INITIALIZED}\n"));
        let (stdout_b, stderr_b, status_b) =
            drive(root_b.path(), &format!("{INITIALIZE}\n{INITIALIZED}\n"));
        assert!(
            status_a.success() && status_b.success(),
            "completed handshakes end 0"
        );
        assert_eq!(stdout_a, stdout_b, "stderr: {stderr_a} / {stderr_b}");
    }
}
