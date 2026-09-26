//! Identity (spec §3, D3 extended to the D10 kinds): repository slugs
//! and typed node ids.
//!
//! A [`RepoSlug`] is the immutable key of a repository; its display name
//! and filesystem path are mutable attributes living on the `Repo` node
//! (spec §7), never part of identity. Epics, tasks, decisions and docs
//! get per-repo sequential integers per kind — allocated by the server,
//! never reused, never renumbered, gaps meaningful. Allocation itself
//! (counters, next-id) is graph logic and lands with the graph; this
//! module owns the id *types*: construction, parse, format, ordering.
//!
//! Display/parse format is `<slug>:<prefix><n>` with the per-kind prefix
//! of §3 — task `U`, epic `E`, decision `D`, doc `S`: `camera:U7`,
//! `camera:D3`. Parsing accepts exactly what formatting produces, so
//! parse→format is byte-identical on every accepted string; leading
//! zeros are rejected because formatting never emits them.

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ValueError;

/// Longest accepted repository slug, in bytes. A guard against
/// pathological keys, not a product limit.
pub const MAX_SLUG_LEN: usize = 64;

/// An immutable repository slug — the key a repository is registered
/// under (spec §3, D3), e.g. `camera` or `ikvm-html`.
///
/// Charset (a judgment call; §3 does not pin one): one or more ASCII
/// lowercase alphanumeric segments joined by single hyphens —
/// `[a-z0-9]+(-[a-z0-9]+)*`, at most [`MAX_SLUG_LEN`] bytes. No
/// uppercase, no leading, trailing or doubled hyphens, and crucially no
/// colon, so a node id splits unambiguously at its first `:`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoSlug(String);

impl RepoSlug {
    /// Validates and wraps an owned string as a slug.
    pub fn new(slug: String) -> Result<Self, ValueError> {
        let well_formed = !slug.is_empty()
            && slug.len() <= MAX_SLUG_LEN
            // Segments split on hyphens; the charset rule makes every
            // split a real boundary, so one all-segments check rejects
            // leading, trailing and doubled hyphens alike.
            && slug.split('-').all(|segment| {
                !segment.is_empty()
                    && segment
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            });
        if well_formed {
            Ok(Self(slug))
        } else {
            Err(ValueError::InvalidSlug { input: slug })
        }
    }

    /// The slug as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RepoSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for RepoSlug {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl Serialize for RepoSlug {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RepoSlug {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::new(raw).map_err(D::Error::custom)
    }
}

/// Builds the four typed node ids of §3. Each is an immutable
/// (slug, number) pair with its per-kind display prefix; ordering is
/// slug first, then number ascending — the total order §5.2's task-id
/// tiebreaker relies on.
macro_rules! node_id {
    ($name:ident, $prefix:literal, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Serializes as its canonical `<slug>:<prefix><n>` string and
        /// deserializes through the same validation as [`FromStr`].
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            slug: RepoSlug,
            number: u64,
        }

        impl $name {
            /// The per-kind display prefix of spec §3.
            pub const PREFIX: char = $prefix;

            /// Wraps an allocated (slug, number) pair. Rejects `0`:
            /// allocation is sequential from 1, so zero is never a
            /// legitimately allocated number.
            pub fn new(slug: RepoSlug, number: u64) -> Result<Self, ValueError> {
                if number == 0 {
                    return Err(ValueError::InvalidId {
                        input: format!("{slug}:{}0", Self::PREFIX),
                    });
                }
                Ok(Self { slug, number })
            }

            /// The repository this id belongs to.
            pub fn repo(&self) -> &RepoSlug {
                &self.slug
            }

            /// The per-repo sequential number within its kind.
            pub fn number(&self) -> u64 {
                self.number
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}{}", self.slug, Self::PREFIX, self.number)
            }
        }

        impl FromStr for $name {
            type Err = ValueError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let malformed = || ValueError::InvalidId {
                    input: s.to_owned(),
                };
                let (slug, rest) = s.split_once(':').ok_or_else(malformed)?;
                let slug = RepoSlug::from_str(slug).map_err(|_| malformed())?;
                let Some(digits) = rest.strip_prefix(Self::PREFIX) else {
                    return Err(malformed());
                };
                // No empty numbers, no leading zeros, ASCII digits only;
                // `parse` then fails only on overflow.
                if digits.is_empty()
                    || (digits.len() > 1 && digits.starts_with('0'))
                    || !digits.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err(malformed());
                }
                let number = digits.parse::<u64>().map_err(|_| malformed())?;
                Self::new(slug, number)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                Self::from_str(&raw).map_err(D::Error::custom)
            }
        }
    };
}

node_id!(
    TaskId,
    'U',
    "A task id (`U` prefix, board heritage), e.g. `camera:U7`."
);
node_id!(EpicId, 'E', "An epic id (`E` prefix), e.g. `camera:E2`.");
node_id!(
    DecisionId,
    'D',
    "A decision id (`D` prefix), e.g. `camera:D3`."
);
node_id!(
    DocId,
    'S',
    "A registered-document id (`S` prefix), e.g. `camera:S1`."
);

#[cfg(test)]
mod tests {
    use super::*;

