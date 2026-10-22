# DOMAIN_MODEL.md

> **The entities, their fields, and the rules that keep them coherent.**
>
> This is the contract for `crates/domain`. It defines *what exists*; `SIMULATION.md`
> defines *what happens to it*.
>
> **Status:** Design intent, revision 2. Field lists are normative for the MVP scope
> (`MVP.md` §2). Revision 2 covers the continuous time model: instants replace months,
> and the continuous quantities are scaled.

---

## 1. Principles

1. **Base state is stored; derived state is computed.** If a number can be calculated
   from other state, it does not live in the state. See §19.
2. **Everything that can be an integer is an integer.** Money, scores, capacities,
   percentages, and time are all integers. Floating point appears only in the UI, for
   display, and only after the value has been decided.
3. **Ids are monotonic `u64` counters**, never random, never reused. They are stable
   across save/load and reproducible from a replay.
4. **The invariant catalogue (§18) is enforced**, not documented. A `domain` type that
   can be constructed in an invalid state is a bug.
5. **A type's fields are the whole truth.** There is no hidden state in a struct, and no
   behaviour in `domain` beyond validation.
6. **The state holds no date.** The clock lives in the simulation, and the calendar is
   a projection of it. See §2.

---

## 2. Time

### 2.1 The clock

Simulated time is an **integer number of milliseconds since the campaign began**.
There is no floating-point time, and the simulation never reads the wall clock — it is
told how much simulated time has passed.

```rust
pub struct Instant(pub u64);     // simulated milliseconds since the campaign began
pub struct Duration(pub u64);    // simulated milliseconds
```

- `Instant` is the **canonical** time value. Every date field in the game is an
  `Instant` — a commitment's target, a milestone's date, a mission's launch window, a
  promise's making, a chronicle entry's date.
- `Duration` is a plain difference of two `Instant`s.
- **There is no wall-clock time anywhere.** An `Instant` is never read from the system
  clock.

**The current instant lives in the simulation, not in the state [D].** The state holds
no date at all, and there is no `next_month` field: two copies of the date would be
two sources of truth, and they would drift.

### 2.2 The civil calendar

The calendar is **derived**, for display and for the periodic triggers that still have a
rhythm.

```rust
pub struct Date { pub year: i32, pub month: u32, pub day: u32, pub hour: u32, pub minute: u32 }

pub struct Calendar { pub year_of_origin: i32 }
```

| Unit | Milliseconds |
|---|---|
| Second | 1,000 |
| Minute | 60,000 |
| Hour | 3,600,000 |
| Day | 86,400,000 |
| Month (30 days) | 2,592,000,000 |
| Year (12 months) | 31,104,000,000 |

- A `Date` is a **display projection** of an `Instant`, computed on demand. It is never
  stored.
- Fiscal years are whole calendar years from the year the agency was founded.
- **Why a uniform calendar [D]:** a real calendar drags in leap years, time zones, and
  epoch assumptions for no benefit, and a uniform one makes "the fiscal year" and "the
  promotion round" trivial to schedule.

### 2.3 Time dilation

```rust
pub enum TimeScale { One, Five, Ten, Fifty, Hundred, FiveHundred, Thousand, FiveThousand, TenThousand }
```

`TimeScale` is a **host-side control, not domain state**. The simulation is never told
which one is in force; the host converts real elapsed time into simulated elapsed time
and hands that over. That is what makes dilation independence hold — a campaign played
at the base rate and at the top rate is the same campaign.

`TimeScale::ALL` holds the nine dilations in order, `from_key` maps the number keys
1-9, and `faster()` / `slower()` step between them.

### 2.4 Scaled quantities

Continuous quantities are stored in **scaled integers**, at 256 units to the percent.

```rust
pub const PERCENT_UNITS: i32 = 256;
```

| Quantity | Stored as | Range |
|---|---|---|
| Fatigue, health | `u16` | 0 ..= 25,600 (0-100%) |
| Morale, loyalty, fame | `i16` | -25,600 ..= 25,600 (-100..+100%) |
| Progress | `u32` | 0 ..= 2^32 (256 units per point) |
| Money | `i64` minor units | unscaled - money is exact already |
| Attributes | `u8` | 0-100 - a *characteristic*, not a continuous state |

**This is not optional [D].** Fatigue accrues at about 0.008% an hour and a progress
rate is a fraction of a point a month. Truncating either to a whole percent loses the
rate entirely, and the visible result is that **nothing ever moves at the base rate**.

**Ratios are not scaled.** The modifier scale (`-100..=+100`) and the effective
performance (basis points) are ratios, not quantities, and they meet the scaled
quantities at the point where a modifier is derived: a scaled morale of 20% becomes a
morale modifier of +10.

**Two rules follow [D]:**

1. **Every threshold that compares against a scaled quantity is itself scaled.** A
   check like `morale <= -30` written against a scaled morale is always true, and the
   symptom is a campaign that eats its own staff for no visible reason.
2. **Any rate applied over a fraction of its period must carry a remainder.** Applying
   `rate x elapsed / period` and truncating loses the rate over many applications -
   136 percentage points of fatigue over a 30-year campaign, and no test would fail.
   See `ROADMAP.md` §4.1.1.

**The remainder is part of the state [D].** `domain::scale::Remainders` holds it, and
it sits on the person, the project, the treasury, and the construction:

```rust
pub struct Remainders {
    pub fatigue: i64,
    pub health: i64,
    pub morale: i64,
    pub progress: i64,
    pub construction: i64,
    pub money: i64,
}
```

