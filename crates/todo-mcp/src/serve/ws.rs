//! The WebSocket push channel (D5): hand-rolled RFC 6455 — no ws
//! crate, the arc's no-new-dependencies discipline. The handshake (the
//! HTTP `Upgrade` on `/ws`: the key plus the magic GUID → SHA-1 → the
//! accept), the frame codec (client frames must be masked, server
//! frames are not; text / ping→pong / close→echo-and-drop; the 7-,
//! 16- and 64-bit length forms; inbound payloads capped), and the
//! connection loop: one reader plus one poller thread per connection
//! ([`super::poll`]), the push a text frame
//! `{"repo":"<slug>","changed":true}` the page's script reloads on.
//!
//! SHA-1 and base64 live here, hand-rolled for the same reason (the
//! t45 test's algorithm lifted to src, pinned by the published
//! vectors): the accept computation is a protocol obligation, not a
//! protection — nothing security-sensitive is hashed.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::serve::ConnState;
use crate::serve::http::{self, Request};
use crate::serve::poll;

/// The RFC 6455 magic GUID concatenated onto the key.
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// The text opcode — the push frames and the client's (discarded)
/// messages.
pub(super) const OPCODE_TEXT: u8 = 0x1;
/// The close opcode.
const OPCODE_CLOSE: u8 = 0x8;
/// The ping opcode.
const OPCODE_PING: u8 = 0x9;
/// The pong opcode.
const OPCODE_PONG: u8 = 0xA;

/// The inbound payload cap (judgment call (d)): the client sends
/// nothing but pings and close — anything larger is a protocol error.
const MAX_INBOUND: u64 = 4096;

