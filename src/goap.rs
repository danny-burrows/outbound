/// The objective of GOAP is for an `Agent` to find a way from the current `State` -> goal `State` through `Action`s.
use std::collections::HashMap;

#[derive(Clone)]
struct Agent {
    position: (f64, f64),
    inventory: HashMap<String, i64>,
}

// State MUST be all-encompassing
// e.g. For an agent to pick something up; the information about the item, where it is, and the agent's inventory must all be included in the state.
// So we need some process for constructing and deconstructing the state for each agent.
// - States can then be augmented with agent perception.
#[derive(Clone)]
struct State {
    agent: Agent,
    items: Vec<String>,
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
trait Action {
    fn act(&self, current_state: State) -> State;

    fn cost(&self) -> f64;

    fn prerequisite(&self) -> Option<()> {
        None
    }
}

struct MoveAction {
    delta_x: f64,
    delta_y: f64,
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

    fn cost(&self) -> f64 {
        (self.delta_x.powf(2.0) + self.delta_y.powf(2.0)).sqrt()
    }
}

struct PickUpAction {
    item_id: String,
}

impl Action for PickUpAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();
        new_state.agent.inventory.insert(self.item_id.clone(), 1);
        new_state
    }

    fn cost(&self) -> f64 {
        1.0
    }
}

trait Planner {
    const PATIENCE: u32 = 100;
    type Plan: Iterator<Item = Box<dyn Action>>;

    fn plan(
        current_state: State,
        available_actions: Vec<impl Action>,
        goal_state: State,
    ) -> Self::Plan {
        // 1. Generate a directed graph sensibly, stopping when exusted with the PATIENTS value.
        // 2. Use some algorithm to find the shortest path from current_state to goal_state.
        // 3. Return that path as a Plan.

        todo!()
    }
}

struct Node<'a> {
    next: &'a Node<'a>,
}
