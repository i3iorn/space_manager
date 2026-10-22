# ROADMAP.md

> **The build plan.** Task-level, with dependencies and exit criteria.
>
> Implements `MVP.md` on the architecture in `ARCHITECTURE.md`, against the
> contracts in `SIMULATION.md` and `DOMAIN_MODEL.md`.
>
> **Status:** Plan, revision 3. M0, M1, and Slices 1 to 4 are complete, except
> for the three playtests and the fourth playtest, which need a human.

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

**The playtest is running, and it has already found four things worth fixing.** They
are recorded here because a playtest that finds nothing is a playtest that did not
happen, and because each of these is a rule the interface should have followed from
the start.

| What the playtester said | What it actually was | The fix |
|---|---|---|
| "I see `programme 0` when I assign somebody" | The interface showed **identifiers** where it should have shown names — in the staff screen, in the order palette, in the report, and in every refusal message | `sim::derived::assignment_description` names the programme, the project, the facility, and the test campaign; the domain's own description no longer carries a number at all, so an identifier cannot leak even from the layer that has no names |
| "It is not the only place" | It was not the only place: the refusals, the order notice, the chronicle's subject filter, and the vehicle's build state all had the same fault | Every message that names a thing goes through one function, and the rule is now in `UI.md` §1 and in the manual |
| "The game just shuts down without warning" | A **panic printed its message to the alternate screen** — which is thrown away when the screen is left, so the game vanished with no explanation and the shell came back looking normal | A panic hook restores the terminal *first* and writes the message *after* it, and the game says plainly that it stopped unexpectedly |
| "I cannot always get out" | **Every modal took the keys** while it was open, and `Ctrl+C` was one of them, so a player who pressed it in the report or the palette found the game still running | `Ctrl+C` quits from any screen and any modal, and is handled before the modals |
| "There is no way out of this campaign" | **The game quit on a single `q`, and the campaign is not saved** — a stray key could end an evening's play | There is now a confirmation: a modal over the screen, with the safe answer selected first, that says the campaign is not saved, and the world waits while the player decides | **Every modal took the keys** while it was open, and `Ctrl+C` was one of them, so a player who pressed it in the report or the palette found the game still running | `Ctrl+C` quits from any screen and any modal, and is handled before the modals |

**The manual was rewritten in the same pass**, because the first playtest showed
that it read as a glossary rather than as an explanation. Every topic now has a
worked example with real numbers, says how the thing ties into the systems
either side of it, and says what the player can do about it. Nine topics were
added - the eight factors, the eight weighting contexts, the first hour, the
capability against reliability, the political capital, the mandate, the
milestones, the shift as an order, and how the screens fit together - and
eighteen new tests now guard the figures the manual quotes against the
balance constants they came from.

**A bug came out of writing the example.** The test campaign's target was a
hundred points and its progress was scaled, so the first month's points passed
the target on their own: every test campaign finished in a month, and the
reliability half of the central tension was nearly free. The target is scaled
now, and a campaign of four takes five months, which is what the manual says.

The campaign itself was not the cause of the crash: a thirty-year campaign with an
order every week, and forty seeds run for fifty years, are both clean, and so is the
interface driven through forty thousand frames of clock, keys, and drawing. What was
wrong was that **the game could not say why it stopped**, and that is the part worth
having fixed.

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

**This table is the state of the world at the end of Slice 1.** Everything in it
has since arrived — the events in Slice 3, the vehicles and the missions in
Slice 3, the live feed and the branch comparison view in Slice 4 — and the
**playtest still has not happened**, because it needs a human. The table is kept
because it says what each gap cost, which is the argument for closing them.

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
| S1-43 | **Every list is filterable and sortable.** `/` filters, `s` sorts, and the filter is a mode so it can contain a `q` without the game quitting. | S1-27 | A player can find one person in three hundred by name, and the filter survives a trip to another screen | Done, for the staff list. The other lists are S1-50 |
| S1-44 | **The staff list sorts by name, age, role, fatigue, attrition risk, and shift.** | S1-43 | The list can be ordered by who is about to leave | Done |
| S1-45 | **The attrition risk is a number the player can see**, on the list and in the detail panel, and it is the same number the simulation rolls against. | M1-10 | The number on the screen is the number the engine uses | Done |
| S1-46 | **The manual.** One store of topics — every stat, every concept, every order, every screen — reached three ways: `F2` for all of it, `I` for the screen you are on, and the info line for a one-line answer. | S1-27 | Every concept in the game has a topic, and no screen shows a number the manual does not explain | Done |
| S1-47 | **The tutorial hints stay until the player acts.** A hint marked as seen when the screen was first drawn is on the screen for one frame. | S1-33 | A hint is still on the screen after ten seconds of looking at it | Done |
| S1-48 | **The shift policy: set everybody's shift at once.** The per-person slider is the surgical tool; this is the tool for three hundred people. | S1-42, S1-49 | A player can put the whole agency on light duties in one action | Done |
| S1-49 | **The policy order.** `SetPolicy` sets the agency's standing instruction, with a week of lead time and a goodwill charge. | M1-08 | An order with a handler, a lead time, and a cost | Done |
| S1-50 | **Every other list is filterable and sortable.** The machinery is built and only the staff list is wired to it. | S1-43 | The research, programme, chronicle, and branches lists all filter | **Not yet** |
| S1-51 | **The filter can express a condition** — "fatigue over 80", "unassigned" — and not only a name. | S1-43 | A player can find everybody who is in trouble without reading three hundred names | **Not yet** |
| S1-52 | **The order palette can set a shift**, so the order is offered where every other order is offered. | S1-42 | The palette offers every order kind that has a handler | Done |

