use crate::{Action, Consideration, Domain, Goal, PlannerConfig, plan};

/// A deliberately tiny domain, which is half the point: implementing [`Domain`] from
/// outside should need nothing that is not public.
struct Counter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Count(u32);

/// Context can be as small as a ceiling. It is borrowed, never copied per node.
struct Limit(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Step {
    Increment,
    Double,
}

impl Domain for Counter {
    type State = Count;
    type Context = Limit;
    type Action = Step;
}

impl Action<Counter> for Step {
    fn cost(&self, _: &Count, _: &Limit) -> u64 {
        1
    }

    fn prerequisite(&self, state: &Count, limit: &Limit) -> bool {
        match *self {
            Self::Increment => state.0 < limit.0,
            Self::Double => state.0 > 0 && state.0.saturating_mul(2) <= limit.0,
        }
    }

    fn act(&self, state: &Count, _: &Limit) -> Count {
        match *self {
            Self::Increment => Count(state.0.saturating_add(1)),
            Self::Double => Count(state.0.saturating_mul(2)),
        }
    }

    fn available(state: &Count, limit: &Limit, out: &mut Vec<Self>) {
        for step in [Self::Increment, Self::Double] {
            if step.prerequisite(state, limit) {
                out.push(step);
            }
        }
    }
}

struct Reach(u32);

impl Goal<Counter> for Reach {
    fn name(&self) -> &'static str {
        "reach"
    }

    fn utility(&self, _: &Count, _: &Limit) -> i64 {
        1
    }

    fn unmet(&self, state: &Count, _: &Limit) -> u32 {
        self.0.saturating_sub(state.0)
    }

    /// The default estimate, one unit-cost action per outstanding condition, would
    /// claim a gap of 16 needs 16 actions when doubling closes it in five. Overshooting
    /// the truth is exactly what costs admissibility, so the domain says what it knows:
    /// doubling is the fastest route, and a gap needs at least its logarithm in steps.
    fn estimate(&self, state: &Count, _: &Limit) -> u64 {
        let mut reached = state.0.max(1);
        let mut steps = u64::from(state.0 == 0);
        while reached < self.0 {
            reached = reached.saturating_mul(2);
            steps = steps.saturating_add(1);
        }
        steps
    }
}

fn goals(target: u32) -> Vec<Box<dyn Goal<Counter>>> {
    vec![Box::new(Reach(target))]
}

#[test]
fn an_admissible_estimate_still_finds_the_cheapest_plan() {
    let found = plan(&Count(0), &Limit(32), &goals(16), &PlannerConfig::default());

    // Increment once, then double four times. Sixteen increments would also arrive.
    assert_eq!(found.goal, Some("reach"));
    assert_eq!(found.cost, 5, "{:?}", found.actions);
    assert_eq!(found.actions.front(), Some(&Step::Increment));
}

#[test]
fn a_satisfied_goal_is_never_planned_for() {
    let found = plan(&Count(9), &Limit(32), &goals(4), &PlannerConfig::default());

    assert!(found.is_empty());
    assert_eq!(found.goal, None);
    assert_eq!(found.expansions, 0, "a satisfied goal cost a search");
    assert_eq!(
        found.considered.first().map(|c| c.outcome),
        Some(Consideration::Satisfied)
    );
}

#[test]
fn an_exhausted_budget_reports_an_unreachable_goal() {
    let found = plan(
        &Count(0),
        &Limit(1_000_000),
        &goals(999_999),
        &PlannerConfig {
            heuristic_weight: 1,
            max_expansions: 4,
        },
    );

    assert!(found.is_empty(), "the budget did not bound the search");
    // Overshoots the budget of 4 by however many nodes were already queued, since
    // draining the frontier still counts. Bounded and cheap, but not exact.
    assert!(
        found.expansions < 50,
        "the budget barely bounded anything: {} expansions",
        found.expansions
    );
    assert_eq!(
        found.considered.first().map(|c| c.outcome),
        Some(Consideration::Unreachable)
    );
}

#[test]
fn goals_are_tried_in_descending_utility() {
    struct Fixed(&'static str, i64, u32);
    impl Goal<Counter> for Fixed {
        fn name(&self) -> &'static str {
            self.0
        }
        fn utility(&self, _: &Count, _: &Limit) -> i64 {
            self.1
        }
        fn unmet(&self, state: &Count, _: &Limit) -> u32 {
            self.2.saturating_sub(state.0)
        }
    }

    // Declared lowest-utility first, so ordering cannot be mistaken for declaration order.
    let goals: Vec<Box<dyn Goal<Counter>>> =
        vec![Box::new(Fixed("meek", 1, 3)), Box::new(Fixed("keen", 9, 3))];
    let found = plan(&Count(0), &Limit(32), &goals, &PlannerConfig::default());

    assert_eq!(found.goal, Some("keen"));
    let outcomes: Vec<_> = found.considered.iter().map(|c| c.outcome).collect();
    assert_eq!(
        outcomes,
        vec![Consideration::NotTried, Consideration::Chosen]
    );
}