    fn slug(s: &str) -> RepoSlug {
        RepoSlug::from_str(s).unwrap()
    }

    #[test]
    fn slug_accepts_valid_charset() {
        for s in [
            "camera",
            "ikvm-html",
            "dacode",
            "x",
            "a1",
            "a-1-b",
            "gen-2-xc3light",
        ] {
            assert_eq!(RepoSlug::from_str(s).unwrap().as_str(), s, "slug {s}");
        }
    }

    #[test]
    fn slug_rejects_malformed() {
        for s in [
            "",         // empty
            "Camera",   // uppercase
            "came_ra",  // underscore
            "camera!",  // punctuation
            "cam era",  // space
            "-camera",  // leading hyphen
            "camera-",  // trailing hyphen
            "cam--era", // doubled hyphen
            "camera:x", // colon would break id splitting
            "カメラ",   // non-ASCII
        ] {
            assert_eq!(
                RepoSlug::from_str(s),
                Err(ValueError::InvalidSlug {
                    input: s.to_owned()
                }),
                "slug {s:?} must be rejected"
            );
        }
    }

    #[test]
    fn slug_rejects_overlong() {
        let long = "a".repeat(MAX_SLUG_LEN + 1);
        assert!(matches!(
            RepoSlug::from_str(&long),
            Err(ValueError::InvalidSlug { .. })
        ));
        // The boundary itself is fine.
        let max = "a".repeat(MAX_SLUG_LEN);
        assert!(RepoSlug::from_str(&max).is_ok());
    }

    #[test]
    fn id_format_is_slug_prefix_number() {
        assert_eq!(
            TaskId::new(slug("camera"), 7).unwrap().to_string(),
            "camera:U7"
        );
        assert_eq!(
            EpicId::new(slug("ikvm-html"), 12).unwrap().to_string(),
            "ikvm-html:E12"
        );
        assert_eq!(
            DecisionId::new(slug("dacode"), 3).unwrap().to_string(),
            "dacode:D3"
        );
        assert_eq!(
            DocId::new(slug("camera"), 1).unwrap().to_string(),
            "camera:S1"
        );
    }

    #[test]
    fn id_parse_format_roundtrip_is_byte_identical() {
        let tasks = ["camera:U7", "camera:U1", "camera:U18446744073709551615"];
        for s in tasks {
            assert_eq!(TaskId::from_str(s).unwrap().to_string(), s);
        }
        for s in ["ikvm-html:E42", "camera:E1"] {
            assert_eq!(EpicId::from_str(s).unwrap().to_string(), s);
        }
        for s in ["dacode:D3", "camera:D1"] {
            assert_eq!(DecisionId::from_str(s).unwrap().to_string(), s);
        }
        for s in ["camera:S1", "ikvm-html:S9"] {
            assert_eq!(DocId::from_str(s).unwrap().to_string(), s);
        }
    }

    #[test]
    fn id_parse_rejects_malformed() {
        for s in [
            "camera:U0",                    // zero is never allocated
            "camera:U07",                   // leading zero: not a format output
            "camera:U",                     // no number
            "camera:U-1",                   // negative
            "camera:U+1",                   // explicit sign
            "camera:Ux",                    // non-digit
            "camera:U1a",                   // trailing garbage
            "camera:T7",                    // wrong prefix
            "camera:7",                     // missing prefix
            "cameraU7",                     // missing colon
            ":U7",                          // missing slug
            "Camera:U7",                    // invalid slug part
            "camera:U7:extra",              // trailing colon segment
            "camera:U18446744073709551616", // u64 overflow
            "",                             // empty
        ] {
            assert_eq!(
                TaskId::from_str(s),
                Err(ValueError::InvalidId {
                    input: s.to_owned()
                }),
                "id {s:?} must be rejected"
            );
        }
        // Prefixes are kind-specific: a U id does not parse as an epic.
        assert_eq!(
            EpicId::from_str("camera:U7"),
            Err(ValueError::InvalidId {
                input: "camera:U7".to_owned()
            })
        );
    }

    #[test]
    fn id_new_rejects_zero() {
        assert!(TaskId::new(slug("camera"), 0).is_err());
        assert!(EpicId::new(slug("camera"), 0).is_err());
        assert!(DecisionId::new(slug("camera"), 0).is_err());
        assert!(DocId::new(slug("camera"), 0).is_err());
    }

    #[test]
    fn id_accessors_and_prefixes() {
        let t = TaskId::new(slug("camera"), 7).unwrap();
        assert_eq!(t.repo().as_str(), "camera");
        assert_eq!(t.number(), 7);
        assert_eq!(TaskId::PREFIX, 'U');
        assert_eq!(EpicId::PREFIX, 'E');
        assert_eq!(DecisionId::PREFIX, 'D');
        assert_eq!(DocId::PREFIX, 'S');
    }

    #[test]
    fn id_order_is_slug_then_number() {
        let a = TaskId::new(slug("camera"), 2).unwrap();
        let b = TaskId::new(slug("camera"), 10).unwrap();
        let c = TaskId::new(slug("dacode"), 1).unwrap();
        assert!(a < b, "numeric ascending, not lexicographic");
        assert!(b < c, "slug first");
    }
}
