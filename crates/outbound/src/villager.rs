use crate::actions::VillagerAction;
use crate::goals::Stockpile;
use crate::item::Good;
use crate::rng::Rng;
use crate::stats::Stats;
use crate::village::{Village, VillagerState};
use crate::world::World;
use goap::{Goal, PlannerConfig, plan};
use std::collections::VecDeque;

/// Most a villager can be born wanting of any one good. Zero is allowed and means not
/// wanting the thing at all.
const MAX_DESIRE: i64 = 12;

/// Steps to wait before reconsidering, once a plan came back with nothing to do. Without it
/// an idle villager searches every step to reach the same conclusion.
const IDLE_REPLAN_INTERVAL: u64 = 120;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Health(Option<u8>);

impl Health {
    pub(crate) const fn new(amount: u8) -> Self {
        Self(Some(amount))
    }

    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that changes it"
    )]
    pub(crate) fn replenish(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = Some(health.saturating_add(amount).min(100));
        }
    }

    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that changes it"
    )]
    pub(crate) const fn sap(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = health.checked_sub(amount);
        }
    }

    pub(crate) const fn check(&self) -> Option<u8> {
        self.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100)
    }
}

/// One agent, entire. [`World`] holds only what villagers share.
pub(crate) struct Villager {
    pub(crate) id: usize,
    /// What the planner reasons over: where they stand and what they carry.
    pub(crate) state: VillagerState,
    /// Trails `state.position` while a walk plays out, and catches up before the next action
    /// runs.
    pub(crate) render_position: (i64, i64),
    pub(crate) current_goal: Option<&'static str>,
    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that reads it"
    )]
    pub(crate) health: Health,
    goals: Vec<Box<dyn Goal<Village>>>,
    plan: VecDeque<VillagerAction>,
    /// Cells left in the walk the current `Travel` is part-way through.
    walk: Vec<(i64, i64)>,
    replan_after: u64,
}

impl Villager {
    pub(crate) fn spawn(id: usize, rng: &mut Rng, extent: i64) -> Self {
        let position = rng.position(extent);
        let goals = Good::ALL
            .into_iter()
            .map(|good| {
                let target = u8::try_from(rng.between(0, MAX_DESIRE)).unwrap_or(0);
                Box::new(Stockpile::new(good, target)) as Box<dyn Goal<Village>>
            })
            .collect();

        Self {
            id,
            state: VillagerState::at_home(position),
            render_position: position,
            current_goal: None,
            health: Health::default(),
            goals,
            plan: VecDeque::new(),
            walk: Vec::new(),
            // Staggered, so a fresh village does not all search on one step.
            replan_after: u64::try_from(id).unwrap_or(0),
        }
    }

    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that reads it"
    )]
    pub(crate) const fn is_alive(&self) -> bool {
        self.health.check().is_some()
    }

    /// Nothing planned and nowhere to be. Either every desire is met or none can be.
    pub(crate) fn is_idle(&self) -> bool {
        self.plan.is_empty() && self.walk.is_empty()
    }

    /// One step: walk a cell, carry out an action, or decide what to do next.
    pub(crate) fn advance(
        &mut self,
        world: &mut World,
        config: &PlannerConfig,
        stats: &mut Stats,
        step: u64,
    ) {
        if let Some(position) = self.walk.pop() {
            self.render_position = position;
            return;
        }

        if let Some(action) = self.plan.pop_front() {
            self.execute(action, world, stats, step);
            return;
        }

        if step >= self.replan_after {
            self.reconsider(world, config, stats, step);
        }
    }

    fn reconsider(&mut self, world: &World, config: &PlannerConfig, stats: &mut Stats, step: u64) {
        let found = plan(&self.state, world, &self.goals, config);
        self.current_goal = found.goal;
        stats.record_plan(found.expansions, !found.is_empty());

        if let Some(goal) = found.goal {
            log::info!(
                "step {step}: villager {} pursuing {goal}: cost {}, {} actions, {} expansions",
                self.id,
                found.cost,
                found.actions.len(),
                found.expansions,
            );
        } else {
            self.replan_after = step.saturating_add(IDLE_REPLAN_INTERVAL);
            log::debug!("step {step}: villager {} has nothing worth doing", self.id);
        }

        for candidate in &found.considered {
            log::debug!(
                "    villager {} {:<8} utility {:>4}  unmet {:>3}  {:?}",
                self.id,
                candidate.goal,
                candidate.utility,
                candidate.unmet,
                candidate.outcome,
            );
        }

        self.plan = found.actions;
    }

    /// Refine one planned action against the real world.
    ///
    /// The plan only asked to be at *a* source; which instance, and the route to it, is
    /// decided here. That keeps the search out of the hundreds of steps a walk actually
    /// takes, and it is where one villager discovers another got there first.
    fn execute(&mut self, action: VillagerAction, world: &mut World, stats: &mut Stats, step: u64) {
        log::trace!("step {step}: villager {} {action:?}", self.id);

        match action {
            VillagerAction::Travel(resource) => {
                let found = world.nearest(self.render_position, resource);
                let Some((destination, _)) = found else {
                    self.abandon_plan(
                        action,
                        "none of that resource left in the world",
                        world,
                        stats,
                        step,
                    );
                    return;
                };
                self.walk = walk_to(self.render_position, destination);
                self.state.position = destination;
                self.state.standing_on = Some(resource);
                self.state.logs_underfoot = 0;
            }
            VillagerAction::Harvest(resource) => {
                if !world.take(resource, self.state.position) {
                    self.abandon_plan(action, "nothing of that kind left here", world, stats, step);
                    return;
                }
                self.state.take(resource);
                self.state.standing_on = None;
                if let Some(good) = resource.direct_good() {
                    self.state.carrying.add(good);
                } else {
                    world.drop_log(self.state.position);
                    self.state.logs_underfoot = self.state.logs_underfoot.saturating_add(1);
                }
            }
            VillagerAction::Gather => {
                if !world.take_log(self.state.position) {
                    self.abandon_plan(action, "no log left here", world, stats, step);
                    return;
                }
                self.state.logs_underfoot = self.state.logs_underfoot.saturating_sub(1);
                self.state.carrying.add(Good::Wood);
            }
            // Neither of these touches the world, so neither can be beaten to it.
            VillagerAction::GoHome => {
                self.walk = walk_to(self.render_position, self.state.home);
                self.state.position = self.state.home;
                self.state.standing_on = None;
                self.state.logs_underfoot = 0;
            }
            VillagerAction::Store => {
                self.state.stored.absorb(self.state.carrying);
                self.state.carrying.clear();
            }
        }
    }

    /// The rest of a plan assumed this action landed, so none of it is trustworthy now.
    ///
    /// Planning state is a belief about the surroundings, and a failure here is proof the
    /// belief is stale. Re-reading it from the world is what stops the next plan repeating
    /// the same impossible action forever.
    fn abandon_plan(
        &mut self,
        action: VillagerAction,
        reason: &'static str,
        world: &World,
        stats: &mut Stats,
        step: u64,
    ) {
        stats.record_refinement_failure();
        log::warn!(
            "step {step}: villager {} could not carry out {action:?}: {reason}; abandoning the plan",
            self.id,
        );
        self.plan.clear();
        self.walk.clear();
        self.current_goal = None;
        self.state.standing_on = world.resource_at(self.state.position);
        self.state.logs_underfoot = 0;
    }
}

