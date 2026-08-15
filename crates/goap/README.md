# goap

Goal-oriented action planning over a caller-defined domain. No knowledge of villages,
items or renderers; implement `Domain` and the planner works against whatever you have.

```rust
impl Domain for MyDomain {
    type State = MyState;      // small and ideally Copy: one per expanded node
    type Context = MyWorld;    // read-only, borrowed, never copied
    type Action = MyAction;
}
```

Three things worth knowing before using it:

**Goals are partial.** A `Goal` reports `unmet` and `estimate` rather than describing a
finished world. That is what makes an informative A* heuristic expressible; comparing
complete states for equality leaves no way to ask how far off you are.

**`estimate` decides whether this is fast.** The default, one unit-cost action per
outstanding condition, is admissible and usually close to useless. Override it with what
your actions actually charge. An estimate that ignores the dominant cost will search
almost as widely as no estimate at all.

**State is not the world.** Keep positional and per-entity detail in `Context` and read
it through `&Context`. Anything in `State` is cloned and hashed on every expansion.

`plan` always returns a `Plan`, and `Plan::considered` records every goal it weighed and
what became of each. Failing to plan is information, not absence.
