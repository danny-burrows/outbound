use crate::actions::VillagerAction;
use crate::item::{Inventory, Resource};
use crate::world::World;
use goap::Domain;

/// The planning problem: one villager choosing what to do next, with the village around them
/// as read-only context.
///
/// The village is the *setting*, not the subject. Nothing here plans for villagers jointly.
pub(crate) struct Village;

impl Domain for Village {
    type State = VillagerState;
    type Context = World;
    type Action = VillagerAction;
}

/// Everything the planner knows about *one villager*: a few dozen bytes, `Copy`, hashed in
/// constant time.
///
/// Not a view of the world; that is [`World`], borrowed as context. Villagers are planned for
/// one at a time and cannot see each other, so no plan can call on anyone else's labour.
///
/// One of these is cloned and hashed per expanded node, which is why nothing positional beyond
/// the villager themselves lives here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct VillagerState {
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

impl VillagerState {
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
