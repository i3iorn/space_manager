# SIMULATION.md

> **How decisions become history.**
>
> This document specifies the engine: the clock, the scheduler, continuous
> integration, the determinism contract, the RNG design, the order model, and the
> event system. It is the contract between the player, the TUI, and the simulation.
>
> **Status:** Design intent, revision 2. Everything here is **[D]** settled unless marked
> **[?]**. Where a number is given it is a starting point for balance, not a promise.
>
> **Revision 2** replaced the monthly turn with continuous time. The old eleven-phase
> monthly tick is gone; the phases survive as the *order within an instant*, and a
> scheduler and continuous integration take its place.

---

## 1. Core principles

These are the rules the rest of the document exists to serve. If an implementation
conflicts with one of them, the implementation is wrong.

1. **The simulation never reads the UI.** Information flows one way: player → orders →
   sim → state → feed → UI.
2. **The simulation never reads wall-clock time, the filesystem, or system
   randomness.** Not during resolution, not anywhere.
3. **The player never mutates state.** They issue *orders*, which are validated and
   scheduled. An order never takes effect at the instant it is issued.
4. **The simulation is the only writer of state.**
5. **Every random draw comes from a named substream**, and **every random draw is
   anchored to a scheduled instant** — never to an integration sub-step (§3.4).
6. **The chronicle is the output, not a log.** A quiet moment still produces an entry;
   a catastrophic one still produces a readable history.
7. **No order is silently ignored.** Rejection is reported with a reason.
8. **The simulation is a function of simulated time, not of frames [D].** The time
   dilation the player has chosen **cannot change an outcome** (§6.3).

---

## 2. The clock

### 2.1 Instants and durations [D]

```rust
pub struct Instant(pub u64);    // simulated milliseconds since the campaign began
pub struct Duration(pub u64);   // simulated milliseconds
```

- **Integer milliseconds.** No floating-point time anywhere.
- A `Duration` is a plain difference of two `Instant`s.
- The simulation never converts a `Duration` into a real-time duration. It is told how
  much *simulated* time has passed and nothing else.

### 2.2 The civil calendar [D]

A calendar is **derived** from the instant, for display and for the periodic triggers
that still have a rhythm.

| Unit | Milliseconds |
|---|---|
| Second | 1,000 |
| Minute | 60,000 |
| Hour | 3,600,000 |
| Day | 86,400,000 |
| Month (30 days) | 2,592,000,000 |
| Year (12 months) | 31,104,000,000 |

A uniform calendar keeps the arithmetic trivial. `Month` and `Date` become **projections**
— `Month` is no longer a value the game stores, only one it prints.

### 2.3 Time dilation [D]

```rust
pub enum TimeScale { One, Five, Ten, Fifty, Hundred, FiveHundred, Thousand, FiveThousand, TenThousand }
```

`TimeScale::simulated_per_real_second()` returns the multiplier. **The dilation lives in
the host, not in the simulation.** The host converts real elapsed time into simulated
elapsed time and hands that to the simulation. The simulation has no idea a dilation
exists, which is exactly why §6.3 holds.

| Dilation | Simulated s per real s | A 90-minute launch takes | 30 years of campaign takes |
|---|---|---|---|
| 60x | 60 | 90 seconds | 146 days |
| 600x | 600 | 9 seconds | 14.6 days |
| 6,000x | 6,000 | 0.9 seconds | 29 hours |
| 60,000x | 60,000 | 0.09 seconds | 2.9 hours |
| 600,000x | 600,000 | 0.009 seconds | 17 minutes |

*(The last column is real time at that dilation. **The base rate is a minute a
second, and there is nothing slower** — a campaign is thirty to sixty years, and a
rate below that would be a lifetime rather than a game.)*

---

## 3. The engine

### 3.1 The scheduler [D]

Everything that happens at a definite time is on a **scheduler queue**.

```rust
pub struct ScheduledItem {
    pub at: Instant,
    pub priority: u8,     // lower resolves first, for items at the same instant
    pub seq: u64,         // monotonic; the final tie-break
    pub kind: ScheduledKind,
}
```

The queue is a `BTreeSet<ScheduledItem>` ordered by `(at, priority, seq)`. That tuple is
a **total order**, which is what makes the resolution sequence deterministic.

**The scheduler is part of the save [D].** A campaign saved with a launch in flight
must restore that launch, and the recurring work must resume exactly where it was. A
loaded campaign that re-scheduled from scratch would re-fire every month boundary that
had already happened, and the campaign would silently change. The save-and-load
determinism test is what catches that.