/// SHA-1 (FIPS 180-1). The message schedule and the state are
/// positional by the algorithm's own shape; the sixteen-word window is
/// destructured and rebuilt shifted rather than indexed, so the
/// workspace's no-indexing discipline holds here too.
fn sha1(bytes: &[u8]) -> Vec<u8> {
    let [mut h0, mut h1, mut h2, mut h3, mut h4] = [
        0x6745_2301u32,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let bit_len = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());
    for block in message.chunks(64) {
        // The first sixteen words, big-endian.
        let mut window: [u32; 16] = [0; 16];
        for (slot, chunk) in window.iter_mut().zip(block.as_chunks::<4>().0) {
            *slot = u32::from_be_bytes(*chunk);
        }
        let mut schedule: Vec<u32> = Vec::with_capacity(80);
        schedule.extend_from_slice(&window);
        // The schedule: each new word folds the words 3, 8, 14 and 16
        // back — the window holds the last sixteen, rebuilt shifted.
        while schedule.len() < 80 {
            let [
                w0,
                w1,
                w2,
                w3,
                w4,
                w5,
                w6,
                w7,
                w8,
                w9,
                w10,
                w11,
                w12,
                w13,
                w14,
                w15,
            ] = window;
            let fresh = (w13 ^ w8 ^ w2 ^ w0).rotate_left(1);
            schedule.push(fresh);
            window = [
                w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, fresh,
            ];
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (round, &word) in schedule.iter().enumerate() {
            let (f, k) = match round {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    [
        h0.to_be_bytes(),
        h1.to_be_bytes(),
        h2.to_be_bytes(),
        h3.to_be_bytes(),
        h4.to_be_bytes(),
    ]
    .concat()
}

/// Standard base64 with padding — the accept's encoding. The payload
/// is left-aligned in each 24-bit group: a short chunk's missing
/// bytes are zero bits at the low end, and the real byte count picks
/// the padding.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let packed = chunk
            .iter()
            .fold(0u32, |acc, &byte| (acc << 8) | u32::from(byte));
        let triple = packed << (8 * (3 - chunk.len()) as u32);
        out.push(base64_char(triple >> 18));
        out.push(base64_char(triple >> 12));
        if chunk.len() > 1 {
            out.push(base64_char(triple >> 6));
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(base64_char(triple));
        } else {
            out.push('=');
        }
    }
    out
}

/// One base64 character for a six-bit value — arithmetic, not a table
/// lookup, so no index can be out of bounds.
fn base64_char(sextet: u32) -> char {
    let value = sextet & 0x3f;
    match value {
        0..=25 => char::from(b'A' + value as u8),
        26..=51 => char::from(b'a' + (value - 26) as u8),
        52..=61 => char::from(b'0' + (value - 52) as u8),
        62 => '+',
        _ => '/',
    }
}

/// The RFC 6455 accept for a handshake key: base64(SHA-1(key + GUID)).
fn accept_for(key: &str) -> String {
    let mut input = String::with_capacity(key.len() + GUID.len());
    input.push_str(key);
    input.push_str(GUID);
    base64(&sha1(input.as_bytes()))
}

/// The 101 response for a valid upgrade request (D5): the accept
/// computed from the key. A request that is not a complete upgrade —
/// no `Upgrade: websocket`, no `Connection` naming `upgrade`, no key,
/// or a version other than 13 — is a named error; the caller answers
/// 400 and closes.
pub(super) fn handshake_response(request: &Request) -> Result<String, String> {
    let upgrade = request
        .header("upgrade")
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"));
    let connection = request
        .header("connection")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    let key = request.header("sec-websocket-key").unwrap_or_default();
    let version = request.header("sec-websocket-version").unwrap_or_default();
    let connection_upgrade = connection.split(',').any(|token| token.trim() == "upgrade");
    if !upgrade || !connection_upgrade || key.is_empty() || version != "13" {
        return Err("the request is not a WebSocket upgrade".to_owned());
    }
    Ok(format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        accept_for(key)
    ))
}

/// One parsed client frame.
#[derive(Debug)]
struct ClientFrame {
    /// The opcode.
    opcode: u8,
    /// The unmasked payload.
    payload: Vec<u8>,
}

/// Reads one client frame (D5): the mask bit must be set (an unmasked
/// client frame is a protocol error), the payload capped, the length
/// in all three forms. The FIN bit is not inspected — the client sends
/// nothing but pings and close, neither of which fragments.
fn read_client_frame(reader: &mut impl BufRead) -> Result<ClientFrame, String> {
    let mut head = [0u8; 2];
    reader
        .read_exact(&mut head)
        .map_err(|error| format!("read the frame head: {error}"))?;
    let [first, second] = head;
    let opcode = first & 0x0f;
    let masked = second & 0x80 != 0;
    let length = match u64::from(second & 0x7f) {
        126 => {
            let mut bytes = [0u8; 2];
            reader
                .read_exact(&mut bytes)
                .map_err(|error| format!("read the frame length: {error}"))?;
            u64::from(u16::from_be_bytes(bytes))
        }
        127 => {
            let mut bytes = [0u8; 8];
            reader
                .read_exact(&mut bytes)
                .map_err(|error| format!("read the frame length: {error}"))?;
            u64::from_be_bytes(bytes)
        }
        small => small,
    };
    if !masked {
        return Err("the client frame is not masked".to_owned());
    }
    if length > MAX_INBOUND {
        return Err("the client frame exceeds the inbound cap".to_owned());
    }
    let mut mask = [0u8; 4];
    reader
        .read_exact(&mut mask)
        .map_err(|error| format!("read the frame mask: {error}"))?;
    // The cap bounds the length, so the cast cannot truncate.
    let mut payload = vec![0u8; length as usize];
    reader
        .read_exact(&mut payload)
        .map_err(|error| format!("read the frame payload: {error}"))?;
    for (byte, key) in payload.iter_mut().zip(mask.iter().cycle()) {
        *byte ^= key;
    }
    Ok(ClientFrame { opcode, payload })
}

/// Encodes one server frame (D5): FIN set, unmasked, the length in
/// the same three forms.
pub(super) fn encode_server_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(0x80 | opcode);
    let length = payload.len();
    if length < 126 {
        frame.push(length as u8);
    } else if length <= usize::from(u16::MAX) {
        frame.push(126);
        frame.extend_from_slice(&(length as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(length as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    frame
}

/// The WS connection (D5): the repo resolves and the handshake
/// completes before anything else — a failure on either answers a
/// plain 500/400 and the connection ends. Then the write half is
/// shared with the poller behind one mutex (the reader's pongs and
/// the poller's pushes must not interleave mid-frame), the poller
/// spawns, and the reader loop serves the client's frames: ping →
/// pong, close → echo-and-drop, text → discarded (the client has
/// nothing to say). The connection ends on the close frame, a
/// protocol error or an IO error; the alive flag takes the poller
/// down with it.
pub(super) fn handle_ws_connection(
    mut reader: BufReader<TcpStream>,
    state: &ConnState,
    request: &Request,
) {
    let slug = match super::repo_for_request(state, request) {
        Ok(slug) => slug,
        Err(message) => {
            eprintln!("todo-mcp serve: the WebSocket repo did not resolve: {message}");
            http::write_plain_response(
                reader.get_mut(),
                500,
                "text/plain; charset=utf-8",
                super::RENDER_FAILED_BODY.as_bytes(),
            );
            return;
        }
    };
    let response = match handshake_response(request) {
        Ok(response) => response,
        Err(message) => {
            eprintln!("todo-mcp serve: {message}");
            http::write_plain_response(
                reader.get_mut(),
                400,
                "text/plain; charset=utf-8",
                b"not a websocket upgrade\n",
            );
            return;
        }
    };
    let Ok(write_half) = reader.get_ref().try_clone() else {
        return;
    };
    let write_half = Arc::new(Mutex::new(write_half));
    {
        let mut stream = write_half.lock().unwrap_or_else(PoisonError::into_inner);
        if stream.write_all(response.as_bytes()).is_err() {
            return;
        }
    }
    let alive = Arc::new(AtomicBool::new(true));
    let paths = [state.root.log_path(&slug), state.root.snapshot_path(&slug)];
    let poller = match poll::spawn_poller(
        paths,
        state.poll_interval,
        alive.clone(),
        write_half.clone(),
        &slug,
    ) {
        Ok(poller) => poller,
        Err(message) => {
            eprintln!("todo-mcp serve: {message}");
            return;
        }
    };
    while let Ok(frame) = read_client_frame(&mut reader) {
        match frame.opcode {
            OPCODE_PING => {
                let pong = encode_server_frame(OPCODE_PONG, &frame.payload);
                let mut stream = write_half.lock().unwrap_or_else(PoisonError::into_inner);
                let _ = stream.write_all(&pong);
            }
            OPCODE_CLOSE => {
                let echo = encode_server_frame(OPCODE_CLOSE, &frame.payload);
                let mut stream = write_half.lock().unwrap_or_else(PoisonError::into_inner);
                let _ = stream.write_all(&echo);
                break;
            }
            OPCODE_TEXT => {}
            _ => break,
        }
    }
    alive.store(false, Ordering::SeqCst);
    let _ = poller.join();
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    /// SHA-1's published vectors.
    #[test]
    fn sha1_matches_the_published_vectors() {
        fn hex(bytes: &[u8]) -> String {
            bytes.iter().map(|byte| format!("{byte:02x}")).collect()
        }
        assert_eq!(
            hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    /// base64's padding cases — the RFC example key decodes to
    /// "the sample nonce", so encoding it must reproduce the key.
    #[test]
    fn base64_pads_the_tail() {
        assert_eq!(base64(b"the sample nonce"), "dGhlIHNhbXBsZSBub25jZQ==");
        assert_eq!(base64(b"abc"), "YWJj");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(base64(b"a"), "YQ==");
    }

    /// The RFC 6455 example: the key computes the published accept.
    #[test]
    fn the_rfc6455_example_key_computes_the_published_accept() {
        assert_eq!(
            accept_for("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    /// A masked text frame parses with its payload unmasked.
    #[test]
    fn a_masked_text_frame_parses() {
        let mask = [0x01u8, 0x02, 0x03, 0x04];
        let mut bytes = vec![0x81, 0x85];
        bytes.extend_from_slice(&mask);
        for (index, byte) in b"hello".iter().enumerate() {
            bytes.push(byte ^ mask[index % 4]);
        }
        let frame = read_client_frame(&mut Cursor::new(bytes)).expect("the masked frame parses");
        assert_eq!(frame.opcode, OPCODE_TEXT);
        assert_eq!(frame.payload, b"hello");
    }

    /// The 16-bit and 64-bit length forms parse; the 64-bit form is
    /// the sender's choice, so it carries a small length here.
    #[test]
    fn the_extended_length_forms_parse() {
        let mask = [0x0Au8, 0x0B, 0x0C, 0x0D];
        let mut sixteen = vec![0x81, 0xFE, 0x00, 0xC8];
        sixteen.extend_from_slice(&mask);
        for index in 0..200 {
            sixteen.push(0x61 ^ mask[index % 4]);
        }
        let frame = read_client_frame(&mut Cursor::new(sixteen)).expect("the 16-bit form parses");
        assert_eq!(frame.payload.len(), 200);
        assert!(frame.payload.iter().all(|&byte| byte == 0x61));

        let mut sixty_four = vec![0x81, 0xFF];
        sixty_four.extend_from_slice(&100u64.to_be_bytes());
        sixty_four.extend_from_slice(&mask);
        for index in 0..100 {
            sixty_four.push(0x62 ^ mask[index % 4]);
        }
        let frame =
            read_client_frame(&mut Cursor::new(sixty_four)).expect("the 64-bit form parses");
        assert_eq!(frame.payload.len(), 100);
        assert!(frame.payload.iter().all(|&byte| byte == 0x62));
    }

    /// An unmasked client frame and an over-cap frame are protocol
    /// errors.
    #[test]
    fn the_protocol_errors_refuse() {
        let unmasked = vec![0x81, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let error =
            read_client_frame(&mut Cursor::new(unmasked)).expect_err("the mask is required");
        assert!(error.contains("not masked"), "{error}");
        // The 16-bit form claiming 5000 bytes: over the cap before any
        // payload is read.
        let oversized = vec![0x81, 0xFE, 0x13, 0x88];
        let error =
            read_client_frame(&mut Cursor::new(oversized)).expect_err("the cap is enforced");
        assert!(error.contains("cap"), "{error}");
    }

    /// The server frame encodes the three length forms — unmasked, so
    /// the markers are 0x7E and 0x7F (the client's masked forms set
    /// the high bit).
    #[test]
    fn the_server_frame_encodes_the_length_forms() {
        assert_eq!(
            encode_server_frame(OPCODE_TEXT, b"hi"),
            vec![0x81, 0x02, b'h', b'i']
        );
        let frame = encode_server_frame(OPCODE_TEXT, &[b'x'; 200]);
        assert_eq!(&frame[..4], &[0x81, 0x7E, 0x00, 0xC8]);
        let frame = encode_server_frame(OPCODE_TEXT, &[b'x'; 70_000]);
        assert_eq!(&frame[..10], &[0x81, 0x7F, 0, 0, 0, 0, 0, 0x01, 0x11, 0x70]);
    }

    /// The handshake validation: the complete upgrade accepts (header
    /// case-insensitive), each missing piece refuses.
    #[test]
    fn the_handshake_validates_the_upgrade() {
        let full = "GET /ws HTTP/1.1\r\nHost: x\r\nUPGRADE: WebSocket\r\nConnection: keep-alive, Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n";
        let request = http::read_request(&mut Cursor::new(full.as_bytes().to_vec()))
            .expect("the head parses");
        let response = handshake_response(&request).expect("the upgrade accepts");
        assert!(response.starts_with("HTTP/1.1 101 Switching Protocols\r\n"));
        assert!(response.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"));

        for broken in [
            full.replace("UPGRADE: WebSocket\r\n", ""),
            full.replace(
                "Connection: keep-alive, Upgrade\r\n",
                "Connection: keep-alive\r\n",
            ),
            full.replace("Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n", ""),
            full.replace("Sec-WebSocket-Version: 13", "Sec-WebSocket-Version: 8"),
        ] {
            let request = http::read_request(&mut Cursor::new(broken.clone().into_bytes()))
                .expect("the head parses");
            assert!(handshake_response(&request).is_err(), "refused: {broken:?}");
        }
    }
}
