# BALANCE.md

> **The numbers, where they live, and how they get tuned.**
>
> **Status:** Design intent, revision 1. **Every value below is provisional.** They exist
> to give the implementation a shape to start from and the harness something to measure.
> They are not predictions.

---

## 1. Principles

1. **One file.** Every tunable number lives in `sim::balance`, in one module, with
   named constants and no magic numbers anywhere else in the codebase. A number in the
   middle of a function is a bug waiting to be un-tuned.
2. **Balance is measured, not felt.** The harness (`TESTING.md` §8) reports outcome
   distributions. A change is only accepted when the distributions move in the intended
   direction.
3. **Three strategies must survive.** Cautious, fast, and science-first. If one
   collapses, the trade-offs are fake and the game has a single answer.
4. **Every number has a reason.** If nobody can say why a constant is 7, it is either
   wrong or meaningless.
5. **Prefer relationships over absolutes.** "A test campaign costs about three
   person-months" survives retuning; "a test campaign costs 26,000 credits" does not.

---

## 2. Scale and units

| Unit | Value |
|---|---|
| **Money** | Integer, in minor units. **1 credit = 100 minor units** |
| **Time** | Integer milliseconds of *simulated* time. 1 s = 1,000 ms, 1 min = 60 s, 1 hour = 60 min, 1 day = 24 h, 1 month = 30 d, 1 year = 12 months |
| **Rates** | Per simulated day, unless stated |
| **Attributes** | 0-100 |
| **Modifiers** | -100..=+100 |
| **Continuous quantities** | Scaled integers at **256 units per percent** |
| **Percentages** | Basis points internally (0-10,000), rendered as a percentage |
| **Probabilities** | Basis points, drawn as integers |

**Everything is an integer.** Percentages are stored as basis points so that a 14% risk
is exactly 1,400, and comparisons are exact. The continuous quantities — fatigue,
morale, progress — are stored at 256 units per percent, because a rate of 0.008% an
hour must not be truncated to nothing.

### 2.1 Order lead times

Every order takes time to implement. These are the numbers behind
`MANAGEABLE_PARTS.md` §3.4.

| Order | Lead time | Direct cost | Goodwill |
|---|---|---|---|
| Hire | 2-6 weeks, by role and seniority | One month's salary | 1 |
| Fire | Immediate | Three months' salary | 4 |
| Assign / reassign | 1-3 days | — | 2 |
| Promote | The next quarterly round (at most 3 months) | — | 3 |
| Train | 3-9 months | The course fee | 2 |
| Start a research project | 1-2 weeks | — | 1 |
| Start a test campaign | 1 week | — | 1 |
| Build a facility | 6 months to 3 years | The capital cost | 5-15 |
| Authorise a launch | The mission's flight time | The vehicle and the mission | 3 |
| Set an objective or a deadline | Immediate | — | 1 |
| Set a programme's budget | Immediate | — | 2 |
| Publish a result | 1 week | — | 1 |
| Change a policy | 1 week | — | 5 |

**A hire's lead time scales with the seniority of the role being filled**, so the agency
cannot buy a division head in a fortnight. It is two weeks at the entry salary and
six weeks at the top of the band, and the salary is what decides.

**Where there is nothing to scale by, the middle of the range is used, and it is a
constant.** A lead time may never be random: an order that landed at a random time
would make the campaign depend on the draw, and the order log would not replay. The
numbers are in `sim::balance` and `sim::orders`.

---

## 3. Money

### 3.1 Salaries (credits per month)

| Role | Entry | Experienced | Senior | Head of division |
|---|---|---|---|---|
| Astronaut | 1,100 | 1,700 | 2,600 | — |
| Scientist | 1,200 | 1,700 | 2,600 | 4,500 |
| Engineer | 1,300 | 1,900 | 2,900 | 5,000 |
| Administrator | 900 | 1,300 | 2,000 | 3,800 |
| Manager | 1,100 | 1,600 | 2,400 | 4,200 |

**A division head earns roughly twice a senior specialist.** That gap is what makes
promotion into management an attractive move for the ambitious and a sacrifice for the
loyal.

**Flight bonuses** are additional for astronauts on a manifest: 3,000 per mission,
paid on return regardless of outcome.

### 3.2 Research and testing (credits per month)

```
research_project  = 12,000 × |team| + 6,000
test_campaign     =  7,000 × |team| + 5,000
```