```rust
pub enum ScheduledKind {
    MonthBoundary,                  // the month's work, and the periodic triggers
    EventCheck,                     // a random event is drawn
    OrderLands { order: OrderId },  // an order takes effect
    MissionPhase { mission: MissionId, phase: MissionPhase },
    ProjectComplete { project: ResearchProjectId },  // a project reached its target
    CommitmentDue { commitment: CommitmentId },
    AttritionCheck { person: PersonId },
}
```

**The monthly work is a scheduled item [D].** The old eleven phases are now the
order of work at an instant, and the month boundary is what triggers them. The
periodic triggers — payroll, the fiscal year, the promotion round — are **inside** the
month's work rather than separate items, because they happen *at* a month boundary
rather than at instants of their own.

**A project completion is an item [D].** The progress is a rate and it crosses the
target inside the integration, where there is no random source — and the completion
draws. So the crossing schedules a `ProjectComplete` item, and the completion happens
when the item fires, at a definite instant.

**Everything random is a scheduled item [D].** An event check, a mission outcome, an
attrition check — each is placed on the queue at a definite instant, and the draw happens
when it is reached. **No random draw may occur inside an integration sub-step**, because
the number of sub-steps depends on the gap size, and the gap size depends on the
dilation. This rule is what makes §6.3 true.

### 3.2 Advancing [D]

```rust
pub fn advance_to(&mut self, target: Instant) -> Result<Vec<MonthReport>, SimError> {
    // The clock lands on the integration grid: the last whole quantum at or
    // before the target, plus whatever the last call did not use.
    let end = self.now + whole_quarters(self.now, target, &mut self.carry);
    while self.now < end {
        // Everything due at this instant, in the scheduler's total order.
        while let Some(item) = self.scheduler.peek() {
            if item.at > self.now { break; }
            self.apply_item(self.scheduler.pop())?;
        }
        // One quantum of continuous work.
        let step = (self.now + QUANTUM).min(end);
        let completed = rates::integrate(&mut self.state, step - self.now);
        self.now = step;
        // A project that reached its target cannot be completed here: the
        // completion draws, and a draw may only happen at a scheduled instant.
        for project in completed {
            self.scheduler.schedule(self.now, ScheduledKind::ProjectComplete { project });
        }
    }
}
```

**The items at an instant are applied before the rates for that instant.** The
month's work at a boundary therefore sees the world it woke up to, and an event
check at the same instant sees the month's work — which is what "events resolve last
within an instant" means in practice.

**The campaign never overshoots the target [D].** The simulation's clock lands on the
integration grid, and the fraction beyond it is carried. The target is therefore
measured from the campaign's **effective time** (`now + carry`), and the grid point
the campaign lands on is the last one at or before it. Without this, the carried
fraction could push a campaign past the frame the host asked for — and at the top rate a
frame is nearly three hours — and two campaigns at different dilations would end in
different months.

Three properties this preserves from the old tick:

- **The order at an instant is fixed.** When several items fall on the same instant, the
  priority and sequence decide, and the order is the same every time.
- **Events resolve last within an instant.** The priority field puts the event check
  after the work that instant's events react to.
- **The commit is atomic.** The month-boundary commit becomes an *instant* boundary
  commit: the changes made at one instant are applied together, and a failure part-way
  through leaves the instant unapplied.

### 3.3 The phase order, preserved [D]

The old eleven monthly phases become **the order of work at an instant**:

| Order | Phase | What it does |
|---|---|---|
| 1 | calendar | Fires the periodic triggers due now |
| 2 | intake | Validates the orders landing at this instant. Writes nothing |
| 3 | personnel | The assignments that land, and the attrition check |
| 4 | research | The projects that start, and the ones that finish |
| 5 | construction | Builds completing, facility output |
| 6 | missions | Mission phases and their outcomes |
| 7 | finance | Income, outgoings, the treasury, the crisis ladder |
| 8 | politics | Approval, promises falling due, public opinion |
| 9 | world | Rivals, the market, milestone primacy |
| 10 | commit | The atomic application of the instant's changes |

**The events phase is not in this list.** It is a *scheduled item* now
(`ScheduledKind::EventCheck`), and it resolves last within an instant so that it reacts
to the work that has just happened. See §5.2.

