# MVP.md

> **The minimum viable *game*.**
>
> Not a demo, not a prototype — a complete, playable, saveable campaign that a new
> player can sit down with and lose at. Everything in here is scoped to prove that the
> management loop is fun and that the architecture holds.
>
> **Status:** Proposal, revision 2. Revision 2 adapts the scope to the continuous time
> model: the game is no longer played in monthly turns, and the slice is measured in
> simulated years and real hours rather than in turns.

---

## 1. The MVP in one sentence

> You found a space agency with a small budget, a deadline from the government, and five
> years to get something into orbit — and every order you give takes time to land.

**Working title:** *Five Years to Orbit*.

### Why this scope

The MVP is chosen to **exercise every risky part of the design at once**, in the smallest
package that can do so:

| Risky part | Where the MVP exercises it |
|---|---|
| **Continuous time and dilation** | The whole game is five years of running simulation, and the player watches it at nine different rates |
| **Orders that take time** | A hire takes three weeks. A launch takes ninety minutes. The player must plan around lead times from the first hour |
| Target vs. committed dates and slack | The government's orbital deadline is the spine of the scenario |
| C+B performance model | Continuously, with staff too few to hide behind |
| Capability vs. reliability | Research and test stand compete for the same three scientists |
| Context-weighted events | A tired, underfunded agency in year three should *feel* like one |
| Chronicle, feed, and timeline | The player must be able to find out why year two went badly |
| **Dilation independence** | The scenario is a seed plus an order script; the campaign must be the same at the base rate and at the top |
| The TUI | Six live screens, the order palette, and the binding-factor explanation must be legible |

If the MVP is not fun, the design is wrong. Everything after this is elaboration.

**The pacing is part of the scope [D].** Five simulated years is about four and a half
hours of real time at the maximum dilation, and the first launch adds ninety minutes if
the player watches it at the base rate. **The playtest is run at 3,000x-30,000x**, where the launch is
under a minute, and the 1x launch is validated separately, once, by somebody who is
prepared to sit through it.

---

## 2. Scope: what is in

| Domain | MVP scope | Notes |
|---|---|---|
| **Time** | Continuous simulated time, running in real time at nine dilations, ~5 simulated years | The scheduler, the continuous integration, and all ten phases - because the phase order is the architecture |
| **Time** | Continuous calendar, monthly resolution, ~5-8 in-game years | The full tick from `SIMULATION.md` §2.2 — all ten phases, because the phase order is the architecture |
| **Money** | State appropriation (daily accrual, annual true-up), payroll, programme spend, construction spend, upkeep | No investors, no debt, no commercial revenue, no crisis ladder beyond a warning. The **parameterised** revenue model ships with one source wired; the structure is right, the breadth is not |
| **People** | 40-80 staff, roles: Scientists, Engineers, Astronauts, Administrators, Managers | No relationship graph. 2-3 traits per person, traits modify performance directly |
| **Performance** | The full **C+B hybrid** | Clamped, saturating, with the **binding factor** shown. This is the MVP's most important single system |
| **Programmes** | Create, set objectives, set target and committed dates, set budget, assign staff, track progress | One programme category family: *Orbital* and *Lunar* only |
| **Research** | 3 disciplines: Propulsion, Structures & Materials, Life Support | Full capability-vs-reliability tension. Project, PI, funding, duration, breakthrough |
| **Testing** | Test stand runs a testing campaign against an existing capability | Reduces the risk of what the agency already knows how to build. Not a research project — a separate activity competing for the same staff |
| **Facilities** | 4 types: Laboratory, Test Stand, Fabrication Plant, Launch Complex. Each individually built, with **method** (Fast & cheap / Standard / Robust) and **scale** (Small / Large) as construction choices | The full 8-choice construction matrix is deferred; the *mechanism* ships, with 2 of the axes |
| **Vehicles** | 3 archetypes: Sounding Rocket, Orbital Launcher, Orbital Capsule. 4 sliders: Propulsion, Payload, Life Support, Redundancy | Enough to build, orbit, and lose a capsule |
| **Missions** | Preparation → launch → transit → operations → return. Success and partial failure. Results feed research | No sample return, no contamination, no interstellar |
| **Events** | ~35 hand-authored events across all 8 categories, with context weighting and follow-ups | The weight is in the *weighting*, not the count |
| **Politics** | A single state actor: approval, political capital, and one-off milestone demands. A promise tracker | No factions, no elections, no ministers |
| **Public** | Public opinion and prestige as two numbers, with a press line on notable events | No media outlets, no heroes system, no scandals |
| **Chronicle** | Full. Queryable by year, category, subject. The legend list. The live feed | This is the MVP's best surface and ships complete |
| **Timeline** | Full. The next few weeks of scheduled obligations, orders in flight, and their known risk figures | Ships complete - it is the other half of the chronicle |
| **Save / load** | Full, versioned against `sim_version` | |
| **Replay** | The order log, exact reproduction, and **branching** at any instant | The MVP's most expensive feature and the one that most needs proving |
| **Rivals** | **Off** | The toggle exists in the options; the rival model is deferred. The open question in `MANAGEABLE_PARTS.md` §17.1 stays open until the core is proven |
| **Alien life** | **None in the MVP** | Destinations are lifeless. Bacterial life and planetary protection are a late-game pillar and have no place in a five-year slice |

