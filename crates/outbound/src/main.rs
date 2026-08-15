//! Binary concerns only: parse arguments, install a logger, print what came back.
//! Everything the simulation actually does lives in the library.

use clap::Parser;
use log::LevelFilter;

/// Reproducible by default, so a run can be described by its flags alone.
const DEFAULT_SEED: u64 = 0x0000_0B0A_7000_D1CE;
const DEFAULT_HEADLESS_STEPS: u64 = 100_000;

/// A goal-oriented action planning sandbox where villagers plan over world state.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Run without a window and report what happened
    #[arg(long)]
    headless: bool,

    /// World seed. The same seed rebuilds the same world, windowed or headless
    #[arg(long, default_value_t = DEFAULT_SEED)]
    seed: u64,

    /// Simulation steps to run headless
    #[arg(long, default_value_t = DEFAULT_HEADLESS_STEPS)]
    steps: u64,

    /// How much of each decision to report: info names the goal chosen, debug adds the
    /// goals it weighed against, trace adds every action carried out
    #[arg(long, default_value_t = LevelFilter::Info, value_name = "LEVEL")]
    log_level: LevelFilter,
}

#[expect(
    clippy::print_stdout,
    reason = "stdout is the entire interface of this binary"
)]
fn main() {
    let cli = Cli::parse();

    // The flag sets the level; RUST_LOG still wins if it is set.
    env_logger::Builder::new()
        .filter_level(cli.log_level)
        .parse_default_env()
        .init();

    if cli.headless {
        println!("{}", outbound::run_headless(cli.seed, cli.steps));
        return;
    }

    #[cfg(feature = "render")]
    outbound::run(cli.seed);

    #[cfg(not(feature = "render"))]
    {
        log::warn!("built without the `render` feature; running headless instead");
        println!("{}", outbound::run_headless(cli.seed, cli.steps));
    }
}
