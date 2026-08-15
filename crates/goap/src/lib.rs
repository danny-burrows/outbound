//! Goal-oriented action planning over a caller-defined domain.
//!
//! A [`Domain`] bundles the three things the planner keeps deliberately apart:
//!
//! - [`Domain::State`]: the *planning* state. One is cloned and hashed per expanded node, so
//!   it wants to be small and ideally `Copy`. It is not the world.
//! - [`Domain::Context`]: read-only world data actions may consult but that the planner never
//!   copies. Item positions, prices, who knows whom.
//! - [`Domain::Action`]: the operators. Cost and prerequisite both see state and context, so
//!   an action's price can reflect where an agent stands and how good it is at the job.
//!
//! Goals are *partial*. A [`Goal`] reports how far off it is rather than describing a finished
//! world, which is what makes [`Goal::estimate`], and therefore an informative A* heuristic,
//! expressible at all.
//!
//! [`plan`] always returns a [`Plan`], carrying every goal it weighed in
//! [`Plan::considered`] whether or not it found anything to do. Failing to plan is
//! information, not absence, and it is exactly when you most want to know why.

#[cfg(test)]
mod tests;

use pathfinding::directed::astar::astar;
use std::collections::VecDeque;

/// Requirements a planning state must meet. Blanket-implemented; the bound exists to
/// keep signatures readable.
pub trait State: Clone + Eq + std::hash::Hash + std::fmt::Debug {}

impl<T: Clone + Eq + std::hash::Hash + std::fmt::Debug> State for T {}

pub trait Domain: Sized {
    type State: State;
    type Context;
    type Action: Action<Self>;
}

pub trait Action<D: Domain>: std::fmt::Debug + Clone + Eq + std::hash::Hash + Sized {
    fn cost(&self, state: &D::State, context: &D::Context) -> u64;

    fn prerequisite(&self, state: &D::State, context: &D::Context) -> bool;

    fn act(&self, state: &D::State, context: &D::Context) -> D::State;

    /// Append every action whose prerequisite holds in `state`. Takes a buffer rather
    /// than returning a `Vec` so the planner can reuse one allocation for the whole
    /// search.
    fn available(state: &D::State, context: &D::Context, out: &mut Vec<Self>);
}

pub trait Goal<D: Domain> {
    fn name(&self) -> &'static str;

    /// How badly the agent wants this right now. Highest is planned first.
    fn utility(&self, state: &D::State, context: &D::Context) -> i64;

    /// Conditions still outstanding; zero means satisfied.
    fn unmet(&self, state: &D::State, context: &D::Context) -> u32;

    /// Optimistic remaining cost: the A* heuristic. Overshooting the true remaining cost costs
    /// admissibility, and with it the guarantee that the first plan found is the cheapest.
    ///
    /// The default assumes each outstanding condition needs at least one unit-cost
    /// action. Domains that know what their actions charge should say so: an estimate
    /// that ignores the dominant cost is admissible and nearly useless.
    fn estimate(&self, state: &D::State, context: &D::Context) -> u64 {
        u64::from(self.unmet(state, context))
    }

    fn is_satisfied(&self, state: &D::State, context: &D::Context) -> bool {
        self.unmet(state, context) == 0
    }
}

pub struct PlannerConfig {
    /// Multiplier on [`Goal::estimate`]. `0` discards the heuristic and degrades to
    /// uniform-cost search, `1` is admissible A*, above that is weighted A*: fewer
    /// expansions in exchange for plans up to that factor from optimal.
    pub heuristic_weight: u64,
    /// Expansions allowed per goal before the search gives up and the next goal is
    /// tried. Bounds the worst case so one unreachable goal cannot stall a frame.
    ///
    /// A soft bound: once it trips, the frontier still has queued nodes to pop, and
    /// [`Plan::expansions`] counts those too. They generate no successors, so the
    /// overshoot is bounded by the frontier's size and costs next to nothing.
    pub max_expansions: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            heuristic_weight: 1,
            max_expansions: 20_000,
        }
    }
}

/// What became of one goal during a call to [`plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consideration {
    /// Nothing outstanding, so it was never planned for.
    Satisfied,
    /// Planned for, and its plan is the one returned.
    Chosen,
    /// Planned for, but no plan exists inside [`PlannerConfig::max_expansions`].
    Unreachable,
    /// A goal it outranked, or that outranked it, was served first.
    NotTried,
}

