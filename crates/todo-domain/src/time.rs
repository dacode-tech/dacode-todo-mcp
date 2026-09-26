//! Time values (spec §3, §8): RFC 3339 UTC timestamps and calendar
//! dates, both caller-passed — nothing in this crate reads a wall clock.
//!
//! Validation is hand-rolled (no chrono/time dependency is approved):
//! strict canonical forms only, so parse∘format is the identity and
//! lexicographic order over the stored strings is a total order.
//!
//! - [`Timestamp`]: exactly `YYYY-MM-DDTHH:MM:SSZ` — a canonical subset
//!   of RFC 3339. Uppercase `T`/`Z` only; numeric UTC offsets (including
//!   `+00:00`) and fractional seconds are rejected, so each instant has
//!   one spelling and the round trip is byte-stable. Second `60` is
//!   accepted: RFC 3339 permits leap seconds.
//! - [`Date`]: `YYYY-MM-DD` — the shape of a task's optional `until`
//!   (D9), which is pure metadata; selection reads no calendar (§8).

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ValueError;

/// Length of a canonical `YYYY-MM-DDTHH:MM:SSZ` timestamp, in bytes.
const TIMESTAMP_LEN: usize = 20;
/// Length of a canonical `YYYY-MM-DD` date, in bytes.
const DATE_LEN: usize = 10;
/// Index of the seconds field's first digit in a canonical timestamp.
const SECONDS_AT: usize = 17;

/// An RFC 3339 UTC timestamp in canonical form, exactly
/// `YYYY-MM-DDTHH:MM:SSZ` (e.g. `2026-09-26T12:34:56Z`). Validated at
/// construction; the stored bytes are exactly what formatting emits.
///
/// The canonical form is a deliberate subset of RFC 3339: uppercase
/// `T`/`Z` only, no numeric UTC offset (not even `+00:00`), and no
/// fractional seconds — one spelling per instant, so parse∘format is
/// byte-identical and [`Ord`] (lexicographic over the fixed-width
/// string) is exactly chronological order. Second `60` is accepted
/// because RFC 3339 permits leap seconds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(String);

impl Timestamp {
    /// Parses and validates an RFC 3339 UTC timestamp in canonical form.
    pub fn parse(s: &str) -> Result<Self, ValueError> {
        validate_timestamp(s)?;
        Ok(Self(s.to_owned()))
    }

    /// The canonical timestamp string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Timestamp {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(D::Error::custom)
    }
}

/// A calendar date `YYYY-MM-DD` — the shape of a task's optional
/// `until` field (D9). [`Ord`] is lexicographic over the canonical
/// string, which for this fixed shape is chronological order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(String);

impl Date {
    /// Parses and validates a `YYYY-MM-DD` date.
    pub fn parse(s: &str) -> Result<Self, ValueError> {
        validate_date(s)?;
        Ok(Self(s.to_owned()))
    }

    /// The canonical date string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Date {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(D::Error::custom)
    }
}

fn validate_timestamp(s: &str) -> Result<(), ValueError> {
    let invalid = || ValueError::InvalidTimestamp {
        input: s.to_owned(),
    };
    let b = s.as_bytes();
    // Canonical form is exactly `YYYY-MM-DDTHH:MM:SSZ`: 20 bytes, no
    // fractional seconds, terminating `Z` (see the type docs).
    if b.len() != TIMESTAMP_LEN {
        return Err(invalid());
    }
    let (year, month, day) = date_fields(b).ok_or_else(invalid)?;
    // Fixed positions: the shape is not negotiable.
    let fixed_ok = b.get(10).copied() == Some(b'T')
        && b.get(13).copied() == Some(b':')
        && b.get(16).copied() == Some(b':')
        && b.get(SECONDS_AT + 2).copied() == Some(b'Z');
    if !fixed_ok {
        return Err(invalid());
    }
    let hour = two_digits(b, 11).ok_or_else(invalid)?;
    let minute = two_digits(b, 14).ok_or_else(invalid)?;
    let second = two_digits(b, SECONDS_AT).ok_or_else(invalid)?;
    // RFC 3339 admits second 60 for leap seconds.
    if hour > 23 || minute > 59 || second > 60 {
        return Err(invalid());
    }
    if calendar_valid(year, month, day) {
        Ok(())
    } else {
        Err(invalid())
    }
}

fn validate_date(s: &str) -> Result<(), ValueError> {
    let invalid = || ValueError::InvalidDate {
        input: s.to_owned(),
    };
    let b = s.as_bytes();
    if b.len() != DATE_LEN {
        return Err(invalid());
    }
    let (year, month, day) = date_fields(b).ok_or_else(invalid)?;
    if calendar_valid(year, month, day) {
        Ok(())
    } else {
        Err(invalid())
    }
}

/// The `YYYY-MM-DD` prefix fields of a canonical timestamp or date.
fn date_fields(b: &[u8]) -> Option<(u32, u32, u32)> {
    if b.get(4).copied() != Some(b'-') || b.get(7).copied() != Some(b'-') {
        return None;
    }
    let year = four_digits(b, 0)?;
    let month = two_digits(b, 5)?;
    let day = two_digits(b, 8)?;
    Some((year, month, day))
}

