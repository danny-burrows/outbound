use crate::item::{GATHER_COST, Good, Resource};
use crate::village::{Village, VillagerState};
use crate::world::World;
use goap::Action;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum VillagerAction {
    /// Stand at the nearest unworked instance. Refined into an actual route at execution
    /// time, not during the search.
    Travel(Resource),
    Harvest(Resource),
    Gather,
}

impl Action<Village> for VillagerAction {
    fn cost(&self, state: &VillagerState, context: &World) -> u64 {
        match *self {
            Self::Travel(resource) => context
                .trip_cost(state.position, resource, state.taken(resource))
                .unwrap_or(u64::MAX),
            Self::Harvest(resource) => resource.harvest_cost(),
            Self::Gather => GATHER_COST,
        }
    }

    fn prerequisite(&self, state: &VillagerState, context: &World) -> bool {
        match *self {
            // Travelling to where you already stand would be a free self-loop for the
            // search to churn on.
            Self::Travel(resource) => {
                state.standing_on != Some(resource)
                    && context
                        .trip_cost(state.position, resource, state.taken(resource))
                        .is_some()
            }
            Self::Harvest(resource) => state.standing_on == Some(resource),
            Self::Gather => state.logs_underfoot > 0,
        }
    }

    fn act(&self, state: &VillagerState, context: &World) -> VillagerState {
        let mut next = *state;

        match *self {
            Self::Travel(resource) => {
                if let Some((position, _)) = context.nearest(state.position, resource) {
                    next.position = position;
                }
                next.standing_on = Some(resource);
                next.logs_underfoot = 0;
            }
            Self::Harvest(resource) => {
                next.take(resource);
                next.standing_on = None;
                match resource.direct_good() {
                    Some(good) => next.carrying.add(good),
                    None => next.logs_underfoot = next.logs_underfoot.saturating_add(1),
                }
            }
            Self::Gather => {
                next.logs_underfoot = next.logs_underfoot.saturating_sub(1);
                next.carrying.add(Good::Wood);
            }
        }

        next
    }

    fn available(state: &VillagerState, context: &World, out: &mut Vec<Self>) {
        for resource in Resource::ALL {
            for action in [Self::Travel(resource), Self::Harvest(resource)] {
                if action.prerequisite(state, context) {
                    out.push(action);
                }
            }
        }

        if Self::Gather.prerequisite(state, context) {
            out.push(Self::Gather);
        }
    }
}
