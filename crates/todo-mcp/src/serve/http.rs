//! The hand-rolled HTTP/1.1 layer (D4): no HTTP crate — the arc's
//! no-new-dependencies discipline. One request shape (a GET's head),
//! one response shape (status, Content-Type, Content-Length,
//! `Connection: close` — the connection closes after every plain
//! response, no keep-alive; the WS connection persists by protocol),
//! and D2's route classification. Header names are case-insensitive.
//! No date is emitted: the lib reads no clock (§8).

use std::io::{BufRead, Write};
use std::net::TcpStream;

/// How many bytes of request head one connection accepts — the bound
/// on a hostile or broken client's stream.
const MAX_HEAD: usize = 8 * 1024;

/// One parsed request: the request line and the headers, with the
/// target split at `?` (D2's query rule).
pub(super) struct Request {
    /// The method, as sent.
    method: String,
    /// The path, query string removed.
    path: String,
    /// The raw query string — everything after the first `?`.
    query: String,
    /// The headers: names lower-cased, values trimmed, in arrival
    /// order (the first match wins).
    headers: Vec<(String, String)>,
}

impl Request {
    /// The first value of `name` (case-insensitive on both sides —
    /// the stored name is lower-cased, the asked name may be any
    /// case), trimmed.
    pub(super) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Reads one request head: lines through the first blank line, then
/// the request line and the headers parsed out of them. A connection
/// that closes mid-head, a head over the cap, or a malformed line is
/// a named error — the caller answers 400 and closes.
pub(super) fn read_request(reader: &mut impl BufRead) -> Result<Request, String> {
    let mut head = String::new();
    loop {
        let mut line = Vec::new();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("read the request: {error}"))?;
        if read == 0 {
            return Err("the connection closed before the request head completed".to_owned());
        }
        if !line.ends_with(b"\n") {
            return Err("the connection closed mid-line".to_owned());
        }
        if head.len() + line.len() > MAX_HEAD {
            return Err("the request head exceeds the size cap".to_owned());
        }
        head.push_str(&String::from_utf8_lossy(&line));
        if line == b"\r\n" || line == b"\n" {
            break;
        }
    }
    parse_head(&head)
}

/// Parses the head text: `METHOD SP TARGET SP VERSION`, then `Name:
/// value` lines. The version is not validated beyond its presence —
/// the routes do not branch on it.
fn parse_head(head: &str) -> Result<Request, String> {
    let mut lines = head.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "the request head is empty".to_owned())?;
    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default().to_owned();
    let version = parts.next();
    if method.is_empty() || target.is_empty() || version.is_none() || parts.next().is_some() {
        return Err(format!("the request line is malformed: {request_line:?}"));
    }
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target, String::new()),
    };
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(format!("the header line is malformed: {line:?}"));
        };
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
    Ok(Request {
        method,
        path,
        query,
        headers,
    })
}

/// D2's routes: `/health` (no render), `/` and `/json` (the render
/// pair), `/ws` (the upgrade) — anything else 404s BEFORE any render,
/// so a browser's favicon probe never pays one; a non-GET method is
/// refused before the render for the same reason.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Route {
    /// `/health`: the marker, no render.
    Health,
    /// `/`: the page.
    Page,
    /// `/json`: the JSON twin.
    Json,
    /// `/ws`: the WebSocket upgrade.
    Ws,
    /// An unknown path.
    NotFound,
    /// A method other than GET.
    MethodNotAllowed,
}

/// Classifies the request's route.
pub(super) fn route(request: &Request) -> Route {
    if request.method != "GET" {
        return Route::MethodNotAllowed;
    }
    match request.path.as_str() {
        "/health" => Route::Health,
        "/" => Route::Page,
        "/json" => Route::Json,
        "/ws" => Route::Ws,
        _ => Route::NotFound,
    }
}

/// The `repo` query parameter's raw value (D2): the first `repo=` pair
/// wins, unknown parameters are ignored — the browser surface, not
/// the wire's deny_unknown_fields discipline.
pub(super) fn repo_param(request: &Request) -> Option<&str> {
    request
        .query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .find_map(|pair| pair.strip_prefix("repo="))
}

/// Writes one plain response (D4): the status line, Content-Type,
/// Content-Length and `Connection: close`, then the body — the
/// connection closes when the caller drops the stream. Write failures
/// are ignored: a peer gone mid-response has nothing left to recover.
pub(super) fn write_plain_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) {
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reason_phrase(status),
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

/// The reason phrase for the statuses this server emits.
fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    /// Parses one request from raw head bytes.
    fn parse(head: &[u8]) -> Request {
        read_request(&mut Cursor::new(head.to_vec())).expect("the head parses")
    }

    /// The request parse: the line, the query split, the headers
    /// case-insensitive.
    #[test]
    fn the_request_parses() {
        let request = parse(
            b"GET /json?repo=alpha&x=1 HTTP/1.1\r\nHost: 127.0.0.1\r\nSec-WebSocket-Key: abc\r\n\r\n",
        );
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/json");
        assert_eq!(request.query, "repo=alpha&x=1");
        assert_eq!(request.header("host"), Some("127.0.0.1"));
        assert_eq!(request.header("SEC-WEBSOCKET-KEY"), Some("abc"));
        assert_eq!(request.header("absent"), None);
    }

    /// A target without a query parses to an empty query; a malformed
    /// request line or header is a named error.
    #[test]
    fn the_malformed_heads_are_named_errors() {
        let request = parse(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        assert_eq!(request.query, "");
        assert!(read_request(&mut Cursor::new(Vec::new())).is_err());
        assert!(read_request(&mut Cursor::new(b"GET /\r\n\r\n".to_vec())).is_err());
        assert!(
            read_request(&mut Cursor::new(
                b"GET / HTTP/1.1\r\nno-colon\r\n\r\n".to_vec()
            ))
            .is_err()
        );
        assert!(read_request(&mut Cursor::new(b"GET / HTTP/1.1\r\nHost: x\r\n".to_vec())).is_err());
    }

    /// The repo param: the first wins, unknown params ignored, a bare
    /// `repo` is not a parameter.
    #[test]
    fn the_repo_param_extracts() {
        for (query, expected) in [
            ("repo=alpha&repo=beta", Some("alpha")),
            ("x=1&repo=alpha", Some("alpha")),
            ("repo=alpha&x=1", Some("alpha")),
            ("x=1", None),
            ("repo", None),
            ("", None),
        ] {
            let request = parse(format!("GET /?{query} HTTP/1.1\r\n\r\n").as_bytes());
            assert_eq!(repo_param(&request), expected, "query: {query:?}");
        }
    }

    /// The routes: the four known paths, the 404, the non-GET refusal.
    #[test]
    fn the_routes_classify() {
        for (path, expected) in [
            ("/", Route::Page),
            ("/json", Route::Json),
            ("/health", Route::Health),
            ("/ws", Route::Ws),
            ("/nope", Route::NotFound),
            ("/json/sub", Route::NotFound),
        ] {
            let request = parse(format!("GET {path} HTTP/1.1\r\n\r\n").as_bytes());
            assert_eq!(route(&request), expected, "path: {path}");
        }
        let request = parse(b"POST / HTTP/1.1\r\n\r\n");
        assert_eq!(route(&request), Route::MethodNotAllowed);
    }
}
