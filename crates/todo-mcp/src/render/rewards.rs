//! The rewards section (§15's display half, U63): the model slice the
//! progress surfaces gain when the root's config enables rewards —
//! the per-root and per-repo totals, the recent grants, the
//! reflections verbatim, and the planned/actual size columns' data.
//! Its own module because both `mod.rs` and `page.rs` ran over the
//! 600-line target with recorded debt when the arc landed (U57 and
//! U58 have since paid them): the arc's display code lands here
//! rather than growing them, and the shared files gain only the
//! joins — the gather call, the model field, the row's size cells,
//! the section's insertion, the JSON twin's block.
//!
//! §15's fence is the module's first rule: a root with no config — or
//! `enabled: false` — answers [`gather_rewards`] with `None`, and
//! every consumer renders exactly today's bytes (no section, no
//! columns, no CSS). A **malformed** config fails the render with its
//! named error instead — the config module's own discipline, that
//! damage is never a silent inert root.
//!
//! The amounts and the answers are rendered **as recorded** (§15: the
//! server never interprets an answer): the store's events read is the
//! single source, and this module formats without recomputing — the
//! one derivation it owns is the actuals map (each task's latest
//! grant's size), which is a lookup over the recorded events, not an
//! arithmetic. The currency label is the config's — the display's
//! (§15) — so it is the one config value the section shows.
//!
//! Determinism (§8): the section is a pure function of the model —
//! the store's root-order events, the `BTreeMap`-keyed totals, the
//! named [`RECENT_GRANTS`] bound — so two renders over unchanged
//! state are byte-identical, and the enabled goldens pin the bytes.

use std::collections::BTreeMap;

use serde::Serialize;
use todo_domain::{RewardsConfig, Size, Task, TaskId};
use todo_store::{DataRoot, GrantedReward, RewardReflection, RewardsEvents, RewardsTotals};

use crate::ToolError;
use crate::rewards::size_label;

use super::esc;

/// How many grants the "recent rewards" list shows: the bounded
/// suffix of the root's grant sequence (root order — the registry's
/// ascending-slug order across repos, log order within one), newest
/// last, the ledger reading. A named constant (§8: the bound is part
/// of the pinned bytes, never a hidden default); the enabled goldens
/// and the unit test below pin it.
pub(crate) const RECENT_GRANTS: usize = 10;

/// The rewards model slice (§15's display half): everything the
/// enabled surfaces render, gathered once per render and shared by
/// every surface so they can never disagree. The grants and
/// reflections are the store's read verbatim; the actuals map is the
/// display's one derivation — each task's **latest** grant's size (a
/// task closed, reopened and re-closed keeps its latest actual), the
/// honesty anchor's right-hand column.
pub(crate) struct RewardsModel {
    /// The config's currency label — the display's (§15).
    pub(crate) currency: String,
    /// The derived per-root and per-repo totals.
    pub(crate) totals: RewardsTotals,
    /// Every grant, root order — the recent list is the bounded
    /// suffix.
    pub(crate) grants: Vec<GrantedReward>,
    /// Every reflection, root order — displayed verbatim and
    /// unbounded: they are the experiment's data, and rare by
    /// construction (one per milestone closure, one per threshold
    /// crossing).
    pub(crate) reflections: Vec<RewardReflection>,
    /// Each task's actual size: its latest grant's size, keyed by
    /// task id — the ids embed the repo, so one root-wide map cannot
    /// collide across repos.
    pub(crate) actuals: BTreeMap<TaskId, Size>,
}

impl RewardsModel {
    /// The recent grants: the bounded suffix of the root-order
    /// sequence, still in root order (newest last).
    pub(crate) fn recent_grants(&self) -> &[GrantedReward] {
        let skip = self.grants.len().saturating_sub(RECENT_GRANTS);
        self.grants.get(skip..).unwrap_or(&[])
    }
}

