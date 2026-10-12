//! The rewards easter egg's pure half (spec §15, D22 — signed
//! 2026-10-13): the S/M/L/XL size vocabulary, the per-root config's
//! domain types, and [`reward`], the closure computation over them.
//!
//! Boundary (§2): no I/O, no async, nothing read but the arguments.
//! The rest of the feature lives above — the `enabled` gate and the
//! config document (the store, U61), the closure response and
//! `reward_reflect` (the server, U62), the rewards section (the
//! display, U63) — and the task node's `estimate` field is U61's
//! storage story: the sizes exist here as a vocabulary, the node
//! wiring lands with the `FORMAT_VERSION` bump it needs.
//!
//! Determinism (§8): the arithmetic is all-integer and checked —
//! `round_half_up(base × priority_pct × kind_pct / 10000)`, no float
//! anywhere, and a config whose product would overflow `u64` grants
//! nothing rather than panicking. The config types are the persisted
//! schema's domain half: derives only, field order = declaration
//! order, field names = wire names (the size, priority and kind
//! spellings), so the store's goldens pin their bytes in U61 — the
//! crate's test codec serializes no `bool`, so the bool-bearing block
//! itself is pinned there, while its u64 sub-structs round-trip under
//! the codec here.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::status::{Priority, Status, TaskKind};

/// A task's size bucket (§15, D22): S/M/L/XL, the volume axis the
/// reward scales on. Two buckets exist per task — the `estimate`
/// (planned, set at creation) and the actual (reported at closure) —
/// but they share this one vocabulary; the node fields are U61's
/// storage story. Wire spelling is `snake_case` like every enum in
/// the crate: `"s"`, `"m"`, `"l"`, `"xl"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    /// The smallest bucket.
    S,
    /// The default-sized bucket in a typical config.
    M,
    /// The large bucket.
    L,
    /// The largest bucket.
    Xl,
}

impl Size {
    /// The stable wire spelling, e.g. `"xl"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::S => "s",
            Self::M => "m",
            Self::L => "l",
            Self::Xl => "xl",
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The per-bucket base costs (§15): the reward's starting amount per
/// size. A fixed-field struct, not a map: the bucket vocabulary is
/// closed ([`Size`]'s four variants), declaration order is the
/// canonical order, and a missing bucket is a deserialization error
/// rather than a silently absent entry — the named-field discipline
/// for persisted schemas. Field names are the [`Size`] wire
/// spellings, so the config's keys and the task's size fields share
/// one vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseCosts {
    /// The base cost of an `s` task.
    pub s: u64,
    /// The base cost of an `m` task.
    pub m: u64,
    /// The base cost of an `l` task.
    pub l: u64,
    /// The base cost of an `xl` task.
    pub xl: u64,
}

impl BaseCosts {
    /// The base cost of `size`'s bucket.
    pub fn cost(&self, size: Size) -> u64 {
        match size {
            Size::S => self.s,
            Size::M => self.m,
            Size::L => self.l,
            Size::Xl => self.xl,
        }
    }
}

/// The priority multipliers (§15): integer percents, 100 = ×1, keyed
/// by the [`Priority`] wire spellings. A task with no priority does
/// not read these — it multiplies by 100 (see [`reward`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityMultipliers {
    /// The multiplier percent of a `high`-priority task.
    pub high: u64,
    /// The multiplier percent of a `medium`-priority task.
    pub medium: u64,
    /// The multiplier percent of a `low`-priority task.
    pub low: u64,
}

impl PriorityMultipliers {
    /// The multiplier percent of `priority`, e.g. 150 for `high`.
    pub fn pct(&self, priority: Priority) -> u64 {
        match priority {
            Priority::High => self.high,
            Priority::Medium => self.medium,
            Priority::Low => self.low,
        }
    }
}