/// One goal as the planner weighed it. Answers "why did this agent do that?" after the
/// fact, which is the only way to debug a decision made from a dozen competing numbers.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub goal: &'static str,
    pub utility: i64,
    pub unmet: u32,
    pub outcome: Consideration,
}

#[derive(Debug)]
pub struct Plan<A> {
    /// The goal being served, or `None` when nothing could be planned.
    pub goal: Option<&'static str>,
    pub actions: VecDeque<A>,
    pub cost: u64,
    /// Nodes expanded across every goal attempted. The number to watch when tuning a
    /// heuristic.
    pub expansions: usize,
    pub considered: Vec<Candidate>,
}

impl<A> Plan<A> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// Plan for the most-wanted goal that is actually reachable.
///
/// Goals are tried in descending utility and the first that yields a plan wins, so a
/// goal nothing can currently satisfy costs a bounded search rather than an idle agent.
#[must_use]
pub fn plan<D: Domain>(
    state: &D::State,
    context: &D::Context,
    goals: &[Box<dyn Goal<D>>],
    config: &PlannerConfig,
) -> Plan<D::Action> {
    let mut considered: Vec<Candidate> = goals
        .iter()
        .map(|goal| {
            let unmet = goal.unmet(state, context);
            Candidate {
                goal: goal.name(),
                utility: goal.utility(state, context),
                unmet,
                outcome: if unmet == 0 {
                    Consideration::Satisfied
                } else {
                    Consideration::NotTried
                },
            }
        })
        .collect();

    // Stable, so equal utilities break toward declaration order instead of whichever
    // way the sort happens to fall.
    let mut order: Vec<usize> = considered
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.outcome != Consideration::Satisfied)
        .map(|(index, _)| index)
        .collect();
    order.sort_by_key(|&index| {
        std::cmp::Reverse(considered.get(index).map_or(i64::MIN, |c| c.utility))
    });

    let mut expansions = 0;
    for index in order {
        let Some(goal) = goals.get(index) else {
            continue;
        };

        if let Some((path, cost)) =
            search::<D>(state, context, goal.as_ref(), config, &mut expansions)
        {
            if let Some(candidate) = considered.get_mut(index) {
                candidate.outcome = Consideration::Chosen;
            }
            return Plan {
                goal: Some(goal.name()),
                actions: recover_actions::<D>(&path, context),
                cost,
                expansions,
                considered,
            };
        }

        if let Some(candidate) = considered.get_mut(index) {
            candidate.outcome = Consideration::Unreachable;
        }
    }

    Plan {
        goal: None,
        actions: VecDeque::new(),
        cost: 0,
        expansions,
        considered,
    }
}

fn search<D: Domain>(
    start: &D::State,
    context: &D::Context,
    goal: &dyn Goal<D>,
    config: &PlannerConfig,
    expansions: &mut usize,
) -> Option<(Vec<D::State>, u64)> {
    let budget = expansions.saturating_add(config.max_expansions);
    let mut scratch: Vec<D::Action> = Vec::new();

    astar(
        start,
        |state| {
            *expansions = expansions.saturating_add(1);
            // `astar` offers no abort hook, so the budget is enforced by starving the
            // frontier: no successors means the search runs dry and reports failure.
            if *expansions > budget {
                return Vec::new();
            }

            scratch.clear();
            D::Action::available(state, context, &mut scratch);
            scratch
                .iter()
                .map(|action| (action.act(state, context), action.cost(state, context)))
                .collect()
        },
        |state| {
            goal.estimate(state, context)
                .saturating_mul(config.heuristic_weight)
        },
        |state| goal.is_satisfied(state, context),
    )
}

/// Re-derive the actions that walk `path`, one adjacent state pair at a time.
///
/// The action is deliberately absent from the search node. `pathfinding` keys its
/// frontier by node and rewrites only the parent link when it finds a cheaper route,
/// so an action carried inside the key would still describe the route it was first inserted
/// with. Keeping the node as bare state also lets it stay `Copy` and hash in constant time.
fn recover_actions<D: Domain>(path: &[D::State], context: &D::Context) -> VecDeque<D::Action> {
    let mut scratch: Vec<D::Action> = Vec::new();
    let mut actions = VecDeque::with_capacity(path.len().saturating_sub(1));

    for pair in path.windows(2) {
        if let [from, to] = pair {
            scratch.clear();
            D::Action::available(from, context, &mut scratch);

            let step = scratch
                .iter()
                .filter(|action| &action.act(from, context) == to)
                .min_by_key(|action| action.cost(from, context));

            if let Some(action) = step {
                actions.push_back(action.clone());
            }
        }
    }

    actions
}
