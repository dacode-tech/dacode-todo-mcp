//! The rewards surface (§15, D24 — the surface half of the rewards
//! arc): the closure response's reward line and reflection question,
//! the `reward_reflect` tool, and the milestone-ness detection the
//! milestone trigger and the render share (moved here from
//! `render/mod.rs` at U62 — the heuristic's one home).
//!
//! The cluster follows the nodes cluster's shape: params and the
//! answer helper as plain functions, the tool on the router below,
//! every failure through the one error-mapping layer (lib.rs's
//! [`ToolError`]). The reward AMOUNT is never chosen here — the
//! domain computes it, the store grants it, and this module only
//! renders what the store's `TaskClosure` reports (§15: "the agent
//! never picks its own amount").
//!
//! # The triggers (§15)
//!
//! Two, both detected on the closure transition alone — the store's
//! own before-check mirrored, so a patch re-closing an already-closed
//! task asks nothing a second time. The **milestone** trigger: the
//! closure that leaves its epic's every task terminal, under an epic
//! whose provenance description names a board milestone
//! ([`milestone_of`]). The **threshold** trigger: the root total
//! crossing `reflect_every` from below — strictly below before this
//! grant, at or above after. The pre/post comparison is the semantics
//! (the E13 handover's carried note): comparing only the post-grant
//! total would re-ask on every later closure above the threshold.
//! When both fire on one closure the milestone wins — the completion
//! moment subsumes the accumulation one, and the response carries one
//! question.
//!
//! # The enabled gate
//!
//! §15's fence: a root with no config — or `enabled: false` — is
//! byte-identical on every response. The gate is read here, the
//! caller's per the store's contract (the store logs a reflection
//! verbatim whenever asked; the server decides whether the root is
//! enabled), and the config is only consulted on a closing patch —
//! the one patch that can move a task into `closed` — so every other
//! update's bytes cannot move at all.
//!
//! # The answer's label
//!
//! `reward_reflect` takes the trigger as a parameter — the spelling
//! the question itself teaches — because the label is the experiment's
//! data (which kind of moment the answer answers) and no stateless
//! derivation can recover it: the current totals cannot distinguish a
//! milestone answer from a threshold one once the total sits above
//! the threshold, and remembered per-process state would not survive
//! the daemon's restart. The answer text itself is never interpreted
//! (§15) — only its label is echoed.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use todo_domain::{Epic, EpicId, Graph, Priority, Reward, Size, Status};
use todo_store::ReflectionTrigger;

use crate::{ServerState, TodoServer, ToolError, error_carrier, repo_of, structured, time};

// =================================================================
// The milestone-ness detection — the render's heuristic, moved
// =================================================================

/// The board's M id for a milestone epic, recovered from the mapping's
/// provenance description — the one contract every dialect writes:
/// `the <board> board's §<n> milestone <M-id>` (own, camera and ikvm
/// §2; dacode §5, where the board's milestones live). The synthetic
/// containers (`… §6 open follow-ups, not tied to a milestone`) do not
/// match: the section part must be digits and the milestone word must
/// be followed by exactly one token.
pub(crate) fn milestone_of(epic: &Epic) -> Option<String> {
    let (head, tail) = epic.description.split_once(" milestone ")?;
    if !head.starts_with("the ") || !head.contains(" board's §") {
        return None;
    }
    let section = head.rsplit(" board's §").next().unwrap_or_default();
    if section.is_empty() || !section.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut tokens = tail.split_whitespace();
    let id = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }
    Some(id.to_owned())
}

/// Whether the epic's every task is terminal (§15's milestone
/// completion): `closed` or `wontfix`. The milestone trigger watches
/// this over the graph as the closure left it — the epic's own status
/// field is the user's bookkeeping, not the work's fact.
pub(crate) fn epic_complete(graph: &Graph, epic: &EpicId) -> bool {
    graph
        .tasks_of_epic(epic)
        .iter()
        .all(|task| matches!(task.status, Status::Closed | Status::Wontfix))
}

// =================================================================
// The two lines the closure response gains
// =================================================================

/// The closure response's reward line (§15): the exact shape the spec
/// spells — `reward: 25 coffee (M × high). root total: 1234 coffee.`
/// — the currency from the config, the size and the priority shown,
/// the root total derived. The size is spelled uppercase (the spec's
/// example) and the priority by its wire spelling, `none` when the
/// task carries none (the neutral ×100 the arithmetic applied).
pub(crate) fn reward_line(
    granted: &Reward,
    priority: Option<Priority>,
    currency: &str,
    root_total: u64,
) -> String {
    format!(
        "reward: {} {} ({} × {}). root total: {} {}.",
        granted.amount,
        currency,
        size_label(granted.size),
        priority.map(Priority::as_str).unwrap_or("none"),
        root_total,
        currency
    )
}