**Why the shift is an order and not a direct control.** The architecture says the
player never mutates state — they issue orders — and the shift is the most-used
control in the game, so it must not be the one thing that breaks the rule. **A
one-hour lead time is the price of that**, and it is a price worth paying: the
player sees the change in flight, the order log records it, and a campaign replayed
from the log reproduces it.

**Why the shift is not the whole answer to three hundred people.** It is the
*surgical* tool, and it scales because it is one keystroke on a screen the player is
already looking at. It is not the *policy* tool, and the policy tool is S1-48.

**How the policy is built (S1-48, S1-49, S1-52).** **A policy is a standing
instruction, not a one-off.** The agency's shift policy says two things: **who it
covers** and **what they work**. With no threshold, everybody works the shift. With
one — "anybody over 60% fatigue" — only the people who are tireder than it are eased
off, and the people who are fresher keep their own shift. That threshold makes the
policy a **thermostat**: an eased-off person recovers, comes back up to the threshold,
and is eased off again, so the agency never has anybody work themselves to pieces.
**A person the manager has moved individually is moved off the policy** and the staff
screen says so.

**The per-person shift is now an override of the policy**, and the state says which
of the two is in force: a person's `shift` is `None` when the policy decides and a
value when the manager has decided better. That is why the policy is a *standing*
instruction and not a bulk write — a bulk write would have to be repeated, and the
"tired ones" version would only ever catch the tired ones *at that moment*.

**The policy order has a week of bureaucracy and costs five goodwill**, because a
standing instruction about how the whole agency works is a public act, and the
design's order table says so. The per-person shift remains free and immediate: a
player must be able to make a hundred of them.

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
| Below −40 | Events in which the state asks why | Not yet — the event catalogue is here, but no event in it watches the goodwill |
| Below −80 | The state reconsiders the mandate | **A warning on the dashboard**; the crisis itself is not built |

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

**This is the record of that moment, and it is kept because it says what the
order model was missing.** The order palette is here now (S1-37), the event
catalogue is here now (Slice 3), and the live feed is here now (S1-37's
successor). **The report lines are still not saved** — they are the things that
have happened since the last report, the chronicle is the durable record, and a
loaded campaign starts with an empty buffer. That is the one item here that is
still true.

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
## 6. Slice 3 - *First Orbit* (months 36-96)

*The payoff, and the danger.*

**Status: complete, except for the playtest.** S3-01 to S3-22 were already
built; the Slice 3 work added S3-23 to S3-27, and **S3-28 needs a human**.

**A vehicle is a set of sliders, the numbers follow from the sliders, and
they are never stored as an authority.** Two things bound a slider: the
archetype, which says what kind of machine it is, and the capabilities, which
say what the agency has developed. **A slider the agency has not researched is
pinned at zero**, whatever the archetype would otherwise allow, and the
capability behind each of the gated sliders is now named in the catalogue:
orbital propulsion for the propulsion slider, basic re-entry for the heat
shield, closed-loop life support for the life support, and orbital launch for
the avionics.

**A mission is a chain of scheduled instants, not a monthly step.** The phases
have real durations - three weeks of preparation, ninety minutes of launch, four
days of transit, a week of operations, four days of return, a fortnight of
archival - and each phase schedules the next. The outcome is drawn at a scheduled
instant, on the mission's own stream, and **a partial failure is the common
branch**: the capsule comes back, the data is partial, and the agency learns
something and loses something. **Two failures in three are partial ones**, and
the third writes the vehicle off.

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
fifteen points of failure risk out of the capability it tested, the campaigns
stack, and the risk stays taken out after the campaign that found it is gone.
The vehicles screen says what the testing would cost and what it would buy,
because the choice the player is making is *test this* or *build a better
vehicle*.

**And the launch is the point.** A launch is authorised against a risk figure
and a delta-v, and the engine refuses the impossible ones: a vehicle that is
not finished, a destination the agency has not developed the technology for,
and a rocket without the delta-v to get there. **The orbital programme's
objective is finished by reaching orbit**, and the objective's progress is the
programme's staff working while the launch is the thing that completes it.

