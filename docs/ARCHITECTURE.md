# ARCHITECTURE.md

> **Where the code lives, and what each crate may know about the others.**
>
> **Status:** Design intent, revision 1. The crate layout, the dependency direction,
> and the module map below all match the code as it stands.

---

## 1. Principles

1. **Dependencies point one way.** `ui` → `sim` → `domain`. Never the reverse. `domain`
   depends on nothing of ours.
2. **The simulation never imports the UI.** Not for types, not for constants, not
   "temporarily".
3. **The simulation never panics.** Every fallible path returns a typed error. A panic
   in the sim is a bug that corrupts a campaign.
4. **The UI never mutates simulation state.** It issues *orders* and renders what it is
   given.
5. **`domain` holds data and invariants; `sim` holds behaviour.** If a rule needs to be
   enforced on every write, it belongs in `domain` as an invariant or a checked
   constructor.
6. **One concept, one module.** A module that holds two unrelated concepts will be
   changed for two unrelated reasons.
7. **The simulation never knows the frame rate.** The host tells it how much *simulated*
   time has passed. The dilation lives in the host, and the simulation has no idea it
   exists — which is what makes dilation independence hold.

---

## 2. Crate layout

```
space_manager/                    (repo root)
├─ Cargo.toml                     [workspace] — members = ["crates/*"]
├─ Cargo.lock                     — workspace lockfile, at the root
├─ README.md
├─ .github/workflows/ci.yml
├─ docs/
└─ crates/
   ├─ domain/                     domain model, ids, invariants. No behaviour beyond checks.
   │  └─ src/
   ├─ sim/                        the engine: scheduler, phases, rates, orders, events
   │  └─ src/
   ├─ ui/                         ratatui: screens, widgets, input
   │  └─ src/
   └─ space_manager/              the binary package. Name MUST stay "space_manager".
      └─ src/main.rs              boots the TUI and wires the crates together
```

**Why the binary lives in `crates/space_manager`:** the IntelliJ run configuration in
`.idea/workspace.xml` invokes `cargo run --package space_manager --bin space_manager`.
The **package name** must stay `space_manager` — the directory may move, the package
name may not. This is also why the `non_kebab_case_bins` manifest warning is silenced
rather than "fixed": see §7.

**Why the domain crate is called `domain` and not `core` [D]:** a package named `core`
**shadows the standard library's `core` crate in the extern prelude**, and every
dependent crate then fails to resolve `core::fmt` and friends:

```text
error[E0433]: cannot find `fmt` in `core`
 --> crates/sim/src/probe.rs:1:25
  |
1 | pub fn probe() -> core::fmt::Result { Ok(()) }
  |                         ^^^^^^ could not find `fmt` in `core`
```

This was found during M0 and verified before the rename, not assumed. `domain` is
short, descriptive, and collides with nothing.

**The root `Cargo.toml` is a virtual workspace manifest** — `[workspace]` only, no
`[package]`. The root package lives in `crates/space_manager`.

```toml
# Cargo.toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
```

`resolver = "3"` is required for the 2024 edition and is the correct default; it is
pinned explicitly so it is never ambiguous.

---

## 3. Dependency graph

```
        ┌──────────────────────┐
        │      space_manager   │  binary: main(), wiring, panic boundary
        └──────────┬───────────┘
                   │
      ┌────────────┼────────────┐
      ▼            ▼            ▼
  ┌───────┐   ┌────────┐   ┌────────┐
  │  ui   │   │  sim   │   │ domain │      domain ──▶ (nothing)
  └───────┘   └───┬────┘   └────────┘
                  └──────────▶ domain
```

| Crate | May depend on | Must not depend on |
|---|---|---|
| `domain` | `serde` (derive only) | `sim`, `ui`, anything of ours |
| `sim` | `domain`, `rand_chacha`, `serde` | `ui` |
| `ui` | `domain`, `sim`, `ratatui`, `crossterm` | - |
| `space_manager` | all three | - |

