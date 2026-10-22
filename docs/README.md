# Documentation

Design documentation for `space_manager` — a space agency management simulation.

## Reading order

| # | Document | What it is | Status |
|---|---|---|---|
| 1 | `PLAYER_INTRO.md` | The in-fiction framing. Who the player is and what the game feels like. | Current |
| 2 | `DEVELOPER_INTRO.md` | The original vision: pillars, core loop, long-term progression. | Current |
| 3 | `MANAGEABLE_PARTS.md` | **The management surface** — every system the player can act on, and therefore every system the simulation must model. The scoping document. | Current |
| 4 | `MVP.md` | The minimum viable *game*: scope, build order, success criteria, risks. | Current |
| 5 | `SIMULATION.md` | The engine: the clock and resolution order, determinism contract, RNG, orders, event system. | Current |
| 6 | `ROADMAP.md` | **The build plan** — task-level, with dependencies and exit criteria, and what is still left. | Current |
| 7 | `ARCHITECTURE.md` | Crate layout, module boundaries, dependency direction, the public API of each crate. | Current |
| 8 | `DOMAIN_MODEL.md` | The entities, their fields, the invariants, and what is derived rather than stored. | Current |
| 9 | `PERSISTENCE.md` | Save format, `sim_version`, the order log, replay, and branching. | Current |
| 10 | `TESTING.md` | The test strategy: golden, determinism, property, fuzz, balance harness, performance. | Current |
| 11 | `UI.md` | The seven screens, the key map, the binding-factor widget, colour and layout rules. | Current |
| 12 | `BALANCE.md` | The numbers, where they live, and how they get tuned. | Current |
| 13 | `ECONOMY.md` | Money in depth: income parameters, escalation, the budget-crisis ladder. | Deferred |
| 14 | `PEOPLE.md` | Staff model in depth: careers, development, relationships, attrition. | Deferred |
| 15 | `PROGRAMMES.md` | Programmes in depth: objectives, milestones, risk, failure aftermath. | Deferred |
| 16 | `MISSIONS.md` | Vehicles, subsystem sliders, mission construction, success and failure. | Deferred |
| 17 | `RESEARCH.md` | Disciplines, projects, capability vs. reliability, discovery. | Deferred |
| 18 | `INFRASTRUCTURE.md` | Facilities, the full construction matrix, degradation, industrial capacity. | Deferred |
| 19 | `POLITICS.md` | The state, factions, approval, promises, terms and elections. | Deferred |
| 20 | `PUBLIC.md` | Opinion, prestige, media, heroes, scandals, ceremonies. | Deferred |
| 21 | `WORLD.md` | Rival agencies, relations, suppliers, the alien-life scope. | Deferred |
| 22 | `DECISIONS.md` | The decision log — every settled design decision and why. | Deferred |

**Start with `MANAGEABLE_PARTS.md`.** It defines the scope of the game; every later
document elaborates one domain of it, and `SIMULATION.md` is the one that makes it move.
`MVP.md` then says which parts are built first, and `ROADMAP.md` says in what order.

### Documents 1-12 form the MVP set

Everything needed to build the MVP is written, and all twelve are kept in step with
the code. **Documents 13-22 are deferred, not promised** — they are written when the
game proves a domain needs more than `MANAGEABLE_PARTS.md` and `BALANCE.md` give it,
and nothing in the build depends on them.

### Progress

| Milestone | State |
|---|---|
| **M0 - Foundation** | **Complete.** The workspace, the four crates, the pinned RNG, the save envelope, the terminal shell, the dependency-direction check, CI, and the README. |
| **S1 - Ground Game** | **Complete, and rebuilt on the continuous model.** The engine, the domain model, the performance model, the playable loop, the interface - including the order palette, the binding-factor panel, the manual, the tutorial, and the main menu - and the golden fixture. The hands-on play that ran alongside the build found the five things in `ROADMAP.md` §3.6; the formal five-person playtest is still outstanding. |
| **M1 - Continuous Time** | **Complete.** The clock, the scaled quantities, the host's clock, the scheduler, the advance loop, the draw anchors, the rates, the order model, the goodwill, the stable buckets, the save format and the version, the host loop, the top bar, the dilation-independence test, and the fork. See `ROADMAP.md` §4. |
| **S2 - The Programme** | **Complete, except for the playtest.** The commitment mechanic, the slack, the facilities, the industrial capacity, the budget-crisis warning, and the programme and facilities screens. See `ROADMAP.md` §5. |
| **S3 - First Orbit** | **Complete, except for the playtest.** The vehicles, the missions, the capability gating, the event catalogue, the politics, the **"First Orbit" scenario**, the **balance harness**, the cross-system invariants, the **fuzz test**, and the **performance test**. Six bugs and a balance rewrite came out of it; they are in `ROADMAP.md` §6.5. |
| **S4 - Proof** | **Complete, except for the playtest.** Replay, branching, the branch comparison view, the onboarding briefing, the balance pass, the strategy check, the error-message pass, the save and load, and the documentation pass. See `ROADMAP.md` §7. |

