use crate::actions::VillagerAction;
use crate::item::{Good, Resource};
use crate::world::World;
use goap::Domain;

/// Marker tying the village's state, context and actions together.
pub(crate) struct Village;

impl Domain for Village {
    type State = PlanState;
    type Context = World;
    type Action = VillagerAction;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Inventory([u8; Good::COUNT]);

impl Inventory {
    pub(crate) fn count(self, good: Good) -> u8 {
        self.0.get(good.index()).copied().unwrap_or(0)
    }

    pub(crate) fn add(&mut self, good: Good) {
        if let Some(slot) = self.0.get_mut(good.index()) {
            *slot = slot.saturating_add(1);
        }
    }
}

/// The planner's entire view of the world: a few dozen bytes, `Copy`, hashed in
/// constant time.
///
/// Everything positional beyond the villager itself lives in [`World`] and is read
/// through it. The old state carried the whole item list, so each of the hundreds of
/// node expansions in a search cloned and hashed 325 items it never reasoned about.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PlanState {
    pub(crate) position: (i64, i64),
    /// Resource underfoot, when the villager is standing at one.
    pub(crate) standing_on: Option<Resource>,
    /// Logs felled at the current position. Left behind on moving away.
    pub(crate) logs_underfoot: u8,
    pub(crate) carrying: Inventory,
    /// Instances of each resource this plan has consumed. Prices the next trip without
    /// the state having to remember which instances they were.
    taken: [u16; Resource::COUNT],
}

impl PlanState {
    /// Standing somewhere, carrying nothing.
    pub(crate) fn at(position: (i64, i64)) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }

    pub(crate) fn taken(&self, resource: Resource) -> u16 {
        self.taken.get(resource.index()).copied().unwrap_or(0)
    }

    pub(crate) fn take(&mut self, resource: Resource) {
        if let Some(slot) = self.taken.get_mut(resource.index()) {
            *slot = slot.saturating_add(1);
        }
    }
}
