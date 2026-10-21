# AGENTS.md

## What this repo actually is

A space agency management simulation, in Rust, built as a cargo workspace of four
crates: `domain` (the data), `sim` (the engine), `ui` (the terminal interface), and
`space_manager` (the binary). 175 tests pass.

**The design is complete and the code is behind it.** The engine in `crates/` still
implements the old monthly-turn model; Milestone M1 (`docs/ROADMAP.md` §4) rebuilds it
around continuous time. Read the time-model note below before touching the engine.

## Commands (verified working on this machine)

| Task | Command |
|---|---|
| Build everything | `cargo build` |
| Run the game | `cargo run -p space_manager` |
| Test everything | `cargo test --workspace` |
| Test one crate | `cargo test -p sim` |
| Format check | `cargo fmt --all --check` (fix with `cargo fmt --all`) |
| Lint | `cargo clippy --workspace --all-targets` |

All of these exit 0 today. There is **no `rustfmt.toml`, no `clippy.toml`, no
`.cargo/config.toml`, and no `rust-toolchain.toml`** — rustfmt and clippy run on
stock defaults, and the toolchain follows the machine's rustup default. CI lives in
`.github/workflows/ci.yml` and runs the first four plus the dependency-direction test
on both Linux and Windows.

**`cargo test --workspace` currently runs 494 tests and they all pass.** That number
matters: it is the baseline a change is measured against. If you add behaviour, add
tests with it, or "green" means nothing.

**`cargo build` alone no longer builds every member** in the usual sense — use
`cargo build --workspace` when you want to be explicit, and `cargo build -p <crate>`
when working in one crate.

**`cargo run -p space_manager` starts the interface and waits for a key forever.** Use
`cargo run -p space_manager -- --headless 24` for a non-interactive smoke test, and
`--seed N` to reproduce a campaign exactly.

## Toolchain

- rustup's **default toolchain on this machine is `nightly`** (`cargo 1.100.0-nightly`);
  `stable` (1.96.1) is also installed. There is **no `rust-toolchain.toml`**, so the
  toolchain follows the machine's rustup default — it can differ on another box.
- `edition = "2024"` requires Rust ≥ 1.85, so stable works too. Pin deliberately if the
  nightly default starts causing surprises; don't assume the default is intentional.
- Host is `x86_64-pc-windows-msvc`. New crates come from crates.io, so a network fetch
  is required the first time each dependency is added.
- **Pinned dependencies** (do not bump without regenerating the golden fixtures):
  `rand` 0.10, `rand_chacha` 0.10, `serde` 1, `serde_json` 1, `thiserror` 2,
  `anyhow` 1, `ratatui` 0.30, `crossterm` 0.29.

## Gotchas

- **A crate named `core` is unusable.** It shadows the standard library's `core` crate
  in the extern prelude, and every dependent crate then fails to resolve `core::fmt`.
  The domain crate is `domain` for this reason. Do not rename it.
- **Git does not work in this tree.** `C:\Users\micro\Code\Projects` is a *bare* repository
  (it holds `HEAD`, `config`, `objects/`, `refs/`), and this project is not inside a work
  tree, so `git status` fails with `this operation must be run in a work tree`. There is no
  history, branch, or PR workflow here — don't plan around commits.
- The IDE is **IntelliJ IDEA with the Rust plugin** (`.idea/`). `.idea/workspace.xml` is
  machine-local and holds the run config; keep the package name in sync with it. It
  needs a re-index after the workspace migration.
- The shell here is **PowerShell on Windows** — use PowerShell syntax
  (`Get-ChildItem`, `Select-String`), not bash-isms.
- **`rand` 0.10 moved `random_range` to the `RngExt` trait**, not `Rng`. Import
  `rand::RngExt` or the method will not resolve.
- **`std`'s `DefaultHasher` is not stability-guaranteed.** The stream hash in
  `sim::rng` is FNV-1a, written out in the file, for exactly this reason. The seed
  expansion is SplitMix64 rather than `SeedableRng::seed_from_u64`, for the same reason.