/// The size's label in the reward line's multiplier pair: the spec's
/// example spells the buckets uppercase — `S`, `M`, `L`, `XL`.
fn size_label(size: Size) -> &'static str {
    match size {
        Size::S => "S",
        Size::M => "M",
        Size::L => "L",
        Size::Xl => "XL",
    }
}

/// The milestone trigger's question (§15): names the epic and the
/// trigger spelling the answer echoes through `reward_reflect`.
pub(crate) fn milestone_question(epic: &EpicId) -> String {
    format!(
        "The milestone epic {epic} closed. How does the reward feel? Answer honestly, once, \
         through reward_reflect with trigger {{\"trigger\":\"epic_closure\",\"epic\":\"{epic}\"}}."
    )
}

/// The threshold trigger's question (§15): names the threshold
/// crossed, the total now, and the trigger spelling the answer echoes
/// through `reward_reflect`.
pub(crate) fn threshold_question(reflect_every: u64, root_total: u64, currency: &str) -> String {
    format!(
        "The root's reward total crossed {reflect_every} {currency} (now {root_total} \
         {currency}). How does the reward feel? Answer honestly, once, through reward_reflect \
         with trigger {{\"trigger\":\"threshold\"}}."
    )
}

// =================================================================
// reward_reflect — the params, the payload, the answer
// =================================================================

/// The wire mirror of the reflection triggers (§15, D23): the shape the log's `reward_reflect` event spells — a milestone epic's closure or the threshold crossing — so the answer's payload echoes the event and the question can teach the spelling the answer passes back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "trigger", rename_all = "snake_case")]
pub(crate) enum ReflectionTriggerDto {
    /// A milestone epic's closure asked.
    EpicClosure {
        /// The epic whose closure asked, e.g. `camera:E1`.
        epic: String,
    },
    /// The root's accumulated total crossed reflect_every.
    Threshold,
}

/// `reward_reflect`'s parameters (§6, D24): the repo, the verbatim answer, and the trigger the closure response's question named — the label the logged event carries.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RewardReflectParams {
    /// The repository's stable slug key (§7): ASCII lowercase segments joined by single hyphens; the repo whose graph the reflection answers over; absent means the configured default repo; an explicit value always wins.
    pub repo: Option<String>,
    /// The verbatim answer to the reflection question, as given — never interpreted, logged as-is (§15).
    pub answer: String,
    /// The trigger that asked — the spelling the question carried.
    pub trigger: ReflectionTriggerDto,
}

/// `reward_reflect`'s success payload (§15, D24): the reflection as it was logged — the verbatim answer, the trigger that asked, and the derived totals at the moment the answer landed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub(crate) struct ReflectionDto {
    /// The verbatim answer, as given.
    pub answer: String,
    /// Why the question was asked: a milestone epic's closure, or the root total crossing reflect_every.
    pub trigger: ReflectionTriggerDto,
    /// The root's accumulated total at the moment the answer landed.
    pub root_total: u64,
    /// This repo's accumulated total at the moment the answer landed.
    pub repo_total: u64,
}

/// Parses one id parameter: a malformed value is §6.1's `invalid`
/// before the store is touched — this cluster's own copy of the
/// per-cluster parse the nodes cluster documents (a shared surface
/// change would be worse than the copy).
fn parse_value<T>(value: &str) -> Result<T, ToolError>
where
    T: FromStr<Err = todo_domain::ValueError>,
{
    T::from_str(value).map_err(ToolError::from)
}

/// `reward_reflect`'s answer: parse, resolve, gate, derive, log. The
/// totals are derived at the answer's moment — the caller-derived
/// numbers the store records (§15) — and the answer is logged
/// verbatim through the store's `log_reflection`.
fn answer_reward_reflect(
    state: &mut ServerState<'_>,
    params: &RewardReflectParams,
) -> Result<ReflectionDto, ToolError> {
    let slug = repo_of(state, params.repo.as_deref())?;
    // The parse-before-resolve convention: a malformed epic id is
    // `invalid` before the store is touched.
    let trigger = match &params.trigger {
        ReflectionTriggerDto::EpicClosure { epic } => ReflectionTrigger::EpicClosure {
            epic: parse_value::<EpicId>(epic)?,
        },
        ReflectionTriggerDto::Threshold => ReflectionTrigger::Threshold,
    };
    let root = state.core.root.clone();
    let handle = state.resolve(&slug)?;
    // The enabled gate is the caller's (§15): the question is asked
    // only on a rewards-enabled root, so an answer anywhere else is
    // refused — the store would log it verbatim, but nothing asked.
    let enabled = root.load_config()?.is_some_and(|config| config.enabled);
    if !enabled {
        return Err(ToolError::invalid(
            "the root's rewards are not enabled — no reflection was asked",
        ));
    }
    let totals = root.rewards_totals()?;
    let repo_total = totals.by_repo.get(&slug).copied().unwrap_or(0);
    handle.log_reflection(params.answer.clone(), trigger, totals.clone(), time::now())?;
    Ok(ReflectionDto {
        answer: params.answer.clone(),
        trigger: params.trigger.clone(),
        root_total: totals.total,
        repo_total,
    })
}

