use crate::goals::Stockpile;
use crate::item::{Good, Item, Resource};
use crate::rng::Rng;
use crate::sim::{Simulation, WORLD_EXTENT};
use crate::village::{CARRY_CAPACITY, Village, VillagerState};
use crate::world::World;
use goap::{Action, Consideration, Goal, PlannerConfig, plan};

const EXTENT: i64 = 48;
const SEED: u64 = 0x0BAD_C0DE;

fn fixture() -> World {
    let mut rng = Rng::new(SEED);
    let mut items = Vec::new();
    for (resource, count) in [
        (Resource::Tree, 14),
        (Resource::Stone, 8),
        (Resource::Berry, 10),
    ] {
        for _ in 0..count {
            items.push(Item::new(resource, rng.position(EXTENT)));
        }
    }
    World::new(&items, u64::try_from(EXTENT).unwrap_or(1))
}

fn wood_goal(target: u8) -> Vec<Box<dyn Goal<Village>>> {
    vec![Box::new(Stockpile::new(Good::Wood, target))]
}

/// The measurement that justifies the planner's shape: a `heuristic_weight` of 0 is the
/// behaviour before it had a heuristic at all, where A* degraded to uniform-cost search.
#[test]
#[expect(
    clippy::print_stdout,
    reason = "the numbers are the point of this test; `cargo test -- --nocapture` shows them"
)]
fn heuristic_cuts_expansions_without_changing_the_plan() {
    let world = fixture();
    let state = VillagerState::default();
    let goals = wood_goal(3);

    let generous = 2_000_000;
    let blind = plan(
        &state,
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 0,
            max_expansions: generous,
            ..PlannerConfig::default()
        },
    );
    let guided = plan(
        &state,
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 1,
            max_expansions: generous,
            ..PlannerConfig::default()
        },
    );

    assert!(!blind.is_empty(), "uniform-cost search found no plan");
    assert!(!guided.is_empty(), "guided search found no plan");

    println!(
        "uniform-cost: {:>7} expansions, cost {}",
        blind.expansions, blind.cost
    );
    println!(
        "guided:       {:>7} expansions, cost {}",
        guided.expansions, guided.cost
    );

    // The estimate is admissible, so guiding the search must not cost optimality.
    assert_eq!(
        guided.cost, blind.cost,
        "heuristic changed the plan it settled on"
    );
    assert!(
        guided.expansions.saturating_mul(10) < blind.expansions,
        "expected an order of magnitude fewer expansions, got {} vs {}",
        guided.expansions,
        blind.expansions
    );
}

/// The same comparison at the size the simulation actually runs. The gap widens with the
/// world: guiding the search matters more, not less, as there is more to search.
#[test]
#[expect(
    clippy::print_stdout,
    reason = "the numbers are the point of this test; `cargo test -- --nocapture` shows them"
)]
fn the_gap_widens_with_world_size() {
    let mut rng = Rng::new(0x5EED);
    let mut items = Vec::new();
    for (resource, count) in [
        (Resource::Tree, 250),
        (Resource::Stone, 25),
        (Resource::Berry, 50),
    ] {
        for _ in 0..count {
            items.push(Item::new(resource, rng.position(WORLD_EXTENT)));
        }
    }
    let world = World::new(&items, 150);
    let goals = wood_goal(10);

    let generous = 4_000_000;
    let blind = plan(
        &VillagerState::default(),
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 0,
            max_expansions: generous,
            ..PlannerConfig::default()
        },
    );
    let guided = plan(
        &VillagerState::default(),
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 1,
            max_expansions: generous,
            ..PlannerConfig::default()
        },
    );

    assert!(!blind.is_empty(), "uniform-cost search found no plan");
    assert!(!guided.is_empty(), "guided search found no plan");

    println!(
        "10 wood among 250 trees | uniform-cost: {:>7} expansions, cost {} | guided: {:>7}, cost {}",
        blind.expansions, blind.cost, guided.expansions, guided.cost
    );

    assert_eq!(guided.cost, blind.cost, "heuristic cost optimality");
    // Ten rounds of travel, fell and gather, plus a trip home to store for each of the two
    // loads a capacity of eight forces.
    assert_eq!(guided.actions.len(), 10 * 3 + 2 * 2);
    assert!(
        guided.expansions.saturating_mul(50) < blind.expansions,
        "expected the gap to widen with world size, got {} vs {}",
        guided.expansions,
        blind.expansions
    );
}

