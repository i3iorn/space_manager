# space_manager

> A space agency management simulation.
>
> You are the Director of the Stellar Expansion Authority. You do not fly anything.
> You hire and keep people, decide which programmes the agency pursues, split a finite
> budget across them, absorb political pressure and public reaction, and live with the
> consequences for thirty years or more.

**Status: pre-alpha.** The design is complete and the build plan is written; the game
itself is not built. See [Where things stand](#where-things-stand).

---

## Building and running

Requires a Rust toolchain with edition 2024 support (**Rust 1.85 or newer**).

```sh
cargo build                       # build the workspace
cargo test  --workspace           # run every test
cargo run -p space_manager        # play (the interface arrives in Slice 1)
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

Options:

```sh
cargo run -p space_manager -- --seed 12345   # start from a fixed campaign seed
cargo run -p space_manager -- --help
```

**Campaigns are reproducible from their seed.** Every random draw in the simulation is
derived from the campaign's root seed, so the same seed and the same decisions always
produce the same history.

---

## The crates

```
crates/
├─ domain/         the data: entities, ids, invariants. Depends on nothing of ours.
├─ sim/            the engine: the clock, determinism, orders, events.
├─ ui/             the terminal interface (ratatui).
└─ space_manager/  the binary. A thin shell that wires the others together.
```

**Dependencies point one way:** `space_manager` → `ui` → `sim` → `domain`.
The direction is enforced by a test
(`crates/space_manager/tests/dependency_direction.rs`) that fails the build if it is
broken.

---

## The documentation

Start at [`docs/README.md`](docs/README.md) — it indexes everything and explains how
the documents fit together.

| Document | What it is |
|---|---|
| [`docs/MANAGEABLE_PARTS.md`](docs/MANAGEABLE_PARTS.md) | **The management surface** — every system the player can act on. The scoping document. |
| [`docs/SIMULATION.md`](docs/SIMULATION.md) | The engine contract: the clock, determinism, the RNG, orders, events. |
| [`docs/MVP.md`](docs/MVP.md) | The minimum viable *game*: scope, slices, success criteria, risks. |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | The build plan, task by task. |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Crate layout, module boundaries, dependency direction. |
| [`docs/DOMAIN_MODEL.md`](docs/DOMAIN_MODEL.md) | The entities, their fields, the invariants. |
| [`docs/PERSISTENCE.md`](docs/PERSISTENCE.md) | Save format, versioning, replay, branching. |
| [`docs/TESTING.md`](docs/TESTING.md) | The test strategy. |
| [`docs/UI.md`](docs/UI.md) | The screens and the information design. |
| [`docs/BALANCE.md`](docs/BALANCE.md) | The numbers, and how they get tuned. |
| [`docs/DEVELOPER_INTRO.md`](docs/DEVELOPER_INTRO.md) | The original vision and its pillars. |
| [`docs/PLAYER_INTRO.md`](docs/PLAYER_INTRO.md) | The in-fiction framing. |

[`AGENTS.md`](AGENTS.md) holds the conventions an automated session needs: the exact
commands, the toolchain quirks, the manifest warning that must not be "fixed", and the
architectural rules that must not be broken.

---

## Where things stand

The **design** is done: the management surface, the simulation contract, the domain
model, the interface specification, and the balance model are all written and
interlocked.

The **code** is well past the foundation. The engine runs on **continuous simulated
time** — a campaign is a running agency on a clock, not a monthly tick — with a
scheduler, the continuous rates, the order model with its lead times and its two
validations, the goodwill, and a save format that restores a campaign with work in
flight. The interface has seven screens, an **order palette** that shows what an order
will cost before the player commits to it, a tutorial, and a fork that lets the player
ask what would have happened if they had not cancelled that programme.

What does not exist yet is the **event catalogue**, so the agency is still nearly
deterministic in practice, and the **first playtest** has not happened.

The build plan is in [`docs/ROADMAP.md`](docs/ROADMAP.md). Slice 1 (*Ground Game*)
is the first year of a small agency: a budget, a few scientists, a research project,
and the first month where the player sees **why their team is slow**.

---

## The rules that matter

These are not style preferences. Breaking them breaks the game.

1. **No wall-clock time, no system randomness, and no `HashMap` iteration order** in
   the simulation, and **no floating point in decision-making**. All randomness comes
   from a named substream derived from `(root_seed, domain, bucket, discriminator)`,
   where the bucket is a **stable identifier** and never an instant.
2. **The player never mutates state.** They issue orders, which are validated
   without side effects. A rejected order is reported with a reason, never
   silently ignored.
3. **Every collection is a `BTreeMap`.** A total iteration order is a determinism
   requirement.
4. **No `unwrap`, no `expect`, no `panic!`** outside tests.
5. **The terminal is always restored**, even on panic.

---

## Licence

MIT OR Apache-2.0.
