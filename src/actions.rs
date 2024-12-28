use super::goap::{Action, State};

pub fn generate_available_actions(current_state: &State) -> Vec<Box<dyn Action>> {
    let mut available_actions: Vec<Box<dyn Action>> = vec![];
    let (agent_x, agent_y) = current_state.agent.position;

    // Needed to allow agent to return to start point.
    available_actions.push(Box::new(MoveAction {
        delta_x: -agent_x,
        delta_y: -agent_y,
    }));

    let tree_action = MoveToNearestItemAction {
        target_item_id: "tree".to_string(),
    };
    if tree_action.prerequisite(current_state) {
        available_actions.push(Box::new(tree_action));
    }

    let stone_action = MoveToNearestItemAction {
        target_item_id: "stone".to_string(),
    };
    if stone_action.prerequisite(current_state) {
        available_actions.push(Box::new(stone_action));
    }

    let berry_action = MoveToNearestItemAction {
        target_item_id: "berry".to_string(),
    };
    if berry_action.prerequisite(current_state) {
        available_actions.push(Box::new(berry_action));
    }

    for item in current_state.items.clone() {
        if item.position == current_state.agent.position {
            if item.id == *"tree" {
                let action = ChopTreeAction { item: item.clone() };
                if action.prerequisite(current_state) {
                    available_actions.push(Box::new(action));
                }
            } else {
                let action = PickUpAction { item: item.clone() };
                if action.prerequisite(current_state) {
                    available_actions.push(Box::new(action));
                }
            }
        }
    }

    // println!("{}", available_actions.len());

    available_actions
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveToNearestItemAction {
    target_item_id: String,
}

impl MoveToNearestItemAction {
    fn get_new_position(&self, current_state: &State) -> Option<(i64, i64)> {
        let (current_x, current_y) = current_state.agent.position;
        current_state
            .items
            .iter()
            .filter_map(|item| (item.id == self.target_item_id).then_some(item.position))
            .min_by_key(|(ix, iy)| {
                (((ix - current_x) as f64).powf(2.0) + ((iy - current_y) as f64).powf(2.0)).sqrt()
                    as u64
            })
    }
}

impl Action for MoveToNearestItemAction {
    fn act(&self, current_state: State) -> State {
        let mut new_state = current_state.clone();
        new_state.agent.position = self
            .get_new_position(&current_state)
            .expect("If we passed prerequisite there should be something here!");
        new_state
    }

    fn cost(&self) -> u64 {
        // TODO
        5
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        // Check that new position is not out of bounds
        const WORLD_MAX: i64 = 151;

        if let Some(new_position) = self.get_new_position(current_state) {
            new_position.0 > -1
                && new_position.0 < WORLD_MAX
                && new_position.1 > -1
                && new_position.1 < WORLD_MAX
        } else {
            false
        }
    }
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
        let fdelta_x: f64 = self.delta_x as f64;
        let fdelta_y: f64 = self.delta_y as f64;
        (fdelta_x.powf(2.0) + fdelta_y.powf(2.0)).sqrt() as u64
    }

    fn prerequisite(&self, current_state: &State) -> bool {
        // Check that new position is not out of bounds
        let new_position = self.get_new_position(current_state);
        const WORLD_MAX: i64 = 26;
        new_position.0 > -1
            && new_position.0 < WORLD_MAX
            && new_position.1 > -1
            && new_position.1 < WORLD_MAX
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