**A test campaign is roughly half the cost of a research project of the same size**, and
produces no new capability. That ratio is the balance of the central tension: if
testing were as cheap as research, everyone would always test, and the research
half of the game would be pointless.

### 3.3 Facilities

Cost and upkeep are derived from a base, a scale multiplier, and a method multiplier.

| Type | Base cost | Base upkeep/mo | Build months (standard) |
|---|---|---|---|
| Laboratory | 350,000 | 4,000 | 12-18 |
| Test Stand | 600,000 | 6,000 | 18-24 |
| Fabrication Plant | 800,000 | 8,000 | 18-30 |
| Launch Complex | 1,200,000 | 14,000 | 24-36 |

| Scale | Cost × | Upkeep × | Staff needed |
|---|---|---|---|
| Small | 0.70 | 0.6 | 2-4 |
| Standard | 1.00 | 1.0 | 4-8 |
| Large | 1.90 | 2.2 | 8-16 |
| Oversized | 2.80 | 3.6 | 16-28 |

| Method | Cost × | Upkeep × | Build time × | Reliability |
|---|---|---|---|---|
| Fast & cheap | 0.70 | 0.75 | 0.65 | −15 |
| Standard | 1.00 | 1.0 | 1.0 | 0 |
| Robust | 1.45 | 1.30 | 1.35 | +12 |
| Experimental | 0.85 | 1.15 | 0.85 | −5, but may unlock capability |

**Fast & cheap is 30% cheaper and 35% quicker, and costs 15 reliability.** Over a
forty-year campaign, a reliability penalty compounds into the difference between a
facility that still works and one that has been rebuilt twice. That is the intended
lesson, and it is learned rather than told.

### 3.4 Vehicles

| Archetype | Cost | Build months | Notes |
|---|---|---|---|
| Sounding rocket | 250,000 | 6 | Sub-orbital. The agency can reach orbit without one |
| Orbital launcher | 1,900,000 | 12 | The workhorse |
| Orbital capsule | 600,000 | 8 | Reusable only with high investment |

**A launch vehicle costs more than a laboratory, a test stand, and a fabrication plant
put together.** That is deliberate: the first launch should feel like a real commitment.

### 3.6 The reference economy

A typical mid-game month, from the example in `UI.md`:

| Line | Credits |
|---|---|
| Payroll (62 staff) | ~90,000 |
| Facility upkeep | ~28,000 |
| Programme spend | ~40,000 |
| **Total outgoings** | **~158,000** |
| State appropriation | ~117,800 |
| **Net** | **−40,200** |
| Treasury | 190,000 |
| **Runway** | **4 months** |

**The reference agency runs a deficit.** That is the design: a competent player must
find revenue, or cut costs, or ask for a larger appropriation, and every one of those
is a political act. An agency that balances its books by default is an agency with no
interesting decisions.

### 3.6.1 Facilities: wear and capacity [D]

| Constant | Value | What it is |
|---|---|---|
| `FACILITY_DECAY_PER_MONTH` | 4% | How much a facility's condition falls a month when it is short of staff |
| `CAPACITY_OVERLOAD_TOOLS_BP` | 5,000 | How much of the tools factor is lost per unit of overload |

**A facility nobody is looking after falls apart at four percent a month**, and a
fully staffed one holds its condition. The upkeep is the money and the staff are
the maintenance, and the agency can pay for one and not the other.

**Going over the industrial capacity costs the tools factor**, five percent of it
per unit of overload. Every member of staff working at once needs a place to
work, and the facilities are the places.

### 3.7 The appropriation

The state allocation is **not a fixed sum**. It scales with the agency:

```
allocation = max(12,000, 1,900 × headcount) × political_multiplier
political_multiplier = 1 + approval × 0.005
```

**The state funds the agency in proportion to what it employs.** A fixed allocation
would make a small agency absurdly rich and a large one bankrupt, and neither is true.
The floor of 12,000 is low enough that a three-person agency can afford roughly one
researcher — and hiring a second puts it on a clock. The political multiplier runs from
0.5 (approval −100) to 1.5 (approval +100).

**The pressure comes from ambition, not from existence.** A three-person agency that
does nothing runs a small surplus indefinitely. A crisis arrives when the player starts
a project, hires, or builds — which is exactly right: the game should punish overreach,
not survival.

---

## 4. People

### 4.1 Attributes

| Band | Value | Reading |
|---|---|---|
| Exceptional | 85-100 | One of a handful in the agency |
| Strong | 65-84 | Above expectations |
| Competent | 45-64 | The average professional |
| Mediocre | 25-44 | A weak point in the team |
| Poor | 0-24 | A liability |

