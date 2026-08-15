use crate::item::{Good, Item, Resource};
use crate::rng::Rng;
use crate::stats::Stats;
use crate::villager::Villager;
use crate::world::World;
use goap::PlannerConfig;

pub(crate) const WORLD_EXTENT: i64 = 150;

/// Stocked for a village rather than a single villager: eight people each wanting up to a
/// dozen of each good will exhaust a sparse world and spend the rest of the run finding
/// their desires unreachable.
const MAX_TREES: usize = 250;
const MAX_BERRIES: usize = 120;
const MAX_STONE: usize = 90;

/// The whole simulation, with no renderer anywhere in it.
///
/// Everything here is reproducible from a seed and steppable without a window, which is what
/// makes a ten-thousand-step run something you can diff rather than watch.
pub(crate) struct Simulation {
    pub(crate) world: World,
    pub(crate) villagers: Vec<Villager>,
    pub(crate) stats: Stats,
    config: PlannerConfig,
    steps: u64,
}

impl Simulation {
    /// The world is generated before anyone is spawned into it, so one seed describes one
    /// world whatever the population.
    pub(crate) fn new(seed: u64, population: usize) -> Self {
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

        let villagers = (0..population)
            .map(|id| Villager::spawn(id, &mut rng, WORLD_EXTENT))
            .collect();

        let extent = u64::try_from(WORLD_EXTENT).unwrap_or(1);
        Self {
            world: World::new(&items, extent),
            villagers,
            stats: Stats::default(),
            config: PlannerConfig::default(),
            steps: 0,
        }
    }

    pub(crate) const fn steps(&self) -> u64 {
        self.steps
    }

    /// One simulation step: every villager does one thing.
    pub(crate) fn advance(&mut self) {
        self.steps = self.steps.saturating_add(1);

        // Disjoint fields, so every villager can be stepped against a mutable world.
        for villager in &mut self.villagers {
            villager.advance(&mut self.world, &self.config, &mut self.stats, self.steps);
        }
    }

    /// Villagers with nothing planned: every desire either met or out of reach.
    pub(crate) fn idle_villagers(&self) -> usize {
        self.villagers
            .iter()
            .filter(|villager| villager.is_idle())
            .count()
    }

    /// How much of a good the village is carrying between them.
    pub(crate) fn carried(&self, good: Good) -> u64 {
        self.villagers
            .iter()
            .map(|villager| u64::from(villager.state.carrying.count(good)))
            .sum()
    }
}
