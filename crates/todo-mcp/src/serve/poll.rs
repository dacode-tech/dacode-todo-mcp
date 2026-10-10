//! The stat-polling change detection (D5): one poller thread per WS
//! connection watches the repo's event log and its snapshot — `mtime`
//! plus length, every poll interval — and pushes one text frame when
//! the fingerprint moves. No watch crate (notify): two `stat()`s per
//! tick per connected tab is nothing, and the event-driven
//! alternative is recorded in the spec as the user's flip.

use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use todo_domain::RepoSlug;

use crate::serve::ws;

/// One watched file's fingerprint: `mtime` plus length, or `None` when
/// the file is absent — a file appearing or vanishing is a change.
type Fingerprint = Option<(SystemTime, u64)>;

/// Stats one file. A platform that cannot report `mtime` reads as the
/// epoch — a documented default, not a failure: the length still
/// detects every append.
fn stat_of(path: &Path) -> Fingerprint {
    fs::metadata(path).ok().map(|meta| {
        (
            meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            meta.len(),
        )
    })
}

/// The watched pair's fingerprint.
fn fingerprint(paths: &[PathBuf; 2]) -> [Fingerprint; 2] {
    let [log, snapshot] = paths;
    [stat_of(log), stat_of(snapshot)]
}

/// Spawns the poller thread: stat, sleep, stat again — on a change,
/// one text frame `{"repo":"<slug>","changed":true}` through the
/// shared write half; on a write failure (the peer gone) the thread
/// exits. The alive flag ends it within one interval of the
/// connection's end. The slug's charset needs no JSON escaping.
pub(super) fn spawn_poller(
    paths: [PathBuf; 2],
    interval: Duration,
    alive: Arc<AtomicBool>,
    write: Arc<Mutex<TcpStream>>,
    slug: &RepoSlug,
) -> Result<JoinHandle<()>, String> {
    let payload = format!("{{\"repo\":\"{}\",\"changed\":true}}", slug.as_str()).into_bytes();
    thread::Builder::new()
        .name("serve-ws-poll".to_owned())
        .spawn(move || {
            let mut baseline = fingerprint(&paths);
            loop {
                thread::sleep(interval);
                if !alive.load(Ordering::SeqCst) {
                    return;
                }
                let current = fingerprint(&paths);
                if current == baseline {
                    continue;
                }
                baseline = current;
                let frame = ws::encode_server_frame(ws::OPCODE_TEXT, &payload);
                let mut stream = write.lock().unwrap_or_else(PoisonError::into_inner);
                if stream.write_all(&frame).is_err() {
                    return;
                }
            }
        })
        .map_err(|error| format!("spawn the poller thread: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Read;
    use std::str::FromStr;

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

    /// The poller over a real TCP pair: a file change pushes exactly
    /// the spec's frame; no change pushes nothing within several
    /// intervals; the alive flag ends the thread.
    #[test]
    fn the_poller_pushes_on_change_and_stays_quiet_without_it() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let watched = dir.path().join("watched.log");
        std::fs::write(&watched, b"baseline\n").expect("the watched file lands");
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("the pair binds");
        let port = listener.local_addr().expect("the address").port();
        let mut client =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the client connects");
        let (server, _) = listener.accept().expect("the server side accepts");
        let alive = Arc::new(AtomicBool::new(true));
        let write = Arc::new(Mutex::new(server));
        let poller = spawn_poller(
            [watched.clone(), dir.path().join("absent.snapshot")],
            Duration::from_millis(20),
            alive.clone(),
            write,
            &RepoSlug::from_str("fixture").expect("a valid slug"),
        )
        .expect("the poller spawns");

        // No change: quiet for several intervals.
        client
            .set_read_timeout(Some(Duration::from_millis(120)))
            .expect("the read timeout");
        let mut probe = [0u8; 1];
        assert!(
            client.read(&mut probe).is_err(),
            "no frame without a change"
        );

        // A change: the push frame arrives.
        std::fs::write(&watched, b"baseline\nchanged\n").expect("the file changes");
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("the read timeout");
        let (opcode, payload) = read_frame(&mut client);
        assert_eq!(opcode, 0x1);
        assert_eq!(payload, b"{\"repo\":\"fixture\",\"changed\":true}");

        alive.store(false, Ordering::SeqCst);
        poller.join().expect("the poller exits");
    }
}