**Enforce it in CI.** A dependency-direction check is a few lines of script and it
prevents the single most likely architectural decay. It is a `cargo tree` assertion in
a shell script, or a small Rust test.

---

## 4. Module map

### `crates/domain` — the domain model

```
src/
├─ lib.rs              re-exports; the only public surface
├─ ids.rs              typed id newtypes and the id allocator
├─ time.rs             Instant, Duration, the civil calendar, TimeScale
├─ money.rs            Money (integer minor units)
├─ scale.rs            the fixed-point scale: PERCENT_UNITS and the conversions
├─ person.rs           Person, Role, Attributes, Traits, Assignment, Contract
├─ performance.rs      the C+B modifier set and the input types it reads
├─ programme.rs        Programme, Objective, Commitment, Milestone, Status
├─ research.rs         Discipline, ResearchProject, Capability, Knowledge
├─ facility.rs         Facility, FacilityType, Construction, Condition
├─ vehicle.rs          Archetype, Slider, Vehicle
├─ mission.rs          Mission, MissionPhase, Outcome
├─ finance.rs          Budget, Treasury, LedgerEntry, IncomeSource
├─ political.rs        Approval, PoliticalCapital, Promise, Goodwill
├─ public.rs           PublicOpinion, Prestige
├─ chronicle.rs        ChronicleEntry, Severity, Tag
├─ event.rs            EventDef, Effect, Condition, Category
├─ order.rs            Order, OrderKind, OrderId — a decision, and the policy it can carry
├─ report.rs           MonthReport and the report's sections
├─ state.rs            CampaignState — the aggregate root — and its invariants
└─ invariant.rs        the checked constructors and assertions
```

`domain` has **no logic that decides anything**. It holds data, enforces that the data
is coherent, and defines the vocabulary. Everything that *computes* lives in `sim`.

### `crates/sim` - the engine

```
src/
├─ lib.rs              Sim, advance_to(), the public entry points
├─ tick.rs             the phase order, and the month's work
├─ phases/             the ten phases - the order of work at an instant
│  ├─ mod.rs           phase trait, phase ordering table
│  ├─ calendar.rs      phase 0
│  ├─ intake.rs        phase 1 - validation only, writes nothing
│  ├─ personnel.rs     phase 2
│  ├─ research.rs      phase 3
│  ├─ construction.rs  phase 4
│  ├─ missions.rs      phase 5
│  ├─ finance.rs       phase 6
│  ├─ politics.rs      phase 7
│  ├─ world.rs         phase 8
│  ├─ events.rs        the events — a scheduled item, not a monthly phase
│  └─ commit.rs        phase 9
├─ scheduler.rs        the queue, the ordering rules, the item kinds
├─ rates.rs            the continuous rates, and the integration over elapsed time
├─ orders.rs           the order registry: the lead times, the costs, and the handlers
├─ derived.rs          every derived value (performance, morale, approval, ...)
├─ performance.rs      the C+B model implementation
├─ shift.rs            the shift: the output, the fatigue, and the morale it costs
├─ report.rs           building the month report
├─ feed.rs             the live feed: the chronicle read live, and the decisions
├─ persist.rs          save, load, version checks, the state hash
├─ version.rs          the behaviour-defining inputs, and the version they hash to
├─ error.rs            SimError and OrderError
├─ scenario.rs         scenario definitions and the shipped scenarios
├─ branch.rs           replay, forking, and the comparison of two branches
├─ balance.rs          all tunable numbers, in one place
├─ recruitment.rs      generating the people the agency hires
├─ names.rs            the name generator, one culture at a time
├─ deadlines.rs        the commitments, and the slack between a promise and a plan
├─ harness.rs          the balance harness: three reference policies, played out
├─ summary.rs          the plain-text report of a campaign
├─ rng.rs              stream derivation, the pinned hash, the PRNG wrapper
└─ events/             the event catalogue (data, one module per category)
   ├─ mod.rs
   ├─ technical.rs
   ├─ personnel.rs
   ├─ discovery.rs
   ├─ political.rs
   ├─ financial.rs
   ├─ environmental.rs
   ├─ public.rs
   └─ external.rs
```

