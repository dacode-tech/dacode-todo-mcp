//! The golden-file helper
//! (`.agents/knowledge/workflows/golden-files.md`): the one
//! implementation of the compare-and-regenerate contract, shared by every
//! golden test of this crate. A second byte-compare implementation
//! anywhere is a review finding.
//!
//! **This is a copy.** The original lives at
//! `crates/todo-store/tests/common/mod.rs`; the workflow prescribes one
//! shared helper *per crate*, and a cross-crate test-helper dependency
//! would be worse than the copy. It is kept verbatim — the only
//! difference is the crate name in the missing-golden regeneration
//! command, which has to name *this* crate to be true here. A fix to one
//! must propagate to the other.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// How many differing line pairs the mismatch renderer shows at most —
/// bounded so a whole-file drift cannot flood the failure output.
const MAX_DIFF_LINES: usize = 3;
/// How many characters of each rendered diff line are shown at most.
const MAX_LINE_CHARS: usize = 200;

/// Resolves a golden's path from its subject `name` (the part before
/// the `.golden` suffix), anchored at the crate manifest directory —
/// never the process cwd, so the tests pass from any invocation root.
pub(crate) fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{name}.golden"))
}

/// Asserts `actual` is byte-identical to the golden `name`, or — under
/// `UPDATE_GOLDEN=1` — regenerates the golden and passes.
///
/// Regeneration writes through a temporary file in the golden directory
/// plus an atomic rename, so an interrupted update cannot leave a torn
/// golden. Deliberately no `fsync` (judgment call, t9): §4.3's
/// durability discipline binds the store's own writes; a golden lost to
/// a power failure is a regenerated test fixture, never a state of
/// record, and rename-atomicity is the property this helper owes.
///
/// Failure messages per the workflow contract: a missing golden names
/// the regeneration command; a mismatch names the path, both byte
/// lengths, the first differing byte offset, and a bounded line-level
/// diff of the first differing region.
pub(crate) fn assert_golden(name: &str, actual: &[u8]) {
    let path = golden_path(name);
    if update_requested() {
        write_golden(&path, actual);
        return;
    }
    let expected = match fs::read(&path) {
        Ok(expected) => expected,
        Err(err) if err.kind() == ErrorKind::NotFound => panic!(
            "golden file {} is missing; regenerate it with:\n  \
             UPDATE_GOLDEN=1 cargo test -p todo-cli -- golden",
            path.display()
        ),
        Err(err) => panic!("golden file {} could not be read: {err}", path.display()),
    };
    if expected != actual {
        panic!("{}", mismatch_report(&path, &expected, actual));
    }
}

/// Whether this run regenerates goldens instead of comparing them.
fn update_requested() -> bool {
    std::env::var("UPDATE_GOLDEN").is_ok_and(|value| value == "1")
}

/// Writes `actual` to `path` via temp+rename in the golden directory,
/// then reports what was written.
fn write_golden(path: &Path, actual: &[u8]) {
    let directory = path
        .parent()
        .expect("a golden path always has the golden directory as its parent");
    fs::create_dir_all(directory).expect("the golden directory is creatable");
    let file_name = path
        .file_name()
        .expect("a golden path always ends in its file name")
        .to_string_lossy();
    // The pid keeps parallel test binaries from sharing a temp name.
    let temporary = directory.join(format!(".{file_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, actual).expect("the golden temp file is writable");
    fs::rename(&temporary, path).expect("rename within one directory is atomic");
    println!(
        "UPDATE_GOLDEN: wrote {} bytes to {}",
        actual.len(),
        path.display()
    );
}

/// The mismatch failure message: path, both byte lengths, the first
/// differing byte offset, and the bounded line-level diff.
fn mismatch_report(path: &Path, expected: &[u8], actual: &[u8]) -> String {
    let offset = first_difference(expected, actual);
    let mut report = format!(
        "golden mismatch: {}\n  expected {} bytes, got {} bytes\n  \
         first differing byte at offset {offset}",
        path.display(),
        expected.len(),
        actual.len()
    );
    report.push_str(&line_diff(expected, actual));
    report
}

/// The offset of the first differing byte; when one side is a prefix of
/// the other, the length of the shorter.
fn first_difference(expected: &[u8], actual: &[u8]) -> usize {
    expected
        .iter()
        .zip(actual)
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| expected.len().min(actual.len()))
}

/// A line-level diff of the two byte slices: walks line pairs in index
/// order (deterministic — nothing unordered is iterated), renders at
/// most [`MAX_DIFF_LINES`] differing pairs, truncates each rendered
/// line to [`MAX_LINE_CHARS`], and counts the remainder.
fn line_diff(expected: &[u8], actual: &[u8]) -> String {
    let expected_text = String::from_utf8_lossy(expected);
    let actual_text = String::from_utf8_lossy(actual);
    let expected_lines: Vec<&str> = expected_text.lines().collect();
    let actual_lines: Vec<&str> = actual_text.lines().collect();
    let mut diff = String::new();
    let mut shown = 0usize;
    let mut differing = 0usize;
    for index in 0..expected_lines.len().max(actual_lines.len()) {
        let left = expected_lines.get(index).copied();
        let right = actual_lines.get(index).copied();
        if left == right {
            continue;
        }
        differing += 1;
        if shown < MAX_DIFF_LINES {
            shown += 1;
            diff.push_str(&format!(
                "\n  line {}:\n    expected: {}\n    actual:   {}",
                index + 1,
                bounded(left.unwrap_or("<absent>")),
                bounded(right.unwrap_or("<absent>"))
            ));
        }
    }
    if differing > shown {
        diff.push_str(&format!(
            "\n  … and {} more differing line(s)",
            differing - shown
        ));
    }
    diff
}

/// A diff line, cut to [`MAX_LINE_CHARS`] characters with an ellipsis
/// marking the cut.
fn bounded(line: &str) -> String {
    if line.chars().count() <= MAX_LINE_CHARS {
        return line.to_owned();
    }
    let mut cut: String = line.chars().take(MAX_LINE_CHARS).collect();
    cut.push('…');
    cut
}
