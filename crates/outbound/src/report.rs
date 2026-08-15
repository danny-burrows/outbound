/// What a headless run amounted to.
///
/// Plain numbers, so a caller can assert on them or format them differently. The library
/// hands this back rather than printing it — writing to stdout is the binary's business,
/// which is why nothing under `src/` but `main.rs` needs a `print_stdout` exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub seed: u64,
    pub steps: u64,
    pub villagers: usize,
    /// Villagers with nothing planned: every desire either met or out of reach.
    pub idle_villagers: usize,
    /// Carried between the whole village.
    pub wood: u64,
    pub stone: u64,
    pub berries: u64,
    pub plans: u64,
    /// Plans that came back with nothing to do.
    pub empty_plans: u64,
    pub expansions: u64,
    pub peak_expansions: usize,
    /// Planned actions that could not be carried out, almost always because another
    /// villager got there first.
    pub refinement_failures: u64,
    pub trees_left: u64,
    pub stone_left: u64,
    pub berries_left: u64,
    /// Logs felled and then walked away from. Stranded: the planner only knows about logs
    /// under the villager who felled them.
    pub logs_stranded: usize,
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "outbound headless | seed {} | {} villagers | {} steps",
            self.seed, self.villagers, self.steps
        )?;
        writeln!(
            f,
            "  village carries: {} wood, {} stone, {} berries",
            self.wood, self.stone, self.berries
        )?;
        writeln!(
            f,
            "  world left:      {} trees, {} stone, {} berries ({} logs stranded)",
            self.trees_left, self.stone_left, self.berries_left, self.logs_stranded
        )?;
        writeln!(
            f,
            "  idle villagers:  {} of {}",
            self.idle_villagers, self.villagers
        )?;
        writeln!(
            f,
            "  plans:           {} ({} with nothing to do)",
            self.plans, self.empty_plans
        )?;
        writeln!(
            f,
            "  expansions:      {} total, {} peak",
            self.expansions, self.peak_expansions
        )?;
        write!(
            f,
            "  contested:       {} refinements failed",
            self.refinement_failures
        )
    }
}