**A generated specialist lands at 35-75.** A generated ordinary lands at 20-60. Stars
are rare, and they are not generated — they arrive as applicants the player must notice
and compete for.

### 4.2 State dynamics (continuous rates)

Every one of these is now a **rate**, integrated over elapsed time rather than a delta
applied once a month. The units are given per simulated day, because that is the
smallest period a player thinks in.

| Value | Range | Rate | Notes |
|---|---|---|---|
| Morale | -100..=+100 | Toward a target set by assignment satisfaction, over weeks | A good assignment holds it near +30; a bad one pulls it down |
| Fatigue | 0..=100 | +0.2% per day on a full assignment; -0.33% resting | Above 70, output falls sharply; above 90, attrition risk |
| Health | 0..=100 | Recovers over weeks, or not at all | Below 40, forced rest |
| Loyalty | -100..=+100 | Toward a target set by treatment and promises | Below -50, a resignation risk; above +30, half the risk |
| Fame | -100..=100 | Only moves on public events | A consequence, not a lever |

**The rates are stored in scaled units** - 256 units to the percent - so that a rate of
0.008% an hour is not truncated away. `FATIGUE_ON_ASSIGNMENT` is 1,536 units, which is
6% a month of full work, or 0.2% a day.

**The code applies these as rates [D].** `sim::balance` holds the rates in *scaled
units per month*, and `sim::rates` integrates them over the elapsed time, on the
one-hour grid. **The remainder is carried between applications** - `domain::scale::Remainders`
- or the rate is quietly lost: 0.133 units an hour is 136 percentage points of fatigue
over a thirty-year campaign, and nothing would look broken. See `ROADMAP.md` §4.1.1
and §4.3.

**The treasury drains continuously [D].** The outgoings - payroll, research, testing,
and upkeep - are a rate, and the balance falls as the days pass. The allocation from
the state still arrives monthly, because that is a decision rather than a rate.

**Fatigue accrues at roughly 6% a month of full work** - 0.2% a day. A twelve-month
project with no relief puts a competent engineer at 72% fatigue: very slow, but not yet
broken. A two-year project costs the agency a good engineer unless somebody rests.
**Rest is a management decision with a real price**, which is what makes long assignments
interesting.

### 4.2.1 The shift [D]

**The shift is the player's throttle on a person**, and it is a scaled percentage of a
full load. A hundred percent is a full load; the range is zero to two hundred, in
steps of ten.

| Constant | Value | What it is |
|---|---|---|
| `SHIFT_MIN_PERCENT` | 0 | The lightest shift. The same as being idle |
| `SHIFT_MAX_PERCENT` | 200 | The hardest shift. Twice a full load |
| `SHIFT_STEP_PERCENT` | 10 | The step the shift moves by |
| `SHIFT_OVERTIME_OUTPUT_BP` | 6,000 | How much of the effort above a full load turns into output: 60% |
| `SHIFT_OVERTIME_MORALE_PERCENT` | 15 | How much less satisfied a person is for a full extra load |
| `SHIFT_LEAD_TIME_HOURS` | 1 | How long a shift change takes to land |

**The three effects, and the numbers.**

| Shift | Output | Fatigue per month | Morale target |
|---|---|---|---|
| 0% | 0% | −8.5% (the resting rate) | unaffected |
| 50% | 50% | −1.25% — it recovers | unaffected |
| 100% | 100% | +6% (the ordinary rate) | unaffected |
| 150% | 130% | +9% | −7.5% |
| 200% | 160% | +12% | −15% |

**The output above a full load is not proportional, on purpose.** At a hundred and
fifty percent the person produces thirty percent more and tires fifty percent more.
**Overtime is never free**, and the gap between the two columns is the whole of the
mechanic. The rate is `100% + (shift − 100%) × 60%`.

**The fatigue rate below a full load is interpolated between resting and working**, so
a person on light duties *recovers* while they are still on the team. **That is what
the shift buys and what an unassignment cannot**: taking somebody off a project costs
the agency the person, their place on the team, and their output all at once, and a
shift costs none of that.

**A shift change lands at the next hour and costs nothing.** A shift change is a
rostering decision and not a project. It draws no goodwill, because the player must be
able to make a hundred of them.

### 4.3 The scale, and the thresholds that go with it