/// The kind multipliers (§15): integer percents, 100 = ×1, keyed by
/// the [`TaskKind`] wire spellings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindMultipliers {
    /// The multiplier percent of a `task` node.
    pub task: u64,
    /// The multiplier percent of a `gate` node.
    pub gate: u64,
    /// The multiplier percent of a `finding` node.
    pub finding: u64,
    /// The multiplier percent of a `debt` node.
    pub debt: u64,
}

impl KindMultipliers {
    /// The multiplier percent of `kind`, e.g. 50 for `finding`.
    pub fn pct(&self, kind: TaskKind) -> u64 {
        match kind {
            TaskKind::Task => self.task,
            TaskKind::Gate => self.gate,
            TaskKind::Finding => self.finding,
            TaskKind::Debt => self.debt,
        }
    }
}

/// The rewards block of the per-root config (§15, D22): the numbers
/// and labels [`reward`] computes over. The store persists this as
/// part of `<root>/config.json` (U61); an absent file or
/// `enabled: false` leaves the root inert — a gate the caller reads,
/// not the computation. Field order follows §15's enumeration of the
/// block and is the canonical serialization order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardsConfig {
    /// Whether the root's rewards are live (§15). The gate is read
    /// above this crate: [`reward`] computes regardless, and the
    /// caller decides whether to ask.
    pub enabled: bool,
    /// The free-text currency label (§15), displayed verbatim in
    /// reward lines and totals. Deliberately unvalidated: §15 pins
    /// no constraint on the label — it is display text, and an empty
    /// label violates no invariant this crate owns.
    pub currency: String,
    /// The per-bucket base costs.
    pub base_costs: BaseCosts,
    /// The bucket an absent actual size falls to at closure (§15's
    /// default bucket).
    pub default_size: Size,
    /// The priority multipliers, integer percents (100 = ×1).
    pub priority_multipliers: PriorityMultipliers,
    /// The kind multipliers, integer percents (100 = ×1).
    pub kind_multipliers: KindMultipliers,
    /// The accumulated-currency threshold the reflection trigger
    /// watches (§15). `0` is the never-fires spelling — a total that
    /// starts at zero never crosses a zero threshold from below —
    /// chosen over an `Option` so the wire keeps one spelling and
    /// never `null`.
    pub reflect_every: u64,
}

/// The closure-side inputs of [`reward`]: the status being closed
/// under, the task's kind and priority, and the actual size reported
/// at the closure call — §15's second size bucket. A computation
/// argument, not a persisted schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardInput {
    /// The status the task is being closed under.
    pub status: Status,
    /// The closing task's kind.
    pub kind: TaskKind,
    /// The closing task's priority, if it carries one.
    pub priority: Option<Priority>,
    /// The actual size reported at the closure call, if any.
    pub size: Option<Size>,
}

/// A granted reward: the amount plus the inputs the arithmetic
/// actually applied — the size after the default-bucket fallback and
/// the two percents. §15's reward event records exactly these ("the
/// task, the amount, the size and multipliers applied"), so the
/// store's event takes its fields from here rather than re-deriving
/// the fallback rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reward {
    /// The granted amount, in currency units.
    pub amount: u64,
    /// The size whose base cost was applied — the reported actual, or
    /// the config's default bucket when the closure reported none.
    pub size: Size,
    /// The priority percent applied (100 when the task carries no
    /// priority).
    pub priority_pct: u64,
    /// The kind percent applied.
    pub kind_pct: u64,
}

/// The arithmetic's fixed denominator (§15): the priority and kind
/// factors are integer percents (100 = ×1), so their product scales
/// the base by 1/10000 — percent squared.
const PERCENT_SQUARED: u64 = 10_000;

/// The percent a task with no priority multiplies by (§15): 100 — an
/// absent priority is neutral, not zero.
const NO_PRIORITY_PCT: u64 = 100;

/// `numerator / PERCENT_SQUARED` rounded half up, all-integer: a
/// remainder of at least half the denominator bumps the quotient. The
/// bump is checked, so no input panics; with the fixed denominator
/// the quotient is bounded far below `u64::MAX`, keeping the `None`
/// arm unreachable from [`reward`].
fn round_half_up(numerator: u64) -> Option<u64> {
    let quotient = numerator / PERCENT_SQUARED;
    let remainder = numerator % PERCENT_SQUARED;
    if remainder >= PERCENT_SQUARED / 2 {
        quotient.checked_add(1)
    } else {
        Some(quotient)
    }
}