It is in the state, and not in the engine, because **it is part of the campaign**: a
save that dropped the remainders would produce a different campaign from the one it
was taken from. The arithmetic is `domain::scale::accrue`, and it telescopes — the
total is exact to within one scaled unit however finely the campaign is integrated.
**The denominator of a rate must not change**, because the carried remainder is only
meaningful against the denominator it was carried with.

The conversions live in `domain::scale`, and the state types expose
`fatigue_percent()`, `health_percent()`, `morale_percent()`, and `percent()` so that
the scale never leaks into the interface.

---

## 3. Money

```rust
pub struct Money(pub i64);   // minor units; 1 credit = 100 units
```

- **Integer only.** Never `f64`, never `f32`.
- Every amount in the game is a `Money`. A bare `i64` in a money position is a bug.
- Arithmetic goes through explicit methods so rounding is visible:
  `Money::multiply_fraction`, `Money::split`, `Money::percent`.
- **Splitting conserves every minor unit.** The remainder is distributed one unit at a
  time to the earliest shares, and a negative amount splits exactly as a positive one
  does, with the remainder keeping the sign of the amount.

---

## 4. The Agency

```rust
pub struct Agency {
    pub id: AgencyId,
    pub name: String,
    pub focus: String,
    pub founded: i32,
}
```

`founded` is the year the agency was founded, and it is what the calendar is
projected onto. The agency is the only thing that knows when the campaign began.

---

## 5. People

**Scale [D]:** 150-300 staff. The player knows the famous ones intimately and the
rest through aggregate screens, filters, and reports.

### 5.1 Roles

Scientists, Engineers, Astronauts, Administrators, Managers — plus Pilots/Range and
Controllers, and later commercial/industrial staff. Each role has a distinct career
ladder, a distinct pay band, and a distinct failure mode.

### 5.2 The person

| Field | Notes |
|---|---|
| Identity | Name, origin, species, homeworld, age, family ties |
**The names are drawn from ten cultures [D], and the two halves of a name
belong together.** A recruit draws a culture and then a given name and a family
name from it, so "Kwame Adeyemi" reads as a person and "Kwame Kowalski" does
not. **A recruit never gets a family name somebody in the agency already has** —
two people called Okonjo are two people the player cannot tell apart on the
staff screen. The catalogue is code rather than saved state, so a name can be
reworded without invalidating anybody's save.
| Career | Rank, grade, years of service, education, certifications |
| Attributes | A small set — Technical, Creative, Leadership, Physical, Diplomatic — plus role-specific specialisms (propulsion, structures, avionics, life support, geology, medicine…) |
| Traits | 1-3 personality hooks: *frail but brilliant*, *won't accept a bad plan*, *needs the credit*, *loyal to the old regime* |
| State | Morale, loyalty, fatigue, health, fame, stress — all scaled |
| **Shift** | **How hard they work, as a scaled percentage of a full load: 0 to 200** |
| Relationships | Mentor, rivals, friends, spouse/partner, faction clique, political sponsor |
| Assignment | Division, programme, project, facility, test campaign, or "unassigned" (a cost, not a neutral state) |
| History | Past projects, successes, failures, incidents, awards, disciplinary notes |
| Contract | Salary, term, demands (a senior scientist may demand a programme of their own) |
| Remainders | The fractional remainders carried between rate applications — see §2.4 |

**Only one assignment at a time [D].** Enforced as an invariant, because a rejected
order has to be able to say *why* it was rejected, and that reason can only be
produced if the state is unambiguous.

**An assignment is a move [D].** A person has one assignment, and giving them a
new one takes them off whatever they were on — the project, the programme, the
facility. The design's own order list calls the order "assign / reassign", and
the phase that applies it says "moved from the propulsion laboratory to the
combustion study". **The one thing refused is a no-op**: putting somebody where
they already are is not a move, and the order says so.

**Any number of people can be on one team [D].** The points are summed, so a
bigger team finishes sooner and the target never moves — and that is the whole
of the research model.

**The traits are part of the person, and they are applied [D].** A trait is a
small permanent difference in how somebody works, and it lands on the factor it
describes: a team player raises the support factor, a perfectionist raises the
tools factor, a restless mind raises creativity and costs morale. **There are
fourteen of them, everybody has two or three, and the catalogue is code rather
than saved state** - a campaign stores trait *identifiers*, never the
catalogue, so a trait can be reworded without invalidating anybody's save.
The one factor a trait cannot improve is fatigue, which is the only factor that
can only hurt.
**Being assigned to a project and being on its team are different things [D].** The
assignment is what the person *is*; the team list is what the project draws on. The
two must be kept in step, and the sim updates both together.

**The shift is part of the state, and it is scaled [D].** It is a percentage of a
full load, so it is `PERCENT_UNITS` scaled units per percent, and a full load is
`FULL_PERCENT` — a hundred of them. **The shift scales three things**: the person's
contribution to whatever they are on, the fatigue they accrue, and the morale target
they drift towards. It is set by an order (`SetShift`), it lands at the next hour,
and it costs nothing. See `MANAGEABLE_PARTS.md` §5.2.1.

**A person's shift is an override, and `None` means the policy decides [D].** The
agency has one standing shift instruction — `CampaignState::shift_policy`, a
`Policy::Shift { shift, only_over }` — and a person's `shift` is `None` when the
policy decides and a value when the manager has decided better. `only_over` is the
fatigue threshold the policy covers: `None` covers everybody, and a threshold covers
only the people who are tireder than it. **The effective shift is a pure function of
the state**, so the campaign is still deterministic and still replayable. **A shift
order with `None` puts somebody back on the policy**, which is the other half of the
override. See `MANAGEABLE_PARTS.md` §5.2.1 and `ROADMAP.md` S1-48.

