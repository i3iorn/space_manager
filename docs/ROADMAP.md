# ROADMAP.md

> **The build plan.** Task-level, with dependencies and exit criteria.
>
> Implements `MVP.md` on the architecture in `ARCHITECTURE.md`, against the
> contracts in `SIMULATION.md` and `DOMAIN_MODEL.md`.
>
> **Status:** Plan, revision 2. Milestones M0 and S1 are complete; M1 is in progress.

---

## 1. How to read this plan

Each task has an ID, a deliverable, and a **done-when** condition. The done-when
conditions are the plan's real content — they are what makes the plan checkable
rather than aspirational.

| Field | Meaning |
|---|---|
| **ID** | `M<n>-<nn>` for milestones, `S<n>-<nn>` for slices. Stable. Referenced by issues and commits |
| **Depends on** | Task IDs that must be complete first |
| **Done when** | The verifiable condition. If it cannot be checked, it is not a task — it is a wish |

**Ordering rule:** within a milestone, tasks are listed in the order they should be
started. Independent tasks may be done in parallel, but a task's *done when* never depends
on work that has not landed.

**Slices are vertical.** Each one ends with something playable. No slice ends with "the
model is done" — it ends with "a person can do this".

---

## 2. Milestone M0 - Foundation

*Before any gameplay. Small, and it must not grow.*

**Status: complete.** All eleven tasks are done. One deviation and two adjustments are
recorded in §2.1; §2.2 lists what M0 actually produced.

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M0-01 | Convert the repo to a cargo workspace. Root `Cargo.toml` becomes a virtual manifest with `[workspace] members = ["crates/*"]` and `resolver = "3"`. Move the package to `crates/space_manager`. | - | `cargo build` succeeds from the root; `cargo run -p space_manager` still runs |
| M0-02 | **Keep the package name `space_manager`.** Verify the IntelliJ run configuration still resolves; re-index the project in IDEA. | M0-01 | The IDEA run configuration launches the binary without a manual path fix |
| M0-03 | Silence the manifest warning: `[lints.cargo] non_kebab_case_bins = "allow"`, declared once in the **root** manifest and inherited by every member. | M0-01 | `cargo build` prints no warning at all |
| M0-04 | Create the three library crates with the module skeletons from `ARCHITECTURE.md` §4. | M0-01 | `cargo build` compiles three empty-but-structured crates |
| M0-05 | Add the dependency-direction check. | M0-04 | The check fails if a forbidden dependency is introduced |
| M0-06 | Pin the PRNG: `rand` + `rand_chacha`, using `ChaCha12Rng` explicitly. Record the exact versions in `Cargo.lock`. | M0-04 | `Cargo.lock` contains `rand_chacha` and the code names `ChaCha12Rng` |
| M0-07 | Add `thiserror` to `domain`, `sim`, `ui`; `anyhow` to the binary only. | M0-04 | Errors are typed in the libraries |
| M0-08 | Add `serde` + `serde_json`. | M0-04 | A test type round-trips through JSON |
| M0-09 | Add a clippy configuration: `clippy::pedantic` as a warning, with a short allow-list for the specific lints that fight the domain model. | M0-04 | `cargo clippy` is clean on the skeleton |
| M0-10 | Write the README for the repo: what the game is, how to build, how to run, where the docs are. | - | A new contributor can build and run without asking |
| M0-11 | Set up `rustfmt` defaults (no config file - the defaults are correct for now). | M0-04 | `cargo fmt --check` is clean |

**M0 exit:** `cargo build`, `cargo clippy`, `cargo fmt --check` and `cargo test` are all
clean, the workspace builds from the root, and the docs index points at the new files.

### 2.1 Deviations from the plan

**The domain crate is named `domain`, not `core`.** The plan specified `core`, and the
name was found to be unusable during M0-04: a package named `core` shadows the
standard library's `core` crate in the extern prelude, and every dependent crate then
fails to resolve `core::fmt`. The failure was **reproduced before the rename**, not
assumed. `ARCHITECTURE.md` §2 records the error and the reasoning.

**`[lints.cargo]` moved to the workspace level.** A package cannot combine
`[lints] workspace = true` with its own `[lints.cargo]` table, so the single lint that
needs silencing is declared once in the root manifest and inherited by every member.

**`serde_json` is a dependency of `sim`, not `domain`.** The save format is JSON and the
serialisation lives in `sim::persist`; `domain` only needs `serde`'s derives.

### 2.2 What M0 actually produced

Beyond the eleven tasks, M0 landed a few things early because they were the honest way
to complete a task rather than a way to expand the scope:

- **`sim::rng` is complete**, not a stub: the pinned FNV-1a hash, the pinned
  SplitMix64 seed expansion, named substreams, and five tests including golden
  values for the derived seeds. The golden values exist to catch a change to the hash
  or the expansion, either of which would silently invalidate every replay.
- **`sim::persist` has a working save envelope** with format and version checking, and
  four tests covering round-tripping, garbled files, a newer format, and a foreign
  engine version.
- **`ui` has a working terminal shell** - raw mode, the alternate screen, a `Drop`
  guard that restores the terminal even on panic, a render pass, and global key
  handling. The screens are still placeholders.
- **`domain::ids` is complete**, with the typed id newtypes, the allocation counters,
  and three tests.

---

## 3. Slice 1 - *Ground Game* (months 0-11)

*A playable first year.*

> **This slice was built on the monthly model, which the time redesign replaced.**
> Everything here still works and its tests still pass, but the shape was wrong: the
> simulation resolved in months, the player issued directives for "the next month",
> and the interface had no way to give an order at all. Milestone M1 (§4) has now
> rebuilt it, and the order palette (S1-37) has given the player the last of it.
> What follows is the record of what was learned, and what is still missing.

**Status: nearly complete.** The engine, the domain model, the performance model, the
playable loop, the interface — **including the order palette** — the golden fixture,
the property tests, and the tutorial are all built. **What is missing is the one
thing a machine cannot do: the first playtest (S1-36).** The player can now watch a
year happen *and* act on it.

### 3.1 The engine skeleton

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-01 | **The RNG stream derivation.** `hash(root_seed, domain, period, discriminator)` using a hash pinned in our own code, plus `ChaCha12Rng` per stream. | - | A test asserts known stream outputs for fixed inputs, and the test's expected values are written down as literals | Done |
| S1-02 | **The determinism harness.** A test that runs a campaign twice and asserts identical state hashes. | S1-01 | The test exists and passes on a trivial campaign | Done |
| S1-03 | **The id allocator.** Monotonic `u64` per type, in `CampaignState`. | M0-04 | Ids are stable across save/load | Done |
| S1-04 | **The tick driver.** The phase table, with each phase a module and a `Phase` trait. Phases may be no-ops at first. | S1-02 | A month advances and produces a report | Done |
| S1-05 | **The state container.** `CampaignState` with `BTreeMap` collections and the invariant checker. | S1-03, S1-04 | The invariant catalogue has a test per row | Done |
| S1-06 | **The directive registry.** A trait, a registry, and one handler (`Hire`) to prove the mechanism. | S1-04 | A directive with no handler is rejected with a reason | Done |
| S1-07 | **Directive validation.** An error per failure mode, each with a player-readable message. | S1-06 | The "already assigned" rejection produces the exact message in `SIMULATION.md` §4.1 | Done |
| S1-08 | **The month report.** The structured report and its sections. | S1-04 | A report renders, with empty sections shown as empty | Done |

### 3.2 The domain, minimum viable

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-09 | `Person`, `Role`, `Attributes`, `PersonState`, `Assignment`, `Contract`. | S1-05 | People can be created, and the attribute ranges are enforced | Done |
| S1-10 | `Money` with the shared, deterministic split helper. | S1-05 | Splitting 100 among 3 gives 34/33/33 and never loses a unit | Done |
| S1-11 | `ResearchProject`, `Discipline`, `Progress`. | S1-05 | A project accrues progress and can complete | Done |
| S1-12 | `Facility` with a fixed set of types and a stub construction. | S1-05 | A facility exists, is staffed, and produces output | Done |
| S1-13 | `ChronicleEntry` and the append-only chronicle. | S1-05 | Entries are written and can be filtered by category | Done |
| S1-14 | `Budget`, `Treasury`, `IncomeSource` with the base wired. | S1-10 | The monthly appropriation is a function of the political multiplier | Done |

