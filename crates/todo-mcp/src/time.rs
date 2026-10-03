//! The server's clock: the one wall-clock read this binary makes,
//! stamped into `registered_at` at registration — §7's audit instant.
//!
//! §8's "no wall clock" is about *selection* — `next_task` derives
//! from graph state alone — and the store's events already carry
//! caller-passed timestamps by design; the binary is the caller here.
//! The arithmetic is pinned by unit tests over known instants, and the
//! domain validates what the binary formats: every formatted string is
//! fed through [`Timestamp::parse`], with the round trip pinned below,
//! so a formatting bug is a parse rejection, never a silent bad
//! timestamp. The representable range is bounded on both sides by
//! clamps rather than faults — before the Unix epoch, and past the
//! domain's last acceptable instant — because a clock fault is not
//! worth dying for; both clamps are observable in the stored
//! `registered_at`.
//!
//! The civil-calendar conversion is Howard Hinnant's `civil_from_days`
//! (the standard days-from-epoch algorithm), hand-rolled because no
//! date/time dependency is approved for this workspace.

use std::time::{SystemTime, UNIX_EPOCH};

use todo_domain::Timestamp;

/// Seconds in one day.
const SECS_PER_DAY: u64 = 86_400;
/// Seconds in one hour.
const SECS_PER_HOUR: u64 = 3_600;
/// Seconds in one minute.
const SECS_PER_MINUTE: u64 = 60;
/// The last second of year 9999 in Unix seconds — the saturation
/// ceiling, because the domain's [`Timestamp`] accepts a four-digit
/// year at most and the formatter would emit five digits past it. The
/// value is pinned by test against the domain's own canonical
/// spelling (the oracle), so a hand-computation error fails the pin
/// rather than reaching production.
const MAX_SECS: u64 = 253_402_300_799;

/// Reads the clock and returns the instant as a canonical
/// [`Timestamp`].
///
/// A clock before the Unix epoch — a corrupted host clock — clamps to
/// the epoch instant rather than failing: registration is not the
/// operation that should die to a clock fault, and the clamp is
/// observable in the stored `registered_at`. A clock past the domain's
/// last acceptable instant saturates there (see [`from_unix`]).
pub fn now() -> Timestamp {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    from_unix(duration.as_secs(), duration.subsec_nanos())
}

/// The instant `secs` seconds and `nanos` nanoseconds after the Unix
/// epoch, as a canonical [`Timestamp`]. Instants past the domain's
/// last acceptable one saturate there — symmetric to `now`'s pre-epoch
/// clamp, and for the same reason: the formatter would emit a
/// five-digit year the domain rejects, and a clock fault is not worth
/// dying for. The clamp is observable in the stored `registered_at`.
fn from_unix(secs: u64, nanos: u32) -> Timestamp {
    let (secs, nanos) = if secs > MAX_SECS {
        (MAX_SECS, 999_999_999)
    } else {
        (secs, nanos)
    };
    let formatted = format_unix(secs, nanos);
    #[allow(
        clippy::expect_used,
        reason = "past saturation the year is at most 9999, so format_unix emits exactly \
                  the domain's canonical 30-byte spelling and Timestamp::parse accepts \
                  it by construction; the round trip is pinned by this module's tests"
    )]
    {
        Timestamp::parse(&formatted).expect("the canonical spelling parses")
    }
}

/// Formats the instant as the canonical 30-byte RFC 3339 UTC spelling
/// `YYYY-MM-DDTHH:MM:SS.sssssssssZ` — the domain [`Timestamp`] form,
/// with the fraction always present at nine digits.
fn format_unix(secs: u64, nanos: u32) -> String {
    let days = (secs / SECS_PER_DAY) as i64;
    let seconds_of_day = secs % SECS_PER_DAY;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / SECS_PER_HOUR;
    let minute = (seconds_of_day % SECS_PER_HOUR) / SECS_PER_MINUTE;
    let second = seconds_of_day % SECS_PER_MINUTE;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{nanos:09}Z")
}