### 6.1 Hardware and missions

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S3-01 | `Archetype`, `SliderValues`, `Vehicle`, and the **derived statistics calculation**. | S1-05 | A vehicle's cost, reliability, and capacity follow from its sliders | Done |
| S3-02 | The capability gating: a slider is bounded by the available capability. | S3-01, S1-21 | A radioisotope power slider is unavailable before the capability unlocks | Done, and the gate was extended to four sliders |
| S3-03 | The three MVP archetypes: sounding rocket, orbital launcher, orbital capsule. | S3-01 | Each has a distinct derived profile | Done |
| S3-04 | `Mission` and the phase machine, with the flight phases in their real durations. | M1 | A mission advances through its phases, and the ascent takes 60-90 minutes | Done |
| S3-05 | Success resolution, with partial failure as the common branch, resolved at a scheduled instant. | S3-04 | A failed mission usually produces partial results, not a total loss | Done, and the partial band is now two thirds of the failures |
| S3-06 | Results feeding research: a successful mission's data advances a discipline. | S3-05, S1-21 | A mission's outcome measurably advances knowledge | Done |
| S3-07 | The `DesignVehicle`, `SetSubsystem`, `AuthorizeLaunch` handlers. | S1-06, S3-04 | A player can design, build, and launch | Done, and the palette now offers all three |
| S3-08 | Vehicle construction, using industrial capacity, as a rate. | S2-11, S3-07 | A vehicle takes real time and capacity to build | Done |
| S3-09 | The vehicles and missions screen: sliders, derived stats, risk, authorisation, and the time to orbit. | M1-18, S3-01 | The risk figure and the flight time are visible before authorisation | Done |

### 6.2 Capability against reliability

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S3-10 | `TestCampaign` as a distinct activity from a research project. | S1-11 | A test campaign consumes staff and money like a project, but produces no new capability | Done |
| S3-11 | Test campaigns reducing the risk of existing capability. | S3-10, S3-01 | A tested capability measurably lowers a launch's failure probability | Done |
| S3-12 | The capability-versus-reliability decision surfaced in the UI: what testing would cost and what it would buy. | S3-11, S3-09 | The player can see the trade before choosing | Done |

### 6.3 The world reacts

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S3-13 | `EventDef`, `Condition`, `Effect`, `WeightMod` and the event registry. | S1-05 | An event can be defined purely as data | Done |
| S3-14 | The context weighting: weight is base x modifiers over live state. | S3-13 | A fatigued team measurably raises the weight of its bad events | Done |
| S3-15 | The event phase: draw, apply effects atomically, write chronicle entries. | S3-14, S1-13 | Events apply as deltas and never break an invariant | Done |
| S3-16 | `FollowUp`: immediate, delayed, and conditional. | S3-15 | A delayed follow-up fires in a later month and appears in the forecast | Done |
| S3-17 | The **event catalogue** - ~35 events across the eight categories. | S3-15 | Done | Done |
| S3-18 | The event phase runs **last**, after every other phase. | S1-04, S3-15 | The order matches `SIMULATION.md` §2.2 and is covered by a golden test | Done |
| S3-19 | Politics: approval, political capital, promises, milestone demands. | S1-14 | A kept promise raises capital; a broken one costs it | Done |
| S3-20 | Public opinion and prestige, and their link to the chronicle. | S3-19 | A disaster moves public opinion in the same month it happens | Done |
| S3-21 | The forecast: the next twelve months of scheduled obligations and their risk figures. | S3-16 | Done | Done |
| S3-22 | The full chronicle screen: filter by year, category, and subject. | S1-32, S3-15 | Done | Done |

### 6.4 Proving it

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S3-23 | The **"First Orbit" scenario**: a tuned seed and start state that reliably produces a five-year arc with real decisions in it. | S3-21, S3-17 | Starting from the scenario, a competent player reaches orbit | **Done** |
| S3-24 | The balance harness: run N seeds under a reference policy, report the outcome distribution. | S3-23 | The harness prints outcome distributions and runs in minutes | **Done** |
| S3-25 | Extend the property tests: invariants that span systems (no orphaned references, every committed date reaches a resolution). | S1-35 | The cross-system invariants are covered | **Done** |
| S3-26 | The fuzz test: random directive sequences never panic and always resolve. | S1-06 | A nightly run of thousands of random sequences finds nothing | **Done** |
| S3-27 | The performance test: a full campaign resolves in under a second. | S3-23 | The test asserts the budget in `ARCHITECTURE.md` §10 | **Done** |
| S3-28 | **The third playtest** - the whole five-year arc. | S3-24 | A playtester reaches orbit, and has lost a programme, a launch, and an engineer along the way | **Not yet** |

**Slice 3 exit test:** a playtester gets to orbit, and on the way loses a
programme, a launch, and an engineer - and can read all three in the chronicle.
**Everything except the playtest is done**, and the playtest needs a human.

### 6.5 What the Slice 3 work found

**The Slice 3 work was supposed to be a scenario, a harness, and three test
suites. It was six bugs and a balance rewrite, and every one of them was
found by the thing the slice was about.**

| What | Why it mattered |
|---|---|
| **A programme's staffing list was never maintained.** Assigning somebody to a programme set their assignment and nothing else, so the programme's rate - which reads the staffing list - never moved. | **The commitment mechanic did not work at all.** The whole of Slice 2 was a promise the simulation never counted |
| **A person who resigned stayed on every team they were on.** | An orphaned reference, and a campaign counting staff it did not have |
| **The knowledge of a research project went to the wrong capability.** The first capability that shared the discipline was credited, and that was "Atmospheric sounding" - a capability with a threshold of zero. | **Orbital propulsion was unreachable in five years.** The flagship capability of every field could only be reached through a twenty percent spill, ten projects at a time |
| **The appropriation multiplied the *scaled* approval by the political scale.** | The agency was funded at 1,900 a head and paid twenty-six times that, and the balance was measured against a budget nobody had voted for |
| **The programme's objective rate was multiplied by its own target.** | Every objective completed in about a month, whatever it was worth. The target cancelled itself out of the rate |
| **The chronicle's identifiers came from the length of the chronicle**, and a batch of entries written in one phase all got the same number. | The chronicle's own filter could not tell two entries apart |