**The phases are ten, not eleven.** The events phase is a *scheduled item* now, and it
resolves last within an instant so that it reacts to the work that has just happened.

**`rates.rs` takes no random source, and that is deliberate.** A draw inside an
integration sub-step would make the campaign depend on the number of sub-steps, and
the number of sub-steps depends on the dilation. There is no `Rng` in the module to
draw from, so the type system makes the mistake impossible rather than merely
unlikely.

**Every module in the map above exists.** The `events/` catalogue arrived with
Slice 3 and `order_palette.rs` with Slice 1-37; `sim::time` is still
`domain::time`, and moves when the clock's host-side half does.

### `crates/ui` — the terminal interface

```
src/
├─ lib.rs              App, run loop, the frame loop
├─ app.rs              application state, current screen, focus, the feed
├─ clock.rs            the host's clock: dilation, pause, real-to-simulated conversion
├─ input.rs            key handling, the global keymap
├─ screen.rs           the Screen enum and the screen trait
├─ order_palette.rs    the modal where the player gives an order
├─ advice.rs           what to do next, read from the campaign
├─ briefing.rs         the opening briefing
├─ confirm.rs          the modal that asks before something irreversible
├─ list_control.rs     the filter and the sort, per screen
├─ tutorial.rs         the hints that wait until the player acts
├─ manual/             the manual's topics, one module per domain
├─ screens/
│  ├─ mod.rs
│  ├─ dashboard.rs
│  ├─ staff.rs
│  ├─ programme.rs
│  ├─ research.rs
│  ├─ vehicles.rs
│  ├─ chronicle.rs
│  ├─ branches.rs      forking the campaign, and comparing the two branches
│  ├─ keymap.rs        the keymap modal
│  ├─ manual.rs        the manual over the interface
│  └─ report.rs        the report modal
├─ widgets/
│  ├─ mod.rs
│  ├─ binding_factor.rs     the performance explanation panel
│  ├─ person_row.rs
│  ├─ budget_bar.rs
│  ├─ commitment_row.rs
│  ├─ slack_meter.rs
│  └─ chronicle_list.rs
└─ theme.rs            colours, glyphs, spacing
```

**`widgets/binding_factor.rs` is the most important file in the UI.** It is the
explanation of why a team is slow, and `MVP.md` §6 identifies UI legibility of the
performance model as the top risk. It is built and iterated on before the model is tuned.

**`order_palette.rs` is the second.** It is the only way the player acts, and it is
where the lead time, the cost, and the goodwill cost of an order are shown before the
order is confirmed.

**`clock.rs` is the host's half of the continuous model.** It owns the dilation and the
pause, and it turns real elapsed time into simulated elapsed time. **The simulation is
never told which dilation is in force** - that is what makes dilation independence
hold, and it is why the clock lives here and not in `sim`.

### `crates/space_manager` — the binary

```rust
fn main() -> std::process::ExitCode {
    // parse args (--seed, --scenario, --headless)
    // build the sim, load or create a campaign
    // hand off to ui::run, and let the panic boundary live here
}
```

---

## 5. The public API of each crate

Boundaries are only real if they are enforced by types.

### `domain` exports

```rust
pub use ids::{AgencyId, PersonId, ProgrammeId, FacilityId, VehicleId, MissionId,
              ResearchProjectId, EventId, ChronicleId, OrderId};
pub use time::{Date, Duration, Instant, TimeScale};
pub use money::Money;
pub use state::CampaignState;
pub use order::{Order, OrderKind};
pub use chronicle::{ChronicleEntry, Severity};
pub use event::{EventDef, Effect, Condition};
// …the domain types
```

### `sim` exports

