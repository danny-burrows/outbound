struct Health(Option<u8>);

impl Health {
    fn default() -> Self {
        Self::new(100)
    }

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

    fn check_health(&self) -> Option<u8> {
        self.0
    }
}

struct Inventory {}

pub struct Villager {
    health: Health,
    inventory: Inventory,
}

impl Villager {
    pub fn new() -> Self {
        Villager {
            health: Health::default(),
            inventory: Inventory {},
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health.check_health().is_some()
    }
}