**And the fuzzer found four more**, which is the argument for having one:
a postponement could invert a commitment, two recruits could have the same
name, the chronicle's dangling-subject rule caught people who had *left*,
and a project's lead could be moved off the project while the project still
named them.

**The balance was rewritten around what the harness said.** The research cost
was a fifth of what it was, because at the old figure no agency could afford
a project at all. The vehicle costs were reduced and the build time made to
follow the price. The mission's base chance, the partial-failure band, and
the risk the testing takes out were all reworked, and the numbers are in
`BALANCE.md` §3, §6, and §7.


## 7. Slice 4 - *Proof*

*Make it a product, not a build.*

**Status: complete, except for the final playtest.** Every task except S4-12
is done, and S4-12 needs a human.

| ID | Task | Depends on | Done when | Status |
|---|---|---|---|---|
| S4-01 | **Replay**: the order log is persisted and reproduces a campaign exactly. | M1-16 | A replayed campaign produces an identical state hash | **Done** |
| S4-02 | **Branching**: fork at any month, re-simulate with a different decision, compare. | S4-01 | The player can fork at month 30 and see a different year 3 | **Done** |
| S4-03 | The branch comparison view: two timelines side by side. | S4-02 | The difference between two branches is legible | **Done** |
| S4-04 | `sim_version`: hash the resolution order, event definitions, directive handlers, the RNG algorithm, and the stream hash. | S1-01 | A change to any of them changes the version | Done |
| S4-05 | Save files carry `sim_version`; replays against a mismatched version are refused with a clear message. | S4-04 | The refusal message is actionable | Done, and the message now names both versions |
| S4-06 | The **onboarding**: the `PLAYER_INTRO.md` framing as an in-game opening. | S3-23 | The game opens with the framing and a clear first goal | **Done** |
| S4-07 | The first-month tutorial, teaching the binding factor. | S1-33 | Sits alongside S1-33 rather than replacing it | Done |
| S4-08 | Balance pass 1: read the harness output, tune the constants in `sim::balance`. | S3-24 | At least three strategies survive with comparable outcomes | **Done** |
| S4-09 | The **strategy check**: the harness must run three reference policies - cautious, fast, and science-first - and all three must be viable. | S4-08 | The harness reports three viable strategies. This is the MVP's most important test | **Done** |
| S4-10 | Error-message pass: every rejection and error reads as a sentence to a player, not a developer. | S1-07 | A full read-through of every message in the game | **Done** |
| S4-11 | Save/load robustness: corrupted, truncated, and foreign-version files fail gracefully. | S1-26 | No crash, and the message says what to do | **Done** |
| S4-12 | The **final playtest** with someone who has never seen it. | S4-09, S4-10 | They reach orbit in one sitting and ask to try a different path | **Not yet** |
| S4-13 | Documentation pass: bring `docs/` in line with the shipped behaviour. | S4-12 | No doc describes behaviour the code does not have | **Done** |

**Slice 4 exit test:** a new player, given nothing but the game, reaches orbit in one
sitting and wants to try a different path - which is exactly what branching is for.
**Everything except the playtest is done**, and the playtest needs a human.

### 7.1 What the Slice 4 work produced

**The replay is an engine feature and not a test.** `Sim::replay` rebuilds a
campaign from the scenario and the order log alone, and the result is the same
campaign down to the state hash. **It is the same mechanism the fork uses**,
which is why the fork is exact: the branch is the same campaign up to the fork
point, and every difference after it is a difference the player caused.

**The branch comparison is two columns and a list of decisions.** The screen
shows the timeline of the campaign - the decisions, with their dates and their
names - and then the two campaigns side by side: the money, the people, the
knowledge, the approval, and the chronicle entries that only one of them has.
**A difference the player cannot see is a difference the player cannot use.**

**The game opens with a briefing.** The `PLAYER_INTRO.md` framing, the
campaign's name, and the first goal - *reach orbit before the state expects
you to* - and the world waits while the player reads it. It is a modal over
the dashboard, and any key dismisses it.

**The save and load are on the command line.** `--save <FILE>` writes the
campaign when the game ends, and `--load <FILE>` starts from one. A damaged
file is a message and not a crash, and the message says which of the three
things is wrong: it is not a campaign, it is truncated, or it was made by
another build of the game.

### 7.2 The last pass: the playtest work, the live feed, and the traits

**Three things were left, and they are the three that a human playtest would
have found first.** A human playtest was running alongside this work, so the
machine did the parts a machine can do and the human did the rest.