```rust
pub struct Sim {
    state: CampaignState,
    scheduler: Scheduler,
    rng: Rng,
    now: Instant,
    sim_version: SimVersion,
}

impl Sim {
    pub fn new(scenario: &Scenario) -> Result<Self, SimError>;
    pub fn state(&self) -> &CampaignState;
    pub fn now(&self) -> Instant;
    pub fn effective_now(&self) -> Instant;
    pub fn log(&self) -> &[LoggedOrder];
    pub fn pending_orders(&self) -> &BTreeMap<OrderId, Order>;
    pub fn pending_lines(&self) -> &[(Instant, ReportLine)];

    /// Advances the simulation to an instant. The only way time moves.
    pub fn advance_to(&mut self, target: Instant) -> Result<Vec<MonthReport>, SimError>;

    /// Issues an order. Validated now, scheduled for its lead time.
    pub fn issue(&mut self, order: OrderKind) -> Result<OrderId, OrderError>;

    pub fn save(&self) -> Result<String, SimError>;
    pub fn load(text: &str) -> Result<Self, SimError>;
    pub fn state_hash(&self) -> u64;

    /// Rebuilds the campaign from the order log alone.
    pub fn replay(&self) -> Result<Self, SimError>;

    /// The same campaign, up to an instant.
    pub fn fork(&self, at: Instant) -> Result<Self, SimError>;
}
```

`advance_to` is the only way time moves, and `issue` is the only way the player acts.

### `ui` exports

```rust
pub struct App { ... }
pub fn run(app: &mut App) -> Result<(), UiError>;
```

`ui` owns the `Sim` and calls `advance_to`. It cannot reach inside it — `state()` is
read-only.

---

## 6. Data flow for one instant

```
  the host's frame
      │
      ▼
  ui::clock          real elapsed × dilation = simulated elapsed
      │
      ▼
  sim::advance_to(now + simulated elapsed)
      │
      ├─ grid         the clock lands on the one-hour integration grid
      ├─ rates        continuous integration, one quantum at a time
      ├─ scheduler    every item due at this instant, in the total order
      ├─ phases       the ten phases, for the month's work
      └─ commit      the instant's changes, applied atomically
      │
      ▼
  new CampaignState (an immutable snapshot)
      │
      ▼
  ui::app            redraw the panels, append to the feed,
                     refresh "needs attention"
```

**The UI holds a snapshot, not a live reference.** After `advance_to` returns, the
state is replaced wholesale. The UI can never hold a borrow across a mutation, and the
"re-simulate and compare" feature in `branch.rs` stays trivial.

**The host owns the clock, the dilation, and the pause.** The simulation is told a
simulated duration and nothing else.

---

## 7. The manifest warning

The `non_kebab_case_bins` warning appears on every cargo invocation because the package
is `space_manager`. Renaming the package to `space-manager` would break the IntelliJ
run configuration, so instead **silence it explicitly** in the root manifest:

```toml
[workspace.lints.cargo]
non_kebab_case_bins = "allow"
```

A package cannot combine `[lints] workspace = true` with its own `[lints.cargo]` table,
so the lint is declared once at the workspace level and inherited by every member.

---

## 8. Error handling

| Layer | Mechanism | Examples |
|---|---|---|
| `domain` | `thiserror` types; checked constructors return `Result` | `invalid_salary`, `self_referential_assignment` |
| `sim` | `thiserror` types. `SimError` for unrecoverable problems, `OrderError` for rejections | `insufficient_funds`, `person_already_assigned` |
| `ui` | `thiserror`; recoverable problems become messages, not errors | `save_failed` |
| binary | `anyhow` for startup, then a clean exit | |

**Rules:**

- **No `unwrap`, `expect`, or `panic!` in `domain` or `sim`.** Not in tests either,
  except where the test *asserts* the panic.
- **No `unwrap` in `ui`** either, for the same reason: a panic there loses the campaign.
  The binary's `main` is the only panic boundary.
- An `OrderError` is **not** a simulation failure. It is information for the player,
  delivered through the order palette. The simulation keeps running.
- A `SimError` means the instant could not be resolved. The state is left untouched,
  because the commit is atomic, and the player is told.

---

## 9. Dependency choices

