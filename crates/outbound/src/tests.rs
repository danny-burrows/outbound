use crate::goals::Collect;
use crate::item::{Good, Item, Resource};
use crate::rng::Rng;
use crate::sim::{Simulation, WORLD_EXTENT};
use crate::village::{PlanState, Village};
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
    vec![Box::new(Collect::new(Good::Wood, target))]
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
    let state = PlanState::default();
    let goals = wood_goal(3);

    let generous = 2_000_000;
    let blind = plan(
        &state,
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 0,
            max_expansions: generous,
        },
    );
    let guided = plan(
        &state,
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 1,
            max_expansions: generous,
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
        &PlanState::default(),
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 0,
            max_expansions: generous,
        },
    );
    let guided = plan(
        &PlanState::default(),
        &world,
        &goals,
        &PlannerConfig {
            heuristic_weight: 1,
            max_expansions: generous,
        },
    );

    assert!(!blind.is_empty(), "uniform-cost search found no plan");
    assert!(!guided.is_empty(), "guided search found no plan");

    println!(
        "10 wood among 250 trees | uniform-cost: {:>7} expansions, cost {} | guided: {:>7}, cost {}",
        blind.expansions, blind.cost, guided.expansions, guided.cost
    );

    assert_eq!(guided.cost, blind.cost, "heuristic cost optimality");
    assert_eq!(
        guided.actions.len(),
        30,
        "10 wood needs 10 rounds of travel, fell, gather"
    );
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
        &PlanState::default(),
        &world,
        &goals,
        &PlannerConfig::default(),
    );

    assert!(!found.is_empty(), "no plan for a reachable goal");

    let goal = Collect::new(Good::Wood, 4);
    let mut state = PlanState::default();
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

/// `PlanState::taken` exists to stop the planner harvesting one instance forever. A
/// single tree yields exactly one log, so a two-log goal has to be unreachable.
#[test]
fn a_plan_cannot_work_the_same_instance_twice() {
    let world = World::new(&[Item::new(Resource::Tree, (5, 5))], 48);
    let config = PlannerConfig::default();

    assert!(
        !plan(&PlanState::default(), &world, &wood_goal(1), &config).is_empty(),
        "one tree should yield one log"
    );
    assert!(
        plan(&PlanState::default(), &world, &wood_goal(2), &config).is_empty(),
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
        Box::new(Collect::new(Good::Wood, 9)),
        Box::new(Collect::new(Good::Berry, 1)),
    ];

    let found = plan(
        &PlanState::default(),
        &world,
        &goals,
        &PlannerConfig::default(),
    );
    assert_eq!(
        found.goal,
        Some("berry"),
        "planner gave up instead of trying berries"
    );

    // The trace should say so, which is the whole point of keeping it.
    let wood = found.considered.iter().find(|c| c.goal == "wood");
    assert!(
        wood.is_some(),
        "the wood goal was not recorded as considered"
    );
    let Some(wood) = wood else { return };
    assert_eq!(wood.outcome, Consideration::Unreachable);
}

/// The riskiest seam in splitting planning state from world state: the planner's `act`
/// and the simulation's real world changes are separate code, and nothing but a test
/// stops them drifting apart.
#[test]
fn the_simulation_carries_out_what_the_planner_intended() {
    let mut simulation = Simulation::new(SEED);
    let trees = simulation.world.count(Resource::Tree);

    for _ in 0..200_000 {
        if simulation.state.carrying.count(Good::Wood) >= 10 {
            break;
        }
        simulation.advance();
    }

    assert_eq!(
        simulation.state.carrying.count(Good::Wood),
        10,
        "executing the plans did not fill the pack"
    );
    assert_eq!(
        simulation.world.count(Resource::Tree),
        trees - 10,
        "the world did not lose exactly the trees the villager felled"
    );
    assert!(
        simulation.world.logs.is_empty(),
        "logs were felled and never gathered: {:?}",
        simulation.world.logs
    );
    assert_eq!(
        simulation.stats.refinement_failures, 0,
        "an abstract action could not be carried out against the real world"
    );
    assert_eq!(
        simulation.world.render_position, simulation.state.position,
        "the drawn position never caught up with the planner's"
    );
}

/// Every goal is reachable in a default world, so a long enough run must finish them all
/// and then settle, exercising the idle path rather than replanning every step.
#[test]
fn a_long_run_satisfies_every_goal_and_settles() {
    let mut simulation = Simulation::new(SEED);
    for _ in 0..100_000 {
        simulation.advance();
    }

    let carrying = simulation.state.carrying;
    assert_eq!(carrying.count(Good::Wood), 10, "wood");
    assert_eq!(carrying.count(Good::Stone), 10, "stone");
    assert_eq!(carrying.count(Good::Berry), 10, "berries");
    assert!(
        simulation.stats.empty_plans > 0,
        "the villager never ran out of things to want"
    );
    assert_eq!(simulation.stats.refinement_failures, 0);
}

/// Seeded generation is what makes a headless run something you can diff.
#[test]
fn a_seed_reproduces_a_run() {
    let mut first = Simulation::new(SEED);
    let mut second = Simulation::new(SEED);
    let mut different = Simulation::new(SEED ^ 1);

    for _ in 0..3_000 {
        first.advance();
        second.advance();
        different.advance();
    }

    assert_eq!(first.state, second.state, "same seed diverged");
    assert_eq!(first.world.render_position, second.world.render_position);
    assert_eq!(first.stats.expansions, second.stats.expansions);
    assert_ne!(
        different.world.render_position, first.world.render_position,
        "different seeds produced the same world"
    );
}

/// The library hands back numbers rather than printing them, which makes a whole headless
/// run assertable in one line — and reproducible enough to compare two runs directly.
#[test]
fn a_headless_run_reports_what_happened() {
    let report = crate::run_headless(SEED, 20_000);

    assert_eq!(report.seed, SEED);
    assert_eq!(report.steps, 20_000);
    assert_eq!((report.wood, report.stone, report.berries), (10, 10, 10));
    assert_eq!(report.refinement_failures, 0);
    assert!(
        report.empty_plans > 0,
        "the villager never ran out of things to want"
    );
    assert_eq!(
        report,
        crate::run_headless(SEED, 20_000),
        "the same seed produced a different report"
    );
}