**The policy's threshold is a latch and not a comparison [D].** `PersonState` carries
a bit, `policy_eased`, and it is the whole of the thermostat: the agency eases a
person when their fatigue reaches the threshold, and does not release them until
their fatigue has come down by `SHIFT_POLICY_BAND_PERCENT`. **A comparison of
fatigue against a limit is not enough**, because a person whose fatigue settles
exactly on the limit crosses it every few hours — their shift flips on and off, the
output flickers, and the agency gets half of neither. The bit is updated once an
hour, where the fatigue moves, and it is cleared when the person has an individual
override or the policy is withdrawn.

**The attrition risk is derived, not stored [D].** The chance a person leaves this
month is computed from their morale, their loyalty, and whether they are at their
limit, and it is the same number the simulation rolls against — so the interface can
show it and sort by it without a second copy to drift.

### 5.3 Performance model [? — options below]

Performance on a task is **derived**, never set directly. The player manipulates the
*inputs* (who, where, with what, how hard) and reads the *outputs* (progress, quality,
confidence). This is what makes the People pillar a game rather than a spreadsheet.

The formula is the one decision that shapes the entire staffing layer, and it is still
open. Four candidates, with their real costs:

---

#### Option A — Multiplicative cascade

```
effective = skill × role_fit × morale × (1 - fatigue) × tool_quality
            × team_cohesion × (1 - overwork_penalty) × noise
```

Every factor is a multiplier, so a single factor near zero annihilates the team.

- **For:** one line of maths; every factor is obviously important; tuning is easy.
- **Against:** **cliff behaviour**. A team at 4% fatigue is at 96% output, but a team at
  100% fatigue is at 0% — and the player will learn to binary-search the cliff rather than
  manage a spectrum. Small multiplicative penalties compound into nonsense (six factors
  at 0.9 each is 53% output, which feels arbitrary).

#### Option B — Additive competency with thresholds (Football Manager style)

Each task declares a required competency and a contribution window. A staff member
contributes:

```
contribution = base_skill
             + morale_modifier        (-1 .. +1)
             + tool_modifier           (-1 .. +1)
             + support_modifier        (-1 .. +1)   team cohesion, lead, tools
             + fatigue_modifier        (-1 .. +1)
             + motivation_modifier     (-1 .. +1)   personal stakes, interest, prestige
```

The task progresses when the **sum** crosses a threshold.

- **For:** **maximally explainable** — "Marsha has 62 propulsion against the 55 this
  needs, but her morale is -1 and her fatigue is -1, so the team lands at 60: marginal".
  Scales cleanly to 300 people. Each modifier is a **separate lever**.
- **Against:** the threshold is a cliff of a different kind. Needs per-task tuning, and
  forty task types means forty threshold sets.

#### Option C — Diminishing-returns dimensional

One continuous performance number per person per activity, built from clamped factors
with diminishing returns:

```
skill_fit   = attributes × role_fit
drive       = skill_fit × motivation
condition   = drive × (1 - fatigue) × health
effective   = condition × support × tools × creativity
```

with every factor **floored above zero** and **saturating** toward its maximum.

- **For:** **no cliffs and no degenerate stacking** — a saturated factor stops helping,
  so the player is forced to spread effort across several levers. Smooth, so the UI can
  show a clear "this team is at 62%, held back mainly by fatigue and tool quality".
- **Against:** the least self-explanatory of the four, and it **demands a good UI** to
  stay legible. Also needs the saturation curves designed carefully or everything
  converges to the average.

#### Option D — Full individual event simulation

Each staff member takes their own actions each month, interacting with their colleagues,
their projects, and their circumstances.

- **For:** maximum emergence. Relationships acquire genuine causal power.
- **Against:** the most expensive option by a wide margin, and at 150-300 staff it
  generates noise faster than meaning. Balance is close to intractable, and it makes
  the "why did that fail?" question unanswerable.

---