/// The reward a closure grants (§15, D22): the pure, all-integer
/// computation
///
/// `round_half_up(base[size] × priority_pct × kind_pct / 10000)`
///
/// over the config's numbers and the closing task's fields — the
/// agent never picks its own amount. The rules:
///
/// - Only [`Status::Closed`] grants. `wontfix` grants nothing —
///   declined work is not rewarded work (§15) — and because the
///   computation answers a *closure*, every other status grants
///   nothing too: the function is total, never an error.
/// - An absent `size` falls to the config's `default_size` bucket
///   (§15: tasks created before E13 close against the default).
/// - An absent `priority` multiplies by 100 — no priority is neutral,
///   not zero.
/// - The arithmetic is checked throughout (§8: no floats): a config
///   whose product would overflow `u64` grants nothing (`None`)
///   rather than panicking.
/// - A zero base cost grants a zero amount, not `None` — only the
///   status rule and overflow return `None`.
///
/// The config's `enabled` gate is deliberately not read here: an
/// absent config or `enabled: false` leaves the root inert (§15), and
/// that policy call belongs to the caller — the store and server
/// layers decide whether to compute at all.
pub fn reward(config: &RewardsConfig, input: RewardInput) -> Option<Reward> {
    if input.status != Status::Closed {
        return None;
    }
    let size = input.size.unwrap_or(config.default_size);
    let priority_pct = input
        .priority
        .map_or(NO_PRIORITY_PCT, |p| config.priority_multipliers.pct(p));
    let kind_pct = config.kind_multipliers.pct(input.kind);
    let product = config
        .base_costs
        .cost(size)
        .checked_mul(priority_pct)?
        .checked_mul(kind_pct)?;
    let amount = round_half_up(product)?;
    Some(Reward {
        amount,
        size,
        priority_pct,
        kind_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testcodec::{from_bytes, to_bytes};

    fn config() -> RewardsConfig {
        RewardsConfig {
            enabled: true,
            currency: "coffee".to_owned(),
            base_costs: BaseCosts {
                s: 10,
                m: 25,
                l: 50,
                xl: 100,
            },
            default_size: Size::M,
            priority_multipliers: PriorityMultipliers {
                high: 150,
                medium: 100,
                low: 50,
            },
            kind_multipliers: KindMultipliers {
                task: 100,
                gate: 100,
                finding: 50,
                debt: 25,
            },
            reflect_every: 500,
        }
    }

    /// A config whose S base, `high` priority and `task` kind carry
    /// the table's numbers; the arithmetic rows read them through
    /// `closing(Some(High), Some(S))`.
    fn arithmetic_config(base: u64, priority_pct: u64, kind_pct: u64) -> RewardsConfig {
        let mut cfg = config();
        cfg.base_costs.s = base;
        cfg.priority_multipliers.high = priority_pct;
        cfg.kind_multipliers.task = kind_pct;
        cfg
    }

    fn closing(priority: Option<Priority>, size: Option<Size>) -> RewardInput {
        RewardInput {
            status: Status::Closed,
            kind: TaskKind::Task,
            priority,
            size,
        }
    }

    #[test]
    fn size_wire_spellings() {
        assert_eq!(Size::S.as_str(), "s");
        assert_eq!(Size::M.as_str(), "m");
        assert_eq!(Size::L.as_str(), "l");
        assert_eq!(Size::Xl.as_str(), "xl");
        assert_eq!(Size::Xl.to_string(), "xl");
        assert_eq!(to_bytes(&Size::S).unwrap(), b"1:s");
        assert_eq!(to_bytes(&Size::M).unwrap(), b"1:m");
        assert_eq!(to_bytes(&Size::L).unwrap(), b"1:l");
        assert_eq!(to_bytes(&Size::Xl).unwrap(), b"2:xl");
        for size in [Size::S, Size::M, Size::L, Size::Xl] {
            assert_eq!(from_bytes::<Size>(&to_bytes(&size).unwrap()).unwrap(), size);
        }
    }

    #[test]
    fn config_structs_pin_declaration_order() {
        // The u64-bearing sub-structs round-trip under the crate's
        // test codec, pinning field names and declaration order (§8);
        // the bool-bearing block itself cannot — the codec serializes
        // no bool — so its bytes are the store's golden story (U61).
        let costs = BaseCosts {
            s: 10,
            m: 25,
            l: 50,
            xl: 100,
        };
        let bytes = to_bytes(&costs).unwrap();
        assert_eq!(bytes, b"d1:si10e1:mi25e1:li50e2:xli100ee");
        assert_eq!(from_bytes::<BaseCosts>(&bytes).unwrap(), costs);

        let priority = PriorityMultipliers {
            high: 150,
            medium: 100,
            low: 50,
        };
        let bytes = to_bytes(&priority).unwrap();
        assert_eq!(bytes, b"d4:highi150e6:mediumi100e3:lowi50ee");
        assert_eq!(from_bytes::<PriorityMultipliers>(&bytes).unwrap(), priority);

        let kind = KindMultipliers {
            task: 100,
            gate: 100,
            finding: 50,
            debt: 25,
        };
        let bytes = to_bytes(&kind).unwrap();
        assert_eq!(bytes, b"d4:taski100e4:gatei100e7:findingi50e4:debti25ee");
        assert_eq!(from_bytes::<KindMultipliers>(&bytes).unwrap(), kind);
    }

    #[test]
    fn arithmetic_boundaries_across_both_percent_axes() {
        // (base, priority_pct, kind_pct, expected amount). The rows
        // pin the half-up boundaries — exact quotients, the .5 cases
        // rounding up, just-below-half rounding down, tiny and zero
        // bases — across both percent axes.
        for (base, pct, kind_pct, expected) in [
            (10u64, 100u64, 100u64, 10u64), // exact: 10 × 1 × 1
            (1, 50, 100, 1),                // 0.5 rounds up
            (1, 49, 100, 0),                // 0.49 rounds down
            (1, 100, 50, 1),                // 0.5 on the kind axis
            (1, 100, 49, 0),                // 0.49 on the kind axis
            (3, 50, 100, 2),                // 1.5 rounds up
            (1, 150, 100, 2),               // 1.5, non-unit percent
            (10, 100, 25, 3),               // 2.5 rounds up
            (2, 25, 100, 1),                // 0.5, non-unit base
            (1, 99, 100, 1),                // 0.99 rounds up
            (1, 1, 1, 0),                   // 0.0001 — tiny
            (0, 100, 100, 0),               // zero base grants zero, not None
            (1, 9999, 100, 100),            // 99.99 rounds up to 100
            (1, 10001, 100, 100),           // 100.01 stays 100
        ] {
            let got = reward(
                &arithmetic_config(base, pct, kind_pct),
                closing(Some(Priority::High), Some(Size::S)),
            );
            assert_eq!(
                got.map(|r| r.amount),
                Some(expected),
                "base {base} × priority {pct} × kind {kind_pct}"
            );
        }
    }

    #[test]
    fn absent_size_falls_to_the_default_bucket() {
        let cfg = config();
        let fell = reward(&cfg, closing(None, None));
        assert_eq!(fell.map(|r| (r.amount, r.size)), Some((25, Size::M)));
        let reported = reward(&cfg, closing(None, Some(Size::Xl)));
        assert_eq!(reported.map(|r| (r.amount, r.size)), Some((100, Size::Xl)));
    }

    #[test]
    fn no_priority_multiplies_by_100() {
        let cfg = config();
        let none = reward(&cfg, closing(None, Some(Size::S)));
        assert_eq!(none.map(|r| (r.amount, r.priority_pct)), Some((10, 100)));
        let high = reward(&cfg, closing(Some(Priority::High), Some(Size::S)));
        assert_eq!(high.map(|r| (r.amount, r.priority_pct)), Some((15, 150)));
        let low = reward(&cfg, closing(Some(Priority::Low), Some(Size::S)));
        assert_eq!(low.map(|r| (r.amount, r.priority_pct)), Some((5, 50)));
    }

    #[test]
    fn wontfix_and_non_closure_statuses_grant_nothing() {
        let cfg = config();
        for status in [
            Status::Wontfix,
            Status::Open,
            Status::InWork,
            Status::Blocked,
            Status::Deferred,
        ] {
            let input = RewardInput {
                status,
                ..closing(None, None)
            };
            assert_eq!(reward(&cfg, input), None, "{status} grants nothing");
        }
        assert!(reward(&cfg, closing(None, None)).is_some());
    }

    #[test]
    fn overflow_grants_nothing_rather_than_panicking() {
        // The largest clean product: base = u64::MAX/10000 with both
        // percents at 100 divides exactly.
        let edge = arithmetic_config(u64::MAX / PERCENT_SQUARED, 100, 100);
        let got = reward(&edge, closing(Some(Priority::High), Some(Size::S)));
        assert_eq!(got.map(|r| r.amount), Some(u64::MAX / PERCENT_SQUARED));
        // One base unit more and the product no longer fits u64.
        let over = arithmetic_config(u64::MAX / PERCENT_SQUARED + 1, 100, 100);
        assert_eq!(
            reward(&over, closing(Some(Priority::High), Some(Size::S))),
            None
        );
        // The overflow can land on either multiplication.
        let first = arithmetic_config(u64::MAX, 2, 100);
        assert_eq!(
            reward(&first, closing(Some(Priority::High), Some(Size::S))),
            None
        );
        let second = arithmetic_config(1 << 32, 2, 1 << 32);
        assert_eq!(
            reward(&second, closing(Some(Priority::High), Some(Size::S))),
            None
        );
    }

    #[test]
    fn the_enabled_gate_is_the_callers_not_the_computations() {
        // §15's opt-in is read above this crate; the pure function
        // computes over whatever numbers it is handed, so a disabled
        // config still answers — the caller decides whether to ask.
        let mut cfg = config();
        cfg.enabled = false;
        assert!(reward(&cfg, closing(None, None)).is_some());
    }

    #[test]
    fn the_grid_matches_the_closed_form() {
        // sizes (absent included) × priorities (absent included) ×
        // kinds, against §15's formula spelled literally —
        // (base × p × k + 5000) / 10000, the +5000 form of half-up —
        // with the applied fields asserted beside the amount. The
        // grid never nears overflow, so the plain form is exact.
        let cfg = config();
        for size in [
            Some(Size::S),
            Some(Size::M),
            Some(Size::L),
            Some(Size::Xl),
            None,
        ] {
            for priority in [
                None,
                Some(Priority::High),
                Some(Priority::Medium),
                Some(Priority::Low),
            ] {
                for kind in [
                    TaskKind::Task,
                    TaskKind::Gate,
                    TaskKind::Finding,
                    TaskKind::Debt,
                ] {
                    let input = RewardInput {
                        kind,
                        priority,
                        size,
                        ..closing(None, None)
                    };
                    let size_used = size.unwrap_or(cfg.default_size);
                    let base = cfg.base_costs.cost(size_used);
                    let p = priority.map_or(NO_PRIORITY_PCT, |pr| cfg.priority_multipliers.pct(pr));
                    let k = cfg.kind_multipliers.pct(kind);
                    let expected = (base * p * k + PERCENT_SQUARED / 2) / PERCENT_SQUARED;
                    assert_eq!(
                        reward(&cfg, input),
                        Some(Reward {
                            amount: expected,
                            size: size_used,
                            priority_pct: p,
                            kind_pct: k,
                        }),
                        "size {size:?} priority {priority:?} kind {kind}"
                    );
                }
            }
        }
    }
}
