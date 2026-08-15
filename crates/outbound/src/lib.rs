mod actions;
mod goals;
mod item;
mod report;
mod rng;
mod sim;
mod stats;
mod village;
mod villager;
mod world;

#[cfg(feature = "render")]
mod render;

#[cfg(test)]
mod tests;

use crate::item::{Good, Resource};
use crate::sim::Simulation;

pub use crate::report::Report;

#[cfg(feature = "render")]
pub use crate::render::run;

/// Run without a window and report what happened.
///
/// This is the mode that makes the simulation testable: no renderer, no frame pacing,
/// reproducible from `seed`, and fast enough to run further than anyone would sit and
/// watch. The narrative of each decision goes to the `log` facade; install a logger to see
/// it.
#[must_use]
pub fn run_headless(seed: u64, population: usize, steps: u64) -> Report {
    let mut simulation = Simulation::new(seed, population);
    for _ in 0..steps {
        simulation.advance();
    }

    let stats = simulation.stats;
    Report {
        seed,
        steps: simulation.steps(),
        villagers: simulation.villagers.len(),
        idle_villagers: simulation.idle_villagers(),
        wood: simulation.carried(Good::Wood),
        stone: simulation.carried(Good::Stone),
        berries: simulation.carried(Good::Berry),
        plans: stats.plans,
        empty_plans: stats.empty_plans,
        expansions: stats.expansions,
        peak_expansions: stats.peak_expansions,
        refinement_failures: stats.refinement_failures,
        trees_left: simulation.world.count(Resource::Tree),
        stone_left: simulation.world.count(Resource::Stone),
        berries_left: simulation.world.count(Resource::Berry),
        logs_stranded: simulation.world.logs.len(),
    }
}