Every quantity above is stored at 256 units per percent, and **every threshold that
compares against one is scaled too**: `FATIGUE_HIGH` is 70%, `FATIGUE_SEVERE` is 90%,
`HEALTH_UNFIT` is 40%, `LOYALTY_LOW` is -50%, and the approval changes are +4% for a
kept promise and -14% for a broken one.

A comparison written against an unscaled threshold is always true or always false, and
the symptom is a campaign that eats its own staff for no visible reason. This is not
hypothetical: it happened during M1-02, and the tests caught it.

**The integration grid is one hour, and it is not a balance number.** It is the
finest step the rates are applied over, and it is fixed, because a rate that depends on
the state gives a different answer at a different granularity. The grid is what makes
dilation independence hold: a campaign played at the base rate and the same
campaign played at the top rate must integrate identically, or they diverge in
`SIMULATION.md` §3.4.

### 4.4 Attrition (per person, per month)

**There are two rates, and they are different mechanics.**

| Rate | Chance | Who it applies to |
|---|---|---|
| The ordinary rate | 0.5% a month — **six percent a year** | Everyone |
| The heavy rate | 25% a month | The people who are **at their limit**: morale at or below −30 **and** fatigue at or above 70 |

**A healthy agency still loses people.** The ordinary rate is the turnover of any
institution, and the cost of replacing them — a signing fee, a month of a weak
recruit, the work that waits — is part of what it means to run one.

**The heavy rate is the case the player can see coming.** The people who are
miserable *and* exhausted are the ones who walk, and they walk quickly. A base
rate alone would make the at-the-limit case almost invisible, and it is the case
the player can act on.

**The multipliers apply to the ordinary rate:**

| Multiplier | When | Effect |
|---|---|---|
| ×3 | Morale at or below −30 | ×3 |
| ×2 | Loyalty at or below −50 | ×2 |
| ×0.5 | Loyalty at or above +30 | ÷2 |
| — | At their limit | The chance is **at least** 25%, whatever the multipliers say |

**Resignation is the most common exit, and it is nearly always caused by the
player.** The multiplier structure makes that legible: a team with morale at or
below −30 will lose people, and the sim will say so.

| Cause | Base rate | Multipliers | Implemented |
|---|---|---|---|
| Resignation | 0.5% a month | ×3 at morale ≤ −30; ×2 at loyalty ≤ −50; ÷2 at loyalty ≥ +30 | **Yes** |
| Retirement | 4% after age 55 | Rises steeply after 62 | Not yet |
| Illness | 3% | ×2 at health < 50 | Not yet |
| Injury | 2% | Rises on active missions | Not yet |
| Death | 0.2% | Rises with age | Not yet |

**The four causes that are not implemented are listed so the design has a place to
put them** — and so that nobody reads this table and believes the agency only
loses staff to unhappiness. They arrive with the personnel work in Slice 2.

---

## 5. The performance model

### 5.1 The modifier scales

Each modifier is an integer in -100..=+100. The effective factor is derived by a
saturating parabola, so that a large modifier helps less and less.

```
gain(v)   = |v| × (10000 − |v|) / 100          # in basis points
factor(v) = 10000 ± gain(v)                    # + for a positive modifier
factor(v) = clamp(factor(v), 2500, 20000)
```

**This is an integer parabola, not an exponential**, and the difference matters: the
whole computation is exact integer arithmetic, with no floating point anywhere in a
decision. At `v = 100` the factor is 1.99; at `v = 50` it is 1.50; at `v = 10` it is
1.10. The curve flattens as `|v|` approaches 100, which is the property that stops the
player from solving staffing with one input.

**Every factor is floored at 0.25** so that no single input can collapse a team.

### 5.2 The composite, and what 100% means

The eight factors are **multiplied**, so everything bad compounds — one exhausted person
really does slow the whole team. The product is taken in `u128` and no intermediate
stage is clamped, because clamping a stage would saturate it and destroy the very
differences the player is being asked to read.

The product is then divided by the **reference composite**, which is the product of one
*good* team's factors (skill +13, role fit +10, morale +10, support +10, tools +5,
fatigue 0, creativity +13, health +15).

**So 100% means "a good team"**, not "as much as possible" and not "a standard team". A
tired team reads around 50%, an exhausted one around 30%, and a team with nobody
resting all summer reads below 40%. The reference is written as a product of a good
team's factors rather than as a bare number, so it is self-explaining and moves with the
curve if the curve changes.

### 5.3 The per-role modifiers

