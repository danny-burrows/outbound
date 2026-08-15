# Outbound

A sandbox for goal-oriented action planning: villagers who decide what to do by searching
over world state, with the aim of a society that assembles itself out of individual
motives.

```
cargo run                                   # watch it
cargo run -- --headless --log-level debug    # run it fast and read why it did that
```

Two crates:

- **[`crates/goap`](crates/goap)** — the planner. Domain-agnostic: implement `Domain` and
  it works against whatever you have.
- **[`crates/outbound`](crates/outbound)** — the simulation and renderer.

Rendering is behind a default-on `render` feature. With it off the simulation still runs
headless and the build needs neither cmake nor any system graphics library, which is how
to iterate on planning without waiting for raylib:

```
cargo test -p goap
cargo test -p outbound --no-default-features
```

Everything else — lints, dependency versions, shared package metadata — is set once at the
workspace root.
