//! The stat-polling change detection (D5): one poller thread per WS
//! connection watches the ROOT — the config, the viewed repo's event
//! log and snapshot, and every registered repo's, the set re-derived
//! from the registry each tick (U80: the rewards section is
//! root-scoped — a sibling repo's grant or a config flip must move
//! the watched page, not only the viewed repo's own writes) —
//! `mtime` plus length, every poll interval — and pushes one text
//! frame when the fingerprint moves. No watch crate (notify): a
//! handful of `stat()`s and one small registry read per tick per
//! connected tab is nothing, and the event-driven alternative is
//! recorded in the spec as the user's flip.

use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use todo_domain::RepoSlug;
use todo_store::{DataRoot, Registry, StoreError};

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

/// The root's fingerprint (U80): the config's, the viewed repo's log
/// and snapshot, then every other registered repo's — the registry
/// read each tick derives the set, so a repo registered
/// mid-connection joins the watch within one interval, and a
/// registration or removal itself moves the list (a change). The
/// viewed repo's pair is watched by name, not membership: the
/// connection resolved the slug, and the page renders it, whatever
/// the registry later says. A registry that will not read is the
/// caller's skip-the-tick answer, never a push: a torn read must not
/// navigate a parked page to a failed render.
fn fingerprint(root: &DataRoot, viewed: &RepoSlug) -> Result<Vec<Fingerprint>, StoreError> {
    let registry = Registry::load(root)?;
    let mut paths = Vec::with_capacity(3 + 2 * registry.slugs().len());
    paths.push(root.config_path());
    paths.push(root.log_path(viewed));
    paths.push(root.snapshot_path(viewed));
    for slug in registry.slugs() {
        if slug == viewed {
            continue;
        }
        paths.push(root.log_path(slug));
        paths.push(root.snapshot_path(slug));
    }
    Ok(paths.iter().map(|path| stat_of(path)).collect())
}

/// Spawns the poller thread: stat, sleep, stat again — on a change,
/// one text frame `{"repo":"<slug>","changed":true}` through the
/// shared write half; on a write failure (the peer gone) the thread
/// exits. The alive flag ends it within one interval of the
/// connection's end. The slug's charset needs no JSON escaping.
///
/// The baseline is statted at thread start, as the pair-watch always
/// did — a change inside the first interval still pushes. A registry
/// that will not read at start (or on any later tick) skips its
/// tick: the first readable fingerprint becomes the baseline, and a
/// torn read neither pushes nor moves it.
pub(super) fn spawn_poller(
    root: DataRoot,
    interval: Duration,
    alive: Arc<AtomicBool>,
    write: Arc<Mutex<TcpStream>>,
    slug: &RepoSlug,
) -> Result<JoinHandle<()>, String> {
    let payload = format!("{{\"repo\":\"{}\",\"changed\":true}}", slug.as_str()).into_bytes();
    let slug = slug.clone();
    thread::Builder::new()
        .name("serve-ws-poll".to_owned())
        .spawn(move || {
            let mut baseline: Option<Vec<Fingerprint>> = fingerprint(&root, &slug).ok();
            loop {
                thread::sleep(interval);
                if !alive.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(current) = fingerprint(&root, &slug) else {
                    continue;
                };
                let changed = baseline.as_ref().is_some_and(|base| base != &current);
                baseline = Some(current);
                if !changed {
                    continue;
                }
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
        let root = DataRoot::new(dir.path());
        let slug = RepoSlug::from_str("fixture").expect("a valid slug");
        let watched = root.log_path(&slug);
        std::fs::create_dir_all(root.store_dir(&slug)).expect("the store dir lands");
        std::fs::write(&watched, b"baseline\n").expect("the watched log lands");
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("the pair binds");
        let port = listener.local_addr().expect("the address").port();
        let mut client =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the client connects");
        let (server, _) = listener.accept().expect("the server side accepts");
        let alive = Arc::new(AtomicBool::new(true));
        let write = Arc::new(Mutex::new(server));
        let poller = spawn_poller(
            root.clone(),
            Duration::from_millis(20),
            alive.clone(),
            write,
            &slug,
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
        std::fs::write(&watched, b"baseline\nchanged\n").expect("the log changes");
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("the read timeout");
        let (opcode, payload) = read_frame(&mut client);
        assert_eq!(opcode, 0x1);
        assert_eq!(payload, b"{\"repo\":\"fixture\",\"changed\":true}");

        alive.store(false, Ordering::SeqCst);
        poller.join().expect("the poller exits");
    }

    /// A config flip pushes (U80's other scenario): the rewards
    /// section's presence and contents are the config's, so the
    /// config file joining the root — absent to present — is a change
    /// the watched page must hear. The poller stats the file, never
    /// parses it, so any bytes serve.
    #[test]
    fn a_config_flip_pushes() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let root = DataRoot::new(dir.path());
        let slug = RepoSlug::from_str("fixture").expect("a valid slug");
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("the pair binds");
        let port = listener.local_addr().expect("the address").port();
        let mut client =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the client connects");
        let (server, _) = listener.accept().expect("the server side accepts");
        let alive = Arc::new(AtomicBool::new(true));
        let write = Arc::new(Mutex::new(server));
        let poller = spawn_poller(
            root.clone(),
            Duration::from_millis(20),
            alive.clone(),
            write,
            &slug,
        )
        .expect("the poller spawns");

        // No store, no config: quiet while the baseline lands.
        client
            .set_read_timeout(Some(Duration::from_millis(120)))
            .expect("the read timeout");
        let mut probe = [0u8; 1];
        assert!(
            client.read(&mut probe).is_err(),
            "no frame without a change"
        );

        // The config appears: the push frame arrives.
        std::fs::write(
            root.config_path(),
            b"any bytes: the poller stats, never parses",
        )
        .expect("the config lands");
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