**The last pass added the three things a human playtest would have found
first**: the **automated playtest** (the Slice 3 exit test, run by a machine),
the **live feed** on the dashboard, and the **traits**. It also found that the
world was reacting at a quarter of the designed rate, because the event pool's
total was compared against a roll scaled differently from the weights choosing
the event. `ROADMAP.md` §7.2 has the whole of it.

**The pass after that answered the questions the first one raised.** A player
could not tell what to do, so the dashboard says what to do; a player could not
put two scientists on one project, so an assignment is a move; the order palette
had the arrows the wrong way round; the feed said everything twice and its
timestamps slid with the clock; a five-year campaign of three hundred people
took ten and a half seconds, so it now takes one and a half; and the player asked
for flare, so the finding was that a text terminal cannot give it and the art
came back out. `ROADMAP.md` §7.3 to §7.9 has each of them.

**What is left is in `ROADMAP.md` §8, in one place.** It is the four formal playtests,
which need a person, and the handful of things the design promises and the game has
not built yet — retirement, illness, injury and death; the numbered save slots; the
deeper applicant pool and the events in which the state asks why; the mandate crisis;
the rivals; the press line; the lunar destination; and the planetary protection of the
late game.

## The build plan, in brief

The game is built as a **simulation with a view** — a deterministic core that turns a
seed plus a sequence of decisions into a forty-year history, and a TUI that renders and
edits that history. Everything below follows from that.

1. **Simulation first, interface second.** The month-resolution engine, the RNG, and the
   domain model are the game. The TUI is a window onto them.
2. **Vertical slices over horizontal layers.** Build a thin slice that plays a year
   end-to-end — budget, one programme, three staff, one launch, one event — then thicken
   each domain. Never build a full data model with nothing to run it.
3. **Determinism from day one.** The seeded RNG and the replayable command log are
   architecture, not a feature to add later. Every domain document must state what is
   random and what is not.
4. **Every pillar earns its place by creating a trade-off.** A system that only adds
   options is a spreadsheet; a system that creates a decision is a game.
5. **Failure is content.** Budget crises, failed programmes, accidents, and cover-ups
   must have defined aftermaths that the simulation can continue from.

## How the documents relate

```
  MANAGEABLE_PARTS.md   what the player manages           ── the scope
         │
         ▼
  SIMULATION.md         what happens to it, and when       ── the engine
         │           DOMAIN_MODEL.md   what it is made of
         │           BALANCE.md        the numbers
         ▼
  ARCHITECTURE.md       where the code lives              ── the structure
         │
         ▼
  PERSISTENCE.md   TESTING.md   UI.md                      ── the edges
         │
         ▼
  ROADMAP.md   MVP.md                                        ── the plan
```

`ROADMAP.md` is the operational document: task-level, with dependencies, exit
criteria, and a critical path. Read it to start work.

## Document conventions

- **Filenames** are `UPPER_SNAKE_CASE.md`, matching the existing two documents.
- **Status** is stated at the top of each document — `Design intent`, `Proposal` or
  `Plan`, with the revision number.
- **Decisions** are marked inline: **[D]** settled, **[?]** open, and every open question
  is repeated in that document's "Open decisions" table so it can be answered in one
  place. Answered questions get promoted into a decision log rather than silently edited.
- **Domain documents describe behaviour, not code.** File and module names belong in
  `ARCHITECTURE.md`; this folder should stay readable by someone who never opens the
  repository.
- **One document, one domain.** Cross-references are by link, not by copy-paste.

## Relationship to the code

The planned crate split (`domain` for the domain model, `sim` for resolution logic, `ui` for
the TUI) mirrors the document split: `DOMAIN_MODEL.md` and the per-domain documents
describe what lives in `domain`; `SIMULATION.md` describes `sim`; `UI.md` describes `ui`.

The authoritative workspace layout and dependency rules live in `AGENTS.md` at the
repository root, not here.