| Modifier | Range in practice | Typical source |
|---|---|---|
| Skill | -20..+30 | Attributes, relative to a 50 baseline |
| Role fit | -25..+30 | Specialism against the task |
| Morale | -60..+30 | The person's state and the assignment's satisfaction |
| Support | -30..+25 | Team cohesion, lead quality, division head |
| Tools | -35..+25 | Facility quality and automation |
| Fatigue | -100..0 | Never positive |
| Creativity | -20..+30 | Attributes and traits |
| Health | -50..+25 | Physical and mental condition |

**Fatigue is the only modifier that is never positive.** It is the one lever that can
only be spent, which is what makes the "rest" decision real.

### 5.4 The binding factor

The binding factor is the modifier with the largest absolute cost to the result. **It is
reported when its cost exceeds 10% of the maximum**, and no binding factor is shown
when the team is performing near its potential — "no significant constraint" is a
useful and often welcome thing to read.

---

## 6. Research and technology

### 6.1 How a project is measured [D]

A project has a **fixed target of 2,400 points**. Each month, every person on the team
contributes their own performance in points — their effective percentage, divided by 100
— and the team's points are **summed**.

**The target never moves. The rate does.** Adding people to a project makes it finish
sooner, because the sum is larger. That is the whole point: the player's staffing
decisions are what change the schedule.

| Team | At full performance | At 50% performance |
|---|---|---|
| 1 | 24 months | 48 months |
| 2 | 12 months | 24 months |
| 3 | 8 months | 16 months |
| 4 | 6 months | 12 months |

In practice nobody works at 100% for a year, because **fatigue compounds**: a project
that is going well in month three is going badly by month nine unless somebody rests.
A realistic three-person project takes fourteen to eighteen months, and the rate visibly
decelerates as the team tires.

### 6.2 Disciplines

| Discipline | Typical project months | Knowledge per completed project | Adjacent disciplines |
|---|---|---|---|
| Propulsion | 18-30 | 120 | Structures, Power |
| Structures & Materials | 12-24 | 110 | Propulsion, Power |
| Life Support | 15-26 | 100 | Medical, Power |
| Power | 16-28 | 115 | Propulsion, Structures |
| Avionics & AI | 14-24 | 105 | Computing, Communications |
| Robotics | 14-24 | 95 | Avionics, Structures |
| Communications | 10-20 | 85 | Avionics, Computing |
| Medical | 12-22 | 90 | Life Support |
| Planetary Science | 12-24 | 100 | — |
| Astrophysics | 10-20 | 80 | Planetary Science, Computing |
| Computing | 8-16 | 70 | Avionics, Robotics |
| Manufacturing | 12-22 | 80 | Structures, Propulsion |

**Adjacent disciplines give 20% spillover** of the knowledge gained. A propulsion
breakthrough nudges structures. This is how progress feels organic without being a
tree, and it means a narrow research portfolio has a real opportunity cost.

### 6.3 Capability thresholds

| Capability | Knowledge required | Unlocks |
|---|---|---|
| Atmospheric sounding | 0 | Sounding rockets |
| Basic re-entry | 300 | Capsule return |
| Orbital propulsion | 240 | Orbital launchers |
| Closed-loop life support | 380 | Long-duration missions |
| Orbital launch | 420 | Orbital missions |
| Landing systems | 560 | Lunar and planetary landings |
| In-situ resource use | 720 | Resource extraction |
| Deep space propulsion | 900 | Interplanetary missions |
| Megastructure engineering | 1,400 | Megastructures |
| Interstellar capability | 2,200 | Interstellar missions |

**The thresholds deliberately do not follow the order they are listed in.** Re-entry
costs 300 and orbital propulsion 240, so a player can reach orbit without ever
developing a heat shield — a legitimate strategy with a specific weakness, and the kind
of shape the capability thresholds should have. A capability with a threshold of zero was
never *developed*; the agency always had it, and it unlocks silently.

### 6.4 Testing

A test campaign reduces the failure risk of a specific capability by **3 to 8 percentage
points**, for the cost in §3.2. **Testing an untested capability is worth less than
testing a used one** — a capability with no flight history has less to reduce, which
means a player with an untested, unused capability should test something else first.

---

## 7. Missions

### 7.1 Phase durations

**A launch to a stable orbit takes 60-90 minutes** (`MANAGEABLE_PARTS.md` §7.4). The
durations below are the game's.