| Crate | Why | Note |
|---|---|---|
| `ratatui` | TUI rendering, widget composition, the buffer model | Pin the version in `Cargo.lock`; the API has churned between majors |
| `crossterm` | Terminal input, raw mode, the event stream | Behind ratatui's backend |
| `rand` + `rand_chacha` | The seeded PRNG | **`ChaCha12Rng` explicitly.** Never `StdRng` or `SmallRng` — their algorithms are not stability-guaranteed across releases |
| `serde` + `serde_json` | Save files | JSON, not a binary format: saves stay inspectable, diffable, and hand-editable |
| `thiserror` | Error types in `domain`, `sim`, `ui` | |
| `anyhow` | The binary only | |

### Deliberately absent

| Not used | Why |
|---|---|
| `chrono`, `time` | The sim must not read wall-clock time, and the campaign calendar is a custom 360-day year. A date library would be a standing temptation to violate the determinism contract |
| `uuid` | Ids are monotonic `u64` counters. They must be stable, ordered, and reproducible from a replay — not random |
| `rand::thread_rng` | Same reason as the rest: no system randomness |
| `bevy`, `gamedev` stacks | The game is a simulation with a terminal view. An engine would add a scene graph and a main loop we do not want |
| Any macro-heavy DSL | The event and order systems are plain data plus plain Rust |

---

## 10. Performance budget

| Operation | Budget |
|---|---|
| One instant, 300 staff, 10 projects | < 2 ms |
| A five-year campaign, integrated at one-hour quanta | **< 1 s** in a release build |
| A 30-year campaign, integrated at one-hour quanta | < 5 s of computation in total |
| A fork at any instant | < 100 ms |
| UI redraw | < 16 ms, so input never feels laggy |

**The one-second budget is the MVP's success criterion nine, and it is
asserted by `crates/sim/tests/performance.rs`.** The budget is about the
shipped build, so the release-mode check is a job of its own in CI; a debug
build is five to twenty times slower and the test allows for it rather than
failing for it.

**The integration is the cost.** 30 years at a one-hour quantum is about 263,000
sub-steps, each touching every assigned person and every project — roughly 80 million
updates across the whole campaign, spread over hours of play.

**The sub-step cap matters.** A long gap is integrated in at most 4,096 sub-steps
rather than refused, so a player who leaves the game running for a week of real time
does not stall the frame.

**The engine must stay small enough to be fast.** If `sim` grows past roughly ten
thousand lines, something has been pushed into it that belongs in `domain` (data) or in a
data file (content).

---

## 11. Testing layout

```
crates/domain/tests/    domain invariants
crates/sim/tests/       golden campaigns, determinism, dilation independence,
                        property tests, fuzzing, performance
crates/ui/tests/        snapshot renders of each screen
```

Test fixtures live in `crates/sim/tests/fixtures/` as small JSON files: a scenario, an
order log with its issue instants, and the expected state hash. A golden test is a
fixture plus one assertion.

---

## 12. What is deliberately not split out

| Not a crate | Why |
|---|---|
| `balance` | A module inside `sim` with all numbers in one file. It is data, not a dependency |
| `rates` | A module inside `sim`. It is the core of the engine, not an independent concern |
| `events` | A module tree inside `sim`. Event definitions are data consumed by the engine |
| `persistence` | A module inside `sim`. It serialises `domain` types; it has no independent life |
| `assets` | There are no external assets in a TUI game. Text content lives in the modules that use it |
| `macros` | No DSL |

**A crate split is a promise of independent change.** None of these qualify yet, and
each premature crate adds a version boundary to maintain.

---

## 13. Related documents

- `SIMULATION.md` — the engine contract this layout serves.
- `DOMAIN_MODEL.md` — the types in `domain`.
- `PERSISTENCE.md` — the save format and version checks.
- `TESTING.md` — the test strategy across all crates.
- `UI.md` — the screens and the information design.
- `BALANCE.md` — the numbers.
- `ROADMAP.md` — the tasks that build each of these.
- `../AGENTS.md` — repository conventions.
