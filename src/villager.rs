#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Health(Option<u8>);

impl Health {
    pub(crate) fn new(amount: u8) -> Self {
        Self(Some(amount))
    }

    pub(crate) fn replenish(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = Some(health.saturating_add(amount).min(100))
        }
    }

    pub(crate) fn sap(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = health.checked_sub(amount)
        }
    }

    pub(crate) fn check(&self) -> Option<u8> {
        self.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Villager {
    pub(crate) position: (i64, i64),
    pub(crate) health: Health,
    pub(crate) inventory: Vec<String>,
}

impl Villager {
    pub(crate) fn new(position: (i64, i64)) -> Self {
        Villager {
            position,
            ..Default::default()
        }
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.health.check().is_some()
    }
}
