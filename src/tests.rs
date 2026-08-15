use crate::Executor;
use crate::goals::Collect;
use crate::goap::{Action, Goal, PlannerConfig, plan};
use crate::item::{Good, Item, Resource};
use crate::village::{PlanState, Village};
use crate::world::World;

const EXTENT: u64 = 48;

/// Deterministic scatter, so the expansion counts below are reproducible without
/// dragging the renderer's RNG into a test.
struct Lcg(u64);

impl Lcg {
    const fn step(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn position(&mut self) -> (i64, i64) {
        let span = EXTENT.saturating_add(1);
        let x = i64::try_from(self.step() % span).unwrap_or(0);
        let y = i64::try_from(self.step() % span).unwrap_or(0);
        (x, y)
    }
}

fn fixture() -> World {
    let mut rng = Lcg(0x0BAD_C0DE);
    let mut items = Vec::new();
    for (resource, count) in [
        (Resource::Tree, 14),
        (Resource::Stone, 8),
        (Resource::Berry, 10),
    ] {
        for _ in 0..count {
            items.push(Item::new(resource, rng.position()));
        }
    }
    World::new(&items, EXTENT)
}

fn wood_goal(target: u8) -> Vec<Box<dyn Goal<Village>>> {
    vec![Box::new(Collect::new(Good::Wood, target))]
}

/// The measurement that justifies the whole change: a `heuristic_weight` of 0 is the
/// old behaviour, where `heuristic()` returned a constant zero and A* degraded to
/// uniform-cost search.
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

    assert!(blind.is_some(), "uniform-cost search found no plan");
    assert!(guided.is_some(), "guided search found no plan");
    let (Some(blind), Some(guided)) = (blind, guided) else {
        return;
    };

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

/// The same comparison at the size the simulation actually runs. The gap widens with
/// the world: guiding the search matters more, not less, as there is more to search.
#[test]
#[expect(
    clippy::print_stdout,
    reason = "the numbers are the point of this test; `cargo test -- --nocapture` shows them"
)]
fn the_gap_widens_with_world_size() {
    let mut rng = Lcg(0x5EED);
    let mut items = Vec::new();
    for (resource, count) in [
        (Resource::Tree, 250),
        (Resource::Stone, 25),
        (Resource::Berry, 50),
    ] {
        for _ in 0..count {
            let span = 151;
            let x = i64::try_from(rng.step() % span).unwrap_or(0);
            let y = i64::try_from(rng.step() % span).unwrap_or(0);
            items.push(Item::new(resource, (x, y)));
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

    assert!(blind.is_some(), "uniform-cost search found no plan");
    assert!(guided.is_some(), "guided search found no plan");
    let (Some(blind), Some(guided)) = (blind, guided) else {
        return;
    };

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

    assert!(found.is_some(), "no plan for a reachable goal");
    let Some(found) = found else { return };

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
    let world = World::new(&[Item::new(Resource::Tree, (5, 5))], EXTENT);
    let config = PlannerConfig::default();

    assert!(
        plan(&PlanState::default(), &world, &wood_goal(1), &config).is_some(),
        "one tree should yield one log"
    );
    assert!(
        plan(&PlanState::default(), &world, &wood_goal(2), &config).is_none(),
        "one tree yielded two logs"
    );
}

/// The riskiest seam in splitting planning state from world state: the planner's `act`
/// and the executor's real world changes are now separate code, and nothing but a test
/// stops them drifting apart.
#[test]
fn the_executor_carries_out_what_the_planner_intended() {
    let mut world = fixture();
    let trees = world.count(Resource::Tree);
    let mut state = PlanState::default();
    let mut executor = Executor::default();
    let goals = wood_goal(5);
    let config = PlannerConfig::default();

    // Generous: each action takes a tick, and each walked cell takes one more.
    for _ in 0..20_000 {
        if state.carrying.count(Good::Wood) >= 5 {
            break;
        }
        executor.advance(&mut world, &mut state, &goals, &config);
    }

    assert_eq!(
        state.carrying.count(Good::Wood),
        5,
        "executing the plans did not fill the pack"
    );
    assert_eq!(
        world.count(Resource::Tree),
        trees - 5,
        "the world did not lose exactly the trees the villager felled"
    );
    assert!(
        world.logs.is_empty(),
        "logs were felled and never gathered: {:?}",
        world.logs
    );
    assert_eq!(
        world.render_position, state.position,
        "the drawn position never caught up with the planner's"
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
        EXTENT,
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
    assert!(found.is_some(), "planner gave up instead of trying berries");
    let Some(found) = found else { return };
    assert_eq!(found.goal, "berry");
}