/// Actions are re-derived from adjacent states rather than carried in the search node,
/// so it is worth pinning that the recovered sequence is applicable start to finish and
/// arrives where the planner claimed.
#[test]
fn recovered_actions_replay_the_plan() {
    let world = fixture();
    let goals = wood_goal(4);
    let found = plan(
        &VillagerState::default(),
        &world,
        &goals,
        &PlannerConfig::default(),
    );

    assert!(!found.is_empty(), "no plan for a reachable goal");

    let goal = Stockpile::new(Good::Wood, 4);
    let mut state = VillagerState::default();
    let mut total: u64 = 0;

    for action in &found.actions {
        assert!(
            action.prerequisite(&state, &world),
            "{action:?} is not applicable at this point in its own plan"
        );
        total = total.saturating_add(action.cost(&state, &world));
        state = action.act(&state, &world);
    }

    assert!(
        goal.is_satisfied(&state, &world),
        "replaying the plan left the goal unmet: {:?}",
        state.carrying
    );
    assert_eq!(total, found.cost, "replayed cost differs from planned cost");
}

/// `VillagerState::taken` exists to stop the planner harvesting one instance forever. A
/// single tree yields exactly one log, so a two-log goal has to be unreachable.
#[test]
fn a_plan_cannot_work_the_same_instance_twice() {
    let world = World::new(&[Item::new(Resource::Tree, (5, 5))], 48);
    let config = PlannerConfig::default();

    assert!(
        !plan(&VillagerState::default(), &world, &wood_goal(1), &config).is_empty(),
        "one tree should yield one log"
    );
    assert!(
        plan(&VillagerState::default(), &world, &wood_goal(2), &config).is_empty(),
        "one tree yielded two logs"
    );
}

/// Goals are tried in descending utility, and a goal nothing can satisfy must fall
/// through to the next rather than leaving the villager with nothing to do.
#[test]
fn an_unreachable_goal_falls_through_to_the_next() {
    let world = World::new(
        &[
            Item::new(Resource::Tree, (5, 5)),
            Item::new(Resource::Berry, (7, 7)),
        ],
        48,
    );
    let goals: Vec<Box<dyn Goal<Village>>> = vec![
        // Higher utility, but only one tree exists.
        Box::new(Stockpile::new(Good::Wood, 9)),
        Box::new(Stockpile::new(Good::Berry, 1)),
    ];

    let found = plan(
        &VillagerState::default(),
        &world,
        &goals,
        &PlannerConfig::default(),
    );
    assert_eq!(
        found.goal,
        Some("berry"),
        "planner gave up instead of trying berries"
    );

    let wood = found.considered.iter().find(|c| c.goal == "wood");
    assert!(
        wood.is_some(),
        "the wood goal was not recorded as considered"
    );
    let Some(wood) = wood else { return };
    assert_eq!(wood.outcome, Consideration::Unreachable);
}

/// Runs a village to a standstill, or gives up. Returns whether it settled, so a test can
/// say so rather than silently asserting against a half-finished run.
fn run_until_settled(simulation: &mut Simulation, limit: u64) -> bool {
    for _ in 0..limit {
        simulation.advance();
        // Checked every step rather than at the end, where every pack is empty anyway.
        assert_capacity_is_respected(simulation);
        if simulation.idle_villagers() == simulation.villagers.len() {
            return true;
        }
    }
    false
}

/// Every tree felled becomes exactly one log, every log gathered becomes exactly one wood, and
/// storing a load moves it without changing it. So trees lost must equal wood stored plus wood
/// still in packs plus logs left lying about, whoever felled what.
///
/// This is the invariant that matters with a village rather than a soloist: the planner's `act`
/// and the villager's real world changes are separate code, and eight villagers mutating one
/// world is where they would drift apart.
fn assert_wood_is_conserved(simulation: &Simulation, trees_before: u64) {
    let felled = trees_before - simulation.world.count(Resource::Tree);
    let stored = simulation.stored(Good::Wood);
    let packed: u64 = simulation
        .villagers
        .iter()
        .map(|villager| u64::from(villager.state.carrying.count(Good::Wood)))
        .sum();
    let stranded = u64::try_from(simulation.world.logs.len()).unwrap_or(u64::MAX);

    assert_eq!(
        felled,
        stored + packed + stranded,
        "{felled} trees felled but {stored} stored, {packed} in packs, {stranded} stranded"
    );
}

/// Capacity is only meaningful if it actually binds, so check no villager ever exceeds it.
fn assert_capacity_is_respected(simulation: &Simulation) {
    for villager in &simulation.villagers {
        let load = villager.state.carrying.total();
        assert!(
            load <= CARRY_CAPACITY,
            "villager {} is carrying {load} of a possible {CARRY_CAPACITY}",
            villager.id
        );
    }
}