## Target architecture

The intended shape, confirmed by the maintainer:

1. **Terminal UI** — ratatui/crossterm style, a native Rust binary. No web stack, no
   GUI toolkit, no browser assets. Keep rendering separate from simulation.
2. **Cargo workspace** — split into separate crates rather than growing one binary.

**The workspace exists (M0 is complete) and is being rebuilt for continuous time
(M1).** The layout is:

```
Cargo.toml                     # [workspace] only, virtual. Root Cargo.lock lives here
crates/domain/                 # the data: entities, ids, invariants. Depends on nothing of ours.
crates/sim/                    # the engine: scheduler, phases, rates, orders, events
crates/ui/                     # ratatui: screens, widgets, input
crates/space_manager/          # the binary. Package name MUST stay "space_manager".
```

**The domain crate is named `domain`, not `core`, and must not be renamed.** A package
named `core` shadows the standard library's `core` crate in the extern prelude, and
every dependent crate then fails to resolve `core::fmt`. This was reproduced during
M0; see `docs/ARCHITECTURE.md` §2.

**The package name must stay `space_manager`** — the IntelliJ run config in
`.idea/workspace.xml` invokes `cargo run --package space_manager --bin space_manager`,
and the directory may move but the package name may not.

**Dependencies point one way:** `space_manager` → `ui` → `sim` → `domain`. Keep crates
layered so `domain` and `sim` never depend on `ui`; `ui` may depend on both. This is
enforced by `crates/space_manager/tests/dependency_direction.rs`, which fails the
build if the direction is broken.

**The `non_kebab_case_bins` warning is silenced, not fixed.** It is declared once in
the root manifest as `[workspace.lints.cargo]`, because a package cannot combine
`[lints] workspace = true` with its own `[lints.cargo]` table. Do not "fix" it by
renaming the package.

Settled gameplay decisions (from `docs/MANAGEABLE_PARTS.md`, marked `[D]` there):

- **Time:** continuous simulated time, running in real time at a player-chosen
  dilation, campaign spanning 30-60 years. The player acts whenever they like; every
  order takes time to implement. Missed commitments are the main tension source.
- **Deadlines:** both hard and soft, distinguished by *who holds the promise*. Every
  dated objective carries a soft **target instant** and a hard **committed instant**;
  the gap is the slack the player manages.
- **Hardware:** archetype + ~9 subsystem sliders, no component design, no orbital
  mechanics. The sim derives performance, cost, and risk from the sliders.
- **Scale:** 150-300 staff. Named individuals for the notable, aggregate screens for
  the rest.
- **Performance model:** the **C+B hybrid** — the eight factors are *multiplied* in `u128`
  with no intermediate clamping, then divided by a reference composite so that **100%
  means "a good team"**. The UI must decompose the result into named modifiers plus a
  **binding factor**. The player must never be able to compute the answer by inspection.
- **The saturation curve is an integer parabola**, not an exponential — the whole
  computation must be exact integer arithmetic, and an exponential cannot be.
- **Research projects have a fixed 2,400-point target.** Each team member contributes
  their performance in points and the points are **summed**, so adding people makes a
  project finish sooner and the target never moves.
- **The state appropriation scales with headcount** (1,900 per head, floor 12,000),
  multiplied by political standing. A fixed allocation is wrong at both ends.
- **Infrastructure:** every facility is built, never abstract capacity. Construction
  choices (method, scale, automation, redundancy, materials, siting, financing) determine
  its specification for the rest of the campaign.
- **Alien life:** bacterial only. No sentient species, no artefacts, no first contact.
  Planetary protection and sample-return contamination are first-class mechanics.
- **Rivals:** rival agencies compete for funding and milestone primacy only — no
  hostility — and can be switched off entirely.
- **The central research tension:** capability (research) vs. reliability (testing), and
  they compete for the same scarce staff.

## Testing conventions