/// Gathers the rewards model slice: the config read first — absent or
/// `enabled: false` answers `None`, §15's fence — then, when enabled,
/// the totals and the events, both lock-free (D19). A malformed
/// config fails with its named error, never a silent inert root. One
/// call per render, inside [`super::model::gather`], so every surface shares
/// one answer.
pub(crate) fn gather_rewards(root: &DataRoot) -> Result<Option<RewardsModel>, String> {
    let config = root
        .load_config()
        .map_err(|error| ToolError::from(error).message)?;
    let Some(config) = config.filter(|config: &RewardsConfig| config.enabled) else {
        return Ok(None);
    };
    let totals = root
        .rewards_totals()
        .map_err(|error| ToolError::from(error).message)?;
    let events = root
        .rewards_events()
        .map_err(|error| ToolError::from(error).message)?;
    Ok(Some(model_of(config, totals, events)))
}

/// The model from the config and the two reads: the actuals map folds
/// the grants in root order, so a later grant for the same task wins
/// — the latest actual is the honest one.
fn model_of(config: RewardsConfig, totals: RewardsTotals, events: RewardsEvents) -> RewardsModel {
    let mut actuals = BTreeMap::new();
    for grant in &events.grants {
        actuals.insert(grant.task.clone(), grant.size);
    }
    RewardsModel {
        currency: config.currency,
        totals,
        grants: events.grants,
        reflections: events.reflections,
        actuals,
    }
}

/// The est/actual table cells (§15's honesty anchor): the planned
/// size from the task's `estimate`, the actual from the model's
/// latest-grant map, `-` for either absent — and nothing at all when
/// the root's rewards are disabled (the fence: no columns, no bytes).
/// The label is the reward line's uppercase spelling, so the columns
/// and the closure response cannot drift apart.
pub(crate) fn size_cells(rewards: Option<&RewardsModel>, task: &Task) -> String {
    let Some(rewards) = rewards else {
        return String::new();
    };
    let estimate = task.estimate.map(size_label).unwrap_or("-");
    let actual = rewards
        .actuals
        .get(&task.id)
        .map(|size| size_label(*size))
        .unwrap_or("-");
    format!("<td class=\"mono\">{estimate}</td><td class=\"mono\">{actual}</td>")
}

/// The rewards section's HTML (§15): its own `<h2>` after the
/// activity panel — the root total in the header, the activity
/// header's shape — then the per-repo totals line, the recent grants
/// table and the reflections, verbatim. Each sub-block renders only
/// when it holds something (an empty enabled root shows the header
/// and the totals line, never an empty table or a dangling label).
pub(crate) fn section_html(model: &RewardsModel) -> String {
    let currency = esc(&model.currency);
    let mut out = format!(
        "\n<h2>Rewards — {} {} total</h2>\n<div class=\"rwd\">\n",
        model.totals.total, currency
    );
    let repos: Vec<String> = model
        .totals
        .by_repo
        .iter()
        .map(|(slug, total)| format!("{} <b>{total}</b>", esc(slug.as_str())))
        .collect();
    out.push_str(&format!(
        "<div class=\"rtot\">per-repo totals: {}</div>\n",
        repos.join(" · ")
    ));
    let recent = model.recent_grants();
    if !recent.is_empty() {
        out.push_str(&format!(
            "<p class=\"legend\"><b>Recent rewards</b> — the last {} grant{}, root order</p>\n",
            recent.len(),
            if recent.len() == 1 { "" } else { "s" }
        ));
        out.push_str(&format!(
            "<table><thead><tr><th>task</th><th>reward</th><th>size</th><th>×prio</th>\
             <th>×kind</th><th>when</th></tr></thead>\n<tbody>{}</tbody></table>\n",
            grant_rows(model, recent)
        ));
    }
    if !model.reflections.is_empty() {
        out.push_str("<p class=\"legend\"><b>Reflections</b> — the answers, verbatim</p>\n");
        out.push_str(&reflection_html(model));
    }
    out.push_str("</div>\n");
    out
}

