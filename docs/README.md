# Documentation

Design documentation for `space_manager` — a space agency management simulation.

## Reading order

| # | Document | What it is | Status |
|---|---|---|---|
| 1 | `PLAYER_INTRO.md` | The in-fiction framing. Who the player is and what the game feels like. | Done |
| 2 | `DEVELOPER_INTRO.md` | The original vision: pillars, core loop, long-term progression. | Done |
| 3 | `MANAGEABLE_PARTS.md` | **The management surface** — every system the player can act on, and therefore every system the simulation must model. The scoping document. | Draft (rev 2) |
| 4 | `MVP.md` | The minimum viable *game*: scope, build order, success criteria, risks. | Proposal (rev 1) |
| 5 | `SIMULATION.md` | The engine: the clock and resolution order, determinism contract, RNG, orders, event system. | Draft (rev 2) |
| 6 | `ROADMAP.md` | **The build plan** — task-level, with dependencies and exit criteria. | Plan (rev 1) |
| 7 | `ARCHITECTURE.md` | Crate layout, module boundaries, dependency direction, the public API of each crate. | Draft (rev 1) |
| 8 | `DOMAIN_MODEL.md` | The entities, their fields, the invariants, and what is derived rather than stored. | Draft (rev 1) |
| 9 | `PERSISTENCE.md` | Save format, `sim_version`, the order log, replay, and branching. | Draft (rev 2) |
| 10 | `TESTING.md` | The test strategy: golden, determinism, property, fuzz, balance harness, performance. | Draft (rev 1) |
| 11 | `UI.md` | The seven screens, the key map, the binding-factor widget, colour and layout rules. | Draft (rev 2) |
| 12 | `BALANCE.md` | The numbers, where they live, and how they get tuned. | Draft (rev 1) |
| 13 | `ECONOMY.md` | Money in depth: income parameters, escalation, the budget-crisis ladder. | Planned |
| 14 | `PEOPLE.md` | Staff model in depth: careers, development, relationships, attrition. | Planned |
| 15 | `PROGRAMMES.md` | Programmes in depth: objectives, milestones, risk, failure aftermath. | Planned |
| 16 | `MISSIONS.md` | Vehicles, subsystem sliders, mission construction, success and failure. | Planned |
| 17 | `RESEARCH.md` | Disciplines, projects, capability vs. reliability, discovery. | Planned |
| 18 | `INFRASTRUCTURE.md` | Facilities, the full construction matrix, degradation, industrial capacity. | Planned |
| 19 | `POLITICS.md` | The state, factions, approval, promises, terms and elections. | Planned |
| 20 | `PUBLIC.md` | Opinion, prestige, media, heroes, scandals, ceremonies. | Planned |
| 21 | `WORLD.md` | Rival agencies, relations, suppliers, the alien-life scope. | Planned |
| 22 | `DECISIONS.md` | The decision log — every settled design decision and why. | Planned |

**Start with `MANAGEABLE_PARTS.md`.** It defines the scope of the game; every later
document elaborates one domain of it, and `SIMULATION.md` is the one that makes it move.
`MVP.md` then says which parts are built first, and `ROADMAP.md` says in what order.

### Documents 1-12 form the MVP set

Everything needed to build the MVP is written. Documents 13-22 elaborate domains that
the MVP deliberately defers — they are written when the MVP proves a domain needs more
than `MANAGEABLE_PARTS.md` and `BALANCE.md` give it.

### Progress

| Milestone | State |
|---|---|
| **M0 - Foundation** | **Complete.** The workspace, the four crates, the pinned RNG, the save envelope, the terminal shell, the dependency-direction check, CI, and the README. |
| **S1 - Ground Game** | **Mostly complete, on the old monthly model.** The engine, the domain, the performance model, the playable loop, the interface, and the golden fixture are built; 175 tests pass. The interface cannot yet *give an order*, and the tutorial and the first playtest are outstanding. See `ROADMAP.md` §3.7 for exactly what is missing. |
| **M1 - Continuous Time** | **Complete.** The clock, the scaled quantities, the host's clock, the scheduler, the advance loop, the draw anchors, the rates, the order model, the goodwill, the stable buckets, the save format and the version, the host loop, the top bar, the dilation-independence test, and the fork. The order palette — the interface's ability to give an order — is S1-37. See `ROADMAP.md` §4. |

> **The code is behind the design.** Everything in `crates/` still implements the
> monthly-turn model. It compiles and its tests pass, but it is the wrong shape, and
> Milestone M1 rebuilds it. The design documents are the authority.

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
- **Status** is stated at the top of each document: `Draft`, `Accepted`, or `Superseded`.
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
