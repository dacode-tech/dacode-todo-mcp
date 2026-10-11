//! The activity panel (t84's D5): the census rendered — the store's
//! live statistics, one section after the Totals cards. The header
//! line ("N events since <first_ts>"), the by-kind chips (kind + count,
//! alphabetical — the census map's own order), the day histogram (one
//! bar per day across the by-day map's bounds — the earliest to the
//! latest event day, never the first and last line's: the store API
//! permits caller-passed `at`, so a non-monotonic log must not drop
//! counted days — with zero-event days as zero-height bars, a
//! continuous timeline, no gaps that read as missing data), and the
//! last-activity line. Per-epic counts are
//! deliberately absent: events are node-scoped, so the panel stays
//! event-derived and the sidebar's epic rows carry the per-lane counts.
//!
//! Every dimension is integer arithmetic (the `pct` precedent: no
//! floating point in a rendered dimension), so the bytes are a pure
//! function of the census — deterministic, golden-pinnable.

use std::collections::BTreeMap;

use todo_store::EventCensus;

use super::esc;

/// The tallest bar's height, in pixels — the capped max the
/// proportional scaling maps the busiest day onto.
const BAR_MAX_PX: u64 = 64;

/// The activity panel's HTML: its own `<h2>` section, emitted after the
/// Totals cards. An empty census (no committed events — unreachable
/// behind a successful render, which needs the graph the same log
/// describes) still renders honestly: the count, no since, no bars.
pub(super) fn panel_html(census: &EventCensus) -> String {
    let mut out = String::new();
    match &census.first_ts {
        Some(first) => out.push_str(&format!(
            "\n<h2>Activity — {} events since {}</h2>\n",
            census.total,
            esc(first)
        )),
        None => out.push_str(&format!("\n<h2>Activity — {} events</h2>\n", census.total)),
    }
    out.push_str("<div class=\"act\">");
    let chips: String = census
        .by_kind
        .iter()
        .map(|(kind, count)| format!("<span class=\"chip\">{} <b>{count}</b></span>", esc(kind)))
        .collect();
    out.push_str(&format!("<div class=\"chips\">{chips}</div>"));
    if let (Some((first_day, _)), Some((last_day, _))) = (
        census.by_day.first_key_value(),
        census.by_day.last_key_value(),
    ) {
        let bars = histogram_html(&census.by_day, first_day, last_day);
        out.push_str(&format!("<div class=\"hist\">{bars}</div>"));
    }
    if let Some(last) = &census.last_ts {
        out.push_str(&format!(
            "<div class=\"lastact\">Last activity: {}</div>\n",
            esc(last)
        ));
    }
    out.push_str("</div>\n");
    out
}

/// The day histogram: one bar per day from `first_day` to `last_day`
/// inclusive — the caller passes the by-day map's own first and last
/// keys, so the range is the earliest..latest event day whatever the
/// log's line order (the store API permits caller-passed `at`; the
/// first and last line's days are not the bounds). The keys are
/// `YYYY-MM-DD`, so lexicographic order is chronological and the walk
/// is the timeline. Zero-event days render as zero-height bars; the
/// busiest day in the range maps to [`BAR_MAX_PX`] by integer
/// round-half-up, the `pct` precedent.
fn histogram_html(by_day: &BTreeMap<String, u64>, first_day: &str, last_day: &str) -> String {
    let mut days: Vec<String> = Vec::new();
    let mut day = first_day.to_owned();
    while day.as_str() <= last_day {
        days.push(day.clone());
        day = next_day(&day);
    }
    let max_count = days
        .iter()
        .filter_map(|day| by_day.get(day).copied())
        .max()
        .unwrap_or(0);
    days.iter()
        .map(|day| {
            let count = by_day.get(day).copied().unwrap_or(0);
            let height = (BAR_MAX_PX * count + max_count / 2)
                .checked_div(max_count)
                .unwrap_or(0);
            format!(
                "<div class=\"hday\" title=\"{day}: {count} events\">\
                 <span class=\"hnum\">{count}</span>\
                 <span class=\"hbar\" style=\"height:{height}px\"></span>\
                 <span class=\"hdate\">{day}</span></div>"
            )
        })
        .collect()
}

/// The day after `day` (`YYYY-MM-DD`): the calendar arithmetic the
/// continuous timeline needs — month lengths and leap years, the same
/// rules `todo_domain`'s date validation applies. The keys come from
/// canonical timestamps, so the shape holds; the fallbacks keep the
/// arithmetic total rather than asserting it.
fn next_day(day: &str) -> String {
    let (mut year, mut month, mut day_of_month) = parse_day(day);
    day_of_month += 1;
    if day_of_month > days_in_month(year, month) {
        day_of_month = 1;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    format!("{year:04}-{month:02}-{day_of_month:02}")
}

/// `YYYY-MM-DD` split into its three numbers, zero on a shape that
/// cannot occur (the keys derive from validated timestamps).
fn parse_day(day: &str) -> (u32, u32, u32) {
    let mut parts = day.split('-');
    (
        parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
        parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
        parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
    )
}

/// The days in one month, leap years included.
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            let leap =
                year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
            if leap { 29 } else { 28 }
        }
    }
}

