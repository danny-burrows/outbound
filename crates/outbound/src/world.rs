use crate::item::{Item, Resource};
use crate::villager::Villager;

/// Everything the simulation knows.
///
/// The planner borrows this as its [`crate::goap::Domain::Context`] and never clones
/// it. Only [`crate::village::PlanState`] is copied per search node, which is why the
/// item positions live here instead of in the state.
pub(crate) struct World {
    /// Positions grouped by kind, so a lookup scans only the resource it cares about
    /// rather than every item in the world.
    positions: [Vec<(i64, i64)>; Resource::COUNT],
    /// Typical gap between neighbouring instances, used to price the walk to the
    /// *next* one once a plan has already consumed the closest.
    spacing: [u64; Resource::COUNT],
    /// Felled logs waiting to be gathered.
    pub(crate) logs: Vec<(i64, i64)>,
    #[expect(
        dead_code,
        reason = "the villager's health is modelled ahead of anything that reads it"
    )]
    pub(crate) villager: Villager,
    /// Where the villager is drawn. Trails [`crate::village::PlanState::position`]
    /// while a walk plays out, and catches up before the next action runs.
    pub(crate) render_position: (i64, i64),
}

impl World {
    pub(crate) fn new(items: &[Item], extent: u64) -> Self {
        let mut positions: [Vec<(i64, i64)>; Resource::COUNT] = std::array::from_fn(|_| Vec::new());

        for item in items {
            if let Some(bucket) = positions.get_mut(item.resource.index()) {
                bucket.push(item.position);
            }
        }

        let spacing = std::array::from_fn(|index| {
            let count = positions.get(index).map_or(0, Vec::len);
            // `count` instances scattered over `extent` squared sit roughly
            // `extent / sqrt(count)` apart.
            let rows = u64::try_from(count).unwrap_or(1).max(1).isqrt();
            (extent / rows).max(1)
        });

        Self {
            positions,
            spacing,
            logs: Vec::new(),
            villager: Villager::default(),
            render_position: (0, 0),
        }
    }

    pub(crate) fn count(&self, resource: Resource) -> u64 {
        let count = self.positions.get(resource.index()).map_or(0, Vec::len);
        u64::try_from(count).unwrap_or(u64::MAX)
    }

    /// Position of and Chebyshev distance to the closest instance of `resource`.
    ///
    /// Chebyshev because the walk this abstracts over moves eight-connected at unit
    /// cost, making the distance its exact length rather than merely a bound.
    pub(crate) fn nearest(
        &self,
        from: (i64, i64),
        resource: Resource,
    ) -> Option<((i64, i64), u64)> {
        let (from_x, from_y) = from;
        self.positions
            .get(resource.index())?
            .iter()
            .map(|&(x, y)| {
                let distance = (x - from_x).unsigned_abs().max((y - from_y).unsigned_abs());
                ((x, y), distance)
            })
            .min_by_key(|&(_, distance)| distance)
    }

    /// What a plan should pay to reach the next unworked instance of `resource`, given
    /// it has already consumed `taken` of them. `None` when the world has no more.
    ///
    /// Exact for the first trip. After that it charges one typical `spacing` per
    /// instance already consumed rather than tracking which ones went — the planner
    /// deliberately keeps no item identities, and a cost that only ever grows is
    /// enough to stop it felling the same tree forever.
    pub(crate) fn trip_cost(
        &self,
        from: (i64, i64),
        resource: Resource,
        taken: u16,
    ) -> Option<u64> {
        if u64::from(taken) >= self.count(resource) {
            return None;
        }

        let (_, distance) = self.nearest(from, resource)?;
        let spacing = self.spacing.get(resource.index()).copied().unwrap_or(1);
        Some(distance.saturating_add(u64::from(taken).saturating_mul(spacing)))
    }

    /// Lower bound on the travel still owed for trips numbered `first..=last` in
    /// depletion order, where a trip's number is how many instances of `resource` the
    /// plan had already consumed when it set out.
    ///
    /// Mirrors the depletion half of [`Self::trip_cost`] and drops the geometric half,
    /// since where each later trip starts from is unknown until the plan gets there.
    /// Dropping it keeps the bound under the true cost; keeping the depletion half is
    /// what stops the estimate being nearly useless in a domain where walking, not
    /// working, is what costs.
    pub(crate) fn depletion_cost(&self, resource: Resource, first: u64, last: u64) -> u64 {
        if last < first {
            return 0;
        }

        let spacing = self.spacing.get(resource.index()).copied().unwrap_or(1);
        let trips = last.saturating_sub(first).saturating_add(1);
        // Sum of the arithmetic run first..=last.
        let sum = first.saturating_add(last).saturating_mul(trips) / 2;
        spacing.saturating_mul(sum)
    }

    /// Remove a worked instance for real. Executed against the world, never during a
    /// search.
    pub(crate) fn take(&mut self, resource: Resource, position: (i64, i64)) -> bool {
        let Some(bucket) = self.positions.get_mut(resource.index()) else {
            return false;
        };
        let Some(index) = bucket.iter().position(|&held| held == position) else {
            return false;
        };
        bucket.swap_remove(index);
        true
    }

    pub(crate) fn drop_log(&mut self, position: (i64, i64)) {
        self.logs.push(position);
    }

    pub(crate) fn take_log(&mut self, position: (i64, i64)) -> bool {
        let Some(index) = self.logs.iter().position(|&held| held == position) else {
            return false;
        };
        self.logs.swap_remove(index);
        true
    }

    /// Every item in the world. Only drawing wants the lot in one pass; the planner asks
    /// per resource, which is why they are stored grouped.
    #[cfg(feature = "render")]
    pub(crate) fn items(&self) -> impl Iterator<Item = Item> {
        Resource::ALL.into_iter().flat_map(move |resource| {
            self.positions
                .get(resource.index())
                .map_or(&[][..], Vec::as_slice)
                .iter()
                .map(move |&position| Item::new(resource, position))
        })
    }
}