/// Gregorian calendar check: month 1–12, day within the month, leap
/// years divisible by 4 except centuries unless divisible by 400.
fn calendar_valid(year: u32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if leap {
                29
            } else {
                28
            }
        }
    };
    day <= max_day
}

fn two_digits(b: &[u8], at: usize) -> Option<u32> {
    Some(digit(b, at)? * 10 + digit(b, at + 1)?)
}

fn four_digits(b: &[u8], at: usize) -> Option<u32> {
    Some(digit(b, at)? * 1000 + two_digits(b, at + 1)? * 10 + digit(b, at + 3)?)
}

fn digit(b: &[u8], at: usize) -> Option<u32> {
    b.get(at)
        .copied()
        .filter(u8::is_ascii_digit)
        .map(|c| u32::from(c - b'0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_accepts_canonical_utc() {
        for s in [
            "2026-09-26T12:34:56Z",
            "2026-09-26T00:00:00Z",
            "2026-12-31T23:59:59Z",
            "2024-02-29T06:00:00Z", // leap day
            "2000-02-29T06:00:00Z", // century divisible by 400
            "2016-12-31T23:59:60Z", // leap second, RFC 3339 permits 60
        ] {
            let ts = Timestamp::parse(s);
            assert_eq!(ts.map(|t| t.as_str().to_owned()), Ok(s.to_owned()), "{s}");
        }
    }

    #[test]
    fn timestamp_rejects_malformed() {
        for s in [
            "2026-09-26T12:34:56+00:00",   // numeric offset, even UTC
            "2026-09-26T12:34:56+02:00",   // non-UTC offset
            "2026-09-26T12:34:56-05:00",   // non-UTC offset
            "2026-09-26t12:34:56z",        // lowercase t/z
            "2026-09-26 12:34:56Z",        // space instead of T
            "2026-09-26T12:34:56",         // no zone at all
            "2026-09-26T12:34:56.5Z",      // fractional seconds: canonical form has none
            "2026-09-26T12:34:56.123456Z", // fractional seconds
            "2026-09-26T12:34:56.Z",       // empty fraction
            "2026-09-26T12:34:56.5",       // fraction without Z
            "2026-09-26T12:34:56ZX",       // trailing garbage
            "2026-09-26T24:00:00Z",        // hour 24
            "2026-09-26T12:60:00Z",        // minute 60
            "2026-09-26T12:34:61Z",        // second 61
            "2026-13-01T00:00:00Z",        // month 13
            "2026-00-01T00:00:00Z",        // month 0
            "2026-01-00T00:00:00Z",        // day 0
            "2026-04-31T00:00:00Z",        // April has 30 days
            "2023-02-29T00:00:00Z",        // 2023 is not a leap year
            "2100-02-29T00:00:00Z",        // century, not divisible by 400
            "26-09-26T12:34:56Z",          // short year
            "2026-9-26T12:34:56Z",         // unpadded month
            "",                            // empty
        ] {
            assert_eq!(
                Timestamp::parse(s),
                Err(ValueError::InvalidTimestamp {
                    input: s.to_owned()
                }),
                "{s:?} must be rejected"
            );
        }
    }

    #[test]
    fn date_accepts_canonical() {
        for s in ["2026-09-26", "2024-02-29", "2000-02-29", "2026-12-31"] {
            let d = Date::parse(s);
            assert_eq!(d.map(|d| d.as_str().to_owned()), Ok(s.to_owned()), "{s}");
        }
    }

    #[test]
    fn date_rejects_malformed() {
        for s in [
            "2026-09-26T00:00:00Z", // a timestamp is not a date
            "2026-9-26",            // unpadded month
            "2026-09-2",            // short day
            "2026-09-26 ",          // trailing space
            " 2026-09-26",          // leading space
            "2100-02-29",           // century, not divisible by 400
            "2023-02-29",           // not a leap year
            "2026-13-01",           // month 13
            "2026-01-00",           // day 0
            "2026/09/26",           // wrong separator
            "",                     // empty
        ] {
            assert_eq!(
                Date::parse(s),
                Err(ValueError::InvalidDate {
                    input: s.to_owned()
                }),
                "{s:?} must be rejected"
            );
        }
    }

    #[test]
    fn calendar_rules() {
        assert!(calendar_valid(2024, 2, 29));
        assert!(!calendar_valid(2023, 2, 29));
        assert!(calendar_valid(2000, 2, 29));
        assert!(!calendar_valid(1900, 2, 29));
        assert!(!calendar_valid(2026, 4, 31));
        assert!(calendar_valid(2026, 1, 31));
    }

    #[test]
    fn ord_is_chronological() {
        // The canonical fixed-width form has one precision, so
        // lexicographic Ord is unconditionally chronological order.
        let early = Timestamp::parse("2026-09-26T12:34:56Z").unwrap();
        let late = Timestamp::parse("2026-09-26T12:34:57Z").unwrap();
        assert!(early < late);
        let d1 = Date::parse("2026-09-26").unwrap();
        let d2 = Date::parse("2026-10-01").unwrap();
        assert!(d1 < d2);
    }
}
