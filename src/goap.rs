// The objective of GOAP is to find a way from the current State -> goal State through Actions.

// State MUST be all-encompassing
// e.g. For an agent to pick something up; the information about the item, where it is, and the agent's inventory must all be included in the state.
// So we need some process for constructing and deconstructing the state for each agent.
// - States can then be augmented with agent perception.
#[derive(Clone)]
struct State {
    trees: u32,
    food: i32,
}

// Actions describe changes to the input State and can be generated on the fly. For example, a MoveAction moves the Agent toward a certain item.
trait Action {
    fn act(&self, current_state: State) -> State {
        todo!()
    }
}

struct MoveAction {
    delta_x: i32,
    delta_y: i32,
}

impl Action for MoveAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();

        // new_state.pos.x += self.delta_x;
        // new_state.pos.y += self.delta_y;
        // new_state

        todo!()
    }
}

trait Planner {
    const PATIENTS: u32 = 100;
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
