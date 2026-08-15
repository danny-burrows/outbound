# outbound

The simulation: villagers who plan over world state using [`goap`](../goap), and a
renderer to watch them do it.

```
cargo run                                       # windowed
cargo run -- --headless                         # no window, prints a report
cargo run -- --headless --log-level debug       # ...and why each decision was taken
cargo run -- --headless --log-level trace       # ...and every action carried out
cargo run -- --headless --seed 42 --steps 5000  # a different world, reproducibly
```

`main.rs` holds nothing but binary concerns: arguments, logger, printing. The library
emits its narrative through the `log` facade and returns a `Report` rather than writing to
stdout, which is why no module but `main.rs` needs a `print_stdout` exception.

Layout, roughly in dependency order:

| module | holds |
| --- | --- |
| `item` | `Resource` (what the world holds) and `Good` (what a villager carries) |
| `world` | the simulation's truth, and the `Context` the planner reads through |
| `village` | `PlanState`, the planner's whole view: `Copy` and a few dozen bytes |
| `actions` | `Travel` / `Harvest` / `Gather`, with state-dependent costs |
| `goals` | `Collect`, including the estimate that makes the search fast |
| `sim` | the simulation loop, and refining abstract actions into real changes |
| `stats` | totals over a run; the per-decision narrative goes to `log` |
| `report` | what a headless run amounted to, as plain numbers |
| `render` | every raylib reference in the crate, behind the `render` feature |

Building without `--no-default-features` needs cmake and system graphics libraries for
raylib. `cargo test -p outbound --no-default-features` needs neither.
