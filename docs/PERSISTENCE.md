

**The log is the campaign, and everything else is a cache of it.** That is the
property the whole feature rests on, and it is what makes a campaign verifiable,
forkable, and portable to another machine.# PERSISTENCE.md

> **Saving, loading, replaying, and branching.**
>
> **Status:** Design intent, revision 2. Implements the `sim_version` and order-log
> requirements from `SIMULATION.md` §6 and the replay and branching features from
> `MVP.md` §4.

---

## 1. What is saved

A save is **JSON**, and it contains six things:

| Field | Purpose |
|---|---|
| `sim_version` | The engine version |
| `root_seed` | The campaign seed |
| `now` | The current **instant** |
| `state` | The full `CampaignState` |
| `scheduler` | Every scheduled item still pending |
| `log` | Every order the player has given, with its issue instant |

**The order log is the source of truth for "what the player did".** The state is a
cache of it. This is what makes exact replay and forking possible, and it is why the
log records **simulated** issue instants rather than real ones: the player took ninety
minutes of real time to decide something, and that is irrelevant to the campaign.

**The scheduler is part of the save [D].** A campaign saved with three launches in
flight must restore those launches, their phases, and the instants at which they
complete. A save that dropped the pending work would silently change the campaign.
**This is not hypothetical:** the scheduler was left out of the save during M1, and the
save-and-load determinism test caught it immediately — a loaded campaign re-scheduled
the recurring work from scratch, which re-fired every month boundary that had already
happened.

**The orders in flight are part of the state [D].**
`CampaignState::pending_orders` holds the orders that have been issued and have not
landed, with their issue instants, their landing instants, their costs, and their
goodwill. **A campaign saved with a hire in flight must restore that hire** — and the
scheduler holds the timing of when it lands, so the two must agree. A save that
dropped the orders would leave the agency quietly not doing what the player asked
for.

**The remainders are part of the state [D].** `domain::scale::Remainders` lives on
the person, the project, the treasury, and the construction, and it is inside
`state` — so it is saved with everything else. It is the fractional remainder of each
rate, carried between applications, and it is what makes a rate exact over a long
campaign. **A save that dropped it would produce a different campaign from the one it
was taken from**, in a way that would take years of play to notice.

---

## 2. Save file structure

```json
{
  "format": 10,
  "sim_version": "a3f1c9e2",
  "saved_at": { "day": 87, "month": 3, "year": 2041 },
  "root_seed": 6125084923715548,
  "now": 28537200000,
  "state": { "...": "a CampaignState" },
  "scenario": { "id": "first_year", "...": "a Scenario" },
  "scheduler": [
    { "at": 28800000000, "priority": 90, "seq": 41, "kind": { "EventCheck": {} } },
    { "at": 29000000000, "priority": 50, "seq": 42, "kind": { "OrderLands": { "order": 7 } } }
  ],
  "log": [
    { "issued_at": 0, "seq": 1, "kind": "Hire", "payload": { "role": "Scientist", "salary": 120000 } }
  ]
}
```

| Field | Notes |
|---|---|
| `format` | The **save format** version — how the file itself is laid out. Independent of `sim_version` |
| `sim_version` | The **engine** version — whether the outcomes are reproducible |
| `saved_at` | For the save-slot list. Display only, and **outside the hashed state** |
| `now` | The current instant, in simulated milliseconds |
| `state` | The whole campaign state, including the orders in flight |
| `scenario` | The scenario the campaign started from — a campaign has to be forkable after it is loaded |
| `scheduler` | Every pending item, in the queue's order |
| `log` | Every order, in issue order |

**`format` and `sim_version` are different things [D]** and both are needed. `format`
changes when the file layout changes. `sim_version` changes when the *behaviour*
changes. A save can be readable by a newer engine and still be un-replayable.

---

## 3. The order log

```rust
pub struct LoggedOrder {
    pub issued_at: Instant,   // the simulated instant the player gave it
    pub seq: u64,             // monotonic; the order of orders is never ambiguous
    pub kind: String,         // the order kind, as a stable name
    pub payload: serde_json::Value,
}
```

**The log records intent, not effect [D]:** it stores the order the player issued, not
what it did. A standing order — assigning somebody to a project — appears **once**, at
the instant it was issued, and is re-evaluated internally every moment.

**Orders that were refused are not logged [D].** They had no effect, so replaying them
would be pointless. They appear in the order palette's history, which is not part of the
log.

### Why the log is worth its weight

| Capability | Cost |
|---|---|
| **Exact replay** | The log plus the seed reproduces the campaign byte-for-byte |
| **Branching** | Fork at any instant, re-simulate from there, and compare |
| **Bug reports** | A save plus its log is a complete reproduction case |
| **Balance analysis** | The harness can replay many campaigns and compare outcomes |
| **Undo-by-forking** | "What if I had not cancelled the Orbital Launch?" is a question the player can answer |