/// Turn "be over there" into cells to step through, nearest cell last.
///
/// Eight-connected at unit cost, which is why [`World::nearest`] measures in Chebyshev
/// distance: a straight line covers that many steps whatever its slope, so the planner's
/// price for a `Travel` is the exact length of the walk it defers rather than a guess.
fn walk_to(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (from_x, from_y) = from;
    let (to_x, to_y) = to;
    let run = to_x - from_x;
    let rise = to_y - from_y;
    let steps = run.abs().max(rise.abs());
    if steps == 0 {
        return Vec::new();
    }

    let along = |delta: i64, step: i64| (delta * step + steps / 2 * delta.signum()) / steps;

    (1..=steps)
        .rev()
        .map(|step| (from_x + along(run, step), from_y + along(rise, step)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::walk_to;

    fn forward(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
        let mut cells = walk_to(from, to);
        cells.reverse();
        cells
    }

    /// Every step closes the Chebyshev distance, in every direction. A walk that took a
    /// detour would have to stand still or back off for a step, and neither is allowed here.
    #[test]
    fn a_walk_spends_every_step_getting_closer() {
        for run in -12..=12_i64 {
            for rise in -12..=12_i64 {
                let from = (3, -4);
                let to = (from.0 + run, from.1 + rise);
                let cells = forward(from, to);
                let remaining = |(x, y): (i64, i64)| (to.0 - x).abs().max((to.1 - y).abs());

                let length = usize::try_from(run.abs().max(rise.abs())).unwrap_or(usize::MAX);
                assert_eq!(
                    cells.len(),
                    length,
                    "the walk to {to:?} is not Chebyshev-long"
                );

                let mut previous = from;
                for &cell in &cells {
                    let (x, y) = cell;
                    let (step_x, step_y) = ((x - previous.0).abs(), (y - previous.1).abs());
                    assert!(
                        step_x <= 1 && step_y <= 1,
                        "the walk to {to:?} teleports {previous:?} -> {cell:?}"
                    );
                    assert!(
                        remaining(cell) < remaining(previous),
                        "the walk to {to:?} wanders at {cell:?}"
                    );
                    previous = cell;
                }
                assert_eq!(remaining(previous), 0, "the walk to {to:?} stops short");
            }
        }
    }

    /// The shallow diagonal a plateau-tied A* used to turn into a detour, now interpolated:
    /// the major axis advances every step and the minor axis lags where it should.
    #[test]
    fn a_shallow_diagonal_is_interpolated_not_dog_legged() {
        assert_eq!(
            forward((0, 0), (10, 3)),
            vec![
                (1, 0),
                (2, 1),
                (3, 1),
                (4, 1),
                (5, 2),
                (6, 2),
                (7, 2),
                (8, 2),
                (9, 3),
                (10, 3),
            ]
        );
    }

    #[test]
    fn a_walk_to_where_you_stand_is_no_walk_at_all() {
        assert!(walk_to((7, 7), (7, 7)).is_empty());
    }
}