| Phase | Duration | Note |
|---|---|---|
| Preparation | 3 days to 3 weeks | Crew training, loadout, final checks, and the paperwork that is the real enemy |
| Launch and staging | 0-5 minutes | Ignition, ascent, and staging |
| Ascent | 5-20 minutes | The climb, and the first place something can go wrong |
| Coast to orbital velocity | 20-60 minutes | The long, quiet, dangerous part |
| Circularisation | 60-90 minutes | Insertion, and the moment the vehicle either arrives or does not |
| Transit to the destination | 1 day to 8 weeks | Scales with distance and propulsion |
| Operations | 1 day to 6 months | Science, surface operations |
| Return | 1 day to 8 weeks | |
| Archival | 1-4 weeks | Data processing; the results become available |

**Preparation is where the game is played and the flight is where it is felt.** At
dilation the ninety minutes of ascent are ninety minutes of the player's evening, and
the whole rest of the game is spent making sure that is a good use of it.

### 7.2 Base risk

| Mission type | Base failure risk |
|---|---|
| Sounding rocket | 3% |
| Orbital launch | 8% |
| Orbital mission with return | 11% |
| Lunar mission | 22% |
| Planetary mission | 28% |
| Interplanetary | 35% |

**Risk modifiers, each applied to the base:**

| Modifier | Effect |
|---|---|
| Vehicle reliability, per 10 points above 50 | −1.5 points |
| Test campaigns on the required capability | −1 to −3 points each |
| Crew experience, per 100 flight-hours | −0.8 points |
| Crew fatigue above 70 | +1 point per 10 points of excess |
| Facility condition below 80 | +0.5 points per point below |
| Schedule pressure (slack below 2 months) | +2 points |
| Rushed ground test skipped | +6 points |
| Morale above 60 | −1 point |

**Schedule pressure is the most dangerous modifier and the least visible.** A player
who has let the slack run out is now paying in failure probability whether they
intended to or not.

### 7.3 Failure severity distribution

Given a failure:

| Severity | Share | Consequence |
|---|---|---|
| Partial | 70% | Objective partly met, delay, cost overrun, no catastrophe |
| Severe | 25% | Objective failed, programme set back 3-12 months, injuries likely |
| Catastrophic | 5% | Vehicle and crew lost, programme set back 2-5 years, political crisis |

**A catastrophic loss is 5% of failures, and failures are 8-35% of missions.** So
catastrophe is roughly 0.4-1.8% per mission. Over a 40-launch career, the player will
lose a vehicle and crew perhaps once or twice. That is rare enough to be a story and
frequent enough to matter.

---

## 8. Events

### 8.1 Base weights

Each event carries a base weight between 100 and 1,000. Weights are normalised per
category per month so that the expected number of events per month is:

| Era | Events per month |
|---|---|
| Early (years 0-5) | 1.0 |
| Mid (years 5-20) | 1.5 |
| Late (years 20+) | 2.0 |

**A month with no event is common and legitimate.** The player should not expect
something to happen every month, or every month becomes noise.

### 8.2 The weighting contexts

These are the conditions the weight modifiers read:

| Context | Raises the weight of |
|---|---|
| Average team fatigue above 70 | Personnel incidents |
| Any facility condition below 60 | Technical failures at that facility |
| Treasury below two months of runway | Financial crises, political pressure |
| Public opinion below −30 | Public events, hostile media |
| A programme with zero slack | Programme-level events, deadline events |
| A milestone due within one month | Deadline and pressure events |
| A launch with above 20% risk | Technical failure, investigation |
| An unstaffed facility | Technical failure, staff incident |

**Every weighting context corresponds to something the player did.** That is the
fairness rule from `SIMULATION.md` §5.3, and it is checkable: if a bad event fires
without a matching context, the weighting is wrong.

---

## 9. Politics and public opinion

### 9.1 Approval

| Event | Approval change |
|---|---|
| A committed deadline met | +8 |
| A committed deadline missed | −18 |
| A launch succeeds | +5 |
| A catastrophic failure | −12 |
| A severe failure | −4 |
| A discovery of major scientific value | +6 |
| A promise kept | +4 |
| A promise broken | −14 |
| The appropriation is refused | −10 |
| A scandal is exposed | −9 |

**A broken promise costs more than three times what a kept one rewards.** The
asymmetry is the point: the state remembers failures more than it rewards successes,
which is how realistic political capital behaves.

### 9.2 Political capital

| Action | Cost |
|---|---|
| Postponing a political deadline | 40 |
| Postponing a contractual deadline | 20 |
| Renegotiating an internal deadline | 5 |
| Requesting a larger appropriation | 25 |
| Holding a ceremony | 15, and costs 20 prestige |