### 3.3 The performance model

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-15 | **The modifier set** in `domain`: the eight kinds, integer range -100..=+100. | S1-09 | The enum exists and modifiers are validated | Done |
| S1-16 | **The C+B computation**: clamp above zero, saturate toward the maximum, integer arithmetic, and the **binding factor** returned alongside the result. | S1-15 | A test shows that no single factor can zero a team, and that saturating a factor stops helping | Done |
| S1-17 | **The saturation curves.** The actual numbers, in `sim::balance`. | S1-16 | The curves are named constants, not literals in the code | Done |
| S1-18 | **The explanation.** A structure that decomposes a performance result into its modifiers and names the binding factor. | S1-16 | A test asserts the binding factor is the largest cost for a crafted input | Done |
| S1-19 | **Fatigue, morale, and health dynamics** in the personnel phase. | S1-16 | A person on a full assignment accrues fatigue and a resting one recovers | Done |

### 3.4 The first playable loop

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-20 | Personnel phase: apply assignments, accrue fatigue, drift morale, apply attrition. | S1-19, S1-04 | A person with no assignment drifts in morale | Done |
| S1-21 | Research phase: accrue progress, roll for breakthroughs, unlock capabilities. | S1-11, S1-20 | A project completes and a capability unlocks | Done |
| S1-22 | Finance phase: payroll, programme spend, upkeep, treasury update. | S1-14, S1-20 | A month of salary is paid and the treasury moves | Done |
| S1-23 | Calendar phase: month rollover, pay dates, annual fiscal true-up. | S1-04 | Twelve months advance and the fiscal year closes | Done |
| S1-24 | The `Hire`, `Assign`, and `Train` handlers. | S1-06, S1-09 | A player can hire a scientist and put them on a project | Done |
| S1-25 | **The scenario format** and the first shipped scenario. | S1-23, S1-14 | `cargo run --scenario first_year` starts it | Done |
| S1-26 | Save and load. | S1-25 | A saved campaign reloads to an identical state hash | Done |

### 3.5 The interface

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-27 | The TUI skeleton: app state, screen enum, key handling, the top bar, and a redraw loop. | M0-04, S1-08 | The app runs, quits cleanly, and redraws on input | Done |
| S1-28 | The dashboard screen: date, treasury, burn rate, approval, the headline, and what is due. | S1-27, S1-22 | The dashboard shows the current month and treasury | Done |
| S1-29 | The staff screen: a list with filters, and a detail panel. | S1-27, S1-18 | A person can be selected and their modifiers seen | Done |
| S1-30 | **The binding-factor widget.** The explanation of why a team is slow, in plain words. | S1-29 | The widget names the binding factor and shows the modifier breakdown | Done |
| S1-31 | The research screen: projects, progress, assignments. | S1-27, S1-21 | A project can be started from the UI | Done |
| S1-32 | The chronicle screen: the history, filterable, and the report. | S1-27, S1-13 | The player can find a month and read what happened | Done |
| S1-33 | **A tutorial in the first month** that teaches the binding factor. | S1-30 | A playtester can explain, unprompted, why their team is slow | Built, not yet playtested |
| S1-37 | **The order palette.** The modal where the player gives an order, with the lead time, the cost, and the goodwill charge shown before they confirm. | S1-06, M1-08 | The player can give an order, and sees what it will cost | Done |

### 3.6 The first proof

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S1-34 | The golden-test fixture: a seed and a directive log with the expected state hash. | S1-26, S1-02 | The test passes and the expected hash is in a file, not in the test | Done |
| S1-35 | The property tests for the invariant catalogue. | S1-05 | One test per invariant row | Done |
| S1-36 | **The first playtest.** Five people, one scenario, one hour each. | S1-34 | Notes written, and the "understands what to do in 5 minutes" question is answered honestly | **Not yet** |

**Slice 1 exit test:** a playtester hires two people, runs a research project for a year,
sees it complete, sees a breakthrough unlock something, and can explain why their team
was slow.

### 3.7 What Slice 1 actually delivered, and what it did not

**Delivered:**

| Area | What exists |
|---|---|
| `domain` | The whole minimum-viable model: people, money, programmes, research, facilities, vehicles, missions, finance, politics, public, the chronicle, events, orders, the report, the aggregate, and the invariant catalogue |
| `sim` | The phase driver, the scheduler, the continuous rates, the pinned RNG, the C+B performance model, the derived values, the order registry with its lead times and costs, the save envelope and state hash, the scenarios, the fork, and recruitment |
| `ui` | A working terminal shell with the top bar, the dashboard, the staff screen with the binding-factor panel, the programme, research, vehicles, chronicle, and branches screens, the order palette, the report modal, the manual, the keymap, and the tutorial |
| The binary | `--scenario`, `--seed`, `--headless`, and a help screen |
| Tests | The dependency-direction check, a determinism suite, a golden fixture with a readable chronicle diff, and a generated-state property suite for the invariant catalogue |

**Not delivered, and why it matters:**

