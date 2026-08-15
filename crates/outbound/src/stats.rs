/// Totals over a whole run.
///
/// The narrative of *which* decision was taken and why goes to the `log` facade, where a
/// binary can choose how much of it to surface. What stays here is the arithmetic: it
/// survives a ten-thousand-year run, and it is what tests and reports can assert on.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Stats {
    pub(crate) plans: u64,
    /// Plans that came back with nothing to do.
    pub(crate) empty_plans: u64,
    pub(crate) expansions: u64,
    pub(crate) peak_expansions: usize,
    pub(crate) refinement_failures: u64,
}

impl Stats {
    pub(crate) fn record_plan(&mut self, expansions: usize, planned: bool) {
        self.plans = self.plans.saturating_add(1);
        if !planned {
            self.empty_plans = self.empty_plans.saturating_add(1);
        }
        self.expansions = self
            .expansions
            .saturating_add(u64::try_from(expansions).unwrap_or(u64::MAX));
        self.peak_expansions = self.peak_expansions.max(expansions);
    }

    pub(crate) const fn record_refinement_failure(&mut self) {
        self.refinement_failures = self.refinement_failures.saturating_add(1);
    }
}
