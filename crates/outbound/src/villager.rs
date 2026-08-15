#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Health(Option<u8>);

impl Health {
    pub(crate) const fn new(amount: u8) -> Self {
        Self(Some(amount))
    }

    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that changes it"
    )]
    pub(crate) fn replenish(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = Some(health.saturating_add(amount).min(100));
        }
    }

    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that changes it"
    )]
    pub(crate) const fn sap(&mut self, amount: u8) {
        if let Some(health) = self.0 {
            self.0 = health.checked_sub(amount);
        }
    }

    pub(crate) const fn check(&self) -> Option<u8> {
        self.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100)
    }
}

/// The villager as the *simulation* knows them.
///
/// Position and inventory moved to [`crate::village::PlanState`]: they are what the
/// planner reasons over, and keeping them here meant every search node cloned a
/// `Vec<String>` inventory alongside the whole world.
#[derive(Clone, Debug, Default)]
pub(crate) struct Villager {
    pub(crate) health: Health,
}

impl Villager {
    #[expect(
        dead_code,
        reason = "health is modelled ahead of anything that reads it"
    )]
    pub(crate) const fn is_alive(&self) -> bool {
        self.health.check().is_some()
    }
}
