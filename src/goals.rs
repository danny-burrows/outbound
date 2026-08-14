use crate::{actions::VillageState, goap::Goal};

const TARGET_COUNT: i64 = 10;

fn inventory_count(current_state: &VillageState, id: &str) -> i64 {
    let held = current_state
        .villager
        .inventory
        .iter()
        .filter(|item| *item == id)
        .count();
    i64::try_from(held).unwrap_or(i64::MAX)
}

#[derive(Debug)]
pub(crate) struct CollectWood {}

impl Goal<VillageState> for CollectWood {
    fn priority(&self, current_state: &VillageState) -> i64 {
        TARGET_COUNT - inventory_count(current_state, "wood")
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state;
        goal_state.villager.inventory.push("wood".to_string());
        goal_state
    }
}

#[derive(Debug)]
pub(crate) struct CollectStone {}

impl Goal<VillageState> for CollectStone {
    fn priority(&self, current_state: &VillageState) -> i64 {
        TARGET_COUNT - inventory_count(current_state, "stone")
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state;
        goal_state.villager.inventory.push("stone".to_string());
        goal_state
    }
}

#[derive(Debug)]
pub(crate) struct CollectBerries {}

impl Goal<VillageState> for CollectBerries {
    fn priority(&self, current_state: &VillageState) -> i64 {
        TARGET_COUNT - inventory_count(current_state, "berry")
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state;
        goal_state.villager.inventory.push("berry".to_string());
        goal_state
    }
}