**A month boundary fires *before* the target [D].** The work for a month is therefore
dated at the month it covers, rather than the month after it. This is what the old
monthly model did, and the golden fixture depends on it.

**The scheduled item kinds and their priorities [D]:**

| Priority | Kind | What it is |
|---|---|---|
| 10 | `MonthBoundary` | The month's work, and the periodic triggers |
| 40 | `CommitmentDue` | A dated commitment falling due |
| 50 | `OrderLands` | An order taking effect |
| 60 | `MissionPhase` | A mission changing phase |
| 70 | `AttritionCheck` | A person's fortune being rolled |
| 90 | `EventCheck` | A random event being drawn |

The kinds that are not dispatched yet — `CommitmentDue`, `OrderLands`, `MissionPhase`,
and `AttritionCheck` — are carried and do nothing until the order, mission, and
commitment systems arrive.
### 3.4 Continuous integration [D]

The processes that used to step monthly are now **rates**:

| Process | Rate | Where |
|---|---|---|
| Project progress | Each team member's effective performance, in points per unit time | `sim::rates` |
| Construction progress | The build's rate, in points per unit time | `sim::rates` |
| Fatigue | A percentage per unit time, by assignment | `sim::rates` |
| Morale | Drift toward the target the assignment sets | `sim::rates` |
| Health | Slow recovery | `sim::rates` |
| Money | Continuous burn; the allocation arrives monthly | `sim::rates` and the finance phase |
| Facility condition | Slow degradation, offset by maintenance | still monthly |
| Approval, prestige, opinion | Slow drift | still monthly |

**The integration grid [D].** The rates are applied in **quanta of one hour**, on a
grid anchored at the start of the campaign. The grid is what makes dilation
independence hold.

A rate is often a function of the state — a project's progress depends on how tired
its team is — so the result of an integration depends on how finely it was done. If
the host's frame rate decided that, a campaign played at the base rate and the same campaign
played at the top rate would integrate differently and diverge in the third month, with
nothing on screen to explain it. **The grid is fixed, so the two campaigns integrate
identically.** The test that proves it is in `crates/sim/tests/determinism.rs`: the
same campaign, advanced in one-hour pieces and in one-week pieces, must produce the
same state hash.

**The clock lands on the grid [D].** The simulation's instant is always a whole
number of quanta from the start of the campaign. A host that asks for a span that is
almost never a whole number of hours gets the fraction **carried** until it adds up
to an hour — nothing is lost, and a player who stops for an hour does not come back
to a campaign that skipped an hour.

**The remainder is carried [D].** Every rate application carries the fraction of the
increment that did not fit in a whole scaled unit. Without it the truncation
accumulates: a rate of 0.008% an hour loses 0.133 units an hour, which is 136
percentage points of fatigue over a thirty-year campaign, and no test would fail.

The remainder is **part of the state** (`domain::scale::Remainders`), because it is
part of it: a save that dropped the remainders would produce a different campaign
from the one it was taken from. The arithmetic is in `domain::scale::accrue`, and it
is exact — summing the deltas telescopes, so the total differs from the true value
by less than one scaled unit however finely the campaign is integrated:

```text
sum(delta) = (sum(numerator x elapsed) - final remainder) / denominator
```

**The denominator must not change [D].** The carried remainder is only meaningful
against the denominator it was carried with. A rate whose denominator moves — a
build whose plan changes, say — invalidates its remainder.

**Fixed-point storage is required [D].** Fatigue accrues at about 0.008% an hour, and a
progress rate is a fraction of a point a month. Truncating either loses the rate
entirely, and the result would be that nothing ever moves at the base rate. So the continuous
quantities are stored in **scaled integers** - 256 units to the percent - and the
fractional remainder is carried beside them rather than in a separate accumulator
that could drift. This is a change to the domain types, and it is not optional.

**No random source in the rates [D].** `sim::rates` takes no `Rng`, and that is the
point. A draw inside an integration sub-step would make the campaign depend on the
number of sub-steps, and the number of sub-steps is the number of pieces the host
happened to use. There is no `Rng` in the module to draw from, so the type system
makes the mistake impossible rather than merely unlikely.

**A crossing cannot be completed inside the integration [D].** A research project
reaches its target *inside* the integration, where there is no random source — and
the completion draws. So the crossing schedules a `ProjectComplete` item, and the
completion — the knowledge, the breakthrough, the release of the team — happens when
that item fires, at a definite instant.

