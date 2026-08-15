use crate::item::{GATHER_COST, Good};
use crate::village::{PlanState, Village};
use crate::world::World;
use goap::Goal;

/// Carry `target` units of a good.
pub(crate) struct Collect {
    good: Good,
    target: u8,
}

impl Collect {
    pub(crate) const fn new(good: Good, target: u8) -> Self {
        Self { good, target }
    }

    fn shortfall(&self, state: &PlanState) -> u8 {
        self.target.saturating_sub(state.carrying.count(self.good))
    }
}

impl Goal<Village> for Collect {
    fn name(&self) -> &'static str {
        self.good.label()
    }

    fn utility(&self, state: &PlanState, _: &World) -> i64 {
        i64::from(self.shortfall(state))
    }

    fn unmet(&self, state: &PlanState, _: &World) -> u32 {
        u32::from(self.shortfall(state))
    }

    fn estimate(&self, state: &PlanState, context: &World) -> u64 {
        let shortfall = u64::from(self.shortfall(state));
        if shortfall == 0 {
            return 0;
        }

        // Logs already felled and lying underfoot are part-paid: they only need
        // gathering, so charging them full price would overshoot the true cost.
        let ready = if matches!(self.good, Good::Wood) {
            u64::from(state.logs_underfoot).min(shortfall)
        } else {
            0
        };
        let outstanding = shortfall.saturating_sub(ready);

        let mut estimate = ready
            .saturating_mul(GATHER_COST)
            .saturating_add(outstanding.saturating_mul(self.good.unit_work()));

        if outstanding == 0 {
            return estimate;
        }

        // Every outstanding unit needs an instance of its own, so every one needs a trip;
        // bar the one underfoot, if the villager already stands at a source.
        let source = self.good.source();
        let standing = state.standing_on == Some(source);
        let taken = u64::from(state.taken(source));
        let first = if standing {
            taken.saturating_add(1)
        } else {
            taken
        };
        let last = taken.saturating_add(outstanding).saturating_sub(1);

        estimate = estimate.saturating_add(context.depletion_cost(source, first, last));

        // The nearest source's distance is known exactly, so the first trip can be
        // priced properly rather than bounded.
        if !standing && let Some((_, distance)) = context.nearest(state.position, source) {
            estimate = estimate.saturating_add(distance);
        }

        estimate
    }
}