// =================================================================
// The tool
// =================================================================

#[tool_router(router = rewards_router, vis = "pub(crate)")]
impl TodoServer {
    /// Log the closing agent's reflection (§15, D24): the verbatim answer to a closure response's reflection question, logged as an event with the trigger that asked and the derived totals at the answer's moment — displayed as-is, never interpreted.
    #[tool(
        name = "reward_reflect",
        output_schema = crate::schema::output_for_type::<ReflectionDto>(),
        annotations(open_world_hint = false)
    )]
    async fn reward_reflect(
        &self,
        Parameters(params): Parameters<RewardReflectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // §8's serialization point: the guard is dropped at the end of
        // this sync body — no `.await` follows it.
        let mut state = self.lock_state();
        match answer_reward_reflect(&mut state, &params) {
            Ok(output) => structured("reward_reflect", output),
            Err(error) => Ok(error_carrier(&error.code, &error.message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use todo_domain::References;

    /// An epic with `description`, for the matcher tests.
    fn epic_with(description: &str) -> Epic {
        Epic {
            id: "fixture:E1".parse().unwrap(),
            repo: "fixture".parse().unwrap(),
            title: "an epic".to_owned(),
            description: description.to_owned(),
            status: Status::Open,
            references: References::new(),
        }
    }

    /// All four dialects' provenance prefixes recover their M id —
    /// own, camera and ikvm write §2, dacode §5.
    #[test]
    fn every_dialects_milestone_prefix_recovers_the_id() {
        for (description, id) in [
            ("the dacode-todo-mcp board's §2 milestone M0", "M0"),
            ("the camera board's §2 milestone M3", "M3"),
            ("the ikvm-html board's §2 milestone M1", "M1"),
            ("the dacode board's §5 milestone M6", "M6"),
        ] {
            assert_eq!(milestone_of(&epic_with(description)), Some(id.to_owned()));
        }
    }

    /// The synthetic containers and prose descriptions do not match:
    /// the follow-ups container ends at "milestone" with no trailing
    /// id, a non-digit section fails, and a second token after the id
    /// fails.
    #[test]
    fn synthetic_containers_and_prose_do_not_match() {
        for description in [
            "the fixture board's §6 open follow-ups, not tied to a milestone",
            "the fixture board's §two milestone M0",
            "the fixture board's §2 milestone M0 and more",
            "The sensor bring-up epic.",
            "a milestone word in prose",
        ] {
            assert_eq!(milestone_of(&epic_with(description)), None);
        }
    }

    /// The reward line's exact shape (§15's example): the amount, the
    /// currency, the size uppercase, the priority by name — `none`
    /// when the task carries none — and the derived root total.
    #[test]
    fn the_reward_line_matches_the_specs_shape() {
        let granted = Reward {
            amount: 25,
            size: Size::M,
            priority_pct: 150,
            kind_pct: 100,
        };
        assert_eq!(
            reward_line(&granted, Some(Priority::High), "coffee", 1234),
            "reward: 25 coffee (M × high). root total: 1234 coffee."
        );
        assert_eq!(
            reward_line(&granted, None, "coffee", 25),
            "reward: 25 coffee (M × none). root total: 25 coffee."
        );
        let xl = Reward {
            amount: 100,
            size: Size::Xl,
            priority_pct: 100,
            kind_pct: 100,
        };
        assert_eq!(
            reward_line(&xl, Some(Priority::Low), "tea", 100),
            "reward: 100 tea (XL × low). root total: 100 tea."
        );
    }

    /// The questions name their trigger's exact echo spelling — the
    /// contract the answer's label depends on.
    #[test]
    fn the_questions_teach_the_trigger_spelling() {
        assert_eq!(
            milestone_question(&"fixture:E1".parse().unwrap()),
            "The milestone epic fixture:E1 closed. How does the reward feel? Answer honestly, \
             once, through reward_reflect with trigger \
             {\"trigger\":\"epic_closure\",\"epic\":\"fixture:E1\"}."
        );
        assert_eq!(
            threshold_question(500, 515, "coffee"),
            "The root's reward total crossed 500 coffee (now 515 coffee). How does the reward \
             feel? Answer honestly, once, through reward_reflect with trigger \
             {\"trigger\":\"threshold\"}."
        );
    }
}