### 3.5 The old tick, and what is left of it

| Old | New |
|---|---|
| Eleven phases, once a month | **Ten** phases, once per instant that anything happens — the events phase is a scheduled item now |
| The month's changes, staged and committed | The instant's changes, staged and committed |
| `Month` as a stored value | `Month` as a printed projection of an `Instant` |
| A `MonthReport` modal | A live feed and a standing list of what needs attention |
| A forecast of the next six months | A timeline of the next few weeks |
| Fatigue as a monthly delta | Fatigue as a rate, integrated continuously |
| The budget year as a month index | The budget year as an instant on the calendar |

**What is still monthly, and why.** The allocation from the state, the report of
what the month cost, the crisis ladder, the promises falling due, the attrition
check, and the approval drift are all still monthly. They are **steps, not rates**:
they either happen or they do not, and there is nothing to integrate. The month
boundary is a scheduled item, so they happen at a definite instant and any draw
they make is anchored to one.

---

## 4. Orders

### 4.1 An order is a scheduled decision [D]

```rust
pub struct Order {
    pub id: OrderId,
    pub issued_at: Instant,      // when the player gave it
    pub effective_at: Instant,   // when it lands — issued_at + lead time
    pub kind: OrderKind,
    pub cost: Money,             // charged when it lands
    pub goodwill: i16,           // charged when it lands
}
```

The order lives in `CampaignState::pending_orders` until it lands, and **it is part
of the save**: a campaign saved with a hire in flight must restore that hire.

**Validation happens twice [D]:**

1. **When the order is issued.** The player must be told immediately that it is
   impossible, not three weeks later. A rejected order is reported with a reason and
   costs nothing. `Sim::issue` returns the reason.
2. **When the order lands.** The world has moved on. The project may have been
   cancelled, the person may have left, the vehicle may have been scrapped. An order
   that can no longer be honoured is **rejected at landing**, the cost is not charged,
   and the player is told why — in the report, and in the chronicle, which is the
   durable record.

**An order with no registered handler is rejected, never ignored [D].** The kinds
without handlers yet — the publication and facility-assignment machinery — are refused
at issue, with a reason. **The policy order has a handler**: it sets the agency's
standing shift instruction, and it is the first of the policy machinery to exist.

**A lead time may not be random [D].** An order that landed at a random time would
make the campaign depend on the draw, and the order log would not replay. The lead
times are in `sim::balance` and `sim::orders`, and they are a function of the
order's content: a hire's lead time scales with the seniority of the role, a
promotion waits for the next quarterly round, and a facility takes as long as its
plan says.

**The order lands when the clock reaches it [D].** The `OrderLands` item fires when
the simulation's instant reaches the order's `effective_at`, so the order is applied
in the state of the world at that instant — and a lead time changes that state, not
just the timing.

### 4.2 The order kinds [D]

| Kind | Lead time | Direct cost | Goodwill |
|---|---|---|---|
| `Hire` | 2-6 weeks, by seniority | One month's salary | 1 |
| `Fire` | Immediate | Three months' salary | 4 |
| `Assign` | 1-3 days | — | 2 |
| `Promote` | The next quarterly round | — | 3 |
| `Train` | 3-9 months | The course fee | 2 |
| `StartProject` | 1-2 weeks | — | 1 |
| `StartTestCampaign` | 1 week | — | 1 |
| `BuildFacility` | 6 months to 3 years | The capital cost | 5-15 |
| `AuthorizeLaunch` | The mission's flight time | The vehicle and the mission | 3 |
| `SetObjective` | Immediate | — | 1 |
| `SetBudget` | Immediate | — | 2 |
| `SetShift` | 1 hour | — | 0 |
| `PublishResult` | 1 week | — | 1 |
| `SetPolicy` | 1 week | — | 5 |

**The lead time is shown to the player before the order is confirmed**, and the cost and
goodwill are shown with it. An order the player cannot see the price of is not a
decision.

**How the ranges become numbers [D].** Where the order carries something to scale
by, it does: a hire's lead time is two weeks at the entry salary and six weeks at
the top of the band, a facility's is its own build plan's, and a promotion's is the
distance to the next quarterly round. Where there is nothing to scale by, the
**middle** of the range is used, and it is a constant in `sim::balance` — never a
draw.

### 4.3 Goodwill [D]