| The work | What it is |
|---|---|
| **The automated playtest** | The Slice 3 exit test, run by a machine: the five-year arc reaches orbit, loses a programme, a launch and an engineer, and **the chronicle names all three**. A machine cannot tell whether the game is fun. It can tell whether the arc is possible |
| **The live feed** | The dashboard's "Just happened" panel: the newest thing at the top, appearing the moment it happens, **with the player's own decisions in it** |
| **The traits** | Fourteen of them, two or three per person, each landing on the factor it describes |

**And the feed found a bug the playtest would have found too.** The world was
quieter than the design said by a factor of four, because the event pool's
total was compared against a roll scaled differently from the weights choosing
the event — **the roll was a roll against two different rulers**. The
chronicle of a five-year arc was dominated by whichever event had the largest
weight; it is now about a hundred entries of a dozen different kinds, which is
the difference between a history and a log.

**Two smaller things fell out of the same pass.** A missed primary objective
now **fails the programme**, because a programme the player has lost is a
programme with an aftermath and the aftermath starts with the programme being
marked as failed; and the chronicle entry for a deadline now **names the
programme** rather than the objective, because "Orbital Launch: a deadline the
state was promised was missed" is legible and "Reach orbit: a deadline the
state was promised was missed" is not.
**And every message was read.** The pass is now a test: every refusal the
engine can make is checked for being a sentence, for starting with a capital
letter, for ending with a full stop, and for containing no identifier. **The
pass found two**: a refusal about somebody who had left began with a
lowercase letter, and the message for an order the agency has no way to take
named the order kind - "SetPolicy" - which is the engine's name for it and not
the player's.

### 7.3 The last pass of all: the shift policy, the report, and saving from inside

**Three things, and the first of them is the oldest gap in the design.**

| The work | What it is |
|---|---|
| **The shift policy** | S1-48, S1-49 and S1-52: a standing instruction about shifts for the whole agency, the order that sets it, and the palette recipe that offers it |
| **The report** | `--report <FILE>` writes a plain-text summary of the campaign: the money, the staff, the programmes, the missions, what the agency knows, the worst of the chronicle, and every decision. **It is what a playtester sends back** |
| **Saving from inside the game** | `F3` saves and `F4` loads, so a player who never thought about saving can still save |

**The policy is a standing instruction and the per-person shift is an override
of it.** That is the whole design, and it is the opposite of a bulk write: a
bulk write has to be repeated, and the "tired ones" version of it would only
ever catch the tired ones *at that moment*. A standing instruction keeps
working as people tire, and the threshold makes it a **thermostat** — an
eased-off person recovers, comes back up to the threshold, and is eased off
again. **A person the manager has moved individually is moved off the policy**,
and the staff screen says so, because a person on a light shift because the
policy eased them off is a different thing from a person the manager put on
light duties.

**And the player can put somebody back on the policy**, which is the other half
of the override: "she works flat out until she is tired, and then she eases
off". The shift order's payload is `Option<u16>`, and `None` is that answer.

**The balance moved a little, and the measured table in `BALANCE.md` says so.**
The reference policies now manage the whole agency with one standing
instruction rather than thirty individual changes, and the cautious policy
reaches orbit one campaign in twenty less often — the policy has a week of
bureaucracy, so the first weeks of a campaign are worked at full load. **That is
the price of one order over thirty**, and the design accepts it.

**The report and the in-game save exist for the same reason: a playtester is a
person with an evening and a job to get back to.** The report is a file they can
paste into an email, and it names the seed so the campaign can be reproduced
exactly. The save is a key, because **a player must be able to save without
having planned to** — the campaign they have spent an evening on is not saved
unless they say so, and they should not have to restart the game to say it.
**And a headless run saves too**, because a headless run is a campaign that
ended like any other.

### 7.4 The advice banner: the game says what to do next

**The complaint was that the game was unclear, and the example was the first
programme.** "Reach orbit" is not a clue: it does not say that a launch is what
reaches orbit, and it does not say that the agency cannot design a launch rocket
until it has done the research that unlocks the destination. **A player who
cannot tell what the game wants has to guess, and the guessing is the whole of
the first hour.**

Two things came out of it, and they are different jobs.

| The work | What it is |
|---|---|
| **The objective says how it is done** | Every objective carries a line, `done by`, naming what finishes it: a successful launch to Low orbit, or the programme's own staff. **The domain already knew this and the screen was not showing it** |
| **The advice banner** | A panel at the top of the dashboard that says what the agency should be doing next, read from the campaign as it stands |

**The banner is not a tutorial, and that is the whole design.** A tutorial says
"now do this", and a player who came here to do something else has to ignore
it. **The banner looks at the campaign and finds the thing that is wrong or
missing right now**: an agency without orbital propulsion is told that it is
180 points of research short; an agency whose treasury has four months left is
told about the money; an agency with a rocket on the pad is told to launch it.

**It is read from the campaign every time the screen is drawn, and it is never
stored.** So the banner always matches the campaign the player is actually in,
and doing the thing it said changes what it says next. **The most urgent
advice is the one that is showing, and `n` shows the next one** — the advice
that is showing is the one about the thing that is actually costing the
campaign, and the rest are behind it.

**The keymap had to be trimmed to fit a twenty-four-row terminal**, because a
keymap whose last entry is cut off is a keymap the player cannot read, and the
entries that are the same idea are now on one line.