| Missing | Why it matters |
|---|---|
| **No first playtest** (S1-36) | Needs a human. The exit test above is the criterion it must meet, and it is the one thing left in Slice 1 |
| **No events** (Slice 3) | The phase is in the order and does nothing yet. The campaign is therefore nearly deterministic in practice |
| **No vehicle or mission mechanics** | The types and the phase machine exist; the risk model and the sliders arrive with Slice 3 |
| **The live feed** (S1-37's successor) | The report is still a monthly modal, and the order log is a list rather than a feed |
| **Branching's comparison view** (S4-02) | The fork exists and the divergence is named, but there is no timeline of the two campaigns side by side |
| **The order palette has no free text** | Every parameter is a choice from a list, and a project's title is generated from its discipline |

**The interface can now give an order [D].** The order palette is a modal, opened
with `o`, and it shows the lead time, the price, and the goodwill charge before the
player confirms — and it refuses an impossible order at once, with a reason.

**The interface can also be read [D].** The work in §3.11 came out of the first
hands-on play, and it is a different kind of work from the rest of Slice 1: not new
mechanics, but the difference between an interface that works and one that can be
*used*. The focus is visible, the vocabulary is explained, the lists can be searched,
and the one control that eases the load on a tired team is a keystroke rather than
an order.

### 3.8 Bugs the Slice 1 work found

Four of these were caught by the tests, and each one would have been expensive later:

| Bug | How it was found | Why it mattered |
|---|---|---|
| **A hire overwrote a starting employee.** The scenario inserted people with hand-written ids without advancing the id counter, so the first hire was allocated id 0 - the same id as the agency's lead scientist. | A determinism test comparing two campaigns | The player's first hire would have silently destroyed a member of staff |
| **A project ran on one person while three more were marked busy.** `Assign` set the person's assignment but never added them to the project's team list, so the team the project drew on was always the principal investigator alone - while the others accrued fatigue for work they were not doing. | The golden chronicle came out empty, and a progress trace showed `team 1` | Staffing decisions would have had no effect on anything, and the fatigue mechanic would have punished people for work they were not doing |
| **Adding people to a project did not make it finish sooner.** The team's contribution was *averaged* rather than summed, so a three-person project progressed at exactly the rate of a one-person one. | The progress trace showed a flat rate | The single most important staffing decision in the game would have had no effect |
| **A negative budget split lost a minor unit.** The remainder kept the sign of the divisor, so splitting a deficit between programmes quietly rounded it away. | A unit test on `Money::split` | Budgets would have drifted, and the ledger would not have balanced |

**Two more were found when the order palette was built (M1, S1-37):**

| Bug | How it was found | Why it mattered |
|---|---|---|
| **A facility's staff list was never maintained.** Assigning somebody to a lab set their assignment and nothing else, so the lab's `staff` stayed empty and the tools modifier read as the worst it could be. | Writing the scenario's setup, and asking why the tutorial's binding factor would have been unfixable | The agency could never make its own work faster, and the binding factor said "tools" for reasons the player could not act on |
| **Starting a project left its lead on their old team.** `StartProject` set the new assignment and did not remove them from whatever they were on, so one person was on two project teams at once. | The golden fixture, after the scenario gained a running project | The same bug the first project-team bug above was: a person counted twice is working for both teams, and the project that lost them was quietly slower than it looked |

**The fix for both is one function**, `sim::phases::personnel::assign`, which moves the
assignment and the team lists together. **Being assigned to something and being on
its team are different things**, and a person who is on two teams is working for both.

Two more were found by building the thing rather than by testing it:

- **The performance composite saturated.** Each stage of the computation was clamped to
  200%, so every competent team read as exactly 200% and the fine differences the
  player is being asked to read were destroyed. The stages are now unclamped and the
  product is taken in `u128`.
- **The agency's first hire had no way to be given a name or an ability.** `Hire` was
  validated and then silently dropped. Recruits are now generated deterministically
  from the campaign's random source (`sim::recruitment`).

### 3.9 Design changes Slice 1 forced

These are changes to the *design*, made because the design did not survive contact with
the code. All of them are recorded in `BALANCE.md`.

| Change | Why |
|---|---|
| **The appropriation scales with headcount** (1,900 per head, floor 12,000) rather than being a fixed 120,000 | A fixed allocation made a three-person agency absurdly rich and a 62-person one bankrupt. Neither is true |
| **100% of performance means "a good team", not "a neutral team"** | A neutral team (every modifier at zero) is a mediocre team, and reading it as 100% was misleading. The reference composite is now the product of a good team's factors |
| **The saturation curve is an integer parabola, not an exponential** | The whole computation must be exact integer arithmetic, and an exponential cannot be |
| **A research project has a fixed 2,400-point target** and its rate comes from the team | The target moving with the team made the schedule meaningless |
| **Fatigue accrues at 6% a month, not 1% a day** | At 1% a day every engineer was worthless within a year, and there was no version of "rest" that mattered |
| **Basic re-entry costs 300, not 180** | The thresholds were supposed to overlap so that reaching orbit without a heat shield is a choice. They did not, because the list happened to be sorted. They are now deliberately not sorted |
| **A capability with a threshold of zero unlocks silently** | "The agency has developed Atmospheric sounding" was not an achievement - the agency always had it |

### 3.10 What happens next

**Slice 1 was built on the monthly model, and the monthly model is gone.** The code
works and its tests pass, but it is the wrong shape, and the redesign in §4 rebuilds it.

Milestone M1 (§4) is now **complete**, and so are the **order palette** (S1-37) — the
player can give an order, and sees the lead time, the price, and the goodwill charge
before they confirm it — and the **property tests** (S1-35) and the **tutorial**
(S1-33). What is left in Slice 1 is the one task that cannot be done by a machine:

1. **The first playtest** (S1-36), against the exit test in §3. Five people, one
   scenario, one hour each, and an honest answer to "understands what to do in five
   minutes".

### 3.11 The interface the player can read [S1-40 to S1-52]

**These tasks came out of the first hands-on play.** The player could *use* the game
and could not *read* it: the focus was invisible, the vocabulary was unexplained, and
the one control that eased the load on a tired team — unassigning them — did not scale
past about fifty people. **A management game about three hundred people has to be
navigable, and it has to be legible**, and neither of those is a feature that can be
added at the end.

| Task | What it is | Depends on | Exit test | Status |
|---|---|---|---|---|
| S1-40 | **The focus is obvious everywhere.** The selected row is in reverse video and bold, and carries a `▸` marker, so it is findable in a monochrome terminal as well as a colourful one. | — | Every list with a selection shows it, and the test reads the rendered buffer | Done |
| S1-41 | **The shift.** A per-person work intensity from 0% to 200%, in steps of 10%. It scales the person's output, their fatigue, and their morale, and it replaces "unassign them" as the way to ease the load. | M1-06, S1-27 | A person on light duties recovers while still on the team, and a person on 200% tires twice as fast | Done |
| S1-42 | **The shift is an order**, with a one-hour lead time and no cost, and it is a control on the staff screen — `←` and `→` — so it is reachable while the player is looking at the person. | S1-41, M1-08 | The player changes a shift from the staff screen and sees it land | Done |
| S1-43 | **Every list is filterable and sortable.** `/` filters, `s` sorts, and the filter is a mode so it can contain a `q` without the game quitting. | S1-27 | A player can find one person in three hundred by name, and the filter survives a trip to another screen | Done, for the staff list |
| S1-44 | **The staff list sorts by name, age, role, fatigue, attrition risk, and shift.** | S1-43 | The list can be ordered by who is about to leave | Done |
| S1-45 | **The attrition risk is a number the player can see**, on the list and in the detail panel, and it is the same number the simulation rolls against. | M1-10 | The number on the screen is the number the engine uses | Done |
| S1-46 | **The manual.** One store of topics — every stat, every concept, every order, every screen — reached three ways: `F2` for all of it, `I` for the screen you are on, and the info line for a one-line answer. | S1-27 | Every concept in the game has a topic, and no screen shows a number the manual does not explain | Done |
| S1-47 | **The tutorial hints stay until the player acts.** A hint marked as seen when the screen was first drawn is on the screen for one frame. | S1-33 | A hint is still on the screen after ten seconds of looking at it | Done |
| S1-48 | **The shift policy: set everybody's shift at once.** The per-person slider is the surgical tool; this is the tool for three hundred people, and it needs the policy order machinery that does not exist yet. | S1-42, S1-49 | A player can put the whole agency on light duties in one action | **Not yet** |
| S1-49 | **The policy order.** `SetPolicy` has no handler, and the shift policy is the first thing that wants one. | M1-08 | An order with a handler, a lead time, and a cost | **Not yet** |
| S1-50 | **Every other list is filterable and sortable.** The machinery is built and only the staff list is wired to it. | S1-43 | The research, programme, chronicle, and branches lists all filter | **Not yet** |
| S1-51 | **The filter can express a condition** — "fatigue over 80", "unassigned" — and not only a name. | S1-43 | A player can find everybody who is in trouble without reading three hundred names | **Not yet** |
| S1-52 | **The order palette can set a shift**, so the order is offered where every other order is offered. | S1-42 | The palette offers every order kind that has a handler | **Not yet** |

**Why the shift is an order and not a direct control.** The architecture says the
player never mutates state — they issue orders — and the shift is the most-used
control in the game, so it must not be the one thing that breaks the rule. **A
one-hour lead time is the price of that**, and it is a price worth paying: the
player sees the change in flight, the order log records it, and a campaign replayed
from the log reproduces it.

**Why the shift is not the whole answer to three hundred people.** It is the
*surgical* tool, and it scales because it is one keystroke on a screen the player is
already looking at. It is not the *policy* tool, and the policy tool is S1-48.

---

## 4. Milestone M1 - *Continuous Time* (the redesign)

*The time model changes from monthly turns to continuous simulated time. This milestone
rebuilds the engine around that change, and it is the gate for everything after it.*

**Status: complete.** M1-01 to M1-15 are done: the clock, the scaled quantities, the
host's clock, the scheduler, the advance loop, the draw anchors, the rates, the order
model, the goodwill, the stable buckets, the save format and the version, the host
loop, the top bar, the dilation-independence test, and the fork.

**One thing is still missing, and it is not an engine task.** The interface cannot
give an order — there is no order palette. That is **S1-37**, and it is the last
thing between this milestone and a player who can actually play.

**The governing document is `SIMULATION.md` revision 2.** Read §2 (the clock), §3 (the
engine), §4 (orders), §5 (events), and §6 (determinism) before starting.

### 4.1 The clock and the types

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M1-01 | `Instant` and `Duration` in `domain::time`, in integer milliseconds, with the civil calendar derived from them. `Month` and `Date` stop being stored values and become projections. | - | A date is computed from an instant, and nothing in the state holds a month |
| M1-02 | **The scaled quantities.** Fatigue, health, morale, loyalty, and progress move to scaled integers at 256 units per percent. Every read and write in the codebase is updated. | M1-01 | A rate of 0.008% an hour is not truncated away, and the existing tests still pass |
| M1-03 | `TimeScale` and the host's clock: the dilation, the pause, and the conversion of real elapsed time into simulated elapsed time. | M1-01 | The host can run at any of the nine dilations, and the simulation is never told which |

**M1-01, M1-02, and M1-03 are complete.** What they produced:

| From | To |
|---|---|
| `domain::time::{Month, Date, Calendar}` (a stored calendar) | `domain::time::{Instant, Duration, Date, Calendar, TimeScale}` - the instant is the only record of the date, and the calendar is a projection of it |
| `CampaignState.calendar` | Nothing. The state holds no date at all. The clock lives in the `Sim`, and the calendar is derived from the agency's founding year |
| `Month` in every date field | `Instant` - commitments, milestones, missions, promises, and chronicle entries are all instants |
| `Commitment::slack() -> Option<u32>` (months) | `Option<Duration>` |
| `PersonState` values at 0-100 | Scaled integers at 256 units per percent, with `fatigue_percent()`, `health_percent()`, and `morale_percent()` accessors |
| `Progress` at whole points | Scaled at 256 units per point |
| `Approval`, `PublicOpinion`, `Prestige` at -100..+100 | Scaled, with `percent()` accessors |
| A blocking input loop | A polling loop that ticks the clock every frame whether or not the player is touching anything |
| - | `domain::scale`, a small module holding the conversions and the `PERCENT_UNITS` constant |
| - | `TimeScale`, the nine dilations, in `domain::time` |
| - | `ui::clock`, the host's clock: the dilation, the pause, and the real-to-simulated conversion |
| - | `Sim::advance_to`, the API the clock calls, and the only way time moves |
| - | `Space` to pause, `+`/`-` to step the dilation, `Ctrl+1`..`Ctrl+9` to set it directly |

**The engine still advances a month at a time [D].** `Sim::advance_to` steps the old
monthly tick until it reaches the target, and the commit phase no longer touches the
calendar. That is the old cadence wearing the new clock, and it is replaced by M1-07.

**The dilation is the host's [D].** `ui::clock` owns it, and the simulation is never
told which one is in force. That is what makes dilation independence hold, and M1-14
is the test that proves it.

**Unpausing does not release a backlog [D].** The clock consumes the real time it
passed whether or not it was paused, so a player who stops for an hour does not come
back to a campaign that jumped forward by an hour.

#### 4.1.1 Two findings from M1-02

**Thresholds that were not scaled with their quantities [fixed].** The resignation
check compared a scaled morale against an unscaled `-30`, so every person looked
miserable and the agency was eating its own staff. The same applied to the fatigue
thresholds, the health threshold, the approval changes, and the initial approval in
the scenario. All of them are now scaled, and `domain::invariant` checks the scaled
ranges.

**The integration must carry a remainder [for M1-07].** This is the important one, and
it is easy to get wrong.

The rate is 6% of fatigue a month, which is 1,536 scaled units over 2,592,000
milliseconds. M1-07 will integrate that over elapsed time in sub-steps. If each
sub-step applies `rate x elapsed / month` and truncates, the rate is **lost**:

| Sub-step | True increment | Truncated | Error |
|---|---|---|---|
| 1 hour | 2.133 units | 2 | 0.133 units |
| 1 month | 1,536 units | 1,536 | 0 |

Over a 30-year campaign at hourly quanta - 262,800 applications - the truncation
loses 34,952 units of fatigue, which is 136 percentage points. **The fatigue rate
would be badly wrong and nothing would look broken.**

So M1-07 must carry the remainder between applications. The options are a per-quantity
remainder field, or deriving the quantity from a base plus `rate x elapsed`. Either is
fine; **doing neither is not**, and the symptom - a rate that is quietly a fraction of
what it should be - would not show up in any test that does not check the rate
directly.

### 4.2 The engine

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M1-04 | **The scheduler.** `ScheduledItem`, the queue, the ordering rules, the item kinds. | M1-01 | Items pop in a total order, and the sequence number is never reused |
| M1-05 | `Sim::advance_to`, the advance loop, replacing the monthly stepping. | M1-04 | The clock advances to an arbitrary instant, and everything due has happened |
| M1-16 | The `SaveEnvelope` carries the instant, the scheduler, and the order log. | M1-05 | A save restores everything the campaign was waiting on |

**M1-04 and M1-05 are complete.** What they produced:

| From | To |
|---|---|
| `Sim::step` resolving a month directly | `Sim::advance_to`, the only way time moves. `step` is now a thin wrapper over it, so there is exactly one path |
| The eleven monthly phases | **Ten**, plus the events phase as a scheduled item |
| The events phase running at the end of a month | `ScheduledKind::EventCheck`, at its own instants, on a weekly interval that tightens under stress |
| - | `sim::scheduler`, the queue, the total order, and the item kinds |
| - | `domain::ids::{OrderId, CommitmentId}`, for the kinds that arrive with M1-08 and Slice 2 |
| - | The scheduler is in the save, because a campaign that lost its pending work was a campaign that silently changed |

**The ordering [D].** A scheduled item is ordered by `(at, priority, seq)`, and the
priorities are: month boundary 10, commitment 40, order 50, mission 60, attrition 70,
**events 90**. Events resolve last within an instant, so they react to the work that
has just happened.

**A boundary fires *before* the target [D].** The work for a month is therefore dated
at the month it covers, rather than the month after it - which is what the old model
did, and what the golden fixture depends on.

#### 4.2.1 Two bugs M1-04 and M1-05 found

**The scheduler was not in the save [fixed].** A loaded campaign re-scheduled the
recurring work from scratch, which re-fired every month boundary that had already
happened, and the campaign silently changed. **The determinism test caught it** - the
save-and-load test produced a different campaign from the original. The scheduler is now
part of the save, and `Sim::load` restores it rather than re-scheduling.

**The events phase ran outside the month's commit [fixed].** The events phase now runs
as a scheduled item, outside the commit phase that writes the chronicle - so an event
would have fired and never been recorded. `resolve_events` now writes its own chronicle
entries. The event catalogue is still deferred, so nothing was lost yet, but it would
have been lost the moment it arrived.

#### 4.2.2 A note on the golden fixture

The M1-04/M1-05 refactor changed the golden state hash while leaving the **chronicle
byte-identical**. The narrative was checked and confirmed unchanged; the state hash
covers fields the chronicle does not show.

What was proved, by experiment rather than by inspection:

- The scheduler path and a direct path through `resolve_month` produce **the same
  state hash**.
- The event checks are **state-neutral**: disabling them changes nothing.
- The events phase's position in the order changes nothing.

So the refactor is behaviour-preserving in every way that could be measured, and the
hash difference lies in a state field outside the chronicle - the accumulated points and
dates of a research project that is still running at the end of the fixture. **The lesson
for the next milestone: the golden test's *chronicle* is the legible signal and its
*hash* is not.** The MVP should grow a test that asserts the campaign's *structure* -
the dates, the project progress, the treasury - rather than relying on a hash alone.

### 4.3 The rates, and the draw anchors

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M1-06 | Every random draw moves to a scheduled instant. **No draw may occur inside an integration sub-step.** | M1-04, M1-07 | The rates take no random source, and a campaign advanced in different-sized pieces is the same campaign |
| M1-07 | **`rates.rs`.** Every continuous quantity becomes a rate, and the integration over elapsed time is implemented, on the one-hour grid. **The remainder must be carried between applications** - see §4.1.1. | M1-02, M1-05 | A project advances continuously and a person's fatigue rises smoothly, at the right rate |

**M1-06 and M1-07 are complete.** What they produced:

| From | To |
|---|---|
| Fatigue, morale, and health as monthly deltas in the personnel phase | Rates in `sim::rates`, integrated over the elapsed time |
| Project progress as a monthly delta in the research phase | A rate, sampled from the team's effective performance |
| Construction progress as a monthly delta | A rate, and the build takes exactly as long as the plan says |
| The treasury moving by a month's net in the finance phase | A continuous burn, with the allocation still arriving monthly |
| The monthly phase list | The same ten phases, with the continuous work gone |
| - | `domain::scale::Remainders`, and `domain::scale::accrue` - the exact rate arithmetic |
| - | `domain::scale::Remainders` on the person, the project, the treasury, and the construction |
| - | The one-hour integration grid, and the carried leftover |
| - | `ScheduledKind::ProjectComplete`, for the completion that has to draw |
| - | The determinism test that advances a campaign in different-sized pieces |

**The grid is the answer to a question the design did not ask [D].** A rate is often
a function of the state — a project's progress depends on how tired its team is — so
the result of an integration depends on how finely it was done. If the host's frame
rate decided that, a campaign at the base rate and the same campaign at the top rate would diverge
in the third month. The grid is fixed at one hour from the start of the campaign, so
the two campaigns integrate identically, and the test in
`crates/sim/tests/determinism.rs` proves it.

**The remainder is in the state, because it is part of it [D].** A save that dropped
the remainders would produce a different campaign from the one it was taken from.
The arithmetic telescopes, so the total is exact to within one scaled unit however
finely the campaign is integrated.

**No random source in `sim::rates` [D].** The module takes no `Rng`, so the type
system makes a draw inside the integration impossible rather than merely unlikely.
A project that reaches its target inside the integration cannot be completed there,
so the crossing schedules a `ProjectComplete` item and the completion happens at a
definite instant.

### 4.3.1 Two bugs M1-06 and M1-07 found

**A finished project was never recorded [fixed].** The completion runs outside the
month's commit, so it does not share the month's working copy with the phase that
writes the chronicle. The project finished, its team was released, and **nobody was
ever told**. The golden chronicle came out empty, which is what caught it. The
completion now writes its own chronicle entries, exactly as the events phase does.

**The continuous work was applied before the month's work [fixed].** The advance loop
integrated the rates and then applied the scheduled items, which meant the month's
work at a boundary saw a world that had already lived the month. The loop now
applies the items due at an instant **first** and integrates afterwards, so the
month's work sees the world it woke up to, and an event check at the same instant
sees the month's work.

### 4.4 Orders

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M1-08 | The order model: `Order`, `OrderKind`, lead times, and the two validations. | M1-05 | An order takes time to land, and a refused order is reported with a reason |
| M1-09 | **Goodwill.** The resource, the charge on every order, and the effects at high and low levels. | M1-08 | Every landed order charges goodwill, and the dashboard shows it |

**M1-08 and M1-09 are complete.** What they produced:

| From | To |
|---|---|
| `domain::directive::Directive` | `domain::order::OrderKind` — the player's request, renamed to what it is |
| — | `domain::order::Order`, with the issue instant, the landing instant, the cost, and the goodwill |
| — | `CampaignState::pending_orders` — the orders in flight, and part of the save |
| A directive applied at the month boundary | An order that lands at its own lead time, on a `ScheduledKind::OrderLands` item |
| The intake phase validating the month's directives | The **first validation** at issue, and the **second validation** at landing |
| The cost applied when the directive was applied | The cost charged **when the order lands** |
| — | `sim::orders`: the lead times, the costs, the goodwill, and the handlers |
| — | `Sim::issue`, which refuses an impossible order at once, with a reason |

**The two validations [D]:**

1. **When the order is issued.** The player must be told immediately that it is
   impossible, not three weeks later. A refused order is reported with a reason and
   costs nothing. `Sim::issue` returns the reason.
2. **When the order lands.** The world has moved on. The person may have left, the
   project may have been cancelled. An order that can no longer be honoured is
   **refused at landing**, the cost is not charged, and the player is told why — in
   the report, and in the chronicle, which is the durable record.

**An order with no handler is refused, never ignored.** The kinds without handlers
yet — the programme, policy, and publication machinery — are refused at issue with a
reason. **Before M1-08 those kinds were silently dropped**: the intake phase
validated them, no phase consumed them, and they vanished. The doc's rule was in
`DOMAIN_MODEL.md` §16 the whole time and the code did not implement it.

**The lead times are in `sim::balance`, and they are deterministic [D].** A hire's
lead time scales with the seniority of the role being filled, so the agency cannot
buy a division head in a fortnight. A promotion waits for the next quarterly round.
A facility takes as long as its plan says. **A lead time may not be random**: an
order that landed at a random time would make the campaign depend on the draw, and
the order log would not replay.

**The handlers are per-kind.** An order lands at its own instant, and the phase that
owns it applies it — so each phase exposes an `apply_one` that takes a single order
rather than only a month's worth.

### 4.4.1 The goodwill

**The resource.** `PoliticalState::goodwill`, from −100 to +100, on the dashboard. It
is a scaled integer, like every other continuous quantity, and it regenerates at
**three percent a month** up to a cap of a hundred.

**The charge.** Every order that lands draws on it, in proportion to how disruptive
the order is: a hire costs one, a policy change costs five, a facility costs five to
fifteen by its scale. **A refused order costs nothing** — the goodwill goes with the
cost.

**The effects, at both ends [D].** The table in `MANAGEABLE_PARTS.md` §3.5 describes
them, and the implemented ones are:

| Goodwill | Effect | Implemented |
|---|---|---|
| Above +40 | Approvals are fast | **The lead time shortens** — by half at the extreme |
| Above +40 | The state grants favours | **The cost falls** — by a third at the extreme |
| Above +40 | Staff forgive more | **The morale a satisfying assignment holds rises** |
| Above +40 | A deeper applicant pool | Not yet — the recruitment module has no pool |
| Below −40 | Every order takes longer | **The lead time lengthens** — by half at the extreme |
| Below −40 | Costs rise | **The cost rises** — by a third at the extreme |
| Below −40 | Staff disengage | **The morale a satisfying assignment holds falls** |
| Below −40 | Events in which the state asks why | Not yet — the event catalogue is Slice 3 |
| Below −80 | The state reconsiders the mandate | **A warning on the dashboard**; the crisis itself is Slice 3 |

**The effect is zero between the thresholds and grows to its maximum at the
extremes.** That is what the design table says — "above +40", "below −40" — and it
avoids a cliff: a director at +39 and one at +40 are not different people.

**The player sees the delay and the price before they commit.** The lead time and the
cost are both computed at the *issue*, from the goodwill as it stands then, so the
political state is a lever the player can pull rather than a surprise they discover
later.

**A broken promise costs ten goodwill**, which is more than a week of ordinary
orders, and a kept promise and a successful mission each grant a point. The state's
memory is the mechanism behind the whole commitment model, and the goodwill is where
that memory shows up.

### 4.4.2 The golden fixture moved

The M1-08 refactor changed the golden campaign, and the chronicle diff says so:
the project now completes at `2039-12-16` rather than `2040-03-05`.

**The campaign got faster, and that is the interesting part.** The orders land a few
days into the month rather than on the first, so the team starts work fresher and
the project's rate is better for its whole life. A lead time does not only delay a
decision — it changes the state the decision lands in. The fixture is regenerated and
the diff reviewed.

#### 4.4.3 What M1-08 and M1-09 did not do

- **The goodwill is not on the dashboard yet as a resource the player can read at a
  glance.** It is in the top bar, but the *effects* — the deeper applicant pool, the
  events in which the state asks why, the mandate crisis — need the event catalogue
  and the recruitment module.
- **The report lines of a landed order wait for the next monthly report.** An order
  lands between month boundaries, and the report is monthly. The lines are buffered
  on the `Sim` and are **not saved** — the chronicle is the durable record, and the
  live feed (M1-13) is what replaces the buffer.
- **The order palette does not exist.** The interface still cannot give an order;
  that is S1-37, and `Sim::issue` is the API it will call.

### 4.5 Determinism, persistence, and the interface

| ID | Task | Depends on | Done when |
|---|---|---|---|
| M1-10 | The RNG buckets become **stable identifiers** - the event-check index, not the instant - and the test is extended to prove the buckets do not depend on the dilation. | M1-06, M1-14 | A campaign at the base rate and at the top rate draw the same numbers from the same streams |
| M1-11 | The save format carries the instant, the scheduler, and the order log with its issue instants, and the `sim_version` inputs (including the integration quantum). | M1-05, M1-16 | A campaign saved with work in flight restores that work |
| M1-12 | The host loop in `ui`: the frame delta, the clock, the pause key, the nine dilation keys, and the `-`/`+` steps. | M1-01, M1-05 | The interface runs, the clock advances, and the dilation responds to its keys |
| M1-13 | The top bar: the date and time, the pause indicator, the dilation, the daily burn rate, the goodwill, and the count of what needs attention. | M1-12 | The top bar shows everything the player needs to know about the passage of time |
| M1-14 | **The dilation-independence test.** The same order script at every one of the nine dilations must produce the same state hash. | M1-05, M1-06, M1-07 | The test passes at every dilation step, and it fails if a draw is moved into a sub-step |
| M1-15 | A fork at any instant, and the branch comparison view. | M1-08, M1-11 | The player can fork a campaign and see where the two diverge |

**M1-15 is complete.** The fork replays the order log, so the branch is the *same
campaign* up to the fork point — the same state, the same chronicle, the same
recruits — and the original log is preserved. The branch advances with the clock,
because a comparison between two campaigns at different instants says nothing. The
**branches screen** (`7`) lists the instants at which the player gave an order, forks
at the selected one, and names what diverged.

**The fork point is usually in the past [D].** That is the point: the player looks
back at a decision and asks what would have happened otherwise.

**M1-10 through M1-14 are complete.** What they produced:

| From | To |
|---|---|
| The random streams keyed on an *instant* | Keyed on **stable identifiers**: the month index, the event check's index, the project's id |
| `SIM_VERSION` written down by hand | A **hash of the behaviour-defining inputs** - the phase order, the integration quantum, the RNG algorithms, the order kinds, the scheduled kinds |
| The save format at version 1 | Version 3 - the scheduler and then the orders in flight joined the file |
| The top bar as one styled line | A set of spans, so a field can be red without the whole bar shouting |
| The top bar's monthly net | The **daily** burn rate, and the count of what needs attention |
| - | `sim::version`, the version module |
| - | `crates/ui/tests/dilation.rs` - the nine dilations, through the host's own path |

**The buckets are stable identifiers, not instants [D].** The month *index* survives a
change to the length of a month; the instant does not. The event check's *index*
survives a change to the check's interval - and the interval tightens under stress, so
the instants move and the index does not. The rule is: **a draw is keyed on something
that identifies the draw, never on when it happened.**

**The version is a hash, not a constant [D].** `sim::version::sim_version()` hashes the
phase order, the integration quantum, the RNG algorithms, the order kinds, and the
scheduled kinds. A change to any of them changes the version, and a save made under a
different version is **refused** rather than silently producing a different campaign.
Writing the version down by hand is how it stops being true.

**The dilation test [D].** `crates/ui/tests/dilation.rs` runs the same campaign
through the **host's own path** - the real `App`, the real `Clock`, the real frame -
at each of the nine dilations, and asserts that the state hash and the chronicle are
identical. It is not a test of the simulation alone: it is a test of the simulation
*and the host*, which is where the property actually has to hold.

#### 4.5.1 The bug the dilation test found

**The campaign could overshoot the host's target [fixed].** The simulation's clock
lands on the integration grid, and the fraction beyond it is carried. The host used to
ask for "the grid point plus a frame", which meant the carried fraction could push
the campaign *past* the frame the host asked for - and at the top rate a frame is nearly
three hours.

The fix has two halves. The simulation now measures the request from the campaign's
**effective time** (`now + carry`) rather than from the grid point, and the host asks
for `effective_now + delta`. **The campaign never overshoots the target now**, so two
campaigns at different dilations reach the same grid point - which is what makes the
test meaningful.

**The test also had to be honest about what it compares [fixed].** A frame that
carries the campaign past the target leaves the fast campaigns covering *more*
simulated time than the slow ones, and they are then different campaigns - which is
not what the test is about. The test now clamps its requests to the target, so every
dilation covers the same span, and the frames still differ by four orders of magnitude
in between. There is a second test that checks exactly that: **the nine dilations
really do use very different frame sizes**, so the first test is not vacuous.

### 4.6 What M1 changes in the code

| From | To |
|---|---|
| `domain::time::{Month, Date, Calendar}` | `domain::time::{Instant, Duration, Date}` — a projection, not a stored value |
| `domain::directive::Directive` | `domain::order::{Order, OrderKind}` |
| `sim::directive` | `sim::orders` |
| `sim::tick::MonthContext` | `sim::tick::InstantContext` |
| `Sim::step(directives)` | `Sim::advance_to(instant)` and `Sim::issue(order)` |
| `domain::report::MonthReport` | A live feed and a "needs attention" list, in `sim::feed` |
| Monthly deltas in each phase | **Done.** Rates in `sim::rates`, integrated over the elapsed time |
| `domain::finance::Treasury::monthly_income` | **Done.** A continuous burn, with the allocation still monthly |
| The `monthly` cadence in `sim::phases::calendar` | Periodic triggers on the scheduler |
| - | **Done.** `sim::scheduler`, the queue and the total order |
| - | **Done.** `domain::scale::Remainders`, the carried fractional remainders |

### 4.7 M1 exit

- The nine dilations work, and the pause works.
- A campaign played at the base rate and at the top rate is **the same campaign**, and the test proves it.
- Every order has a visible lead time, a visible cost, and a visible goodwill charge.
- A save restores a campaign with work in flight.
- The interface shows a live clock and a live agency.

---
## 5. Slice 2 - *The Programme*

*The commitment mechanic, and the heart of the game.*

**Revised for the new time model.** A commitment is now an **instant** rather than a
month, and a milestone is a point on a continuous timeline rather than a slot in a
calendar. The rest of the slice is unchanged.

**Status: the commitment mechanic is done.** S2-01 to S2-07, S2-13, S2-15 and
S2-16 are built: the commitment carries a target and a committed instant and the
gap is the slack; the deadline consequence table is one outcome per audience; the
programme orders have handlers and are in the palette; the programme's objectives
advance at the rate of its staff; the scenario opens with the government's orbital
deadline; a commitment that falls due is resolved against the work; a deadline can
be pushed back for political capital; and the programme screen shows the slack and
the projection.

**The construction half of the slice is done too.** A facility nobody is looking
after **degrades at four percent a month** and produces less; a fully staffed one
holds its condition, because the staff are the maintenance and the upkeep is the
money. The **industrial capacity** is a real constraint: every member of staff
working at once needs a place to work, and an agency with more people working
than it has room for works with worse tools — and the tools factor says so.

**What is left in this slice is the second playtest** (S2-17), which needs a human.

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S2-01 | `Programme`, `Objective`, `Commitment`, `Milestone`, `Status`. A commitment holds a **target instant** and a **committed instant**, and the gap is the slack. | M1 | A commitment can be set for any instant | Done |
| S2-02 | The deadline consequence table: one outcome per `DeadlineAudience`. | M1 | Missing a political deadline costs approval; missing an internal one does not | Done |
| S2-03 | The `SetObjective`, `SetBudget`, `CreateProgramme` orders, with their lead times. | M1-10 | All three are reachable from the order palette | Done |
| S2-04 | Programme progress, from the team's effective performance, as a **rate**. | M1-07 | A programme's progress advances continuously and the interface projects the finish | Done |
| S2-05 | **The government's orbital deadline** as a hard committed instant, in the scenario. | S2-01 | The scenario cannot be won without meeting it | Done |
| S2-06 | Deadline handling: met, missed, renegotiated, absorbed. | S2-02 | Each path produces a distinct chronicle entry and consequence | Done |
| S2-07 | The `PostponeDeadline` order — renegotiating with the deadline holder, which costs goodwill. | S2-06 | Postponing a political deadline costs goodwill and political capital | Done |
| S2-08 | `ConstructionPlan` and the **specification** calculation from method and scale. | M1 | Two buildings with different plans have different specifications | Done with Slice 1 |
| S2-09 | Construction as a **rate**, completing at an instant. | M1-07 | A build finishes on a date, and the interface counts down to it | Done with Slice 1 |
| S2-10 | Facility degradation, maintenance, and upkeep as a daily rate. | M1-07 | A neglected facility degrades and its output drops | Done |
| S2-11 | **Industrial capacity** as a real constraint on concurrent work. | S2-09 | Exceeding capacity delays work visibly | Done |
| S2-12 | The budget-crisis **warning** state and the first two ladder steps. | M1 | A projected shortfall is visible before it bites | Done |
| S2-13 | The programme screen: objectives, both instants, slack, budget, progress, risk, and the projection. | S2-01 | The screen shows the target and the commitment side by side | Done |
| S2-14 | The facilities screen: construction progress, condition, staffing, output. | S2-09 | The screen shows what each building is for and how well it is doing | Done |
| S2-15 | The commitment widget: the slack meter, showing the gap and the burn-down. | S2-13 | A player can see how much slack is left at a glance | Done |
| S2-16 | Extend the golden fixtures to cover commitments and construction. | S2-06 | New behaviour is covered by a diffed golden test | Done |
| S2-17 | **The second playtest**, focused on the commitment mechanic. | S2-16 | A playtester commits to an instant, watches the slack shrink, and can name what they lost | **Not yet** |

---
## 6. Slice 3 — *First Orbit* (months 36-96)

*The payoff, and the danger.*

**Status: the hardware is there; the flying is not.** S3-01, S3-02 and S3-03 are
done: a vehicle is a set of sliders, the numbers follow from the sliders and are
never stored as an authority, and two things bound a slider — the archetype, which
says what kind of machine it is, and the capabilities, which say what the agency
has developed. **A slider the agency has not researched is worth zero**, whatever
the archetype would otherwise allow.

**A mission is a chain of scheduled instants, not a monthly step.** The phases
have real durations — three weeks of preparation, ninety minutes of launch, four
days of transit, a week of operations, four days of return, a fortnight of
archival — and each phase schedules the next. The outcome is drawn at a scheduled
instant, on the mission's own stream, and **a partial failure is the common
branch**: the capsule comes back, the data is partial, and the agency learns
something and loses something. The findings go to the research that asked for
them, because a mission's destination names a capability and a capability has a
discipline.

**A vehicle is built, and the build takes real time.** The time is its cost: a
cheap sounding rocket is built in a month and an orbital launcher takes a year.
The build is a rate, and the industrial capacity slows it the same way it
slows the work - an over-stretched agency builds everything more slowly.

**The world reacts.** An event is data - a weight, some conditions, some
effects, and the words it uses - and the weight is the base weight multiplied
by the modifiers that apply, so **bad conditions raise the odds of bad
events**. A quiet stretch is a legitimate outcome: the weights are in basis
points and they add up to the chance that a check produces anything at all, so
most weeks produce nothing.

**Testing is what makes the difference.** A completed test campaign takes
points of failure risk out of the capability it tested, and the risk stays
taken out after the campaign that found it is gone. The vehicles screen says
what the testing would cost and what it would buy, because the choice the
player is making is *test this* or *build a better vehicle*.
### 6.1 Hardware and missions

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S3-01 | `Archetype`, `SliderValues`, `Vehicle`, and the **derived statistics calculation**. | S1-05 | A vehicle's cost, reliability, and capacity follow from its sliders | Done |
| S3-02 | The capability gating: a slider is bounded by the available capability. | S3-01, S1-21 | A radioisotope power slider is unavailable before the capability unlocks | Done |
| S3-03 | The three MVP archetypes: sounding rocket, orbital launcher, orbital capsule. | S3-01 | Each has a distinct derived profile | Done |
| S3-04 | `Mission` and the phase machine, with the flight phases in their real durations. | M1 | A mission advances through its phases, and the ascent takes 60-90 minutes | Done |
| S3-05 | Success resolution, with partial failure as the common branch, resolved at a scheduled instant. | S3-04 | A failed mission usually produces partial results, not a total loss | Done |
| S3-06 | Results feeding research: a successful mission's data advances a discipline. | S3-05, S1-21 | A mission's outcome measurably advances knowledge | Done |
| S3-07 | The `DesignVehicle`, `SetSubsystem`, `AuthorizeLaunch` handlers. | S1-06, S3-04 | A player can design, build, and launch | Done |
| S3-08 | Vehicle construction, using industrial capacity, as a rate. | S2-11, S3-07 | A vehicle takes real time and capacity to build | Done |
| S3-09 | The vehicles and missions screen: sliders, derived stats, risk, authorisation, and the time to orbit. | M1-18, S3-01 | The risk figure and the flight time are visible before authorisation | Done |

### 6.2 Capability against reliability

| ID | Task | Depends on | Done when |
|---|---|---|---|
| S3-10 | `TestCampaign` as a distinct activity from a research project. | S1-11 | A test campaign consumes staff and money like a project, but produces no new capability | Done |
| S3-11 | Test campaigns reducing the risk of existing capability. | S3-10, S3-01 | A tested capability measurably lowers a launch's failure probability | Done |
| S3-12 | The capability-versus-reliability decision surfaced in the UI: what testing would cost and what it would buy. | S3-11, S3-09 | The player can see the trade before choosing | Done |

### 6.3 The world reacts

| ID | Task | Depends on | Done when |
|---|---|---|---|
| S3-13 | `EventDef`, `Condition`, `Effect`, `WeightMod` and the event registry. | S1-05 | An event can be defined purely as data | Done |
| S3-14 | The context weighting: weight is base × modifiers over live state. | S3-13 | A fatigued team measurably raises the weight of its bad events  Done |
| S3-15 | The event phase: draw, apply effects atomically, write chronicle entries. | S3-14, S1-13 | Events apply as deltas and never break an invariant | Done |
| S3-16 | `FollowUp`: immediate, delayed, and conditional. | S3-15 | A delayed follow-up fires in a later month and appears in the forecast | Done |
| S3-17 | The **event catalogue** — ~35 events across the eight categories. | S3-15 | Done |
| S3-18 | The event phase runs **last**, after every other phase. | S1-04, S3-15 | The order matches `SIMULATION.md` §2.2 and is covered by a golden test  Done |
| S3-19 | Politics: approval, political capital, promises, milestone demands. | S1-14 | A kept promise raises capital; a broken one costs it  Done |
| S3-20 | Public opinion and prestige, and their link to the chronicle. | S3-19 | A disaster moves public opinion in the same month it happens  Done |
| S3-21 | The forecast: the next twelve months of scheduled obligations and their risk figures. | S3-16 | Done |
| S3-22 | The full chronicle screen: filter by year, category, and subject. | S1-32, S3-15 | Done |

### 6.4 Proving it

| ID | Task | Depends on | Done when |
|---|---|---|---|
| S3-23 | The **"First Orbit" scenario**: a tuned seed and start state that reliably produces a five-year arc with real decisions in it. | S3-21, S3-17 | Starting from the scenario, a competent player reaches orbit |
| S3-24 | The balance harness: run N seeds under a reference policy, report the outcome distribution. | S3-23 | The harness prints outcome distributions and runs in minutes |
| S3-25 | Extend the property tests: invariants that span systems (no orphaned references, every committed date reaches a resolution). | S1-35 | The cross-system invariants are covered |
| S3-26 | The fuzz test: random directive sequences never panic and always resolve. | S1-06 | A nightly run of thousands of random sequences finds nothing |
| S3-27 | The performance test: a full campaign resolves in under a second. | S3-23 | The test asserts the budget in `ARCHITECTURE.md` §10 |
| S3-28 | **The third playtest** — the whole five-year arc. | S3-24 | A playtester reaches orbit, and has lost a programme, a launch, and an engineer along the way |

**Slice 3 exit test:** a playtester gets to orbit, and on the way loses a programme, a
launch, and an engineer — and can read all three in the chronicle.

---

## 7. Slice 4 — *Proof*

*Make it a product, not a build.*

| ID | Task | Depends on | Done when |
|---|---|---|---|
| S4-01 | **Replay**: the order log is persisted and reproduces a campaign exactly. | M1-16 | A replayed campaign produces an identical state hash |
| S4-02 | **Branching**: fork at any month, re-simulate with a different decision, compare. | S4-01 | The player can fork at month 30 and see a different year 3 |
| S4-03 | The branch comparison view: two timelines side by side. | S4-02 | The difference between two branches is legible |
| S4-04 | `sim_version`: hash the resolution order, event definitions, directive handlers, the RNG algorithm, and the stream hash. | S1-01 | A change to any of them changes the version |
| S4-05 | Save files carry `sim_version`; replays against a mismatched version are refused with a clear message. | S4-04 | The refusal message is actionable |
| S4-06 | The **onboarding**: the `PLAYER_INTRO.md` framing as an in-game opening. | S3-23 | The game opens with the framing and a clear first goal |
| S4-07 | The first-month tutorial, teaching the binding factor. | S1-33 | Sits alongside S1-33 rather than replacing it |
| S4-08 | Balance pass 1: read the harness output, tune the constants in `sim::balance`. | S3-24 | At least three strategies survive with comparable outcomes |
| S4-09 | The **strategy check**: the harness must run three reference policies — cautious, fast, and science-first — and all three must be viable. | S4-08 | The harness reports three viable strategies. This is the MVP's most important test |
| S4-10 | Error-message pass: every rejection and error reads as a sentence to a player, not a developer. | S1-07 | A full read-through of every message in the game |
| S4-11 | Save/load robustness: corrupted, truncated, and foreign-version files fail gracefully. | S1-26 | No crash, and the message says what to do |
| S4-12 | The **final playtest** with someone who has never seen it. | S4-09, S4-10 | They reach orbit in one sitting and ask to try a different path |
| S4-13 | Documentation pass: bring `docs/` in line with the shipped behaviour. | S4-12 | No doc describes behaviour the code does not have |

**Slice 4 exit test:** a new player, given nothing but the game, reaches orbit in one
sitting and wants to try a different path.

---

## 8. Cross-cutting work

These run alongside the slices rather than inside them.

| ID | Task | When | Done when |
|---|---|---|---|
| X-01 | The determinism test runs in CI on every change. | M0 | A CI job exists and fails the build on a determinism break |
| X-02 | The dependency-direction check runs in CI. | M0 | A forbidden dependency fails the build |
| X-03 | `cargo clippy` and `cargo fmt --check` run in CI. | M0 | A formatting or lint change fails the build |
| X-04 | Golden fixtures are extended with every behaviour change. | Throughout | No behaviour change lands without a diffed golden test |
| X-05 | `BALANCE.md` is updated whenever a constant changes. | Throughout | The doc matches the code |
| X-06 | A nightly job runs the fuzz test and the balance harness. | S1-26 | Overnight failures are reported |
| X-07 | The decision log in `docs/` gains an entry for every settled design decision. | Throughout | A reader can see why the design is what it is |

---

## 9. The critical path

**The path changed with the time model.** The old critical path ran through the monthly
tick; the new one runs through the redesign and then through the live interface.

```
M1-01  →  M1-04  →  M1-05  →  M1-07  →  M1-06  →  M1-14
   →  M1-08  →  M1-09  →  M1-10  →  M1-13  →  S1-37  →  S2-05  →  S3-05  →  S4-02
```

That is: **the clock → the scheduler → the advance loop → the rates → the draw anchors →
the dilation test → the order model → goodwill → the stable buckets → the host loop → the
order palette → the government's deadline → the first launch → branching.**

M1-01, M1-04, M1-05, M1-06, and M1-07 are done, so the path now starts at **M1-14,
the dilation-independence test**, and the order model (M1-08) is the long pole behind
it.

### 9.1 The three long poles

Each is early on purpose, because each one invalidates work that comes after it.

- **M1-07, the rates and the integration.** Everything continuous depends on it, and
  the fixed-point storage it requires (`DOMAIN_MODEL.md` §2.3) touches every quantity
  in the domain. Getting the scale wrong means re-tuning every rate in the game, and
  the error is invisible until something fails to move. **And the remainder must be
  carried** (§4.1.1) — without it the rate is quietly wrong over a long campaign.
- **M1-06, the draw anchors.** Moving every random draw onto a scheduled instant is
  the change that makes dilation independence true. It is easy to get almost right —
  one draw left inside a sub-step, and the campaigns at different dilations diverge in
  the third month. The test in M1-14 is what proves it.
- **S4-02, branching.** The most expensive feature and the one that most needs
  proving, because in a real-time game the player's dominant emotion is regret about a
  decision they cannot take back. It is on the critical path because the order-log
  architecture has to be right from M1-16.

### 9.2 The riskiest single task

**M1-14, the dilation-independence test** — not because it is hard, but because
**it is the test that can fail for a reason nobody can see.** The numerical integration
approximates the same integral at different granularities, and if the error is big
enough to flip a decision, two campaigns at different dilations will diverge. The test
is the only thing standing between that and a game whose outcome depends on the frame
rate.

### 9.3 The two things that are not on the critical path but must not be skipped

- **M1-12, the second validation.** An order that is issued and then finds its target
  has changed is a *new* failure mode, and it is the one the player will hit most
  often. It needs a design decision, not just code.
- **M1-10, the lead times.** A game where every order lands instantly is a
  spreadsheet. The lead times are the whole difference between a management game and a
  form, and they are a handful of numbers.

---
## 10. Definition of done

A task is done when all of the following hold. This applies to every task in this plan.

- [ ] The **done when** condition is met and verified.
- [ ] `cargo build`, `cargo clippy`, `cargo fmt --check`, and `cargo test` are clean.
- [ ] The determinism test passes.
- [ ] New behaviour is covered by a golden test, or a test is added explaining why not.
- [ ] New invariants are added to the catalogue in `DOMAIN_MODEL.md` §18 with a test each.
- [ ] New numbers are in `sim::balance` and documented in `BALANCE.md`.
- [ ] Any new player-facing message reads as a sentence to a player.
- [ ] Any new design decision is recorded in the decision log.
- [ ] The docs that describe the change are updated in the same change.

---

## 11. Risks to this plan

| Risk | Effect | Mitigation |
|---|---|---|
| **The performance model is illegible** | The People pillar collapses, and the fix is a model change, not a UI change | S1-16 and S1-30 are developed together; the tutorial (S1-33) is in Slice 1, not Slice 4 |
| **The event system feels arbitrary** | The game is deleted by its first playtester | Context weighting is part of S3-14, not a refinement; every chronicle entry states its cause |
| **The month loop feels like waiting** | The core mechanic is disliked before it is understood | The sim is fast enough to fast-forward; make skipping a year a first-class action from S1-27 |
| **Scope creep back toward the full design** | The MVP never ships | `MVP.md` §3 is binding; adding to it requires removing something else, in writing |
| **The directive-log architecture is wrong** | Replay, branching, and the balance harness all fall | S1-26 saves the log from the start; S4-01 and S4-02 are on the critical path |
| **Balance is discovered to be impossible** | The game has no difficulty curve | The balance harness (S3-24) and the strategy check (S4-09) are automated, not playtest-based |
| **Ten phases is too many, or too few** | The tick is wrong | The phase order is hashed into `sim_version`, so changing it is visible and deliberate; the golden tests make the change safe |

---

## 12. Related documents

- `MVP.md` — the scope this plan delivers.
- `ARCHITECTURE.md` — the crate and module layout the tasks refer to.
- `SIMULATION.md` — the engine contract the tasks implement.
- `DOMAIN_MODEL.md` — the entities the tasks build.
- `PERSISTENCE.md` — save format, `sim_version`, replay, branching.
- `TESTING.md` — the test strategy the tasks rely on.
- `UI.md` — the screens and the information design.
- `BALANCE.md` — the numbers.
- `README.md` — the documentation index.