/// The recent grants table's rows: the full task id (it names the
/// repo — the list is root-scoped, where short ids would collide),
/// the amount with the currency, the size and the two multiplier
/// percents as recorded, and the audit instant.
fn grant_rows(model: &RewardsModel, grants: &[GrantedReward]) -> String {
    grants
        .iter()
        .map(|grant| {
            format!(
                "<tr><td class=\"mono\">{}</td><td>{} {}</td><td class=\"mono\">{}</td>\
                 <td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td></tr>",
                esc(&grant.task.to_string()),
                grant.amount,
                esc(&model.currency),
                size_label(grant.size),
                grant.priority_pct,
                grant.kind_pct,
                esc(grant.ts.as_str())
            )
        })
        .collect()
}

/// The reflections, one block each: the audit instant, the trigger as
/// the log spells it, the totals at the moment (the currency label
/// appended — the display's, §15), and the answer verbatim beneath.
fn reflection_html(model: &RewardsModel) -> String {
    model
        .reflections
        .iter()
        .map(|reflection| {
            format!(
                "<div class=\"refl\"><div class=\"rmeta\"><span class=\"mono\">{}</span> · {} \
                 · root <b>{} {}</b> · {} <b>{}</b></div><p>{}</p></div>\n",
                esc(reflection.ts.as_str()),
                trigger_label(reflection),
                reflection.root_total,
                esc(&model.currency),
                esc(reflection.repo.as_str()),
                reflection.repo_total,
                esc(&reflection.answer)
            )
        })
        .collect()
}

/// One reflection's trigger, as the log spells it: the wire tag, plus
/// the epic for a milestone closure.
fn trigger_label(reflection: &RewardReflection) -> String {
    match &reflection.trigger {
        todo_store::ReflectionTrigger::EpicClosure { epic } => {
            format!("epic_closure {}", esc(&epic.to_string()))
        }
        todo_store::ReflectionTrigger::Threshold => "threshold".to_owned(),
    }
}

/// The est/actual column heads (§15) — the one home for the shared
/// row shape: the progress page's and the epic page's tables render
/// the same heads beside the same [`size_cells`] order.
pub(crate) const SIZE_HEADS: &str = "<th>est</th><th>actual</th>";

/// The rewards section's stylesheet — the activity panel's box shape.
pub(crate) const REWARDS_CSS: &str = r#".rwd { background: #fff; border: 1px solid #e3e3e0;
      border-radius: 10px; padding: 12px 14px; }