**Branching is the reason to do all of this.** In a real-time management game the
player's dominant emotion is regret about a decision they cannot take back, and the
decisions now happen in seconds rather than months. Branching converts that regret into
curiosity, and it is far cheaper on a log than on a save-state diff.

---

## 4. `sim_version`

A hash over everything that can change an outcome:

- the phase order in `SIMULATION.md` §3.3,
- the scheduler's ordering rules,
- every event definition,
- every order handler,
- the RNG algorithm and the stream-hash function,
- **the integration quantum and the sub-step cap**,
- the source version.

**The integration quantum is in the version [D].** Changing it changes the rates, and
therefore the outcomes. It is a behavioural constant and it must be hashed like one.

**The version is computed, not written down [D].** `sim::version::sim_version()` hashes
the phase order, the integration quantum, the RNG algorithms, the order kinds, and the
scheduled kinds. It must be automatic; a hand-maintained version constant will be
forgotten the first time an event changes, and the failure mode is silent.

**The save format version is separate, and it is about the file.** The format was 1
for the original layout, 2 when the scheduler joined the save, 3 when the orders
in flight joined the state, and it is now 10. **A file of any other format is
refused** rather than half-read — the older ones included, because they are the
dangerous ones.

**The last two of those are the interesting ones.** A change to the *shape* of
the state needs a new format, because the old bytes would otherwise be read as
the new thing and mean something else. The knowledge is the case in point: the
per-capability and per-discipline maps have the same JSON, so a file of the
old format would have loaded without complaint and every threshold would have
been checked against the wrong number. **The shift is the other one:** it went
from a number to an override of the agency's policy, and a shift change went
from a number to "this person, or nobody".

---

## 5. Loading

Loading validates in this order, and stops at the first failure with an actionable
message:

1. **The file parses as JSON.** Otherwise: *"This is not a save file."*
2. **`format` is the current one.** Otherwise: *"This save was made by a newer version of the
   game"*, or — for an older file — *"It was made by an older build of the game, and it cannot
   be read safely."*
3. **`sim_version` matches.** Warn if it does not, but continue.
4. **The state validates against the invariant catalogue.** Otherwise: *"This save is
   damaged (…, …)"* and offer to load the last autosave.
5. **The log parses into typed orders.** Otherwise: *"This save contains a decision the
   current version does not understand."*

**A damaged save must never crash the game [D].** The loader returns an error, the UI
shows it in plain language, and the player keeps whatever they have.

### Autosaves

**There is one, and the game writes it by itself.** The design had three — one
before each significant action, one on quit, and numbered slots for the player
— and the shipped game has the first two merged into one:

| When | Where | Built |
|---|---|---|
| At every month boundary | The campaign file | **Yes** |
| When the player quits | The campaign file | **Yes** |
| Before each significant action | `<saves>/autosave.json` | No — the month boundary is the checkpoint |
| Manually, into a slot | `<saves>/slot-<n>.json` | No |

**The campaign file is the game's own.** It is written at every month boundary,
so the worst a player can lose by closing the terminal is a month, and it is
written again when the player quits, so the worst they can lose by quitting is
nothing. `F3` writes it by hand and `F4` reads it back.

**The automatic save is quiet.** A save that says something every month is a
message the player learns to ignore, and the one time it matters — the save
that failed — is the one that would be lost in the noise. A failure is worth
saying out loud, though, because a save that is not happening is worse than
one that is.

**It is throttled.** At the top dilation a month boundary arrives several times
a second, and writing the whole campaign each time is a lot of disk for no
gain. The game waits ten seconds of real time between automatic saves, which at
the base rate is never and at the top rate is a file every ten seconds.

**The save slots are still on the list.** A player who wants to keep an old
campaign has nowhere to put it, and the design's numbered slots are the answer.

**Simulated time only advances while the game is open [D].** Closing the game is a
pause that costs nothing, and it is the only "skip" in a game that has no skip.

---

## 6. Scenario files

A scenario is the starting point of a campaign: the initial state, the seed, and
optionally a scripted opening.

```json
{
  "format": 10,
  "id": "first_orbit",
  "name": "First Orbit",
  "description": "Five years from founding to orbit.",
  "seed": 6125084923715548,
  "start_state": { "...": "a partial CampaignState" },
  "opening_orders": [
    { "issued_at": 0, "kind": "Hire", "payload": { "role": "Scientist", "salary": 120000 } }
  ]
}
```

**Scenarios are data, in the same format as saves [D].** A scenario is a save with a
name and a log, so the loader has one code path and the editor needs no special
support.

**The shipped scenarios [D]:**

| Id | Purpose |
|---|---|
| `ground_game` | The tutorial scenario. A small agency, three scientists, and a budget that will not last |
| `first_orbit` | The main scenario. A five-year arc with real decisions, tuned so a competent player reaches orbit |
| `sandbox` | No political deadline, a deep treasury, and a fast clock. For experimenting with a strategy before committing to it |

