/// Effort to pick up one felled log.
pub(crate) const GATHER_COST: u64 = 1;

/// Something the world holds at a position that a villager can work in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Resource {
    Tree,
    Stone,
    Berry,
}

impl Resource {
    pub(crate) const ALL: [Self; 3] = [Self::Tree, Self::Stone, Self::Berry];
    pub(crate) const COUNT: usize = Self::ALL.len();

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Effort to work one instance where it stands.
    pub(crate) const fn harvest_cost(self) -> u64 {
        match self {
            Self::Tree => 2,
            Self::Stone | Self::Berry => 1,
        }
    }

    /// What working it puts straight into the pack. `None` means it leaves something underfoot
    /// to gather: felling a tree drops logs.
    pub(crate) const fn direct_good(self) -> Option<Good> {
        match self {
            Self::Tree => None,
            Self::Stone => Some(Good::Stone),
            Self::Berry => Some(Good::Berry),
        }
    }
}

/// Something a villager can carry. Distinct from [`Resource`] because a tree is not
/// wood until it has been felled and gathered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Good {
    Wood,
    Stone,
    Berry,
}

impl Good {
    pub(crate) const ALL: [Self; 3] = [Self::Wood, Self::Stone, Self::Berry];
    pub(crate) const COUNT: usize = Self::ALL.len();

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) const fn source(self) -> Resource {
        match self {
            Self::Wood => Resource::Tree,
            Self::Stone => Resource::Stone,
            Self::Berry => Resource::Berry,
        }
    }

    /// Action cost of one unit ignoring travel: the harvest, plus gathering when the
    /// harvest leaves it on the ground.
    pub(crate) const fn unit_work(self) -> u64 {
        let source = self.source();
        match source.direct_good() {
            Some(_) => source.harvest_cost(),
            None => source.harvest_cost() + GATHER_COST,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Wood => "wood",
            Self::Stone => "stone",
            Self::Berry => "berry",
        }
    }
}

/// A tally of goods. Carried in a pack, piled in a stockpile, or delivered to a build site.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Inventory([u8; Good::COUNT]);

impl Inventory {
    pub(crate) fn count(self, good: Good) -> u8 {
        self.0.get(good.index()).copied().unwrap_or(0)
    }

    pub(crate) fn add(&mut self, good: Good) {
        if let Some(slot) = self.0.get_mut(good.index()) {
            *slot = slot.saturating_add(1);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) position: (i64, i64),
    pub(crate) resource: Resource,
}

impl Item {
    pub(crate) const fn new(resource: Resource, position: (i64, i64)) -> Self {
        Self { position, resource }
    }
}