- No test framework is chosen. Prefer built-in `#[cfg(test)]` unit tests; don't add a
  framework dependency until there's a concrete need for it.
- `domain`/`sim` must stay usable without a terminal, so the sim crate's public API should
  accept inputs and return state/events — the TUI is a *view* over it, not the driver.
- Procedural generation means results must be **reproducible from a seed**; pick one RNG
  crate and thread the seed through the public API so tests are deterministic.
- Unit tests belong next to the code they test; integration tests go in
  `crates/<crate>/tests/`.

## Design constraints (read before coding)

`docs/DEVELOPER_INTRO.md` is the original game design spec — **not** a setup guide, despite
the name. `docs/PLAYER_INTRO.md` is the in-fiction framing ("Director of the Stellar
Expansion Authority").

**Start at `docs/README.md`** — it indexes every design document and states the
build-plan rules (simulation first, vertical slices, determinism from day one, every
pillar must create a trade-off). `docs/MANAGEABLE_PARTS.md` is the **scoping document**:
it enumerates every system the player can act on, and the sections marked `[D]` (settled)
vs. `[?]` (open) are the current decisions. Read it before designing any feature — if a
mechanic is not in that document, it is not in the game.

`docs/SIMULATION.md` is the **engine contract** — the clock, the scheduler, the
continuous integration, the determinism rules, the named-substream RNG, the order model,
and the event system. Its rules are normative, and `docs/MVP.md` says which parts get
built first. The constraints from it are architectural and must not be violated:

- **No wall-clock time, no system randomness, no `HashMap` iteration order** anywhere in
  the simulation, and **no floating-point in decision-making** (integers or fixed-point
  only; floats are presentation-only).
- **The player never mutates state** — they issue *orders*, which are validated
  without side effects and then scheduled for their lead time. A rejected order is
  reported with a reason, never silently ignored or partially applied.
- **All randomness comes from a named substream** derived from `(root_seed, domain,
  bucket, discriminator)`, the bucket is a stable identifier and never an instant, and
  the stream-hash function is pinned in code - not `DefaultHasher`, whose algorithm
  Rust may change between releases.
- **The state has exactly one record of the date** - the instant. Do not add a
  `next_month` field or a stored calendar; two copies of the date will drift.
- **Every collection in `domain` is a `BTreeMap`.** A total iteration order is a
  determinism requirement, not a style preference.
- **Assigning someone to a project must add them to the project's `team` list.** Being
  assigned and being on the team are different things, and the bug where they came apart
  made every staffing decision meaningless.
- **Use `[System.IO.File]::ReadAllText` / `WriteAllText` for scripted edits to source
  files.** `Get-Content`/`Set-Content` round-trips corrupt non-ASCII characters (em
  dashes and section signs became mojibake) and silently drop a byte here and there.

The MVP's build plan is `docs/ROADMAP.md` (task-level, with dependencies and exit
criteria). `docs/ARCHITECTURE.md` fixes the crate and module layout the plan refers
to: `domain` (data and invariants), `sim` (the engine), `ui` (ratatui), and a thin
binary package. `domain` must never depend on `sim`, and `sim` must never depend on
`ui` - there is a CI check for this. `docs/DOMAIN_MODEL.md` is the field-level
contract, and `docs/BALANCE.md` holds every tunable number; **numbers belong in
`sim::balance` and nowhere else.**

## The time model (the big one)

**The game is not turn-based.** It is continuous simulated time running in real time at
a player-chosen dilation. There is no month to resolve and no "end month" button.
See `docs/SIMULATION.md` revision 2 and `docs/MANAGEABLE_PARTS.md` §3.

- **The nine dilations are 60x, 300x, 600x, 3,000x, 6,000x, 30,000x, 60,000x, 300,000x, and 600,000x.**
  **The base rate is 60x — one second in life is one minute in game — and there is
  nothing slower.** A campaign is thirty to sixty years, and a rate below a minute a
  second would be a lifetime rather than a game. The dilation lives in the
  **host** (`ui::clock`), never in the
  simulation — the simulation is only ever told a *duration*.
