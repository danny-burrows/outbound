/// The objective of GOAP is for an `Agent` to find a way from the current `State` -> goal `State` through `Action`s.

// State MUST be all-encompassing
// e.g. For an agent to pick something up; the information about the item, where it is, and the agent's inventory must all be included in the state.
// So we need some process for constructing and deconstructing the state for each agent.
// - States can then be augmented with agent perception.
trait State: std::fmt::Debug + Clone + PartialEq + Eq {
    fn generate_available_actions(&self) -> GenericActions<Self>;
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
trait Action<S: State> {
    fn act(&self, current_state: S) -> S;

    fn cost(&self) -> u64;

    fn prerequisite(&self, _current_state: &S) -> bool;
}

struct GenericAction<S: State>(Box<dyn Action<S>>);
struct GenericActions<S: State>(Vec<GenericAction<S>>);

struct Node<S: State> {
    state: S,
    available_actions: GenericActions<S>,
    action: Option<GenericAction<S>>,
}

pub fn plan<S: State>(current_state: S, goal_state: S) -> Option<GenericActions<S>> {
    // 1. Generate a directed graph sensibly, stopping when exusted with the PATIENCE value.
    // 2. Use some algorithm to find the shortest path from current_state to goal_state.
    // 3. Return that path as a Plan.

    let start = Node {
        state: current_state.clone(),
        available_actions: current_state.generate_available_actions(),
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
            .filter_map(|node| node.action.clone())
            .collect();

        Some(actions)
    } else {
        None
    }
}

fn successors<S: State>(node: &Node<S>) -> Vec<(Node<S>, u64)> {
    node.available_actions
        .iter()
        .map(|agent_action| {
            let new_state = agent_action.act(node.state.clone());
            let new_available_actions = new_state.generate_available_actions();

            let cost = agent_action.cost();
            let next_node = Node {
                state: new_state,
                available_actions: new_available_actions,
                action: Some(agent_action.clone()),
            };

            (next_node, cost)
        })
        .collect()
}

fn heuristic<S: State>(_: &Node<S>) -> u64 {
    0
}

fn success<S: State>(state: &S, goal_state: &S) -> bool {
    state == goal_state
}

pub fn print_plan<S: State>(plan: GenericActions<S>) {
    for agent_action in plan.0 {
        println!("---------------------------");
        println!("Action: {:#?}", agent_action);
    }
    println!("---------------------------");
}