### TUI screens (6)

| Screen | Purpose |
|---|---|
| **Dashboard** | The clock, the treasury, the burn rate, the headline, what needs attention, and what is in flight |
| **Staff** | List, filters, detail panel with the modifier breakdown and the **binding factor** |
| **Programme** | Objectives, target vs. committed dates, slack, budget, progress, risk |
| **Research & Facilities** | Projects, test campaigns, facilities, construction progress, outputs |
| **Vehicles & Missions** | Design, sliders, derived stats, risk, launch authorisation, mission status |
| **Chronicle** | The live feed and the timeline, filterable |

Plus a persistent top bar (date and time, the dilation, the treasury, the approval, the goodwill, and what is due), the **order palette**, and the pause control.

---

## 3. Scope: what is out, and why

| Deferred | Reason |
|---|---|
| Rival agencies | Needs the simulation to be multi-agent. Unproven architecture, and the toggle defaults to off |
| Political factions, elections, ministers | One state actor is enough to test approval, capital, and promises. Factions are a refinement |
| Media, heroes, scandals, ceremonies | All downstream of the public-opinion number. The number ships; the texture waits |
| Relationship graph | Emergent social dynamics are the most expensive and least certain system. Traits ship first |
| Investors, commercial revenue, debt, crisis ladder | The revenue model ships *parameterised*; only the state source is wired. The ladder waits until there is a second source to fight over |
| Publish / patent / license | Depends on rivals existing to license to |
| 12 research disciplines | Three is enough to test progression and unlock gating. The rest is data |
| Full construction matrix | Method and scale prove the mechanism; automation, redundancy, materials, siting, financing, and public face are more axes, not more ideas |
| Planetary protection, contamination, microbial life | A late-game pillar. No bacterial life in the MVP means no MVP risk of the player reaching a destination that does not exist yet |
| 150-300 staff | 40-80 exercises the same code path. The aggregate screens and filters get tested at the smaller number, which is the harder case anyway |
| Nine vehicle sliders | Four prove the derived-statistics pipeline |
| Milestone primacy | Requires rivals |
| Rebalancing, difficulty modes, scenario editor | After the loop is proven fun |

**The discipline:** nothing in the "out" list may leak into the MVP. If a mechanic is not
in §2, it does not get built, and if it turns out to be needed, the MVP gets longer by
explicit decision rather than by drift.

---

## 4. Build order — four vertical slices

Each slice ends with something a playtester can actually sit down and play. No slice is
"just infrastructure".

### Slice 1 — *Ground Game* (months 0-11)

A playable first year.

- Workspace, crate split, `domain` / `sim` / `ui`
- The tick: all ten phases, with only personnel, finance, and research doing real work
- The RNG contract and the determinism test, from day one
- State, order log, save/load
- The domain model: people, budgets, research projects, facilities (minimum viable)
- The performance model with the binding factor, and **the screen that explains it**
- TUI: dashboard, staff, research, chronicle + the live feed
- **A scenario**: a fixed seed that starts a small agency with a budget, three
  scientists, and a research project already running

**Exit test:** a playtester hires two people, runs a research project for a year, sees it
complete, sees a breakthrough unlock something, and can explain why their team was slow.

### Slice 2 — *The Programme* (months 12-35)

The commitment mechanic, and the heart of the game.

- Programmes with objectives, **target and committed dates**, and **slack**
- The government's orbital deadline as a hard committed date
- Facilities: construction, completion, staffing, upkeep, degradation
- Facility **method** and **scale** as construction choices
- Industrial capacity as a real constraint
- Deadline handling: met, missed, renegotiated, absorbed
- The budget-crisis warning state
- TUI: programme screen, facilities screen
- **Exit test:** a playtester commits to a date, watches the slack shrink, and either
  hits it, moves it, or eats the consequence — and can name what they lost.

### Slice 3 — *First Orbit* (months 36-96)

The payoff, and the danger.

- Vehicles, sliders, derived statistics, cost and risk
- Missions as state machines; success, **partial failure**, and full failure
- Results feeding research, and research gating better vehicles
- The capability-vs-reliability choice in earnest: a launch now, or a test first
- ~35 events, context-weighted, with follow-ups
- The full chronicle and the full forecast
- The political layer: approval, capital, promises, milestone demands
- Public opinion and prestige
- **Exit test:** a playtester gets to orbit, and on the way loses a programme, a
  launch, and an engineer — and can read all three in the chronicle.

### Slice 4 — *Proof*

Make it a *product*, not a build.