- **Dilation independence [D]:** a campaign played at the base rate and at the top rate must be **the
  same campaign**. It holds because every random draw is anchored to a *scheduled
  instant* and never to an integration sub-step. If a draw drifts into a sub-step, the
  two campaigns diverge in the third month and nobody can see why.
- **`Space` pauses, `+`/`-` step the dilation, `Ctrl+1`..`Ctrl+9` set it.** `Ctrl+digit`
  rather than `Shift+digit`, because the shifted character is layout-dependent and the
  control modifier is not.
- **Unpausing does not release a backlog.** The clock consumes the real time it passed
  whether or not it was paused, so stopping for an hour does not jump the campaign
  forward by an hour.
- **Everything that happens at a definite time is on the scheduler**, ordered by
  `(at, priority, seq)` — a total order. Events resolve last within an instant, so
  they react to the work that has just happened.
- **A random draw is keyed on a stable identifier, never on an instant.** The month
  *index*, the event check's *index*, an entity's id. The month index survives a
  change to the length of a month, and the check's index survives a change to the
  check's interval — which tightens under stress, so the instants move and the index
  does not.
- **The campaign's clock is on the integration grid, and the request is measured
  from the campaign's *effective* time** (`Sim::effective_now()` = `now + carry`),
  not from the grid point. Otherwise the carried fraction pushes the campaign past
  the frame the host asked for, and two campaigns at different dilations end in
  different months. The host asks for `effective_now + delta`.
- **The version is computed, not written down.** `sim::version::sim_version()` hashes
  the phase order, the integration quantum, the RNG algorithms, the order kinds, and
  the scheduled kinds. A hand-written version stops being true the first time a rate
  moves.
- **The scheduler is part of the save.** A loaded campaign restores its pending work
  rather than re-scheduling it; otherwise every month boundary that had already
  happened fires again and the campaign silently changes. The save-and-load
  determinism test is what catches that.
- **The continuous quantities are rates, and they live in `sim::rates`.** The module
  takes **no `Rng`** — that is how "no draw inside an integration sub-step" is
  enforced: by the type system, not by discipline. A project that reaches its target
  inside the integration schedules a `ProjectComplete` item instead, and the
  completion draws when the item fires.
- **The integration grid is one hour, anchored at the start of the campaign.** A rate
  is often a function of the state, so the result of an integration depends on how
  finely it was done. The grid is what makes dilation independence hold: the same
  campaign advanced in one-hour pieces and in one-week pieces produces the same
  state hash. `crates/sim/tests/determinism.rs` proves it.
- **The remainder is carried, and it is in the state.** `domain::scale::accrue`
  telescopes, so the total is exact to within one scaled unit however finely the
  campaign is integrated. The remainder lives in `domain::scale::Remainders` on the
  person, the project, the treasury, and the construction — **a save that dropped it
  would produce a different campaign from the one it was taken from.**
- **The denominator of a rate must not change.** The carried remainder is only
  meaningful against the denominator it was carried with.
- **Orders take time.** Every order has a lead time, a direct cost, and a goodwill
  charge, and all three are shown before it is confirmed. An order is validated when
  it is issued *and* when it lands.
- **An order with no registered handler is rejected, never ignored.** `Sim::issue`
  refuses it with a reason. The kinds without handlers yet are the programme, policy,
  and publication machinery.
- **A lead time may not be random.** An order that landed at a random time would
  make the campaign depend on the draw, and the order log would not replay. The lead
  times are in `sim::balance` and `sim::orders`, and they are a function of the
  order's content.
- **A campaign is forked by replaying the order log** (`Sim::fork`). The branch is
  the *same campaign* up to the fork point, so the comparison is meaningful. The
  order log therefore records the **effective time** of each issue, not the grid
  point — the carried fraction has to be reproduced exactly or the branch and the
  original drift apart.