### 7.5 The arrows: up and down navigate, left and right change the value

**The order palette had it the wrong way round, and its own hint said so.** The
form said "(1 of 4 — ← → to change)" while the code made the left and right
arrows move between the *parameters* — which are one per line, so moving between
them looks like moving the highlight down the screen — and made the up and down
arrows change the value. **The interface was lying about its own keys.**

**The convention is now the same everywhere [D].** Up and down navigate, and
left and right change the value. On the staff screen the left and right arrows
are the shift, which is a value and not a field; in the order palette they are
the chosen option; on a list of orders there is no value to change, so they do
nothing and the up and down arrows move the highlight.

**Two navigation bugs came out of the same pass.**

- **The selection could walk off the end of a list, and the focus with it.** The
  down arrow added one to the index with nothing to stop it, so a player who
  pressed it a few times too often lost the highlight entirely — and had to
  press up a dozen times to get it back. **The selection is now clamped to the
  list**, when the arrow is pressed and every time the screen is drawn, because
  a list can change under the player.
- **The branches screen's list could not be moved at all.** The screen showed
  the decisions and marked one of them, but the arrows moved a different index
  entirely, so the player could not choose which decision to fork at. **There is
  one selection per screen now**, and the branches screen's is the decision the
  fork happens at.

**And the test that was supposed to catch it was passing for the wrong reason.**
"Every screen with a selection moves it" compared the *rendered text* before and
after the arrow, and the text changed because the tutorial hint had been
dismissed — not because the selection moved. It now compares the selection
itself.

### 7.6 The pictures: a terminal is a pixel canvas

**The player wanted some flare, and the finding is that a text terminal cannot
give them icons.** The ceiling is the character cell, not the tooling.

| Technique | Pixels per cell | Colours per cell |
|---|---|---|
| **The half-blocks** `▀▄█` | 1 across, 2 down — two | **two** |
| **The braille block** `⠿⣿` | 2 across, 4 down — **eight** | one |

**Both give square pixels**, because a cell is about twice as tall as it is wide.
The braille block is the finest a text terminal goes, and it is what the picture
converters use — **but the dots have gaps between them**, so a shape drawn in
braille reads as a dot matrix rather than as a shape. It is the right tool for a
*photograph*, where the dottiness reads as a gradient, and the wrong one for an
icon.

**So the finest solid pixel a terminal can draw is a whole cell across**, and a
picture you can read needs many cells. **An icon in the corner of a panel is
therefore a blob, or it eats the panel's width** — nine columns of a
sixty-column panel for a shape too coarse to read, and the panel is where the
money is.

**Both attempts were tried and both looked bad**, which is the finding: the
half-blocks are solid but the shapes are too coarse to read, and the braille is
fine but the shapes are made of dots. **The interface has no pictures.** The flare
is in the panels, the glyphs and the colours.

**Beyond braille there is only real graphics** — sixel and kitty, which put
actual pixels on the screen. **Crossterm does not do them**, they work in a
minority of terminals, and a game that needs a special terminal is not a game
that runs everywhere.

### 7.7 An assignment is a move

**The player reported that they could not put multiple scientists on a research
project.** The mechanic was there — the points are summed, and a project with
two people on it finishes twice as fast — but the order refused to give a
person a second job.

**Everything around it said the opposite.** The design's own order list calls
the order "assign / reassign". The phase that applies it says "moved from the
propulsion laboratory to the combustion study" and takes the person off the old
team. The error variant for the refusal says the person is named "so the player
knows what they are being moved from" — a sentence about a move that the
validation then prevented. And the advice banner told the player to "put
somebody else on it" and then refused the order.

**The order now moves the person**, and the one thing it refuses is a no-op:
putting somebody where they already are. The palette says whose team somebody
is being taken off *before* the order is confirmed, because a laboratory that
loses a scientist is a laboratory that wears out.

**The advice banner names somebody now** — "put Tomas Reyes on it" rather than
"put somebody else on it" — because a name is a sentence the player can act on
and "somebody else" is one they have to act out.

### 7.8 The performance and stability pass

**The player asked for a pass over the codebase, on the grounds that they did not
want to have to do big refactors when the project got much larger.** The game is
designed for three hundred people and the shipped scenarios have twelve, so the
question was what happens at the far end.

**The answer was: a quadratic.** The tools factor — the quality of the agency's
facilities, and whether it is over capacity — is the only part of the
performance model that reads the whole state, and it was being computed once per
person per project per hour of simulated time. A five-year campaign of three
hundred people took **ten and a half seconds**, and it grew faster than the
agency did.

| | Before | After |
|---|---|---|
| **Twelve people** | 0.15 s | 0.08 s |
| **Fifty people** | 0.59 s | 0.24 s |
| **A hundred and fifty** | 3.1 s | 0.75 s |
| **Three hundred** | **10.6 s** | **1.5 s** |

**The tools factor is computed once per hour now** and passed down, and the
performance model keeps its eight modifiers in an array instead of a vector —
**the model is evaluated a hundred and fifty thousand times a second at three
hundred people, and a heap allocation in that path is millions of allocations in
a campaign.** The same fix applies to the binding-factor panel, which is drawn on
every frame and was quadratic in the size of the agency.