/// The activity panel's stylesheet.
pub(super) const PANEL_CSS: &str = r#".act { background: #fff; border: 1px solid #e3e3e0;
      border-radius: 10px; padding: 12px 14px; }
.chips { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
.chip { border: 1px solid #e3e3e0; border-radius: 12px; padding: 2px 10px; font-size: 11.5px;
        background: #fbfbfa; color: #3f444c; }
.chip b { color: #2f6f4f; font-variant-numeric: tabular-nums; }
.hist { display: flex; align-items: flex-end; gap: 5px; flex-wrap: wrap; }
.hday { display: flex; flex-direction: column; align-items: center; gap: 2px; }
.hnum { font-size: 10.5px; color: #6b7280; font-variant-numeric: tabular-nums; }
.hbar { width: 24px; background: #2f6f4f; border-radius: 3px 3px 0 0; }
.hdate { font-size: 9.5px; color: #6b7280;
         font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.lastact { margin-top: 10px; font-size: 11.5px; color: #6b7280; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// The calendar walk: month and year rollovers, a leap February,
    /// and the fixed-width formatting.
    #[test]
    fn next_day_rolls_months_years_and_leap_february() {
        assert_eq!(next_day("2026-10-02"), "2026-10-03");
        assert_eq!(next_day("2026-10-31"), "2026-11-01");
        assert_eq!(next_day("2026-12-31"), "2027-01-01");
        assert_eq!(next_day("2024-02-28"), "2024-02-29");
        assert_eq!(next_day("2023-02-28"), "2023-03-01");
        assert_eq!(next_day("2024-02-29"), "2024-03-01");
    }

    /// The bar heights: the busiest day maps to the capped max, a
    /// zero-event day to zero, and the scaling is integer
    /// round-half-up — the `pct` precedent, no floating point.
    #[test]
    fn bar_heights_scale_integers_with_the_capped_max() {
        let by_day = BTreeMap::from([
            ("2026-10-02".to_owned(), 10),
            ("2026-10-03".to_owned(), 3),
            ("2026-10-04".to_owned(), 0),
            ("2026-10-05".to_owned(), 3),
            ("2026-10-06".to_owned(), 2),
            ("2026-10-07".to_owned(), 5),
        ]);
        let bars = histogram_html(&by_day, "2026-10-02", "2026-10-07");
        assert!(
            bars.contains("height:64px"),
            "the busiest day is the cap: {bars}"
        );
        assert!(bars.contains("height:19px"), "3 of 10 rounds to 19: {bars}");
        assert!(
            bars.contains("height:0px"),
            "the zero day is a zero bar: {bars}"
        );
        assert!(bars.contains("height:13px"), "2 of 10 rounds to 13: {bars}");
        assert!(
            bars.contains("height:32px"),
            "5 of 10 is half the cap: {bars}"
        );
        assert_eq!(
            bars.matches("hday").count(),
            6,
            "one bar per day, gaps included"
        );
    }

    /// The histogram's range is the by-day map's bounds, never the
    /// first and last line's timestamps: the store API permits
    /// caller-passed `at`, so a non-monotonic log must not silently
    /// drop counted days — or the whole histogram, when the first
    /// line's day is the later. The reviewers' round-1 low, fixed
    /// and pinned: under the old range this census rendered zero
    /// bars.
    #[test]
    fn the_histogram_range_is_the_by_day_bounds_not_the_line_order() {
        let census = EventCensus {
            total: 6,
            by_kind: BTreeMap::from([("task_add".to_owned(), 6)]),
            by_day: BTreeMap::from([
                ("2026-10-02".to_owned(), 2),
                ("2026-10-04".to_owned(), 1),
                ("2026-10-07".to_owned(), 3),
            ]),
            // The log order disagrees with the clock order: the first
            // line carries the latest day, the last line an earlier
            // one — the line-order range is empty.
            first_ts: Some("2026-10-07T08:00:00.000000000Z".to_owned()),
            last_ts: Some("2026-10-04T09:00:00.000000000Z".to_owned()),
        };
        let html = panel_html(&census);
        assert_eq!(
            html.matches("hday").count(),
            6,
            "the earliest..latest day, gaps included: {html}"
        );
        assert!(
            html.contains("2026-10-02"),
            "the earliest day is drawn: {html}"
        );
        assert!(
            html.contains("2026-10-07"),
            "the latest day is drawn: {html}"
        );
    }
}
