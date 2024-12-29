/// The objective of GOAP is for an `Agent` to find a way from the current `State` -> goal `State` through `Action`s.

// State MUST be all-encompassing
// e.g. For an agent to pick something up; the information about the item, where it is, and the agent's inventory must all be included in the state.
// So we need some process for constructing and deconstructing the state for each agent.
// - States can then be augmented with agent perception.
pub(crate) trait State: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash {
    fn compare(&self, other_state: &Self) -> bool {
        self == other_state
    }
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
//
// WARN: I've tried a million times to implement `Action` as a trait object but cannot manage it hence `ActionEnum`. A mixture of the pathfinding
// crate (because of Node requiring Eq which is not object safe) and other things have forced this into an Enum.
pub(crate) trait ActionEnum<S: State>: Action<S> {
    fn generate_available_actions(current_state: &S) -> Vec<Self>;
}

pub(crate) trait Action<S: State>:
    std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
{
    fn act(&self, current_state: S) -> S;

    fn cost(&self) -> u64;

    fn prerequisite(&self, _current_state: &S) -> bool;
}

pub(crate) trait Goal<S: State> {
    fn priority(&self, current_state: &S) -> i64;
    fn goal_state(&self, current_state: S) -> S;
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node<S: State, SA: ActionEnum<S>> {
    state: S,
    action: Option<SA>,
}

pub(crate) fn plan<S: State, SA: ActionEnum<S>>(
    current_state: S,
    goals: &[Box<dyn Goal<S>>],
) -> Option<Vec<SA>> {
    // 1. Generate a directed graph sensibly, stopping when exusted with the PATIENCE value.
    // 2. Use some algorithm to find the shortest path from current_state to goal_state.
    // 3. Return that path as a Plan.

    let goal_state = goals
        .iter()
        .max_by_key(|g| g.priority(&current_state))?
        .goal_state(current_state.clone());

    println!("Goal: {:?}", goal_state);

    let start = Node {
        state: current_state,
        action: None,
    };

    println!("Start planning...");
    let best_path_option =
        pathfinding::directed::astar::astar(&start, successors, heuristic, |node| {
            success(&node.state, &goal_state)
        });
    println!("Plan complete!");

    if let Some((best_path, _)) = best_path_option {
        let actions = best_path
            .into_iter()
            .filter_map(|node| node.action)
            .collect();

        Some(actions)
    } else {
        None
    }
}

fn successors<S: State, SA: ActionEnum<S>>(node: &Node<S, SA>) -> Vec<(Node<S, SA>, u64)> {
    let available_actions: Vec<SA> = SA::generate_available_actions(&node.state);
    available_actions
        .iter()
        .map(|agent_action| {
            let new_state = agent_action.act(node.state.clone());
            let cost = agent_action.cost();
            let next_node = Node {
                state: new_state,
                action: Some(agent_action.clone()),
            };

            (next_node, cost)
        })
        .collect()
}

fn heuristic<S: State, SA: ActionEnum<S>>(_: &Node<S, SA>) -> u64 {
    0
}

fn success<S: State>(state: &S, goal_state: &S) -> bool {
    state.compare(goal_state)
}

pub(crate) fn print_plan<S: State, SA: ActionEnum<S>>(plan: Vec<SA>) {
    for agent_action in plan {
        println!("---------------------------");
        println!("Action: {:#?}", agent_action);
    }
    println!("---------------------------");
}