**The guard is a test, and it compares two sizes rather than a clock.** A test
that says "a campaign takes under a second" fails on a slow machine and passes
on a fast one; a test that says "four times the staff costs less than eight
times as much" catches the shape of the cost, which is the thing that matters
and the thing that decides whether the project needs a rewrite.

**The other things the pass turned up:**

- **The live feed copied every headline in the chronicle, every frame.** The
  chronicle is the campaign's whole history and it grows for ever, so the feed
  was copying a thousand strings to show twelve of them. The candidates are
  gathered as references now, and only what the feed shows is copied.
- **The treasury's burn rate was built as a list of named lines on every hour**,
  which allocates, when all the caller wanted was the total.
- **The vehicle builds read the industrial capacity once per vehicle**, and the
  programme rate built a list of target points to check that it was not empty.
- **The state's clone per phase is the next thing to look at**, and the
  chronicle is the part of it that grows. It is not a problem yet — a
  five-year campaign has two hundred entries — but it is the one to watch.

### 7.9 The feed said everything twice, and some of it moved

**The player noticed that some of the timestamps in the "Just happened" panel
moved with the clock, and that most of the entries were duplicated.** Both were
real, and they had the same cause.

**Every event's headline was written to the chronicle and to the month report.**
The chronicle is the durable record and the report is the month's summary, so
the two overlapping is right — but the *live feed* reads both, so it showed
every event twice: once from the chronicle and once from the report lines
waiting for the month. The event is a chronicle entry and nothing else now, and
the month report still shows it, in its events section.

**The report lines carried no instant of their own.** The feed stamped them all
with "now", which is why they slid forward with the clock while the chronicle
entries stayed where they were. **They carry the instant of the step that
produced them now**, and a line says when it happened.

### 7.10 The shift policy stopped buzzing

**The playtest said the shift policy "just toggles on and off every hour or so
around the limit set", and it was right.** The policy's threshold was a
comparison — a person was eased off while their fatigue was at or over the
limit, and put back on a full load the moment it fell below — and the
comparison was made every hour, on a quantity that converges to the limit.

**The measurement was six hundred and sixty-four shift changes a person a
year.** A person worked a full load, tired past the limit, was eased, recovered
to the limit, and was eased again, and the campaign was a few hours later. The
agency got half of neither shift: the output flickered, the performance model's
binding factor moved for no reason the player could see, and a policy the
player had set deliberately was doing the opposite of what it said.

**The threshold is a latch now, and the latch has a margin.** A person is eased
when their fatigue reaches the limit, and stays eased until their fatigue has
come down by ten points. The margin is the difference between a thermostat and
a buzzer, and the same campaign now changes shift **once** a person a year
rather than six hundred and sixty-four times.

**It is a bit of state, and it has to be.** A comparison of fatigue against a
limit cannot express "eased until they have recovered", because the answer in
between the two numbers depends on what happened before. So the person carries
a bit — `policy_eased` — and the bit is the thermostat's memory. It is set once
an hour, where the fatigue moves, and it is cleared when the manager moves the
person or the policy is withdrawn, so that a decision the policy made about
somebody else is never inherited.

**The person who is eased says so.** The staff screen said "50% (policy)" for
both a person the policy had eased and a person the policy had simply never
touched, and those are different things: one is the agency's doing and the
other is not. It says "eased by the policy" now, and the manual's shift topic
explains the margin.

**The balance moved.** The cautious policy uses the shift policy, and a policy
that holds is a policy that costs: the cautious campaign launches twice where
it used to launch once, and the fast one gets to orbit three months sooner
because its people are not being held at half a load for eight months at a
time. All three reference policies still reach orbit and still meet the
deadline. `BALANCE.md` §12 has the new distribution.

### 7.11 The game saves itself, and it saves where it is allowed to

**The quit dialog said "the campaign is not saved. Quitting ends it", and that
was true — and it was also the single worst thing about the game.** A player
who has spent an evening on a campaign and believes that quitting loses it
will not quit when they should, and will not close the terminal, and will
leave the game running overnight to avoid losing the evening.

**The game writes the campaign at every month boundary now**, so the worst a
player can lose by closing the terminal is a month, and it writes it again
when the player quits, so the worst they can lose by quitting is nothing. The
dialog says what is true.

**And the game writes to the platform's configuration directory**, which is the
one thing `PERSISTENCE.md` §9 has said since the beginning: `%APPDATA%` on
Windows, `~/Library/Application Support` on macOS, and the XDG directory
everywhere else. **The game used to write `campaign.json` beside its own
executable**, which is a save file that turns up in whatever folder the player
happened to run the game from, and a save file that moves is a save file that
is lost. The directory is made if it is not there, because the first save is
the one that has to create it.

**The automatic save is quiet and throttled.** A save that says something every
month is a message the player learns to ignore, and the one time it matters —
the save that failed — is the one that would be lost in the noise. And at the
top dilation a month boundary arrives several times a second, so the game
waits ten seconds of real time between automatic saves.

**The save slots are still on the list.** A player who wants to keep an old
campaign has nowhere to put it, and the design's numbered slots are the answer.

## 8. What is left

