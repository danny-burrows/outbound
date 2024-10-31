/// The objective of GOAP is for an `Agent` to find a way from the current `State` -> goal `State` through `Action`s.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent {
    position: (i64, i64),
    inventory: Vec<String>,
}

impl Agent {
    pub fn new(position: (i64, i64)) -> Agent {
        Agent {
            position,
            inventory: vec![],
        }
    }
}

// State MUST be all-encompassing
// e.g. For an agent to pick something up; the information about the item, where it is, and the agent's inventory must all be included in the state.
// So we need some process for constructing and deconstructing the state for each agent.
// - States can then be augmented with agent perception.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    agent: Agent,
    items: Vec<Item>,
}

impl State {
    pub fn construct(agent: &Agent, items: &Vec<Item>) -> State {
        State {
            agent: agent.clone(),
            items: items.clone(),
        }
    }
}

pub fn generate_goal_state(current_state: &State) -> State {
    let mut goal_state = current_state.clone();
    goal_state.agent.inventory.push("wood".to_string());
    goal_state.agent.inventory.push("stone".to_string());
    goal_state.agent.inventory.push("wood".to_string());
    goal_state.agent.inventory.push("berry".to_string());
    goal_state
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentAction {
    MoveAction(MoveAction),
    PickUpAction(PickUpAction),
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
trait Action: Clone {
    fn act(&self, current_state: State) -> State;

    fn cost(&self) -> u64;

    fn prerequisite(&self, _current_state: &State) -> Option<bool> {
        None
    }
}

pub fn generate_available_actions(agent: Agent, items: Vec<Item>) -> Vec<AgentAction> {
    let mut actions = vec![];

    for i in (-1..2) {
        for j in (-1..2) {
            actions.push(AgentAction::MoveAction(MoveAction {
                delta_x: i,
                delta_y: j,
            }));
        }
    }

    for item in items {
        // actions.push(AgentAction::MoveAction(MoveAction {
        //     delta_x: item.position.0 - agent.position.0,
        //     delta_y: item.position.1 - agent.position.1,
        // }));

        actions.push(AgentAction::PickUpAction(PickUpAction {
            item: item.clone(),
        }));
    }

    actions
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MoveAction {
    delta_x: i64,
    delta_y: i64,
}

impl Action for MoveAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();
        new_state.agent.position = (
            new_state.agent.position.0 + self.delta_x,
            new_state.agent.position.1 + self.delta_y,
        );
        new_state
    }

    fn cost(&self) -> u64 {
        1
        // (self.delta_x.pow(2) + self.delta_y.pow(2)).unsigned_abs()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    position: (i64, i64),
    id: String,
}

impl Item {
    pub fn new(id: String, position: (i64, i64)) -> Item {
        Item { id, position }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PickUpAction {
    item: Item,
}

impl Action for PickUpAction {
    fn act(&self, current_state: State) -> State {
        if self.prerequisite(&current_state) == Some(false) {
            return current_state;
        }

        let mut new_state = current_state.clone();
        new_state.agent.inventory.push(self.item.id.clone());
        new_state
    }

    fn cost(&self) -> u64 {
        // Arbitrary cost to pick up an item. Maybe this should be weight?
        1
    }

    fn prerequisite(&self, current_state: &State) -> Option<bool> {
        Some(current_state.agent.position == self.item.position)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    state: State,
    action: Option<AgentAction>,
}

pub fn plan(
    current_state: State,
    available_actions: Vec<AgentAction>,
    goal_state: State,
) -> Option<Vec<AgentAction>> {
    // 1. Generate a directed graph sensibly, stopping when exusted with the PATIENCE value.
    // 2. Use some algorithm to find the shortest path from current_state to goal_state.
    // 3. Return that path as a Plan.

    let start = Node {
        state: current_state,
        action: None,
    };

    println!("Test");
    let best_path_option = pathfinding::directed::astar::astar(
        &start,
        |node| successors(&node.state, &available_actions),
        |_| heuristic(),
        |node| success(&node.state, &goal_state),
    );
    println!("Test");

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

fn successors(state: &State, available_actions: &[AgentAction]) -> Vec<(Node, u64)> {
    let mut successors = vec![];
    for action in available_actions.into_iter() {
        let next_node = match action {
            agent_action @ AgentAction::MoveAction(move_action) => (
                Node {
                    state: move_action.act(state.clone()),
                    action: Some(agent_action.to_owned()),
                },
                move_action.cost(),
            ),
            agent_action @ AgentAction::PickUpAction(pick_up_action) => (
                Node {
                    state: pick_up_action.act(state.clone()),
                    action: Some(agent_action.to_owned()),
                },
                pick_up_action.cost(),
            ),
        };

        successors.push(next_node);
    }

    successors
}

fn heuristic() -> u64 {
    1
}

fn success(state: &State, goal_state: &State) -> bool {
    // println!("S {:?}", state);
    // println!("G {:?}", goal_state);
    state == goal_state
}

pub fn print_plan(plan: Vec<AgentAction>) {
    for agent_action in plan {
        println!("---------------------------");
        println!("Action: {:#?}", agent_action);
    }
}
