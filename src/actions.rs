use crate::goap::{Action, ActionEnum, State};
use crate::item::Item;
use crate::villager::Villager;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct VillageState {
    pub(crate) villager: Villager,
    pub(crate) items: Vec<Item>,
}

impl State for VillageState {
    fn compare(&self, other_state: &Self) -> bool {
        self.villager.inventory == other_state.villager.inventory
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum VillagerActionEnum {
    MoveToNearestItem(MoveToNearestItem),
    Move(Move),
    ChopTree(ChopTree),
    PickUpItem(PickUpItem),
}

impl ActionEnum<VillageState> for VillagerActionEnum {
    fn generate_available_actions(current_state: &VillageState) -> Vec<Self> {
        let mut available_actions: Vec<Self> = vec![];

        let (agent_x, agent_y) = current_state.villager.position;

        // Needed to allow agent to return to start point.
        available_actions.push(Self::Move(Move {
            delta_x: { -agent_x },
            delta_y: { -agent_y },
        }));

        let tree_action = Self::MoveToNearestItem(MoveToNearestItem {
            target_item_id: "tree".to_string(),
        });
        if tree_action.prerequisite(current_state) {
            available_actions.push(tree_action);
        }

        let stone_action = Self::MoveToNearestItem(MoveToNearestItem {
            target_item_id: "stone".to_string(),
        });
        if stone_action.prerequisite(current_state) {
            available_actions.push(stone_action);
        }

        let berry_action = Self::MoveToNearestItem(MoveToNearestItem {
            target_item_id: "berry".to_string(),
        });
        if berry_action.prerequisite(current_state) {
            available_actions.push(berry_action);
        }

        for item in current_state.items.clone() {
            if item.position == (agent_x, agent_y) {
                if item.id == *"tree" {
                    let action = ChopTree { item: item.clone() };
                    if action.prerequisite(current_state) {
                        available_actions.push(Self::ChopTree(action));
                    }
                } else {
                    let action = PickUpItem { item: item.clone() };
                    if action.prerequisite(current_state) {
                        available_actions.push(Self::PickUpItem(action));
                    }
                }
            }
        }

        available_actions
    }
}

impl Action<VillageState> for VillagerActionEnum {
    // Uuugh

    fn act(&self, current_state: VillageState) -> VillageState {
        match self {
            VillagerActionEnum::MoveToNearestItem(a) => a.act(current_state),
            VillagerActionEnum::Move(a) => a.act(current_state),
            VillagerActionEnum::ChopTree(a) => a.act(current_state),
            VillagerActionEnum::PickUpItem(a) => a.act(current_state),
        }
    }

    fn cost(&self) -> u64 {
        match self {
            VillagerActionEnum::MoveToNearestItem(a) => a.cost(),
            VillagerActionEnum::Move(a) => a.cost(),
            VillagerActionEnum::ChopTree(a) => a.cost(),
            VillagerActionEnum::PickUpItem(a) => a.cost(),
        }
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        match self {
            VillagerActionEnum::MoveToNearestItem(a) => a.prerequisite(current_state),
            VillagerActionEnum::Move(a) => a.prerequisite(current_state),
            VillagerActionEnum::ChopTree(a) => a.prerequisite(current_state),
            VillagerActionEnum::PickUpItem(a) => a.prerequisite(current_state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MoveToNearestItem {
    target_item_id: String,
}

impl MoveToNearestItem {
    fn get_new_position(&self, current_state: &VillageState) -> Option<(i64, i64)> {
        let (current_x, current_y) = current_state.villager.position;
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

impl Action<VillageState> for MoveToNearestItem {
    fn act(&self, current_state: VillageState) -> VillageState {
        let mut new_state = current_state.clone();
        new_state.villager.position = self
            .get_new_position(&current_state)
            .expect("If we passed prerequisite there should be something here!");
        new_state
    }

    fn cost(&self) -> u64 {
        // TODO
        5
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
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
pub(crate) struct Move {
    delta_x: i64,
    delta_y: i64,
}

impl Move {
    fn get_new_position(&self, current_state: &VillageState) -> (i64, i64) {
        let (current_x, current_y) = current_state.villager.position;
        (current_x + self.delta_x, current_y + self.delta_y)
    }
}

impl Action<VillageState> for Move {
    fn act(&self, current_state: VillageState) -> VillageState {
        let mut new_state = current_state.clone();
        new_state.villager.position = self.get_new_position(&current_state);
        new_state
    }

    fn cost(&self) -> u64 {
        let fdelta_x: f64 = self.delta_x as f64;
        let fdelta_y: f64 = self.delta_y as f64;
        (fdelta_x.powf(2.0) + fdelta_y.powf(2.0)).sqrt() as u64
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
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
pub(crate) struct PickUpItem {
    item: Item,
}

impl Action<VillageState> for PickUpItem {
    fn act(&self, current_state: VillageState) -> VillageState {
        let mut new_state = current_state.clone();
        new_state.villager.inventory.push(self.item.id.clone());

        if let Some(i) = new_state.items.iter().position(|item| item == &self.item) {
            new_state.items.remove(i);
        }

        new_state
    }

    fn cost(&self) -> u64 {
        // Arbitrary cost to pick up an item. Maybe this should be weight?
        1
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        current_state.villager.position == self.item.position && self.item.id != *"tree"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChopTree {
    item: Item,
}

impl Action<VillageState> for ChopTree {
    fn act(&self, current_state: VillageState) -> VillageState {
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

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        current_state.villager.position == self.item.position
    }
}