#[test]
fn one_villager_carries_out_what_the_planner_intended() {
    let mut simulation = Simulation::new(SEED, 1);
    let trees = simulation.world.count(Resource::Tree);

    assert!(
        run_until_settled(&mut simulation, 200_000),
        "one villager never settled"
    );
    assert_wood_is_conserved(&simulation, trees);

    assert_eq!(
        simulation.stats.refinement_failures, 0,
        "nobody to be beaten to a tree, so nothing should have failed"
    );
    assert!(
        simulation.world.logs.is_empty(),
        "a lone villager abandoned logs: {:?}",
        simulation.world.logs
    );

    let Some(villager) = simulation.villagers.first() else {
        return;
    };
    assert_eq!(
        villager.render_position, villager.state.position,
        "the drawn position never caught up with the planner's"
    );
}

/// The point of the exercise: a village of villagers who each want different things all
/// reach a standstill, without conjuring or destroying anything on the way.
#[test]
fn a_village_settles_without_losing_track_of_anything() {
    let mut simulation = Simulation::new(SEED, 8);
    let trees = simulation.world.count(Resource::Tree);

    assert!(
        run_until_settled(&mut simulation, 400_000),
        "{} of 8 villagers were still busy",
        8 - simulation.idle_villagers()
    );
    assert_wood_is_conserved(&simulation, trees);
    assert!(
        simulation.stats.empty_plans > 0,
        "no villager ever ran out of things to want"
    );
}

/// A villager beaten to a tree must not spend the rest of the run attempting the same
/// impossible action. Nothing changes in a settled world, so its failure count cannot move.
#[test]
fn contention_does_not_become_an_endless_retry() {
    let mut simulation = Simulation::new(SEED, 8);
    assert!(
        run_until_settled(&mut simulation, 400_000),
        "the village never settled"
    );

    let settled = simulation.stats.refinement_failures;
    for _ in 0..50_000 {
        simulation.advance();
    }

    assert_eq!(
        simulation.stats.refinement_failures, settled,
        "a settled village went on failing refinements"
    );
}

/// Villagers are born apart and wanting different things, or a village is just one villager
/// copied eight times, and they would all walk to the same tree.
#[test]
fn villagers_are_born_different() {
    let simulation = Simulation::new(SEED, 8);

    let positions: std::collections::HashSet<_> = simulation
        .villagers
        .iter()
        .map(|villager| villager.state.position)
        .collect();
    assert!(
        positions.len() > 1,
        "every villager spawned on the same spot"
    );

    let mut settled = Simulation::new(SEED, 8);
    run_until_settled(&mut settled, 400_000);
    let packs: std::collections::HashSet<_> = settled
        .villagers
        .iter()
        .map(|villager| villager.state.stored)
        .collect();
    assert!(
        packs.len() > 1,
        "every villager ended up carrying exactly the same things"
    );
}

/// Seeded generation is what makes a headless run something you can diff.
#[test]
fn a_seed_reproduces_a_run() {
    let mut first = Simulation::new(SEED, 4);
    let mut second = Simulation::new(SEED, 4);
    let mut different = Simulation::new(SEED ^ 1, 4);

    for _ in 0..3_000 {
        first.advance();
        second.advance();
        different.advance();
    }

    let states = |simulation: &Simulation| -> Vec<_> {
        simulation
            .villagers
            .iter()
            .map(|villager| (villager.state, villager.render_position))
            .collect()
    };

    assert_eq!(states(&first), states(&second), "same seed diverged");
    assert_eq!(first.stats.expansions, second.stats.expansions);
    assert_ne!(
        states(&different),
        states(&first),
        "different seeds produced the same village"
    );
}

/// The world is built before anyone is spawned into it, so population must not disturb it.
#[test]
fn population_does_not_change_the_world() {
    let lone = Simulation::new(SEED, 1);
    let crowd = Simulation::new(SEED, 12);

    for resource in Resource::ALL {
        assert_eq!(
            lone.world.count(resource),
            crowd.world.count(resource),
            "{resource:?} differed with population"
        );
    }
}

/// The library hands back numbers rather than printing them, which makes a whole headless
/// run assertable in one line, and reproducible enough to compare two runs directly.
#[test]
fn a_headless_run_reports_what_happened() {
    let report = crate::run_headless(SEED, 8, 60_000);

    assert_eq!(report.seed, SEED);
    assert_eq!(report.steps, 60_000);
    assert_eq!(report.villagers, 8);
    assert_eq!(report.idle_villagers, 8, "the village never settled");
    assert!(
        report.wood + report.stone + report.berries > 0,
        "the village gathered nothing at all"
    );
    assert!(
        report.empty_plans > 0,
        "no villager ever ran out of things to want"
    );
    assert_eq!(
        report,
        crate::run_headless(SEED, 8, 60_000),
        "the same seed produced a different report"
    );
}
