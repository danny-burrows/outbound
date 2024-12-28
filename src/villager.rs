#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Villager {
    pub position: (i64, i64),
    pub health: Health,
    pub inventory: Vec<String>,
}

impl Villager {
    pub fn new(position: (i64, i64)) -> Self {
        Villager {
            position,
            ..Default::default()
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health.check().is_some()
    }
}
