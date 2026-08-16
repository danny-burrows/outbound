use crate::actions::chebyshev;
use crate::item::{GATHER_COST, Good};
use crate::village::{CARRY_CAPACITY, STORE_COST, Village, VillagerState};
use crate::world::World;
use goap::Goal;

/// Have `target` units of a good in the store at home.
pub(crate) struct Stockpile {
    good: Good,
    target: u8,
}

impl Stockpile {
    pub(crate) const fn new(good: Good, target: u8) -> Self {
        Self { good, target }
    }

    fn shortfall(&self, state: &VillagerState) -> u8 {
        self.target.saturating_sub(state.stored.count(self.good))
    }

    /// Lower bound on reaching every source still needed, ignoring the legs home.
    fn gathering(&self, state: &VillagerState, context: &World, outstanding: u64) -> u64 {
        if outstanding == 0 {
            return 0;
        }

        let source = self.good.source();
        let standing = state.standing_on == Some(source);
        let taken = u64::from(state.taken(source));

        // Standing at a source means one of the outstanding units needs no trip at all.
        let first = if standing {
            taken.saturating_add(1)
        } else {
            taken
        };
        let last = taken.saturating_add(outstanding).saturating_sub(1);
        let mut travel = context.depletion_cost(source, first, last);

        if !standing && let Some((_, distance)) = context.nearest(state.position, source) {
            travel = travel.saturating_add(distance);
        }

        travel
    }

    /// Lower bound on the legs to and from home, ignoring the legs between sources.
    ///
    /// After each load is stored the pack is empty, so every load beyond the first owes a round
    /// trip out to a source and back.
    fn hauling(&self, state: &VillagerState, context: &World, trips: u64) -> u64 {
        let from_home = context
            .nearest(state.home, self.good.source())
            .map_or(0, |(_, distance)| distance);

        chebyshev(state.position, state.home).saturating_add(
            trips
                .saturating_sub(1)
                .saturating_mul(2)
                .saturating_mul(from_home),
        )
    }
}

impl Goal<Village> for Stockpile {
    fn name(&self) -> &'static str {
        self.good.label()
    }

    fn utility(&self, state: &VillagerState, _: &World) -> i64 {
        i64::from(self.shortfall(state))
    }

    fn unmet(&self, state: &VillagerState, _: &World) -> u32 {
        u32::from(self.shortfall(state))
    }

    fn estimate(&self, state: &VillagerState, context: &World) -> u64 {
        let shortfall = u64::from(self.shortfall(state));
        if shortfall == 0 {
            return 0;
        }

        // Units already in hand are part-paid: they need hauling but no more harvesting, and
        // logs underfoot need only gathering. Charging either full price would overshoot.
        let carried = u64::from(state.carrying.count(self.good)).min(shortfall);
        let underfoot = if matches!(self.good, Good::Wood) {
            u64::from(state.logs_underfoot).min(shortfall.saturating_sub(carried))
        } else {
            0
        };
        let outstanding = shortfall.saturating_sub(carried).saturating_sub(underfoot);

        let trips = shortfall.div_ceil(u64::from(CARRY_CAPACITY).max(1));
        let work = outstanding
            .saturating_mul(self.good.unit_work())
            .saturating_add(underfoot.saturating_mul(GATHER_COST))
            .saturating_add(trips.saturating_mul(STORE_COST));

        // The larger of the two travel bounds, never their sum: both describe the same walking,
        // so adding them would charge parts of it twice and overshoot the true remaining cost,
        // which is exactly what costs admissibility.
        let travel = self
            .gathering(state, context, outstanding)
            .max(self.hauling(state, context, trips));

        work.saturating_add(travel)
    }
}
