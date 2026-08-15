use crate::goap::Action;
use crate::item::{GATHER_COST, Good, Resource};
use crate::village::{PlanState, Village};
use crate::world::World;

/// Three operators cover the domain now that item identities live in [`World`].
///
/// The `MoveToNearestItem` / `Move` / `ChopTree` / `PickUpItem` quartet this replaces
/// carried owned `String`s and cloned `Item`s, so every node expansion allocated. This
/// is two bytes and `Copy`. The arbitrary-delta `Move` is gone too: it only existed to
/// walk the villager back to the origin so a *complete* goal state could match, and
/// partial goals no longer care where anyone is standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum VillagerAction {
    /// Stand at the nearest unworked instance. Refined into an actual route at
    /// execution time, not during the search.
    Travel(Resource),
    /// Work the resource underfoot.
    Harvest(Resource),
    /// Pick up one felled log.
    Gather,
}

impl Action<Village> for VillagerAction {
    fn cost(&self, state: &PlanState, context: &World) -> u64 {
        match *self {
            // The estimate for the walk this defers, which is what lets the layer
            // above search without expanding the hundreds of steps below it.
            Self::Travel(resource) => context
                .trip_cost(state.position, resource, state.taken(resource))
                .unwrap_or(u64::MAX),
            Self::Harvest(resource) => resource.harvest_cost(),
            Self::Gather => GATHER_COST,
        }
    }

    fn prerequisite(&self, state: &PlanState, context: &World) -> bool {
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

    fn act(&self, state: &PlanState, context: &World) -> PlanState {
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

    fn available(state: &PlanState, context: &World, out: &mut Vec<Self>) {
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
