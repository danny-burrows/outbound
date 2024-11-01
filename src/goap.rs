/// The objective of GOAP is for an `Agent` to find a way from the current `State` -> goal `State` through `Action`s.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent {
    pub position: (i64, i64),
    pub inventory: Vec<String>,
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
    pub agent: Agent,
    pub items: Vec<Item>,
}

impl State {
    pub fn construct(agent: &Agent, items: &[Item]) -> State {
        State {
            agent: agent.clone(),
            items: items.to_owned(),
        }
    }
}

pub fn generate_goal_state(current_state: &State) -> State {
    let mut goal_state = current_state.clone();
    goal_state.agent.inventory.push("wood".to_string());
    goal_state.agent.inventory.push("stone".to_string());
    goal_state.agent.inventory.push("berry".to_string());
    goal_state.agent.inventory.push("wood".to_string());
    goal_state
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentAction {
    MoveAction(MoveAction),
    PickUpAction(PickUpAction),
    ChopTreeAction(ChopTreeAction),
}

impl Action for AgentAction {
    fn act(&self, current_state: State) -> State {
        match self {
            AgentAction::MoveAction(move_action) => move_action.act(current_state),
            AgentAction::PickUpAction(pick_up_action) => pick_up_action.act(current_state),
            AgentAction::ChopTreeAction(chop_tree_action) => chop_tree_action.act(current_state),
        }
    }

    fn cost(&self) -> u64 {
        match self {
            AgentAction::MoveAction(move_action) => move_action.cost(),
            AgentAction::PickUpAction(pick_up_action) => pick_up_action.cost(),
            AgentAction::ChopTreeAction(chop_tree_action) => chop_tree_action.cost(),
        }
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        match self {
            AgentAction::MoveAction(move_action) => move_action.prerequisite(current_state),
            AgentAction::PickUpAction(pick_up_action) => pick_up_action.prerequisite(current_state),
            AgentAction::ChopTreeAction(chop_tree_action) => {
                chop_tree_action.prerequisite(current_state)
            }
        }
    }
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
pub trait Action: Clone {
    fn act(&self, current_state: State) -> State;

    fn cost(&self) -> u64;

    fn prerequisite(&self, _current_state: &State) -> bool;
}

pub fn generate_available_actions(current_state: &State) -> Vec<AgentAction> {
    let mut available_actions = vec![];

    // All directions of movement - should constrain to the map.
    for i in -1..2 {
        for j in -1..2 {
            let move_action = AgentAction::MoveAction(MoveAction {
                delta_x: i,
                delta_y: j,
            });

            if move_action.prerequisite(current_state) {
                available_actions.push(move_action);
            }
        }
    }

    for item in current_state.items.clone() {
        let action = if item.id == *"tree" {
            AgentAction::ChopTreeAction(ChopTreeAction { item: item.clone() })
        } else {
            AgentAction::PickUpAction(PickUpAction { item: item.clone() })
        };

        if action.prerequisite(current_state) {
            available_actions.push(action);
        }
    }

    available_actions
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveAction {
    delta_x: i64,
    delta_y: i64,
}

impl MoveAction {
    fn get_new_position(&self, current_state: &State) -> (i64, i64) {
        let (current_x, current_y) = current_state.agent.position;
        (current_x + self.delta_x, current_y + self.delta_y)
    }
}

impl Action for MoveAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();
        new_state.agent.position = self.get_new_position(&current_state);
        new_state
    }

    fn cost(&self) -> u64 {
        1
        // (self.delta_x.pow(2) + self.delta_y.pow(2)).unsigned_abs()
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        // Check that new position is not out of bounds
        let new_position = self.get_new_position(current_state);
        new_position.0 > -1 && new_position.0 < 11 && new_position.1 > -1 && new_position.1 < 11
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub position: (i64, i64),
    pub id: String,
}

impl Item {
    pub fn new(id: String, position: (i64, i64)) -> Item {
        Item { id, position }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PickUpAction {
    item: Item,
}

impl Action for PickUpAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();
        new_state.agent.inventory.push(self.item.id.clone());

        if let Some(i) = new_state.items.iter().position(|item| item == &self.item) {
            new_state.items.remove(i);
        }

        new_state
    }

    fn cost(&self) -> u64 {
        // Arbitrary cost to pick up an item. Maybe this should be weight?
        1
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        current_state.agent.position == self.item.position && self.item.id != *"tree"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChopTreeAction {
    item: Item,
}

impl Action for ChopTreeAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();

        let wood = Item {
            position: self.item.position,
            id: "wood".into(),
        };
        new_state.items.push(wood);

        if let Some(i) = new_state.items.iter().position(|item| item == &self.item) {
            new_state.items.remove(i);
        }

        new_state
    }

    fn cost(&self) -> u64 {
        1
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        current_state.agent.position == self.item.position
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    state: State,
    available_actions: Vec<AgentAction>,
    action: Option<AgentAction>,
}

pub fn plan(current_state: State, goal_state: State) -> Option<Vec<AgentAction>> {
    // 1. Generate a directed graph sensibly, stopping when exusted with the PATIENCE value.
    // 2. Use some algorithm to find the shortest path from current_state to goal_state.
    // 3. Return that path as a Plan.

    let start = Node {
        state: current_state.clone(),
        available_actions: generate_available_actions(&current_state),
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

fn successors(node: &Node) -> Vec<(Node, u64)> {
    let mut successors = vec![];
    for action in node.available_actions.iter() {
        let new_state = action.act(node.state.clone());
        let new_available_actions = generate_available_actions(&new_state);

        let cost = action.cost();
        let next_node = Node {
            state: new_state,
            available_actions: new_available_actions,
            action: Some(action.clone()),
        };

        successors.push((next_node, cost));
    }

    successors
}

fn heuristic(_: &Node) -> u64 {
    0
}

fn success(state: &State, goal_state: &State) -> bool {
    // println!("S {:?}", state);
    // println!("G {:?}", goal_state);
    state.agent == goal_state.agent
}

pub fn print_plan(plan: Vec<AgentAction>) {
    for agent_action in plan {
        println!("---------------------------");
        println!("Action: {:#?}", agent_action);
    }
}