**Political capital regenerates slowly**: +3 per month that no commitment is broken, up
to a cap of 100. A player who keeps their promises accumulates the currency to make
one bold ask.

### 9.3 Public opinion and prestige

Public opinion moves with events, roughly ±1 to ±3 per event. Prestige moves only on
significant achievement, ±5 to ±15, and decays by 1 per year. **Prestige is slow,
public opinion is fast** — which is how a public can be enthusiastic about an agency
that is quietly failing.

### 9.4 Goodwill [D]

The institution's patience, and the state's willingness to accommodate the director's
whims. A single resource, from −100 to +100, on the dashboard.

| Order | Goodwill cost |
|---|---|
| Hire, start a project, start a test, set an objective, publish a result | 1 |
| Assign, train, set a budget | 2 |
| Promote | 3 |
| Authorise a launch | 3 |
| Fire | 4 |
| Change a policy | 5 |
| Build a facility | 5-15, by scale and method |

**Goodwill regenerates slowly** — +0.1 a day, up to a cap of 100 — and **a broken
promise costs 10**, which is more than a week of ordinary orders. It regenerates
faster when the agency is doing well: a successful mission and a kept promise each
grant a point.

| Goodwill | Effect |
|---|---|
| **Above +40** | Approvals are fast. The state grants a deadline extension when asked. The applicant pool is deeper. Staff forgive more. |
| **−40 to +40** | The ordinary relationship. Nothing is fast and nothing is hostile. |
| **Below −40** | Every order takes longer to land. Costs rise. The state pushes back. Staff disengage. Events fire in which the state asks why. |
| **Below −80** | The state begins to reconsider the agency's mandate, and a political crisis becomes possible. |

**Goodwill is the anti-spamming mechanism, and it is deliberately not a cap.** There
is no limit on how many orders the player may issue; the cost of spamming is
financial and political.

**How the effect is scaled [D].** The effect is **zero between the thresholds** and
grows to its maximum at the extremes, so there is no cliff at ±40 — a director at +39
and one at +40 are not different people. At the extremes:

| Effect | At +100 | At −100 |
|---|---|---|
| The lead time | Half the time | Half again as long |
| The cost | A third less | A third more |
| The morale a satisfying assignment holds | +20 points | −20 points |

**The numbers are in `sim::balance`:** `GOODWILL_LEAD_TIME_EFFECT_BP`,
`GOODWILL_COST_EFFECT_BP`, `GOODWILL_MORALE_EFFECT_PERCENT`, and the thresholds. is
no limit on how many orders the player may give. A director who changes the plan every
hour will find that the state stops cooperating, and the game becomes about building
the institution rather than steering it. That is a better game than a rationing meter.

---

## 10. The budget crisis ladder

| Stage | Trigger | Effect |
|---|---|---|
| **Warning** | Projected balance below 3 months of runway | Visible on the dashboard and in the top bar |
| **Austerity** | Balance below 1 month | The state withholds 20% of the appropriation; programmes are throttled |
| **Borrowing** | Balance below zero | A loan is taken at 12% annual interest with a 24-month term |
| **Attrition** | Two consecutive months in debt | Unpaid staff leave: seniors first, 5-15% per month |
| **Cancellations** | Three consecutive months in debt | The lowest-priority programme is terminated, publicly |
| **Collapse** | Six consecutive months in debt | The campaign ends. The chronicle is kept |

**The ladder takes six months to reach the bottom.** The player has time to respond at
every stage, and the warning appears two months before the austerity. A crisis should
be the consequence of a run of bad decisions, not a surprise.

---

## 11. Time scale

| Milestone | Target | Acceptable range |
|---|---|---|
| First sounding rocket | Month 6 | 4-12 |
| First launch | Month 18 | 12-36 |
| **First orbit** | **Month 36** | **24-60** |
| First crewed orbital flight | Month 54 | 36-90 |
| First landing on another world | Month 120 | 84-180 |
| Space station | Month 180 | 120-300 |
| Resource extraction | Month 300 | 240-420 |
| Interplanetary | Month 480 | 360-660 |

**A full campaign is 30-60 years, and the design says it should be.** The whole
progression is in `MANAGEABLE_PARTS.md` §3; the numbers here say when each stage
arrives.

**The first orbit at month 36 in a good run, month 60 in a cautious one, and never in a
reckless one** — the reckless strategy should reach orbit sooner and pay for it later.

