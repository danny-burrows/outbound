use crate::goap::{Action, ActionEnum, State};
use crate::item::Item;
use crate::villager::Villager;

const WORLD_MAX: i64 = 151;

const fn in_bounds((x, y): (i64, i64)) -> bool {
    x > -1 && x < WORLD_MAX && y > -1 && y < WORLD_MAX
}

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

        for target_item_id in ["tree", "stone", "berry"] {
            let action = Self::MoveToNearestItem(MoveToNearestItem {
                target_item_id: target_item_id.to_string(),
            });
            if action.prerequisite(current_state) {
                available_actions.push(action);
            }
        }

        for item in &current_state.items {
            if item.position == (agent_x, agent_y) {
                if item.id == "tree" {
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
            Self::MoveToNearestItem(a) => a.act(current_state),
            Self::Move(a) => a.act(current_state),
            Self::ChopTree(a) => a.act(current_state),
            Self::PickUpItem(a) => a.act(current_state),
        }
    }

    fn cost(&self) -> u64 {
        match self {
            Self::MoveToNearestItem(a) => a.cost(),
            Self::Move(a) => a.cost(),
            Self::ChopTree(a) => a.cost(),
            Self::PickUpItem(a) => a.cost(),
        }
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        match self {
            Self::MoveToNearestItem(a) => a.prerequisite(current_state),
            Self::Move(a) => a.prerequisite(current_state),
            Self::ChopTree(a) => a.prerequisite(current_state),
            Self::PickUpItem(a) => a.prerequisite(current_state),
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
            .min_by_key(|&(item_x, item_y)| {
                let delta_x = item_x - current_x;
                let delta_y = item_y - current_y;
                delta_x * delta_x + delta_y * delta_y
            })
    }
}

impl Action<VillageState> for MoveToNearestItem {
    fn act(&self, current_state: VillageState) -> VillageState {
        let new_position = self.get_new_position(&current_state);
        let mut new_state = current_state;
        if let Some(position) = new_position {
            new_state.villager.position = position;
        }
        new_state
    }

    fn cost(&self) -> u64 {
        // TODO
        5
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        self.get_new_position(current_state).is_some_and(in_bounds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Move {
    delta_x: i64,
    delta_y: i64,
}

impl Move {
    const fn get_new_position(&self, current_state: &VillageState) -> (i64, i64) {
        let (current_x, current_y) = current_state.villager.position;
        (current_x + self.delta_x, current_y + self.delta_y)
    }
}

impl Action<VillageState> for Move {
    fn act(&self, current_state: VillageState) -> VillageState {
        let new_position = self.get_new_position(&current_state);
        let mut new_state = current_state;
        new_state.villager.position = new_position;
        new_state
    }

    fn cost(&self) -> u64 {
        let delta_x = self.delta_x.unsigned_abs();
        let delta_y = self.delta_y.unsigned_abs();
        (delta_x * delta_x + delta_y * delta_y).isqrt()
    }

    fn prerequisite(&self, current_state: &VillageState) -> bool {
        in_bounds(self.get_new_position(current_state))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PickUpItem {
    item: Item,
}

impl Action<VillageState> for PickUpItem {
    fn act(&self, current_state: VillageState) -> VillageState {
        let mut new_state = current_state;
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
        current_state.villager.position == self.item.position && self.item.id != "tree"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChopTree {
    item: Item,
}

impl Action<VillageState> for ChopTree {
    fn act(&self, current_state: VillageState) -> VillageState {
        let mut new_state = current_state;

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