A single resource, from −100 to +100, on the dashboard. Every order that lands charges
it. The effects are in `MANAGEABLE_PARTS.md` §3.5; the short version is that high
goodwill buys speed, favours, and a deeper applicant pool, and low goodwill buys
administrative delay, higher costs, and an unhelpful state.

**Goodwill is the anti-spamming mechanism [D], and it is deliberately not a cap.**
There is no limit on how many orders the player may issue.

**The charge is the order's own goodwill, and it is spent when the order lands** —
with the cost, and not if the order is refused.

**The goodwill is a scaled integer** and it regenerates at three percent a month, up
to a cap of a hundred, so a director who spends it can earn it back but not quickly.
**The regeneration is a rate, and it carries a remainder** like every other: three
percent a month is 768 scaled units over thirty days, which is not a whole number of
units an hour.

**The effects are computed when the order is issued [D].** The lead time and the cost
are both read from the goodwill as it stands then, so the player sees the delay and
the price *before* they commit. The effect is zero between ±40 and grows to its
maximum at the extremes — half again as long at −100, half the time at +100, and a
third more or less of the price.

---

## 5. Events

### 5.1 Scheduled and random [D]

| Kind | How it enters the queue | Visible in the forecast? |
|---|---|---|
| **Scheduled** | Placed at a definite instant by an order, a mission, or a periodic trigger | **Yes** — always |
| **Random** | Drawn at an event check | **No** |

### 5.2 The event check [D]

The simulation schedules the next event check at `now + interval`. At the check, an
event is drawn from the context-weighted pool, and the next check is scheduled.

- The **base interval is one simulated week.**
- Most checks produce **nothing**. The agency is usually quiet.
- The interval **tightens under stress** — a thin treasury, a failing facility, a tired
  crew — so a run of bad luck becomes a run of bad luck.
- The check is a **scheduled item**, so the draw happens at a definite instant, and the
  draw sequence is independent of the dilation.

**The events phase runs outside the month's commit [D].** It is a scheduled item, so it
does not share the month's working copy with the commit phase that writes the
chronicle. **It therefore writes its own chronicle entries.** Without that, an event
would fire and never be recorded - which is a bug the implementation actually had, found
while building the scheduler.

### 5.3 Context weighting [D]

```
weight(event) = base_weight × Π modifiers(state)
```

The design rule: **bad conditions raise the odds of bad events, and good conditions
raise the odds of good ones.** This is what makes the simulation feel fair rather than
random. The player should be able to look back at a disaster and say *my fault — I was
running everyone at 80% fatigue and skipped testing*.

**A quiet stretch is a legitimate outcome [D].** If no event clears the weight threshold,
the check produces nothing and the next one is scheduled.

### 5.4 Cascades and follow-ups [D]

A follow-up is one of:

- **Immediate** — resolved at the same instant, after the parent.
- **Delayed** — scheduled for *n* later.
- **Conditional** — scheduled, but only fires if a predicate holds when it arrives.

**Scheduled follow-ups appear in the forecast [D].** Once a crisis is scheduled, the
player can see it coming. This is deliberate: the player should be able to watch a
consequence they set in motion arrive three weeks later and know they could have
prepared. It leaks no information about un-drawn randomness.

**Chain depth is bounded [D]** at three, so the resolution stack stays shallow and the
causal chain stays readable.

### 5.5 The feed [D]

**The month report is a summary, not the interface.** It is what the player sees
when a month boundary is crossed, and it is a prompt: it stays until it is
dismissed, and the chronicle keeps the rest. It is not the way the player reads
the campaign.

The month's report becomes:

- **The feed** — an append-only stream of what has happened, newest first, with
  severity. It is the same data as the chronicle, presented for reading.
- **What needs attention** — a standing, sorted list of the things that are overdue,
  failing, or about to fall due. This is the panel the player looks at constantly, and
  it is the real replacement for "end of month, here is your report".

*Not implemented yet.* The top bar carries the **count** of what needs attention
(`sim::derived::needs_attention`), and the report modal carries the month's
summary, but the live feed itself is Slice 1-37's successor.

---

## 6. Determinism

### 6.1 The rules (normative) [D]

1. No wall-clock time.
2. No system randomness.
3. No `HashMap` iteration order.
4. No dependence on memory addresses, allocation order, or pointer values.
5. **No floating-point arithmetic in decision-making.** Comparisons that can change an
   outcome use integers or fixed point. Floating point is presentation-only.