- **The orders in flight are part of the state** (`CampaignState::pending_orders`),
  and so part of the save. A campaign saved with a hire in flight must restore it.
- **The cost is charged when the order lands**, not when it is issued.
- **Goodwill is a price, not a cap.** `PoliticalState::goodwill`, from −100 to +100.
  Every landed order draws on it, a refused order does not, and it regenerates at
  three percent a month. **The effect is zero between ±40 and grows to its maximum at
  the extremes** — a hostile state makes every order take half again as long and
  charges a third more. The effect is computed at the *issue*, so the player sees the
  delay and the price before they commit.
- **The continuous quantities are scaled integers** at 256 units per percent
  (`domain::scale::PERCENT_UNITS`). This is not optional: a rate of 0.008% an hour
  truncated to an integer never moves, and the visible symptom is that nothing happens
  at the base rate.
- **Being assigned to something and being on its team are different things**, and a
  person who is on two teams is working for both. `sim::phases::personnel::assign`
  moves the assignment and the team lists — the project team, the test campaign
  team, and the facility's staff — together. **Never open-code an assignment**;
  two bugs of exactly this shape have already been found.
- **Every threshold that compares against a scaled quantity must be scaled too.** A
  check like `morale <= -30` written against a scaled morale is always true, and the
  symptom is a campaign that eats its own staff for no visible reason. This happened
  during M1-02 and the tests caught it.
- **The state has exactly one record of the date** — and that record is the `Instant`
  in the `Sim`, not the state. The state holds no date at all. Do not add a calendar
  field or a `next_month`.
- **Every collection in `domain` is a `BTreeMap`.** A total iteration order is a
  determinism requirement, not a style preference.
- **Assigning someone to a project must add them to the project's `team` list.** Being
  assigned and being on the team are different things, and the bug where they came apart
  made every staffing decision meaningless.
- **Use `[System.IO.File]::ReadAllText` / `WriteAllText` for scripted edits to source
  files.** `Get-Content`/`Set-Content` round-trips corrupt non-ASCII characters (em
  dashes and section signs became mojibake) and silently drop a byte here and there.
  **And use single-quoted PowerShell strings for any text containing a backtick** — in
  a double-quoted string `` `0 `` is an escape for NUL, which silently corrupted a
  source file during M1-02.
- **M1-07 must carry a remainder when it integrates rates over elapsed time.** Truncating
  `rate x elapsed / period` on every application loses the rate over many
  applications — 136 percentage points of fatigue over a 30-year campaign, and no test
  would fail. See `docs/ROADMAP.md` §4.1.1.

**Note [D]:** the engine is **continuous now** (M1 is complete). The clock, the
scheduler, the advance loop, the rates, the order model with its lead times, and the
goodwill are all done. **The month boundary is a scheduled item, not a cadence** — the
campaign advances continuously and the boundary is one thing that happens on the way
past. Read `docs/SIMULATION.md` before changing any of it.

The constraints an implementer would otherwise get wrong:

- This is a **management/grand-strategy sim about running a space agency**, not a rocket
  simulator and not a spreadsheet sim. The spec says so explicitly.
- The player manages *people* (scientists, engineers, astronauts, managers) and
  *programmes* (budgets, personnel, objectives, schedules, risks) — not vehicles.
- **Technology emerges from investment and expertise** (research funding, staff
  experience, testing campaigns, mission outcomes, industrial capacity), not from a
  fixed tech tree.
- Outcomes should be **emergent and replayable**, not puzzle-like: no decision has an
  objectively perfect answer, and safety/prestige/exploration/profit pull against
  each other.
- The setting is a **procedurally generated alien civilisation**, so the generator
  (civilisation, staff, programmes) is a first-class subsystem, not fixture data.
- Core loop: fund → hire → invest → design programmes → build/launch → analyse →
  expand, at increasing scale. Long-term arc runs from atmospheric research through
  orbitals, lunar and planetary exploration, stations, resource extraction,
  interplanetary transport, megastructures, interstellar missions.
