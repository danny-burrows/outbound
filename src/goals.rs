use crate::{actions::VillageState, goap::Goal};

#[derive(Debug)]
pub(crate) struct CollectWood {}

impl Goal<VillageState> for CollectWood {
    fn priority(&self, current_state: &VillageState) -> i64 {
        // Aiming to have 10 wood in inventory!
        let wood_count = current_state
            .villager
            .inventory
            .iter()
            .filter(|i| *i == "wood")
            .count() as i64;
        10 - wood_count
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state.clone();
        goal_state.villager.inventory.push("wood".to_string());
        goal_state
    }
}

#[derive(Debug)]
pub(crate) struct CollectStone {}

impl Goal<VillageState> for CollectStone {
    fn priority(&self, current_state: &VillageState) -> i64 {
        // Aiming to have 10 stone in inventory!
        let stone_count = current_state
            .villager
            .inventory
            .iter()
            .filter(|i| *i == "stone")
            .count() as i64;
        10 - stone_count
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state.clone();
        goal_state.villager.inventory.push("stone".to_string());
        goal_state
    }
}

#[derive(Debug)]
pub(crate) struct CollectBerries {}

impl Goal<VillageState> for CollectBerries {
    fn priority(&self, current_state: &VillageState) -> i64 {
        // Aiming to have 10 berries in inventory!
        let berries_count = current_state
            .villager
            .inventory
            .iter()
            .filter(|i| *i == "berry")
            .count() as i64;
        10 - berries_count
    }

    fn goal_state(&self, current_state: VillageState) -> VillageState {
        let mut goal_state = current_state.clone();
        goal_state.villager.inventory.push("berry".to_string());
        goal_state
    }
}