6. Money is **integer**, in a minor unit.
7. All state transitions are pure functions of `(state, orders, rng_streams)`.
8. File I/O happens at load and save only, never during resolution.
9. **No random draw inside an integration sub-step** (§3.1).

### 6.2 The replay

```
replay(seed, order_log, sim_version) -> Result<CampaignState, SimError>
```

The order log records, for every order the player gave: the **simulated instant it was
issued**, its kind, and its payload. Replaying re-issues the orders at those instants and
reproduces the campaign exactly.

**The log records simulated instants, not real ones [D].** The player took 90 minutes
of real time to decide something; that is irrelevant to the campaign. The instant at
which the decision was *made in the simulation* is what matters.

### 6.3 Dilation independence [D]

> **A campaign played at the base rate and the same campaign played at the top rate are the same
> campaign.**

This is a hard guarantee, and it is the property that makes the whole time model
trustworthy. It holds because:

1. The dilation lives in the host. The simulation is told a simulated duration and
   nothing else.
2. **The integration is on a fixed grid** (§3.4). The grid is one hour, anchored at
   the start of the campaign, and every campaign integrates over the same quanta in
   the same order. The host's frame rate decides *how many quanta each request
   covers*, and not one of them.
3. Every random draw is anchored to a **stable identifier** — the month index, the
   event check's index, an entity's id — and never to a sub-step or to an instant
   that the dilation could move (§7).
4. **The campaign never overshoots the host's target.** The simulation's clock lands
   on the grid, the fraction beyond it is carried, and the host asks from the
   campaign's *effective* time. Two campaigns at different dilations therefore reach
   the same grid point, and the state at a grid point is the same for both.

**The test is in `crates/ui/tests/dilation.rs` [D].** It runs the same campaign
through the **host's own path** — the real `App`, the real `Clock`, the real frame —
at each of the nine dilations, and asserts that the state hash and the chronicle are
identical. A second test checks that the nine dilations really do use very different
frame sizes, so the first is not vacuous.

**The residual risk [D]:** a rate that depends on the state is sampled at the start
of each quantum, and the *quantisation* of the grid is a behaviour-defining input.
Changing the quantum changes the path of every rate, and therefore the outcomes —
which is why it is in the version (§6.4) and not merely in the code.

### 6.4 The `sim_version`

A hash over everything that can change an outcome:

- the phase order in §3.3,
- the scheduler's ordering rules,
- every event definition,
- every order handler,
- the RNG algorithm and the stream-hash function,
- the integration quantum and the sub-step cap,
- the source version.

**The integration quantum is in the version [D].** Changing it changes the rates, and
therefore the outcomes. It is a behavioural constant and it must be hashed like one.

**The version is computed, not written down [D].** `sim::version::sim_version()` hashes
the phase order, the integration quantum, the RNG algorithms, the order kinds, and the
scheduled kinds. A version written down by hand is a version that stops being true —
the first change to a rate would leave it saying "the same version" while the outcomes
moved.

---

## 7. The RNG

### 7.1 Named sub-streams [D]

The whole campaign draws from **one root seed**, chosen at campaign start, shown to the
player, and stored in the save.

```
stream = PRNG(seed = expand(hash(root_seed, domain, bucket, discriminator)))
```

| Field | Meaning | Example |
|---|---|---|
| `domain` | Which system is drawing | `"events"`, `"mission"`, `"personnel"` |
| `bucket` | Which instance within that domain | The event-check index, the mission id, the person id |
| `discriminator` | Which draw within that instance | 0, 1, 2… |

**The bucket must be a stable identifier, never a time [D].** Using the instant as the
bucket would make the stream depend on *when* the draw happened, which is exactly what
must not vary. The event check uses its **index**, not its instant.

### 7.2 The pinned algorithms [D]

Two things are pinned in code rather than taken from a dependency, because Rust does not
guarantee their stability across releases:

- **The hash** is FNV-1a, written out. It is deliberately *not*
  `DefaultHasher`, whose algorithm Rust reserves the right to change.
- **The seed expansion** is SplitMix64, also written out. It is deliberately *not*
  `SeedableRng::seed_from_u64`, whose algorithm is not stability-guaranteed.

The generator is `ChaCha12Rng`, seeded through `from_seed`, which is part of the
specified construction.

### 7.3 Drawn values [D]

The sim draws **integers** and derives everything else. Probabilities are in basis
points, so a 14% risk is exactly 1,400 and the comparison is exact.

---

## 8. Save and load