**Recommendation: Option C**, with Option B's *modifiers surfaced in the UI* as an
explanation layer — the underlying math is C's smooth dimensional model, but the
screen shows it decomposed the way B does ("held back mainly by fatigue and tool
quality"). That combination gets C's lack of degenerate strategies and B's legibility,
and it concentrates the hard part (good UI) where the hard part belongs.

The structural point that applies to all four: **the model must never let the player
compute the answer by inspection.** A staffing decision should always require a
judgement about people under pressure, not a sum the player can do in their head.

### 5.4 Lifecycle

Applicants → screening → offer → onboarding → assignment → development → promotion or
exit. Exits are varied and consequential: **resignation** (overwork, low morale, better
offer), **retirement**, **injury**, **illness**, **death**, **scandal**, **poached by a
rival agency**, or **promotion out of the agency into the state apparatus**.

**Hiring takes 2-6 weeks and the applicant is not yours until they arrive.** The player
can offer someone in March and have them start in May, and the person they want may
have taken another offer in the meantime. A hiring spree is therefore a race, and it
costs goodwill for every order in it.

### 5.5 Development

Training courses, mentoring pairs, research sabbaticals, rotational postings, special
assignments, and stretch duties. Each costs money and a slot on someone's calendar, and
each has a payback time. **A course takes 3-9 months**, so development is a decision
whose payoff is years away and whose cost is immediate.

### 5.6 Astronauts

Astronauts carry the public relationship of the agency and carry disproportionate
risk. They accumulate flight hours, EVA hours, injury records, and fame. They age out.
They become national heroes, or national liabilities, or both.

### 5.7 Relationships

A relationship graph over staff and politicians. It is the raw material for emergent
history: two rivals on the same team, a mentor whose student is now the rival, a
relationship that keeps a division from self-destructing, a friendship that turns into
a cover-up. Relationships should be **first-class state**, not flavour text.

---

## 6. Programmes

**Programmes are the strategic layer** — the unit the state funds, the public
understands, and the player is judged on.

### 6.1 Anatomy of a programme

- **Title, category, sponsor** (the political patron funding it), **status**.
- **Status [D]:** Proposed → Funded → Active → (Paused → Resumed) → Completed | Failed |
  Cancelled.
- **Budget envelope** and a burn rate, with a stated ceiling that overruns breach.
- **Staffing plan** — named leads, assigned teams, vacancies.
- **Objectives** — primary (mission-critical) and secondary (bonus), each with a
  **success criterion** and a **deadline**.
  **What finishes an objective** [D] is either the programme's own staff
  finishing the work, or a successful mission to the destination the objective names.
  The primary objective of a programme that is about reaching somewhere is finished by
  reaching it: the orbital programme's objective is "reach orbit", and the only
  thing that reaches orbit is a launch that works. The staff's work still advances
  the objective's progress - that is how the player sees the work going on - but the
  objective is not complete until the thing it names has happened.
- **Milestones** — dated checkpoints the player sets and can move.
- **Risk register** — technical, personnel, schedule, political, budgetary. Risk is
  **visible, priced, and reducible** for money, time, or redundancy.
- **Visibility** — how much the public and the state are watching.

### 6.2 Programme categories

Orbital Exploration, Lunar Landing, Deep Space Probe, Earth Observation, Military
Reconnaissance, Commercial Launch, Space Station, Resource Extraction, Interplanetary
Transport, Megastructure, Interstellar.

### 6.3 Failure is content

A failed programme must have an **aftermath**: a budget hole, a morale crash, an
investigation, a cancelled follow-on, a political enemy. Failure is not a loss
screen; it is a branch in the story.

**A missed primary objective fails the programme [D].** The aftermath starts with
the programme being marked as failed, because a deadline that is missed is a
number that goes down and nothing else, and a player who cannot see that the
work is over cannot feel it. The chronicle entry names the programme rather
than the objective, because a record the player has to decode is not a record.

investigation, a cancelled follow-on, a political enemy. Failure is not a loss
screen; it is a branch in the story.

---

## 7. Missions & Vehicles

**Hardware depth [D]:** archetype plus a handful of **subsystem sliders**. No component
design, no trajectories, no fuel physics. The player chooses the trade-offs; the sim
derives performance, cost, and risk.

### 7.1 Subsystem sliders

| Slider | Trade-off |
|---|---|
| Propulsion | Thrust vs. efficiency (and exotic tech requirements) |
| Fuel | Abundant/cheap vs. high-performance/exotic |
| Structure | Mass vs. durability |
| Power | Batteries vs. fuel cells vs. radioisotope |
| Payload | What you carry vs. what you can afford to build |
| Life Support | Duration vs. complexity vs. mass |
| Avionics & Autonomy | Capability vs. complexity |
| Heat Shield | Reentry capability vs. cost and mass |
| Redundancy | Safety margin vs. cost and schedule |

Each slider has a floor imposed by **available technology** - a player cannot buy a
radioisotope power system before the relevant discipline has advanced.

### 7.2 Vehicle archetypes

Sounding rocket, orbital launcher, heavy launcher, capsule, orbiter, lander, rover,
satellite, station module, tug, probe, megastructure element.

### 7.3 A mission

Vehicle + payload + destination + crew + launch window + flight profile. From this the
sim derives: **success probability**, cost, duration, time-to-results, and the
**discovery potential** of the destination.

### 7.4 The flight [D]

**A launch to a stable orbit takes 60-90 minutes.** This is the game's number, and it
is the number that matters - the player sits and watches it happen, at whatever
dilation they have chosen.

| Phase | Duration | What happens |
|---|---|---|
| Preparation | 3 days to 3 weeks | Crew training, loadout, final checks, and the paperwork that is the real enemy |
| Launch and staging | 0-5 minutes | Ignition, ascent, and staging |
| Ascent | 5-20 minutes | The climb, and the first place something can go wrong |
| Coast to orbital velocity | 20-60 minutes | The long, quiet, dangerous part |
| Circularisation | 60-90 minutes | Insertion, and the moment the vehicle either arrives or does not |

**Preparation is where the game is played and the flight is where it is felt.** A
launch cannot be improvised: the vehicle must exist, the crew must be trained and
rested, the ground support must be operational, and the paperwork must be in order.

### 7.5 Outcomes

Failures should mostly be *partial* rather than cinematic: degraded performance, lost
data, delay, damage, injury, missed objectives, discovery of something unexpected.
Catastrophic loss is possible and probabilistic, and the player is warned - the risk
number is on screen before the launch is approved.

### 7.6 Industrial constraint

Vehicles cannot be conjured. Builds consume **industrial capacity** (personnel,
factories, materials, time). Ambition is throttled by what the agency can actually
manufacture, and the industrial base is itself a thing the player must build and
maintain.

---

## 8. Research & Technology

**Technology emerges from investment and expertise** - this is a design pillar, so the
research model must not be a tech tree with purchases.

### 8.1 Disciplines

Propulsion, Structures & Materials, Life Support, Power, Avionics & AI, Robotics,
Communications, Medical, Planetary Science, Astrophysics, Computing, Manufacturing.

The catalogues are **code constants, not saved state**, and static slices rather than
`Vec`s so they can live in a constant with no allocation and no lazy initialisation.
The campaign stores capability *identifiers*, never the catalogue itself.

### 8.2 Research projects

A project has a principal investigator, a discipline, a target capability, funding,
assigned staff, a duration, and progress. Projects compete for the same people as
missions do.

**Progress is cumulative, not a percentage [D]:** `Progress { points: u32, target: u32 }`,
at 256 units per point. A setback subtracts points; it does not reset the work.

**The target is fixed and the rate moves [D].** A research project's target is 2,400
points, and every person on the team contributes their own effective performance in
points each month; the team's points are **summed**. Adding people to a project makes
it finish sooner, and the target never changes. That is what makes a staffing decision
a schedule decision.

### 8.3 The central research tension [D]

> **Capability vs. reliability.**

- **Research** produces *new capability* - it unlocks better sliders, new
  destinations, new vehicle classes.
- **Testing** produces *reliability* - it reduces the risk of what the agency already
  knows how to build, and produces hard evidence.

Both cost money, time, and the same scarce staff. A player who only researches ships
shaky hardware; a player who only tests never escapes the current era.

### 8.4 Emergent capability

Progress accumulates as **knowledge points per discipline**, plus **evidence** from
successful missions and tests. Capabilities unlock at thresholds. The player perceives a
rising ladder, but the underlying mechanism is continuous and emergent.

**A discipline's knowledge is one number, and every capability in it is a
threshold on that number.** A completed project adds 120 points to its own
field and a fifth of that to each neighbouring field, and a capability unlocks
when its field's total crosses its threshold. **The knowledge is stored per
discipline and not per capability**, and the reason is that a per-capability
total makes the overlapping thresholds meaningless: the flagship capability of
a field would only ever be reached through the adjacency spill.

**The thresholds deliberately do not follow the order they are listed in.** Re-entry
costs 300 and orbital propulsion 240, so a player can reach orbit without ever
developing a heat shield - a legitimate strategy with a specific weakness. A capability
with a threshold of zero was never *developed*; the agency always had it, and it
unlocks silently.

### 8.5 Knowledge and discovery

Missions return data. Data unlocks research directions, reveals the solar system, and
occasionally reveals something about the *civilisation itself*. Discovery is a
first-class reward loop.

### 8.6 Knowledge output [D]

Every research result and every successful mission's data is a **live choice**:

| Choice | Immediate effect | Strategic effect |
|---|---|---|
| **Publish openly** | Reputation up, the world advances | No exclusivity, no income - and rivals get faster too |
| **Patent and license** | Income begins, exclusivity held | Slower cooperation, possible patent disputes |
| **License to a rival** | Immediate cash now | The rival advances, possibly past the player, but the agency is funded |

The choice is made **per result**, and it interacts with the rival race. Publishing or
licensing while the agency is broke is a real, and occasionally correct, decision.

---

## 9. Infrastructure & Industry

**All infrastructure is built. Nothing is abstract capacity.**

### 9.1 A facility

| Field | Notes |
|---|---|
| Name & type | Laboratory, Test Stand, Fabrication Plant, Launch Complex |
| Site | Named location. Affects logistics, security, political exposure, environmental risk |
| Specification | The result of its construction choices - output, reliability, staffing need, upkeep |
| Condition | Degrades over time; affected by maintenance spend and staff attention |
| Staffing | Named staff assigned; understaffed facilities run below specification |
| Construction | Present only while the build is under way |
| Upkeep | What it costs to run each month |
| Output | What it produces each month |

### 9.2 Construction choices

A facility's final specification is **not a preset**. The player chooses how to build
it, and the build method determines how it behaves for the next forty years. The option
set is small and **reusable across facility types**:

| Choice | Options | Affects |
|---|---|---|
| **Method** | Fast & cheap · Standard · Robust · Experimental | Build time, upfront cost, reliability, severity of failure |
| **Scale** | Small · Standard · Large · Oversized | Cost, upkeep, output capacity, staff requirement |
| **Automation** | Manual · Assisted · Automated | Staffing needed, output, labour cost, output quality |
| **Redundancy** | Single-line · N+1 · Fully redundant | Failure rate, and whether a failure is a hiccup or a catastrophe |
| **Materials** | Standard · High-grade · Adaptive | Upkeep cost, degradation rate, obsolescence risk |
| **Siting** | A few concrete choices per type | Logistics, security, political exposure, specific benefits |
| **Financing** | State budget · Private investment · Commercial partner | Cash cost now, but claims on output, revenue share, or obligations |
| **Public face** | For visible sites: austere · civic · prestigious | Prestige, political optics, recruitment appeal |

**Each construction choice must therefore buy something and cost something.**

### 9.3 Construction realities

- Construction is **routinely late**. The build takes as long as it takes, and the
  schedule is a forecast, not a promise.
- Construction draws **political scrutiny**, scaled by cost and by how visible the site
  is.
- Construction consumes **staff** - a build team is people the player does not have on
  programmes.
- Facilities **degrade**. Neglect produces accidents that get blamed on people.

### 9.4 Industrial capacity

Total industrial capacity is the ceiling on how much the agency can build per year, and
it is the sum of the player's own facilities plus what it can contract from industry.

### 9.5 External supply

Some capacity can be bought from contractors and commercial partners rather than owned.
Rented capacity is faster and cheaper up front, and carries exclusivity clauses,
delivery penalties, and dependencies on suppliers the player does not control.

---

## 10. Money

### 10.1 Revenue parameters

| Source | Behaviour |
|---|---|
| **Civilisation base** | The floor. What the civilisation's wealth and industrial maturity can support. Rises across the campaign. |
| **Political standing** | A multiplier on the base: approval, faction support, and the state's priorities. |
| **Investors** | Private capital, in exchange for a revenue share, IP rights, delivery penalties, or exclusivity. |
| **Discoveries** | Major discoveries create new value: resource deposits, biotech, sellable data, salvage, patent income. |
| **Commercial activity** | Launch services, hosted payloads, subscription data, industrial contracts. |
| **Strategic grants** | One-off state funding for prestige objectives, usually attached to a milestone and a date. |
| **Debt** | Emergency borrowing, repaid with interest, available only when the base allocation has already failed. |

**The appropriation scales with the agency [D].** It is not a fixed sum:

```
allocation = max(12,000, 1,900 x headcount) x political_multiplier
political_multiplier = 1 + approval x 0.005
```

**The state funds the agency in proportion to what it employs.** A fixed allocation
would make a small agency absurdly rich and a large one bankrupt, and neither is true.
The floor of 12,000 is low enough that a three-person agency can afford roughly one
researcher - and hiring a second puts it on a clock.

### 10.2 Outgoings

Salaries, programme spend, construction, contracts, research, maintenance, debt service,
and the political cost of a budget that is too large.

### 10.3 The budget crisis ladder [D]

Running the treasury dry is not a game over; it is a **defined escalation ladder**:

1. **Warning** - a funding shortfall is projected. The player sees it coming.
2. **Austerity** - the state imposes a cut. Unfunded programmes are throttled.
3. **Borrowing** - emergency loans at punitive rates, with public and political cost.
4. **Attrition** - unpaid staff leave. Senior staff leave first.
5. **Cancellations** - programmes are terminated, flagship first.
6. **Collapse** - the campaign ends, but the history is kept.

**The pressure comes from ambition, not from existence.** A three-person agency that
does nothing runs a small surplus indefinitely. A crisis arrives when the player starts
a project, hires, or builds.

### 10.4 Escalation

Costs inflate across a multi-decade campaign, and the state does **not** automatically
keep pace.

---

## 11. Politics & the State

### 11.1 The political body

- **Ministers and factions**, each with an agenda: militarisation, pure science,
  commercial expansion, austerity, settlement, expansion at any cost.
- **Approval / trust** with the state, distinct from public approval. The state
  controls the appropriation; the public controls legitimacy. A player can have one
  and not the other.
- **Political capital** is a spendable currency: lobbying, favours, patronage.
- **Goodwill** is the institution's patience, and the state's willingness to
  accommodate the director's whims. It is **not a cap on how many orders the player
  may give** — it is a price. Every order that lands draws on it, in proportion to
  how disruptive it is, and it comes back slowly. A high goodwill means the state
  approves quickly and charges less; a low one means every order takes longer and
  costs more. See `MANAGEABLE_PARTS.md` §3.5.
- **Terms and elections.** A change of government can kill or accelerate programmes.

### 11.2 Promises

The player will be asked for things. A **promise tracker** makes them visible and
accountable. Broken promises are remembered by the specific people who heard them.

### 11.3 Military tension

"Military Reconnaissance Programme" is named in the design doc, so militarisation is a
real axis. A military programme may unlock state support and prestige while alienating
the public.

### 11.4 Goodwill [D]

The institution's patience, and the state's willingness to accommodate the director's
whims. A first-class resource on the dashboard, from -100 to +100.

**Every order that lands charges it**, in proportion to how disruptive the order is.
There is **no limit on how many orders the player may give** - a cap would be a
rationing meter, and this game is not about rationing. The cost of spamming is
financial and political, which is what the theme is about.

| Goodwill | What it buys |
|---|---|
| **High** | Faster approvals, occasional discretionary favours, a deeper applicant pool, staff who forgive more |
| **Neutral** | The ordinary relationship |
| **Low** | Administrative delay on every order, higher costs, political pushback, staff disengagement, and events in which the state asks why |

---

## 12. Public & Media

- **Public approval**, **prestige**, **national pride**, and **fear**.
- **Media outlets** with their own stances and stories.
- **Heroes and villains.** Named individuals become symbols. Fame is a stat; so is
  disgrace.
- **Scandals and cover-ups.** The player may choose to bury a failure. It works, for a
  while. Exposure later is a real event with a real cost.
- **Ceremonies** - state funerals, parades, first landings.

---

## 13. The Wider World

### 13.1 Scope: microbial alien life only [D]

There are **no sentient alien species, no alien artefacts, no ruins of other
civilisations, and no first contact.** The setting's "alien civilisation" is the
*player's own* civilisation, and the game's extraterrestrial biology is **bacterial**.

This buys something valuable: **planetary protection** becomes a first-class
mechanic. A contaminated sample return is a catastrophe with no undo button, and the
risk is knowable but never zero.

### 13.2 Rival agencies [D]

Rivals exist, they are not enemies, and they can be **turned off**.

- **What they compete for:** the state funding pool, commercial contracts, and
  **milestone primacy** - being first to orbit, first to land.
- **What they do not do:** no sabotage, no espionage war, no hostility.
- **The toggle:** a game option, and it may also set how many rivals exist.
- **Implementation note [?]:** whether rivals run the same full simulation as the
  player, or on a lighter progression model, is an architectural decision with a large
  performance impact.

### 13.3 Suppliers and partners

Industry relationships the agency can lean on or be held hostage by. Private investors
arriving with money also arrive with expectations.

---

## 14. Randomness & Events

**Event categories:** technical failure, personnel incident, discovery, political,
financial, environmental, public, external.

**Context weighting [D]:** events are not uniformly random. They are drawn against the
current state of the agency - a tired crew, a stressed project, a neglected facility, a
thin treasury, a hostile press. Bad conditions raise the odds of bad events. The sim is
fair: outcomes follow from the player's own management.

**Determinism [D]:** the entire simulation runs on a seeded RNG. Given the same seed and
the same decisions *at the same simulated instants*, the campaign plays out identically.
**The dilation is not part of this.**

### 14.1 Event visibility [D]

The player sees **both** directions in time.

**History - the chronicle.** A complete, permanent, queryable record of everything that
has happened. This is not a log; it is where the emergent history becomes *readable*.

**Forecast - the timeline.** A rolling horizon of everything the player *knows is
coming**: milestones and committed instants falling due, launches and their windows,
research projects due to complete, construction due to finish, orders in flight,
contracts expiring, political windows, staff returning, and the known risk exposure.

**The forecast is honest but not omniscient.** It shows what is *scheduled*. It does not
show what will go wrong, and it does not show the outcome of any risky action - only
its known risk figure.

---

## 15. Derived, not managed

Computed by the simulation and shown to the player, but not directly set:

Mission success probability · failure severity and shape · time-to-completion ·
knowledge gain rate · technology unlock thresholds · morale and loyalty drift · staff
performance · health outcomes · public and political opinion change · escalation of
interest from rivals · industrial throughput · event draw weights · difficulty pacing ·
the feasibility of a promise.

---

## 16. Orders

An order is a **decision the player has made that has not yet been implemented**. It
carries when it was made, when it lands, and what it costs.

```rust
pub struct Order {
    pub id: OrderId,
    pub issued_at: Instant,     // when the player gave it
    pub effective_at: Instant,  // when it lands: issued_at + lead time
    pub kind: OrderKind,
    pub cost: Money,            // charged when it lands
    pub goodwill: i16,          // charged when it lands
}

pub enum OrderKind {
    Hire { role: Role, salary: Money },
    Fire { person: PersonId },
    Assign { person: PersonId, to: Assignment },
    Unassign { person: PersonId },
    Promote { person: PersonId },
    Train { person: PersonId, course: String },
    SetShift { person: PersonId, shift: Option<u16> },
    CreateProgramme { title: String, category: ProgrammeCategory },
    SetObjective { programme: ProgrammeId, description: String,
                   commitment: Commitment, kind: ObjectiveKind },
    PostponeDeadline { programme: ProgrammeId, objective: ObjectiveId, to: Instant },
    SetBudget { programme: ProgrammeId, allocation: Money },
    StartProject { discipline: DisciplineId, pi: PersonId, title: String },
    StartTestCampaign { capability: CapabilityId },
    BuildFacility { kind: FacilityType, plan: ConstructionPlan },
    AuthorizeLaunch { vehicle: VehicleId, destination: String, crew: Vec<PersonId> },
    DesignVehicle { vehicle: VehicleId, archetype: Archetype },
    SetSubsystem { vehicle: VehicleId, slider: Slider, value: u8 },
    SetPolicy { policy: Policy },
    PublishResult { result: String, mode: KnowledgeMode },
    AssignFacility { facility: FacilityId, programme: ProgrammeId },
    CancelMission { mission: MissionId },
}
```

**The orders in flight are part of the state [D].**
`CampaignState::pending_orders` holds them, and it is saved with everything else: a
campaign saved with a hire in flight must restore that hire, and the scheduler holds
the timing of when it lands.

**Every order carries a lead time [D],** and the lead time is part of the order's
`effective_at`. The table is in `SIMULATION.md` §4.2.

**Each variant has a handler** in `sim::orders`, and an order with no registered
handler is rejected - never ignored.

**An order is validated twice [D]:** when it is issued, so the player is told
immediately, and when it lands, because the world has moved on. An order that can no
longer be honoured is **rejected at landing**, the cost is not charged, and the player
is told why.

---

## 17. CampaignState

```rust
pub struct CampaignState {
    pub agency: Agency,
    pub treasury: Treasury,
    pub political: PoliticalState,   // approval, goodwill, political capital, promises
    pub public: PublicState,

    pub people: BTreeMap<PersonId, Person>,
    pub programmes: BTreeMap<ProgrammeId, Programme>,
    pub projects: BTreeMap<ResearchProjectId, ResearchProject>,
    pub test_campaigns: BTreeMap<TestCampaignId, TestCampaign>,
    pub facilities: BTreeMap<FacilityId, Facility>,
    pub vehicles: BTreeMap<VehicleId, Vehicle>,
    pub missions: BTreeMap<MissionId, Mission>,
    pub pending_orders: BTreeMap<OrderId, Order>,
    pub shift_policy: Option<Policy>,   // the agency's standing shift instruction
    pub chronicle: Vec<ChronicleEntry>,

    pub id_counters: IdCounters,
    pub knowledge: BTreeMap<DisciplineId, u32>,
    pub unlocked: BTreeSet<CapabilityId>,
    pub event_checks: u64,
}
```

**There is no calendar and no `next_month` field [D].** The current instant lives in
the simulation, not in the state, and the calendar is a projection of it derived from
the agency's founding year. Two copies of the date would be two sources of truth, and
they would drift.

**The scheduler is not in the state either [D].** It holds pending work — the next
month boundary, the next event check, a mission in the air, an order about to land —
and it belongs to the engine, not to the campaign. It is saved alongside the state, not
inside it.

**The orders in flight are in the state [D].** `pending_orders` holds the orders the
player has given and the agency has not yet carried out, with their issue instants,
their landing instants, their costs, and their goodwill. They are *campaign data* —
decisions the player made — and they are part of the state hash, so a campaign that
dropped them would be a different campaign. The scheduler holds the *timing* of when
each one lands; the state holds *what* each one is.

**`event_checks` is in the state [D].** It is the count of the random event checks
drawn so far, and it is the **bucket** the event stream is keyed on. The n-th check of
a campaign always draws the same numbers, whether or not it happens at the same
instant — and the check's interval tightens under stress, so the instants move and the
index does not.

**Every collection is a `BTreeMap` [D].** This is not a style preference - it is the
guarantee that iteration order is deterministic, and `SIMULATION.md` §2.3 requires a
total order everywhere. The rule is: *if it is iterated, it is a `BTreeMap` or a sorted
`Vec`.* A `HashMap` in `domain` is a review rejection.

**`CampaignState` is the aggregate root.** It is the only thing the sim mutates, the
only thing the save contains, and the only thing the UI reads. It is cloned at the
month boundary rather than mutated in place, which is what makes the commit atomic and
the branch feature trivial.

---

## 18. Invariant catalogue

These are the rules `domain` enforces, either with a checked constructor or with an
assertion at the point of mutation.

| # | Invariant |
|---|---|
| 1 | Every attribute is 0-100 |
| 2 | Every scaled state value is within its scaled range (fatigue 0-25,600; morale -25,600..=25,600) |
| 3 | A person has **at most one** assignment, and a project team has no duplicates |
| 4 | A `Commitment`'s `target` is never later than its `committed` (the target is the honest plan; the commitment is the promise) |
| 5 | A `Budget` does not silently overspend |
| 6 | A facility under construction has no specification, and a finished one has no construction |
| 7 | Every chronicle entry refers to something that exists |
| 8 | Chronicle entries are never removed or edited, and ids are never reused |
| 9 | A `Promise` is either kept or broken, never both |
| 10 | Money never overflows, and a split conserves every minor unit |

---

## 19. Derived state catalogue

Computed, never stored. Each entry names where it is computed and what invalidates it.

| Derived value | Computed from | Where | Invalidated by |
|---|---|---|---|
| **Effective performance** | modifiers x role fit x tools | `sim::performance` | Any change to the person, their assignment, or their facility |
| **Binding factor** | the same computation, identifying the largest cost | `sim::performance` | Same - it is part of the result, not a separate cache |
| **Role fit** | role vs. specialisms vs. task | `sim::performance` | Specialisms, assignment |
| **Support / cohesion** | team composition, lead, shared traits | `sim::derived` | Any change to the team |
| **Team morale** | individuals, weighted by influence | `sim::derived` | Any morale change |
| **Agency morale** | team morale, weighted by grade | `sim::derived` | Team changes |
| **Political approval** | promises kept, events, public opinion, funding | `sim::derived` | Any of the above |
| **Programme risk** | capability, testing, staff, facility condition, slack | `sim::derived` | All of the above |
| **Effective appropriation** | civilisation base x political standing + other sources | `sim::phases::finance` | Approval, budget year |
| **Slack** | `committed - target`, as a duration | `sim::derived` | Either instant moving |
| **Total burn rate** | sum of programme, staff, upkeep, construction | `sim::phases::finance` | Any of them |
| **Every rate** | the state, as a per-unit-time quantity | `sim::rates` | Whatever the rate reads |
| **Elapsed-time projection** | a rate x the time remaining | `sim::derived` | The rate, or the target |
| **Runway** | balance / burn rate | `sim::derived` | The balance, the burn |
| **Rival standing** | milestone primacy, funding share | deferred | - |

**The rate table is the new "derived" surface [D].** Most of what used to be a monthly
delta is now a rate, and the interface shows both the rate and the projection: *this
project will take another fourteen months at the current rate*, and *the treasury will
last four months at this burn*.

**The rule for caching [D]:** if recomputation is cheap, do not cache. If it is not, cache
with an explicit dirty flag keyed on the inputs. **Never cache by time** - no TTL, no
"refresh every N months". A time-based cache is a non-determinism source.

---

## 20. What is not in the domain model (yet)

| Deferred | Why | Where it will go |
|---|---|---|
| Relationship graph between people | Emergent social dynamics are the most uncertain system | `PEOPLE.md`, later slice |
| Factions, elections, ministers | One state actor suffices for the MVP | `POLITICS.md` |
| Rival agencies | Architecture unproven | `WORLD.md` |
| Media outlets, heroes, scandals | All downstream of the opinion number | `PUBLIC.md` |
| Microbial life, contamination | A late-game pillar | `MISSIONS.md` |
| Knowledge publishing modes beyond `Open` | Needs a rival to license to | `RESEARCH.md` |
| Investor contracts | Needs a second revenue source | `ECONOMY.md` |

**Each of these is a new module or new enum variants - not a change to the aggregate's
shape.** The structure is designed so that adding them does not require a migration.

---

## 21. Related documents

- `SIMULATION.md` - what happens to these entities, and in what order.
- `ARCHITECTURE.md` - where these types live in the code.
- `PERSISTENCE.md` - how they are saved.
- `BALANCE.md` - the numbers they are measured in.
- `UI.md` - how they are shown.
- `MANAGEABLE_PARTS.md` - the design intent behind them.