### 11.1 What the milestones cost in real time [D]

The game runs in real time at a player-chosen dilation, so the milestones have a
wall-clock cost as well as a simulated one.

| Dilation | 30 simulated years | 60 simulated years | A 90-minute launch |
|---|---|---|---|
| 60x | 146 days | 292 days | 90 seconds |
| 600x | 14.6 days | 29 days | 9 seconds |
| 6,000x | 29 hours | 59 hours | 0.9 seconds |
| 60,000x | 2.9 hours | 5.9 hours | 0.09 seconds |
| 600,000x | 17 minutes | 35 minutes | 0.009 seconds |

**A 30-year campaign is about half an hour of real time at maximum dilation.** That is
Football Manager-scale in the other direction, and it is the reason the campaign is
something a player can play in an evening rather than something they leave running.
**Simulated time only advances while the game is open**, so closing it is a pause that
costs nothing.

**The player will spend most of the campaign at high dilation and drop to the base
rate for the moments that matter** — a launch, a political crisis, a deadline. The
nine steps are chosen so that each is a meaningful change of texture rather than a
rounding error.

### 11.2 The event check

| Interval | When |
|---|---|
| One simulated week | The base |
| Tighter | Under stress — a thin treasury, a failing facility, a tired crew |
| Wider | When the agency is quiet and everything is going well |

**Most checks produce nothing.** The agency is usually quiet, and a quiet stretch is a
legitimate outcome rather than a bug.

---

## 12. Tuning method

### The loop

1. Change a constant in `sim::balance`.
2. Run the balance harness over N seeds with the three reference policies.
3. Read the distributions.
4. Accept the change if the distributions moved as intended, and update this document.

**Never tune a constant by playing the game once.** A single run is a sample of one,
and the variance in a 40-year campaign is enormous. The harness exists so that tuning
is a measurement rather than a story.

### What to measure

| Metric | Target |
|---|---|
| Reach-orbit rate, cautious policy | 70-85% |
| Reach-orbit rate, fast policy | 55-75% |
| Reach-orbit rate, science-first policy | 60-80% |
| Any single policy's failure rate | Never above 30% |
| Time to orbit, best run | Not under 18 months |
| Budget crisis reached, per campaign | 0-1, and never more than 2 |
| Catastrophic vehicle loss per 100 missions | 0-4 |
| Research project duration, three people | 14-18 months |
| Research project duration, one person | 24+ months, or never |
| A team's effective performance, rested and competent | 90-110% |
| A team's effective performance, one person at 80% fatigue | 40-60% |

**The spread between the three policies should be roughly 10-20 months of time to orbit
and a comparable spread in final treasury.** If one policy is strictly better on every
axis, the trade-offs are broken.

---

## 13. Change procedure

1. The change is made in `sim::balance`, with a comment giving the reason.
2. The harness is run, before and after, and the distributions are compared.
3. This document is updated in the same change.
4. The golden fixtures are regenerated if the change alters behaviour, and the chronicle
   diff is reviewed.

**Step 4 is not optional.** A balance change that alters outcomes without regenerating
the fixtures will fail the golden tests, which is the system working. The rule is that
the diff gets *reviewed*, not that the fixtures are silently refreshed.

---

## 14. Open questions

| # | Question | Why it matters | Blocks |
|---|---|---|---|
| 1 | Is the currency "credits" right, or should the game name its own? | The setting is an alien civilisation, and a currency named "credits" is a small lie | Naming, early UI |
| 2 | Should research costs scale with the number of disciplines the agency operates? | A broad agency should be more expensive to run | S4-08 |
| 3 | Is one credit 100 minor units or 1,000? | The scale determines whether fractional spending is representable | S1-10 |
| 4 | Do inflation and salary growth track each other? | A thirty-year campaign with static salaries looks wrong | S4-08 |
| 5 | Should the political multiplier be linear in approval, or should high approval have diminishing returns? | A linear multiplier makes approval the only political lever worth having | S4-08 |
| 6 | Should the reference composite be re-derived from the balance harness rather than fixed? | It is a hand-tuned constant, and every rate in the game depends on it | Balance harness |

---

## 15. Related documents

- `SIMULATION.md` — the engine the numbers run inside.
- `DOMAIN_MODEL.md` — the entities the numbers describe.
- `UI.md` — the example figures on the screens come from this document.
- `TESTING.md` — the harness that measures them.
- `ROADMAP.md` — the tasks that implement them.