A save carries:

| Field | Purpose |
|---|---|
| `sim_version` | The engine version |
| `root_seed` | The campaign seed |
| `now` | The current instant |
| `state` | The full `CampaignState` |
| `scheduler` | Every scheduled item still pending |
| `order_log` | Every order the player has given, with its issue instant |
| `order_seq`, `item_seq` | The monotonic counters |

**The scheduler is part of the save [D].** A campaign saved with three launches in
flight must restore those launches, their phases, and their completion instants.

**Autosaves** are written before each significant action and on quit, and **only while
the game is open** — simulated time does not advance while the game is closed.

---

## 9. Performance

| Operation | Budget |
|---|---|
| One instant, 300 staff, 10 projects | < 2 ms |
| A 30-year campaign, integrated at 1-hour quanta | < 5 s of computation in total |
| A fork at any instant | < 100 ms |

**The integration is the cost.** 30 years at a one-hour quantum is about 263,000
sub-steps, each touching every assigned person and every project — roughly 80 million
updates across the whole campaign, spread over hours of play. That is not a problem,
and it is the reason the design uses a one-hour quantum rather than a one-second one.

**The sub-step cap matters [D].** A long gap is integrated in at most 4,096 sub-steps
rather than refused, so that a player who leaves the game running for a week of real
time does not stall the frame.

---

## 10. Extension points

| Adding | Requires | Does **not** require |
|---|---|---|
| A new event | A new `EventDef` and a tag | Touching the scheduler or the tick |
| A new order kind | A handler and a lead time | Changing the phase order |
| A new periodic trigger | An entry in `PeriodicKind` | Changing the scheduler |
| A new continuous rate | An entry in the rate table | Changing the integration |
| A new mission phase | A phase in the state machine | Changing the scheduler |

**The test of a good extension point: the tick file should not need to be edited.** If it
does, the boundary is in the wrong place.

---

## 11. Testing the simulation

### 11.1 The dilation-independence test [D]

The test that the new time model exists to support:

```rust
#[test]
fn the_same_campaign_is_the_same_at_every_dilation() {
    let slow = play_at(1, FIXTURE);
    let fast = play_at(10_000, FIXTURE);
    assert_eq!(slow.state_hash(), fast.state_hash());
}
```

The fixture is a set of orders with fixed **simulated** issue instants. The two runs
differ only in how much simulated time each host frame delivers, and therefore only in
how finely the integration subdivides. **They must produce the same campaign.**

This is the test that catches a random draw inside an integration sub-step, and it is
the one that would catch it most painfully if it were left to a playtest.

### 11.2 The other tests

| Test | What it protects |
|---|---|
| **Determinism** | The same campaign twice is identical |
| **Golden** | A known campaign at a known instant produces a known state hash and a readable chronicle |
| **Property** | Invariants hold for generated states and generated order sequences |
| **Fuzz** | Random order sequences never panic and always resolve |
| **Performance** | The budgets in §9 |

The golden fixtures are now keyed by **instant** rather than by month, and the
chronicle is compared as readable text so that a failure shows the player *what changed
in the world*, not just a number.

---

## 12. Open questions

| # | Question | Why it matters | Blocks |
|---|---|---|---|
| 1 | What happens to a scheduled order whose target has changed by the time it lands? | The main failure mode of the new order model | Orders |
| 2 | Is the integration quantum of one hour right, or should it be finer? | A finer quantum is more accurate and more expensive; the accuracy matters most at the base rate | Engine |
| 3 | Should the player be able to cancel a scheduled order before it lands, and at what cost? | A real-time game needs an undo, and undo has to cost something | Orders |
| 4 | Does the game advance while a menu or an order palette is open? | It changes what "pausing" means | UI, engine |
| 5 | Should the event check interval be a constant, or should it scale with the pace of the campaign? | A 30-year campaign with weekly checks has thousands of them | Events |

---

## 13. Related documents

- `MANAGEABLE_PARTS.md` — the management surface this engine simulates, and §3 for the
  time model in full.
- `DOMAIN_MODEL.md` — the entities, the instants, the orders, and the fixed-point
  storage.
- `PERSISTENCE.md` — the save format, the scheduler, and the replay.
- `BALANCE.md` — the rates, the lead times, the goodwill costs, and the dilation.
- `UI.md` — the live interface, the clock, and the dilation control.
- `TESTING.md` — the wider test strategy.
- `README.md` — the documentation index.