- Replay and **branching** (fork at any month, re-simulate, compare)
- Balance: run the harness across N seeds, read the outcome distributions, tune
- Onboarding: the `PLAYER_INTRO.md` framing as an in-game opening, plus a first-month
  tutorial that teaches the one non-obvious thing: **the binding factor**
- The **"First Orbit" scenario**: a tuned seed and start state that reliably produces a
  five-year arc with real decisions in it
- Save/load polish, error messages, crash-safety
- A performance test asserting a full campaign resolves in under a second

**Exit test:** a new player, given nothing but the game, reaches orbit in one sitting
and wants to try a different path — which is exactly what branching is for.

---

## 5. Success criteria

The MVP is done when all of these are true. They are deliberately measurable, because
"it feels good" is not a test.

| # | Criterion | How it is checked |
|---|---|---|
| 1 | A new player understands what to do within 5 minutes | Observation, in a recorded playtest |
| 2 | A 5-year campaign is completable in under 90 minutes of play | Observation, at the dilations the player chooses |
| 3 | **At least three strategies survive** - cautious/test-heavy, fast/cheap, science-first | The balance harness, across N seeds. If two of the three collapse, the trade-offs are fake |
| 4 | A failed programme produces a legible aftermath the player can trace | Playtest: ask the player why it failed |
| 5 | The player opens the chronicle unprompted | Observation - the best proxy for whether the history is worth having |
| 6 | The binding factor is understood on first sight | Playtest question, or a UI test on the explanation panel |
| 7 | Replay reproduces byte-identically | Automated, every build |
| 8 | **The campaign is identical at the base rate and at the top rate** | Automated, at every dilation step |
| 9 | A full campaign resolves in under a second | Automated performance test |
| 10 | No crash, no soft-lock, no unrecoverable state, from any order sequence | The fuzz test, nightly |

Criterion 3 is the one that matters most. If only one strategy works, the game is a
puzzle with a single answer, and everything the design document says about trade-offs is
decoration.

**Criterion 8 is the one that matters most mechanically.** It is the guarantee that the
time model is sound, and without it the game has a frame-rate-dependent outcome.

---

## 6. Top risks

| Risk | Why it is dangerous | Mitigation |
|---|---|---|
| **The performance model is illegible** | It is the deepest system in the game and the hardest to explain. If the player cannot see why their team is slow, the whole People pillar is noise | Build the **binding-factor explanation screen first**, in Slice 1, before the model is tuned. It is a UI spike, not a feature |
| **The real-time model feels like waiting** | Continuous time is the biggest departure from the genre, and a decade of nothing is a decade of nothing | The dilation is the answer: nine rates, and the player's choice of rate is the choice of how much to watch. Make "needs attention" a standing panel, so there is always something to read |
| **The player stops caring** | A real-time management game can become background noise | The feed, the chronicle, and the projections. A game that tells you "this project will take four months at the current rate" is a game you keep checking |
| **Dilation independence fails** | The outcome would depend on the frame rate, and it would be almost impossible to debug | The test in M1-14, at every dilation step, before any of it is playable |
| **Events feel random rather than earned** | Unearned bad outcomes teach the player nothing and get the game deleted | Context weighting from Slice 3's first event, and the chronicle entry must state the *cause*, not just the effect |
| **Scope creep** | The management surface is huge and every part of it is attractive | The §3 "out" list is binding. Adding to it requires removing something else |
| **Determinism bugs found late** | They are the most expensive kind to fix, because they invalidate replays and golden tests at once | The determinism test and golden test exist from Slice 1, before there is anything to break |
| **The TUI cannot carry the information density** | A management sim's entire value is *legibility* | Six screens is few enough to design well. The MVP is the right moment to find out that ratatui is enough |

---

## 7. What the MVP is *for*

Beyond "is it fun", the MVP has a second job: **it validates the architecture**.

By the end of Slice 4, these questions should have hard answers:

- Does the order-log + resimulation design hold up? (Branching is the test.)
- Is the ten-phase tick the right shape, or is a phase missing or misplaced?
- Is the C+B performance model tunable in practice, or is the saturation curve a pain?
- Does the one-agent-per-system event definition scale to more systems?
- Is the sim fast enough for the UI to re-simulate on demand?
- Did anything in the management surface turn out to be unmanageable — too fiddly, too
  abstract, or too invisible?

**If the MVP ships and the answers are good, the rest of the game is expansion.** The
remaining domains in `MANAGEABLE_PARTS.md` are, mostly, additional data and additional
screens, applied to an engine that already works.

**If the answers are bad, better to find out here**, on five years and one government
deadline, than on forty years and a solar system.

---

## 8. Related documents

- `SIMULATION.md` — the engine this MVP is built on. Read this first.
- `MANAGEABLE_PARTS.md` — the full management surface, of which this is a slice.
- `README.md` — the documentation index and build plan.
- `../AGENTS.md` — workspace layout and repository conventions.
