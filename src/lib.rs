pub mod goap;

use raylib::prelude::Vector2;

struct Health(Option<u8>);

impl Health {
    fn new(amount: u8) -> Self {
        Self(Some(amount))
    }

    fn replenish(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = Some(health.saturating_add(amount).min(100))
        }
    }

    fn sap(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = health.checked_sub(amount)
        }
    }

    fn check(&self) -> Option<u8> {
        self.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100)
    }
}

#[derive(Default)]
pub struct Villager {
    position: Vector2,
    health: Health,
    inventory: Vec<String>,
}

impl Villager {
    pub fn new(position: Vector2) -> Self {
        Villager {
            position,
            ..Default::default()
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health.check().is_some()
    }
}
