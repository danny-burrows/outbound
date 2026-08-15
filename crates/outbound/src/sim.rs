use crate::actions::VillagerAction;
use crate::goals::Collect;
use crate::item::{Good, Item, Resource};
use crate::rng::Rng;
use crate::stats::Stats;
use crate::village::{PlanState, Village};
use crate::world::World;
use goap::{Goal, PlannerConfig, plan};
use pathfinding::directed::astar::astar;
use std::collections::VecDeque;

pub(crate) const WORLD_EXTENT: i64 = 150;

const MAX_TREES: usize = 250;
const MAX_BERRIES: usize = 50;
const MAX_STONE: usize = 25;

const TARGET_WOOD: u8 = 10;
const TARGET_STONE: u8 = 10;
const TARGET_BERRIES: u8 = 10;

/// Steps to wait before reconsidering, once a plan came back with nothing to do.
///
/// Without it an idle villager replans every step, which floods the trace and burns a
/// search per tick to reach the same conclusion. Staggering replans is also how this
/// will have to work once there are hundreds of villagers sharing a frame budget.
const IDLE_REPLAN_INTERVAL: u64 = 120;

/// Turn "be over there" into cells to step through.
///
/// Eight-connected at unit cost, which is why `World::nearest` measures in Chebyshev
/// distance — that makes the planner's price for a `Travel` the exact length of the walk
/// it defers rather than a guess.
fn walk_to(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (goal_x, goal_y) = to;
    let Some((mut steps, _)) = astar(
        &from,
        |&(x, y)| {
            (-1..=1)
                .flat_map(move |i| (-1..=1).map(move |j| ((x + i, y + j), 1_u64)))
                .collect::<Vec<_>>()
        },
        |&(x, y)| (x - goal_x).unsigned_abs().max((y - goal_y).unsigned_abs()),
        |&position| position == to,
    ) else {
        return Vec::new();
    };
    steps.reverse();
    steps
}

/// The plan in progress and how far through it the villager is.
#[derive(Default)]
struct Executor {
    actions: VecDeque<VillagerAction>,
    /// Cells left to step through for the `Travel` under way.
    walk: Vec<(i64, i64)>,
}

/// The whole simulation, with no renderer anywhere in it.
///
/// Everything here is reproducible from a seed and steppable without a window, which is
/// what makes a ten-thousand-step run something you can diff rather than watch.
pub(crate) struct Simulation {
    pub(crate) world: World,
    pub(crate) state: PlanState,
    pub(crate) stats: Stats,
    /// The goal currently being served, for anything that wants to display it.
    pub(crate) current_goal: Option<&'static str>,
    executor: Executor,
    goals: Vec<Box<dyn Goal<Village>>>,
    config: PlannerConfig,
    steps: u64,
    replan_after: u64,
}

impl Simulation {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut items = Vec::with_capacity(MAX_TREES + MAX_BERRIES + MAX_STONE);
        for (resource, count) in [
            (Resource::Tree, MAX_TREES),
            (Resource::Berry, MAX_BERRIES),
            (Resource::Stone, MAX_STONE),
        ] {
            for _ in 0..count {
                items.push(Item::new(resource, rng.position(WORLD_EXTENT)));
            }
        }

        let extent = u64::try_from(WORLD_EXTENT).unwrap_or(1);
        Self {
            world: World::new(&items, extent),
            state: PlanState::default(),
            stats: Stats::default(),
            current_goal: None,
            executor: Executor::default(),
            goals: vec![
                Box::new(Collect::new(Good::Wood, TARGET_WOOD)),
                Box::new(Collect::new(Good::Stone, TARGET_STONE)),
                Box::new(Collect::new(Good::Berry, TARGET_BERRIES)),
            ],
            config: PlannerConfig::default(),
            steps: 0,
            replan_after: 0,
        }
    }

    pub(crate) const fn steps(&self) -> u64 {
        self.steps
    }

    /// One simulation step: walk a cell, carry out an action, or plan the next thing.
    pub(crate) fn advance(&mut self) {
        self.steps = self.steps.saturating_add(1);

        if let Some(position) = self.executor.walk.pop() {
            self.world.render_position = position;
            return;
        }

        if let Some(action) = self.executor.actions.pop_front() {
            self.execute(action);
            return;
        }

        if self.steps < self.replan_after {
            return;
        }

        let found = plan(&self.state, &self.world, &self.goals, &self.config);
        self.current_goal = found.goal;
        self.stats.record_plan(found.expansions, !found.is_empty());

        if let Some(goal) = found.goal {
            log::info!(
                "step {}: pursuing {goal} — cost {}, {} actions, {} expansions",
                self.steps,
                found.cost,
                found.actions.len(),
                found.expansions,
            );
        } else {
            self.replan_after = self.steps.saturating_add(IDLE_REPLAN_INTERVAL);
            log::debug!("step {}: nothing worth doing", self.steps);
        }

        // Why, rather than what: the goals that were on the table and what became of
        // each. This is the record to reach for when a decision looks wrong.
        for candidate in &found.considered {
            log::debug!(
                "    {:<8} utility {:>4}  unmet {:>3}  {:?}",
                candidate.goal,
                candidate.utility,
                candidate.unmet,
                candidate.outcome,
            );
        }

        self.executor.actions = found.actions;
    }

    /// Refine one planned action against the real world.
    ///
    /// The plan only asked to be at *a* source; which instance, and the route to it, is
    /// decided here. Splitting it this way is what keeps the search out of the hundreds
    /// of steps a walk actually takes.
    fn execute(&mut self, action: VillagerAction) {
        log::trace!("step {}: {action:?}", self.steps);

        match action {
            VillagerAction::Travel(resource) => {
                let found = self.world.nearest(self.world.render_position, resource);
                let Some((destination, _)) = found else {
                    self.abandon_plan(action, "no instance of that resource left in the world");
                    return;
                };
                self.executor.walk = walk_to(self.world.render_position, destination);
                self.state.position = destination;
                self.state.standing_on = Some(resource);
                self.state.logs_underfoot = 0;
            }
            VillagerAction::Harvest(resource) => {
                if !self.world.take(resource, self.state.position) {
                    self.abandon_plan(action, "nothing of that kind where the villager stands");
                    return;
                }
                self.state.take(resource);
                self.state.standing_on = None;
                if let Some(good) = resource.direct_good() {
                    self.state.carrying.add(good);
                } else {
                    self.world.drop_log(self.state.position);
                    self.state.logs_underfoot = self.state.logs_underfoot.saturating_add(1);
                }
            }
            VillagerAction::Gather => {
                if !self.world.take_log(self.state.position) {
                    self.abandon_plan(action, "no log where the villager stands");
                    return;
                }
                self.state.logs_underfoot = self.state.logs_underfoot.saturating_sub(1);
                self.state.carrying.add(Good::Wood);
            }
        }
    }

    /// The rest of a plan assumed this action landed, so none of it is trustworthy now.
    fn abandon_plan(&mut self, action: VillagerAction, reason: &'static str) {
        self.stats.record_refinement_failure();
        log::warn!(
            "step {}: {action:?} could not be carried out — {reason}; abandoning the plan",
            self.steps,
        );
        self.executor.actions.clear();
        self.executor.walk.clear();
        self.current_goal = None;
    }
}