**Everything in this plan is done except the things below**, and they are here
so that nobody has to read the whole document to find out what the game still
needs.

**The playtests are done.** They are in §7.10 and in the milestone tables, and
what they found has been acted on.

### The playtests

**They have been run.** All four of them, and the game worked: the campaigns
ran, the objectives were reachable, the order palette did what it said, and
nothing crashed. The findings were about **understanding the mechanics** rather
than about anything being broken, and the ones that have been acted on are:
the shift policy's thermostat (§7.10), the advice banner (§7.4), and the
assignment being a move (§7.7).

| ID | The playtest | What it had to answer | What it said |
|---|---|---|---|
| S1-36 | The first one. Five people, one scenario, one hour each. | Does a new player understand what to do in five minutes? | The mechanics were not obvious; the advice banner and the "done by" line on each objective are the answers |
| S2-17 | The commitment mechanic. | Can a player commit to an instant, watch the slack shrink, and name what they lost? | The commitment reads; the slack meter does not speak for itself |
| S3-28 | The five-year arc. | Does a player reach orbit, having lost something on the way? | Yes, and the shift policy was the thing that fought back |
| S4-12 | The final one, with someone who has never seen it. | Do they reach orbit in one sitting, and ask to try a different path? | Yes |

### The two filter tasks

| ID | The task | Why it matters |
|---|---|---|
| S1-50 | **Every other list is filterable and sortable.** The machinery is built and the staff list is wired to it; the research, programme, chronicle and branches lists are not. | An agency of three hundred people has lists the player cannot search |
| S1-51 | **The filter can express a condition** — "fatigue over 80", "unassigned" — and not only a name. | Finding everybody who is in trouble should not mean reading three hundred names |

### The things the design says and the game does not do yet

These are named in the design documents and are not built. They are listed here
so that they are not mistaken for oversights.

| What | Where it is promised |
|---|---|
| **Retirement, illness, injury and death** | `BALANCE.md` §4.4 — the exits a person can have that are not unhappiness |
| **The numbered save slots** | `PERSISTENCE.md` §9 — a player who wants to keep an old campaign has nowhere to put it |
| **A deeper applicant pool**, and the events in which the state asks why | `ROADMAP.md` §4.4.3 — the goodwill's effects on recruitment |
| **A mandate crisis** — the state's patience running out | `ROADMAP.md` §4.4.3 |
| **Rival agencies** | `MANAGEABLE_PARTS.md` §13.2 — the rivals can be switched off, and they are off |
| **Planetary protection and sample-return contamination** | `MVP.md` §3 — a late-game pillar, and the MVP has no bacterial life |

## 9. Cross-cutting work

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

## 10. The critical path

**The path changed with the time model.** The old critical path ran through the monthly
tick; the new one runs through the redesign and then through the live interface.

```
M1-01  →  M1-04  →  M1-05  →  M1-07  →  M1-06  →  M1-14
   →  M1-08  →  M1-09  →  M1-10  →  M1-13  →  S1-37  →  S2-05  →  S3-05  →  S4-02
```

That is: **the clock → the scheduler → the advance loop → the rates → the draw anchors →
the dilation test → the order model → goodwill → the stable buckets → the host loop → the
order palette → the government's deadline → the first launch → branching.**

**The whole path is done.** M1-01, M1-04, M1-05, M1-06, M1-07, M1-08, M1-09, M1-10, M1-13, M1-14, S1-37, S2-05, S3-05 and S4-02 are all complete, and so is everything else in this plan except the four playtests, which need a human, and the two filter tasks in §3.11. **The path below is the record of how the game was built, and it is here because the order mattered** — each step assumed the one before it.

### 10.1 The three long poles

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

### 10.2 The riskiest single task

**M1-14, the dilation-independence test** — not because it is hard, but because
**it is the test that can fail for a reason nobody can see.** The numerical integration
approximates the same integral at different granularities, and if the error is big
enough to flip a decision, two campaigns at different dilations will diverge. The test
is the only thing standing between that and a game whose outcome depends on the frame
rate.

### 10.3 The two things that are not on the critical path but must not be skipped

- **M1-12, the second validation.** An order that is issued and then finds its target
  has changed is a *new* failure mode, and it is the one the player will hit most
  often. It needs a design decision, not just code.
- **M1-10, the lead times.** A game where every order lands instantly is a
  spreadsheet. The lead times are the whole difference between a management game and a
  form, and they are a handful of numbers.

---
## 11. Definition of done

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

## 12. Risks to this plan

**The plan is delivered, and these are the risks that were retired by it.** They
are kept because they say what the plan was for: each one would have ended the
project, and each one has a mitigation that was built rather than hoped for.

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

## 13. Related documents

- `MVP.md` — the scope this plan delivers.
- `ARCHITECTURE.md` — the crate and module layout the tasks refer to.
- `SIMULATION.md` — the engine contract the tasks implement.
- `DOMAIN_MODEL.md` — the entities the tasks build.
- `PERSISTENCE.md` — save format, `sim_version`, replay, branching.
- `TESTING.md` — the test strategy the tasks rely on.
- `UI.md` — the screens and the information design.
- `BALANCE.md` — the numbers.
- `README.md` — the documentation index.