/// The Gregorian calendar date of Unix day `z` (Howard Hinnant's
/// `civil_from_days`): year, month, day.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_formats_as_the_canonical_epoch_instant() {
        assert_eq!(format_unix(0, 0), "1970-01-01T00:00:00.000000000Z");
    }

    #[test]
    fn leap_day_boundaries_format_correctly() {
        // 2024-02-29 (a leap year's day) and the day after; 2000-02-29
        // (a century leap year); the non-leap 2023-02-28 → 03-01 seam;
        // 2100-03-01, the century non-leap that catches a proleptic
        // every-fourth rule.
        assert_eq!(format_unix(1709164800, 0), "2024-02-29T00:00:00.000000000Z");
        assert_eq!(format_unix(1709251200, 0), "2024-03-01T00:00:00.000000000Z");
        assert_eq!(format_unix(951782400, 0), "2000-02-29T00:00:00.000000000Z");
        assert_eq!(format_unix(1677628799, 0), "2023-02-28T23:59:59.000000000Z");
        assert_eq!(format_unix(1677628800, 0), "2023-03-01T00:00:00.000000000Z");
        assert_eq!(format_unix(4107542400, 0), "2100-03-01T00:00:00.000000000Z");
    }

    #[test]
    fn the_fraction_pads_to_nine_digits_both_ways() {
        // Zero pads to nine zeros; a nine-digit fraction passes through
        // unchanged; a short fraction zero-pads on the right.
        assert_eq!(format_unix(0, 0), "1970-01-01T00:00:00.000000000Z");
        assert_eq!(format_unix(0, 123456789), "1970-01-01T00:00:00.123456789Z");
        assert_eq!(format_unix(0, 5), "1970-01-01T00:00:00.000000005Z");
        assert_eq!(format_unix(0, 900000000), "1970-01-01T00:00:00.900000000Z");
    }

    #[test]
    fn every_formatted_output_parses_and_round_trips() {
        // The domain validates what the binary formats: each formatted
        // string is a canonical Timestamp, and parsing it returns the
        // same bytes. Covers the epoch, leap seams, a far-future year
        // (four-digit width), the max fraction and a mid-range instant.
        for (secs, nanos) in [
            (0u64, 0u32),
            (1709164800, 1),
            (1677628799, 999999999),
            (1999999999, 123456789),
            (253402214399, 0), // 9999-12-30T23:59:59 — the last full day
        ] {
            let formatted = format_unix(secs, nanos);
            let parsed = Timestamp::parse(&formatted)
                .unwrap_or_else(|error| panic!("{}: {error}", formatted));
            assert_eq!(parsed.as_str(), formatted, "round trip for {formatted}");
        }
    }

    #[test]
    fn a_clock_past_the_domain_s_last_instant_saturates_to_it() {
        // The ceiling is the domain's own: the last canonical string
        // Timestamp::parse accepts. Pinned exactly, because hand
        // values for this boundary differ by a day and the pin is the
        // arbiter — the domain's parse is the oracle.
        const MAX: &str = "9999-12-31T23:59:59.999999999Z";
        assert_eq!(format_unix(MAX_SECS, 999_999_999), MAX);
        assert!(
            Timestamp::parse(MAX).is_ok(),
            "the domain accepts the ceiling"
        );
        // Past it, from_unix saturates rather than panicking: the
        // formatter would emit a five-digit year the domain rejects.
        assert_eq!(from_unix(MAX_SECS + 1, 0).as_str(), MAX);
        assert_eq!(from_unix(u64::MAX, 123).as_str(), MAX);
        // The boundary itself is untouched by the clamp.
        assert_eq!(
            from_unix(MAX_SECS, 999_999_999).as_str(),
            MAX,
            "the last acceptable instant passes through unchanged"
        );
    }

    #[test]
    fn a_clock_before_the_epoch_clamps_to_the_epoch() {
        // The clamp is `now`'s documented behaviour; its unit pin is
        // the pure half: a zero duration formats as the epoch instant.
        assert_eq!(format_unix(0, 0), "1970-01-01T00:00:00.000000000Z");
    }
}
