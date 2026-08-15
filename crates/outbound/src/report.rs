/// What a headless run amounted to.
///
/// Plain numbers, so a caller can assert on them or format them differently. The library
/// hands this back rather than printing it — writing to stdout is the binary's business,
/// which is why nothing under `src/` but `main.rs` needs a `print_stdout` exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub seed: u64,
    pub steps: u64,
    pub wood: u8,
    pub stone: u8,
    pub berries: u8,
    pub plans: u64,
    /// Plans that came back with nothing to do.
    pub empty_plans: u64,
    pub expansions: u64,
    pub peak_expansions: usize,
    pub refinement_failures: u64,
    pub trees_left: u64,
    pub stone_left: u64,
    pub berries_left: u64,
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "outbound headless | seed {} | {} steps",
            self.seed, self.steps
        )?;
        writeln!(
            f,
            "  carrying:   {} wood, {} stone, {} berries",
            self.wood, self.stone, self.berries
        )?;
        writeln!(
            f,
            "  world left: {} trees, {} stone, {} berries",
            self.trees_left, self.stone_left, self.berries_left
        )?;
        writeln!(
            f,
            "  plans:      {} ({} with nothing to do)",
            self.plans, self.empty_plans
        )?;
        writeln!(
            f,
            "  expansions: {} total, {} peak",
            self.expansions, self.peak_expansions
        )?;
        write!(f, "  refinements failed: {}", self.refinement_failures)
    }
}