**`sandbox` is not a luxury [D]:** it is the scenario that makes the branching feature
usable, and it is the one a player will use to test a strategy before committing to it.

---

## 7. Replay and branching

### Replay

```
Sim::replay(&self) -> Result<Sim, SimError>
```

The replay re-issues the logged orders at their logged **simulated issue instants** and
reproduces the campaign exactly. **The golden tests in `TESTING.md` rely on exactly this
property**, and so does the fork: the branch is the same campaign up to the fork
point, and every difference after it is a difference the player caused.

**The log is the campaign, and everything else is a cache of it.** That is the
property the whole feature rests on, and it is what makes a campaign verifiable,
forkable, and portable to another machine.
### Branching

```
fork(campaign, at_instant) -> Result<CampaignState, SimError>
```

Implemented by replaying the log up to `at_instant`, taking a snapshot of the state,
and then continuing with a **new log** from that point. The original log is preserved.

**Forking a campaign must be cheap [D]:** under 100 ms for a fork at any point in a
forty-year campaign. This depends entirely on the campaign being fast to simulate, which
is why the performance test is not optional.

**The comparison view [D]:** two branches, side by side, showing the same instants and
where they diverge — treasury, approval, programme status, and the chronicle entries
that differ. The player should be able to see not just *that* the branches differ but
*when* and *why*.

---

## 8. What is not saved

| Not saved | Why |
|---|---|
| Derived values | They are recomputed. Saving them creates two sources of truth that will drift |
| The RNG's internal state | The stream is derived from `(root_seed, domain, bucket, discriminator)`, so it is reconstructible from the seed |
| Caches | Dirty-flagged and recomputed |
| The UI's view state | It is rebuilt on load |
| Any wall-clock timestamp | A save may carry a "saved at" field, but it is display-only and must be outside the hashed state |

**A save must not carry a timestamp into the state [D].** If it did, loading and
re-saving would change the state hash, and every golden test would fail for a reason
nobody could explain.

---

## 9. Directory layout

```
<the platform's configuration directory>/
└─ space_manager/
   └─ campaign.json      the file --save named, or this one by default
```

**What the design intends:**

```
<saves>/
├─ autosave.json
├─ quitsave.json
├─ slot-1.json
└─ slot-2.json

<scenarios>/            installed with the game
├─ first_year.json
├─ first_orbit.json
└─ sandbox.json
```

The locations come from the platform's standard config directory, not the working
directory — a game that writes into its own install directory is a game that breaks the
moment it is installed read-only. **The shipped game writes there now**:
`%APPDATA%\space_manager` on Windows, `~/Library/Application Support/space_manager`
on macOS, and `$XDG_CONFIG_HOME/space_manager` — or `~/.config/space_manager` —
on everything else. **The numbered slots are still on the list**, and they would go
in the same directory.

---

## 10. Open questions

**Four of the five have been answered, and the fifth is the save slots.**

| # | Question | The answer | Where it is written |
|---|---|---|---|
| 1 | Curated `sim_version` inputs versus the whole source tree | **Curated.** The version hashes the phase order, the integration quantum, the random algorithms, the order kinds, and the scheduled kinds — and nothing else, so a comment does not invalidate a replay | `sim::version` |
| 2 | Typed order payloads versus raw JSON in the log | **Both, and deliberately.** The log holds the kind's name and the kind's JSON, and the replay deserialises the JSON back into a typed `OrderKind`. The name is there so the file is readable; the typed order is the source of truth | `sim::Sim::replay` |
| 3 | Do old saves migrate forward, or are they refused? | **Refused.** A migration that silently changes outcomes is worse than none, so a save of any other format is refused with a reason: *"It was made by an older build of the game, and it cannot be read safely."* | §5 |
| 4 | Is the autosave per-campaign or global? | **Neither, because there is one campaign file and it is per-campaign.** The game writes `campaign.json` in the platform's configuration directory at every month boundary and again when the player quits | §5 and §9 |
| 5 | How much of the scheduler is written on every save? | **All of it.** The scheduler is part of the save, because a loaded campaign must restore its pending work rather than re-scheduling it — otherwise every month boundary that had already happened fires again | `SIMULATION.md` §8 |

**And one more that the design has not asked yet:** **where do the save slots
go, and how does a player keep an old campaign?** The answer is the same
directory, and the question is still open.

---

## 11. Related documents

- `SIMULATION.md` — the determinism contract this format exists to serve.
- `DOMAIN_MODEL.md` — the `CampaignState` that is serialised, and the `Order` type.
- `TESTING.md` — the golden, determinism, dilation-independence, and replay tests.
- `BALANCE.md` — the numbers that end up in a save.
- `ROADMAP.md` — the tasks that build this.