.rtot { font-size: 12.5px; color: #3f444c; margin-bottom: 10px; }
.rtot b { color: #2f6f4f; font-variant-numeric: tabular-nums; }
.refl { border-top: 1px solid #eee; padding: 8px 0; }
.refl .rmeta { font-size: 11.5px; color: #6b7280; margin-bottom: 3px; }
.refl .rmeta b { color: #2f6f4f; font-variant-numeric: tabular-nums; }
.refl p { margin: 0; font-size: 12.5px; }
"#;

/// The JSON twin's rewards block (§15): the store's read verbatim —
/// the totals, the bounded recent grants, the reflections — plus the
/// currency label. The activity twin's pattern (t84's D6): what the
/// section shows, the document carries, in the store types' own
/// declaration order.
#[derive(Serialize)]
pub(super) struct RewardsJson {
    /// The config's currency label.
    pub(super) currency: String,
    /// The derived totals, verbatim.
    pub(super) totals: RewardsTotals,
    /// The recent grants — the same bounded suffix the page's table
    /// shows, root order.
    pub(super) recent_grants: Vec<GrantedReward>,
    /// The reflections, verbatim.
    pub(super) reflections: Vec<RewardReflection>,
}

/// The JSON twin's rewards block from the model.
pub(super) fn json(model: &RewardsModel) -> RewardsJson {
    RewardsJson {
        currency: model.currency.clone(),
        totals: model.totals.clone(),
        recent_grants: model.recent_grants().to_vec(),
        reflections: model.reflections.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;
    use todo_domain::{RepoSlug, Timestamp};

    /// A synthetic grant: task `fixture:U<n>`, amount `n`, the rest
    /// fixed — the bound test's units.
    fn grant(n: u64) -> GrantedReward {
        GrantedReward {
            ts: Timestamp::parse("2026-10-02T08:00:00Z").unwrap(),
            repo: RepoSlug::from_str("fixture").unwrap(),
            task: format!("fixture:U{n}").parse().unwrap(),
            amount: n,
            size: Size::M,
            priority_pct: 100,
            kind_pct: 100,
        }
    }

    /// A model over `n` synthetic grants and nothing else.
    fn model_with_grants(n: u64) -> RewardsModel {
        let grants: Vec<GrantedReward> = (1..=n).map(grant).collect();
        RewardsModel {
            currency: "coffee".to_owned(),
            totals: RewardsTotals::default(),
            grants,
            reflections: Vec::new(),
            actuals: BTreeMap::new(),
        }
    }

    /// The recent list is the bounded suffix: eleven grants show ten,
    /// the oldest dropped, the order unchanged (newest last).
    #[test]
    fn the_recent_list_is_the_bounded_suffix() {
        let model = model_with_grants(RECENT_GRANTS as u64 + 1);
        assert_eq!(model.recent_grants().len(), RECENT_GRANTS);
        assert_eq!(
            model.recent_grants().first().map(|grant| &grant.task),
            Some(&"fixture:U2".parse::<TaskId>().unwrap()),
            "the oldest grant is the one dropped"
        );
        let html = section_html(&model);
        assert_eq!(
            html.matches("<tr><td class=\"mono\">fixture:U").count(),
            RECENT_GRANTS,
            "the table shows exactly the bound's rows"
        );
        assert!(
            !html.contains("<td class=\"mono\">fixture:U1</td>"),
            "the dropped grant renders nowhere"
        );
    }

    /// An empty enabled model renders the header and the totals line
    /// only — never an empty table or a dangling reflections label.
    #[test]
    fn an_empty_model_renders_the_header_and_totals_only() {
        let html = section_html(&model_with_grants(0));
        assert!(html.contains("<h2>Rewards — 0 coffee total</h2>"));
        assert!(html.contains("<div class=\"rtot\">per-repo totals: </div>"));
        assert!(!html.contains("<table>"));
        assert!(!html.contains("Reflections"));
    }

    /// The size cells: the estimate and the actual with `-` for
    /// either absent, and nothing at all when disabled — the fence's
    /// unit pin.
    #[test]
    fn the_size_cells_dash_for_absent_and_nothing_when_disabled() {
        let mut model = model_with_grants(0);
        model.actuals.insert("fixture:U1".parse().unwrap(), Size::S);
        let task = |estimate: Option<Size>| Task {
            id: "fixture:U1".parse().unwrap(),
            repo: "fixture".parse().unwrap(),
            epic: "fixture:E1".parse().unwrap(),
            kind: todo_domain::TaskKind::Task,
            title: "the task".to_owned(),
            body: String::new(),
            status: todo_domain::Status::Open,
            verify: None,
            evidence: None,
            note: None,
            references: todo_domain::References::new(),
            priority: None,
            until: None,
            created: Timestamp::parse("2026-10-02T08:00:00Z").unwrap(),
            updated: Timestamp::parse("2026-10-02T08:00:00Z").unwrap(),
            estimate,
        };
        assert_eq!(
            size_cells(Some(&model), &task(Some(Size::L))),
            "<td class=\"mono\">L</td><td class=\"mono\">S</td>",
            "the estimate and the actual"
        );
        assert_eq!(
            size_cells(Some(&model), &task(None)),
            "<td class=\"mono\">-</td><td class=\"mono\">S</td>",
            "an absent estimate is the dash"
        );
        model.actuals.clear();
        assert_eq!(
            size_cells(Some(&model), &task(None)),
            "<td class=\"mono\">-</td><td class=\"mono\">-</td>",
            "an absent actual is the dash"
        );
        assert_eq!(
            size_cells(None, &task(Some(Size::L))),
            "",
            "a disabled root renders no cells at all"
        );
    }
}
