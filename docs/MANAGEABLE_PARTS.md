# The Manageable Parts

> **The management surface of `space_manager`** — every system the player can act upon, and
> therefore every system the simulation must model.
>
> This is the scoping document. If a mechanic is not described here, it is not in the game.
> If something is described here but has no player-facing lever, it belongs in the
> "Derived, not managed" section at the end.

**Status:** Design intent, **revision 3**. Decisions marked **[D]** are settled; **[?]** are
open and listed in [Open decisions](#open-decisions). Numbers are placeholders to be
balanced later, not promises.

**Revision 3** replaced the monthly-turn model with **continuous simulated time running
in real time at a player-chosen dilation**. The player acts whenever they like, orders
take time to implement, and orders cost money and goodwill. This touches every other
document; §3 is the new foundation and §3.8 lists what the change breaks.

**Revision 2** closed the open questions on deadlines, event visibility, infrastructure,
the revenue model, knowledge output, the scope of alien life, rival agencies, and the
staff performance model. Six questions remain, and they are all either balance work or
narrow mechanics rather than foundations.

---

## 1. The vision in one paragraph

The player is the Director of a space agency belonging to a procedurally generated alien
civilisation. They do not fly anything. They hire and keep people, decide which programmes
the agency pursues, split a finite budget across them, absorb political pressure and public
reaction, and live with the consequences for thirty years or more. Every outcome should be
traceable to a decision the player made, and no decision should have been obviously correct
in hindsight.

---

## 2. The management surface at a glance

```
THE AGENCY
│
├─ Time & Campaign          continuous calendar, monthly resolution          [sim]
├─ Identity & Doctrine      focus, policies, public face                     [domain]
├─ Structure               divisions, directorates, their heads             [domain]
├─ People                  150-300 staff: hire, assign, develop, lose        [domain+sim]
├─ Programmes               the strategic layer: goals, money, deadlines     [domain+sim]
│  └─ Missions & Vehicles   the execution layer: hardware, crew, launch    [domain+sim]
├─ Research & Technology    disciplines, projects, breakthroughs            [domain+sim]
├─ Infrastructure & Industry facilities, capacity, upkeep                   [domain+sim]
├─ Money                    treasury, budgets, revenue, escalation           [domain+sim]
├─ Politics & the State     mandate, factions, approval, promises            [domain+sim]
├─ Public & Media           opinion, prestige, heroes, scandals              [domain+sim]
└─ The Wider World          rivals, partners, suppliers, first contact       [domain+sim]
```

| Domain | The player asks | Where it lives |
|---|---|---|
| Time | "What is happening now, and what have I set in motion?" | `sim` |
| Identity & Doctrine | "What kind of agency am I running?" | `domain` |
| Structure | "Who runs which division?" | `domain` |
| People | "Who is the right person for this, and can they take it?" | `domain` + `sim` |
| Programmes | "What am I actually trying to achieve by when?" | `domain` + `sim` |
| Missions & Vehicles | "Can I build and fly this within budget and risk?" | `domain` + `sim` |
| Research | "Do I bet on capability or on reliability?" | `domain` + `sim` |
| Infrastructure | "Do I have the capacity to do what I promised?" | `domain` + `sim` |
| Money | "What can I afford, and what happens if I run dry?" | `domain` + `sim` |
| Politics | "Who do I need to keep happy, and what will it cost me?" | `domain` + `sim` |
| Public | "How do they see us?" | `domain` + `sim` |
| Wider World | "Who is ahead of us, and can I use that?" | `domain` + `sim` |

---

## 3. Time & the Campaign

**Time model [D]:** **continuous simulated time, running in real time at a
player-chosen rate of dilation.** There are no rounds, no turns, and no months to
resolve. The player acts whenever they like, the simulation keeps running, and every
decision they make takes time to implement.

The campaign spans **30-60 simulated years**.

### 3.1 The clock

- Simulated time is an **integer number of milliseconds since the campaign began**.
  There is no floating-point time, and the simulation never reads the wall clock — it is
  told how much simulated time has passed.
- A civil calendar is **derived** from it, for display and for the periodic triggers
  that still have a rhythm:

  | Unit | Length |
  |---|---|
  | Second | 1,000 ms |
  | Minute | 60 seconds |
  | Hour | 60 minutes |
  | Day | 24 hours |
  | Month | 30 days |
  | Year | 12 months (360 days) |

  A uniform calendar keeps the arithmetic trivial and the dates legible. The
  management vocabulary survives: "the fiscal year", "the promotion round", "the
  construction will take two years".

### 3.2 Time dilation [D]

The player sets how fast simulated time passes. There are nine steps.

**The base rate is 60x: one second in life is one minute in game.** There is
nothing slower, and that is a decision rather than an omission — a campaign is
thirty to sixty years, and a rate below a minute a second would be a lifetime
rather than a game.

| Key | Dilation | Simulated seconds per real second | A 90-minute launch takes |
|---|---|---|---|
| `1` | 60x | 60 | 90 seconds |
| `2` | 300x | 300 | 18 seconds |
| `3` | 600x | 600 | 9 seconds |
| `4` | 3,000x | 3,000 | 1.8 seconds |
| `5` | 6,000x | 6,000 | 0.9 seconds |
| `6` | 30,000x | 30,000 | 0.18 seconds |
| `7` | 60,000x | 60,000 | 0.09 seconds |
| `8` | 300,000x | 300,000 | 0.018 seconds |
| `9` | 600,000x | 600,000 | 0.009 seconds |

**The base rate is a minute a second, and the campaign starts there.** At the
base rate a year takes about ninety minutes of watching, which is an evening —
and the player can wind it on whenever the decisions start to arrive faster than
they can think.

**The clock controls are global.** They work on every screen, and they work
while a modal is up: a player who wants to stop and read the report must be
able to stop the clock without dismissing the thing they were reading.

**The dilation is a pacing control, not a simulation control [D].** The outcome of a
campaign depends on the *simulated timestamps* of the player's decisions and never on
how fast the player got there. Dropping to the base rate to watch a launch and
running at 600,000x the rest of the time produce **the same campaign**. This is a hard
guarantee, and the game is built to keep it.

**What the dilation is for.** The player chooses how much of the game they want to
*watch*. At the base rate a launch is an event; at the top of the range it is a line
in a ledger. Both are legitimate ways to play, and the choice is the player's.

**The cost of a long campaign [D]:** at the maximum dilation, 30 simulated years is
about **30 minutes of real time**, and 60 years about **an hour**. A campaign is
something you can play in an evening, and you can leave it running. Saves are
resumable, and **simulated time only advances
while the game is open** — closing the game is the only pause that costs nothing.

### 3.3 The player and a running simulation [D]

- The simulation **runs continuously**. The game does not wait for the player.
- There is a **pause**, and it is a first-class control. A management game in which you
  cannot stop to think is a management game you cannot play.
- The player acts **whenever they like** — the order of actions is entirely theirs.
- The player may also **schedule an order for a future instant**: *hire someone in
  March*, *authorise the launch when the window opens*. This is the order machinery
  generalised from "next month" to "any instant", and it is what lets a player plan
  without leaving the game running.

### 3.4 Orders take time [D]

Every order is a **decision that must be implemented**, and implementation takes time. An
order issued at `t` takes effect at `t + lead time`. The lead time is part of the
order's cost, and it is what stops the game from being a spreadsheet.

| Order | Lead time | Direct cost | Goodwill |
|---|---|---|---|
| Hire | 2-6 weeks, by role and seniority | A signing fee of one month's salary | 1 |
| Fire | Immediate | Severance of three months' salary | 4 |
| Assign / reassign | 1-3 days | — | 2 |
| Promote | The next quarterly round, at most 3 months | — | 3 |
| Train | 3-9 months | The course fee | 2 |
| Start a research project | 1-2 weeks of set-up | — | 1 |
| Start a test campaign | 1 week | — | 1 |
| Build a facility | 6 months to 3 years, by type, scale, and method | The capital cost | 5-15 |
| Authorise a launch | The mission's own flight time | The vehicle and the mission | 3 |
| Set an objective or a deadline | Immediate | — | 1 |
| Set a programme's budget | Immediate | — | 2 |
| Publish a result | 1 week of peer review | — | 1 |
| Change a policy | 1 week of bureaucracy | — | 5 |

**The lead time is visible before the order is confirmed.** The player should always be
able to see that hiring someone takes three weeks, and that a launch takes an hour and
a half, before they commit.

### 3.5 Orders cost money and goodwill [D]

There is **no hard limit on how many orders the player may give.** A cap would be a
rationing meter, and this game is not about rationing. The cost of an order is
**financial and political**, which is what the theme is about.

- **Direct cost.** Most orders have an obvious one — the capital cost of a building, a
  month's salary, a course fee. Some need to be given one, and the table in §3.4 says
  which.
- **Goodwill.** The institution's patience, and the state's willingness to accommodate
  the director's whims. It is a first-class resource, shown on the dashboard, running
  from -100 to +100. **Every order draws on it**, in proportion to how disruptive it is:
  changing a policy costs five, hiring costs one.

| Goodwill | What it buys |
|---|---|
| **High** | Faster approvals, occasional discretionary favours, a deeper applicant pool, staff who forgive more, and a state that grants a deadline extension when asked |
| **Neutral** | The ordinary relationship |
| **Low** | Administrative delay on every order, higher costs, political pushback, staff disengagement, and events in which the state asks why |

**Low goodwill is the answer to "a real-time game invites spamming"** — not a cap, but a
consequence. A director who changes the plan every hour will find that the state stops
cooperating, and the game becomes about building the institution rather than steering it.

### 3.6 The loop [D]

There is no loop any more, in the sense of a cycle the player steps through. The game is
a **running agency**:

1. The player issues orders — now, or scheduled for later.
2. Each order is validated immediately and scheduled for its lead time.
3. The simulation advances. Work progresses at its own rate, budgets burn, fatigue
   accrues, risks materialise, events fire, opinions shift.
4. Something needs attention. The player reacts.

**The simulation does not owe the player the schedule they asked for.** Work slips,
hardware arrives late, people are unavailable, rockets fail. This gap between intention
and outcome remains the game's main source of tension.

### 3.7 Deadlines, dates, and commitment [D]

Commitment remains the core currency. A date is now an **instant** rather than a month,
and everything else follows.

Every date the player commits to has an **audience**, and the audience determines the
consequence of missing it:

| Audience | Deadline type | Consequence of a miss |
|---|---|---|
| The agency itself | Internal target | None directly; the estimate is revised and morale takes a small hit |
| A contractor or partner | Contractual | Financial penalty, loss of that partner's trust, lost future contracts |
| The state / a ministry | Political | Approval loss, a broken promise logged against a *specific* faction, possible defunding |
| The public | Public | Prestige crash, hostile media cycle, sponsor withdrawal |
| A regulator or the sky | Mandatory | Absolute. A missed launch window is simply gone; a lapsed permit suspends operations |

**The buffer is the mechanic.** Every dated objective carries two instants:

- **Target instant** — the agency's honest internal plan. Soft. Slipping is normal.
- **Committed instant** — what was promised to whoever holds the deadline. Hard.

The gap between them is the **slack** the player manages. A programme with fourteen
months of slack absorbs a nine-month delay without consequence. A programme promised in
six months with zero slack is a gamble the player chose to make.

**Converting a date is a real decision.** When the player sees a date will be missed,
they choose how to handle it:

- **Move it openly** — renegotiate with the deadline holder. Costs political capital,
  trust, or goodwill depending on the audience. Honest, and it works.
- **Absorb it silently** — no immediate cost. The miss surfaces later, at a worse moment,
  and costs more.
- **Absorb it completely** — pull resources in and hit the date. Expensive in staff
  fatigue, budget, and risk. Sometimes the only option left.

This is the mechanism behind the design doc's promise that a disastrous launch delays
exploration by decades: the delay is not scripted, it is the slack being consumed.

### 3.8 What the new time model breaks [D]

Recorded so that the old assumptions are not quietly carried forward:

- **The monthly tick is gone.** There is no month to resolve and no "end of month"
  button. The eleven phases of `SIMULATION.md` §2.2 are replaced by a scheduler, ten
  phases, and continuous integration.
- **A decision's effect is never immediate.** It is scheduled, and the player can see
  when it will land.
- **Work does not progress in monthly steps.** It progresses continuously, at a rate
  the player changes by changing who is working on what.
- **The month report is a panel, not a modal.** It sits on the dashboard beside the
  live feed of what has happened and the standing list of what needs attention, so
  the player reads it without the game stopping.
- **The forecast is now a timeline**, not a list of months.
- **Nothing is derived from "which month is it".** Every rule that used to ask that now
  asks "what instant is it", and most of them have become rates rather than events.

---
## 4. The Agency (Identity, Doctrine, Structure)

### 4.1 Identity

- Name, crest, home city, and the nation or polity the agency answers to.
- **Focus [D]:** a one-line strategic identity the player commits to early (e.g. *"the
  science-first authority"*, *"the expansion arm"*, *"the commercial launch house"*).
  Focus biases event weighting, applicant pools, and political receptivity — it is a
  long-term commitment, not a toggle.
- Founding year, founding mandate (the original promise to the state).

### 4.2 Doctrine and Policies

**Policies [D]:** discrete, toggleable, trade-off-bearing rules the player sets. Each one
should buy something real and cost something real. Initial set:

| Policy | Buys | Costs |
|---|---|---|
| Safety Review Board | Fewer accidents, better public trust | Slower approvals, higher overhead |
| Fast-Track Approval | Speed, more launches per year | Higher risk, occasional disaster |
| Recruitment Drive | Deeper applicant pool | Wages rise, integration time |
| Hiring Freeze | Budget relief | Morale drops, attrition rises |
| Intern Programme | Cheaper staff pipeline, long-term quality | Needs mentoring capacity |
| Nationalist Hiring | Loyal, ideologically aligned | Limits talent ceiling |
| Open Research Publishing | Reputation, faster science | Loses exclusivity, no IP income |
| Commercial Licence Programme | Revenue, industrial links | Public trust, state suspicion |
| Formal Accreditation | External credibility, talent access | Years of effort, ongoing cost |

Policies are the Tropico-style edict layer: the player should feel the pull of having
twenty of them, each with a real consequence, and a reason to revisit old ones.

### 4.3 Structure

- **Divisions**, each with a head (a real person with their own skills, morale, and
  ambitions) and a mandate. Typical set: Research & Development, Flight Operations,
  Engineering, Astronaut Corps, Mission Planning, Public Relations, Finance & Procurement.
- The player can **create, rename, split, merge, and abolish** divisions. Each change
  costs administrative energy and unsettles staff.
- Divisions have **headcount budgets** and are where staff are assigned when not on a
  programme.
- Department heads compete for the Director's favour; a neglected division quietly
  underperforms, or worse, lobbies for its own expansion.

### 4.4 Agency-level metrics

Prestige (national standing), Agency Morale (workforce-wide), Public Approval, Political
Capital, Safety Record, Knowledge Index, Industrial Capacity, Treasury.

---

## 5. People

**Scale [D]:** 150-300 staff. The player knows the famous ones intimately and the rest
through aggregate screens, filters, and reports.

### 5.1 Roles

Scientists, Engineers, Astronauts, Administrators, Managers — plus Pilots/Range and
Controllers, and later commercial/industrial staff. Each role has a distinct career ladder,
a distinct pay band, and a distinct failure mode.

### 5.2 The person

| Field | Notes |
|---|---|
| Identity | Name, origin, species, homeworld, age, family ties |
| Career | Rank, grade, years of service, education, certifications |
| Attributes | A small set — Technical, Creative, Leadership, Physical, Diplomatic — plus role-specific specialisms (propulsion, structures, avionics, life support, geology, medicine…) |
| Traits | 1-3 personality hooks: *frail but brilliant*, *won't accept a bad plan*, *needs the credit*, *loyal to the old regime* |

**The traits are built [D].** Fourteen of them, and each one lands on the factor
it describes: a team player raises the support factor, a perfectionist raises the
tools factor, a restless mind raises creativity and costs morale. Everybody has
two or three, and they are drawn deterministically so a campaign is the same
campaign every time. **A trait is worth a few points and not a career** — the
eight factors run from -100 to +100 — and most of them are a trade, because the
interesting decision for the player is not who is strongest (the attributes say
that) but who is pleasant to work beside on a three-year project.
| State | Morale, loyalty, stress/fatigue, health, fame, confidence, burnout |
| Relationships | Mentor, rivals, friends, spouse/partner, faction clique, political sponsor |
| Assignment | Division, programme, project, or "unassigned" (a cost, not a neutral state) |
| **Shift** | **How hard they work: 0% to 200% of a full load, in steps of 10%. The player's throttle.** |
| History | Past projects, successes, failures, incidents, awards, disciplinary notes |
| Contract | Salary, term, demands (a senior scientist may demand a programme of their own) |

### 5.2.1 The shift [D]

**The shift is how hard a person works, and it is the player's throttle on them.** A
hundred percent is a full load. Below it the person contributes less and tires less;
above it they contribute more and tire much more, and the extra effort is worth less
than the extra fatigue it costs.

| Shift | Output | Fatigue | Morale |
|---|---|---|---|
| 0% | nothing | recovers, as if unassigned | unaffected |
| 50% | half | recovers, while still on the team | unaffected |
| 100% | full | the ordinary rate | unaffected |
| 150% | 130% | one and a half times the ordinary rate | −7.5% |
| 200% | 160% | twice the ordinary rate | −15% |

**The output above a full load is deliberately not proportional.** A person on a
hundred and fifty percent produces thirty percent more and tires fifty percent more,
and the gap between those two numbers is the whole of the mechanic. **Overtime is
never free**, and a player who pushes a team past a full load is buying a little
speed with a lot of wear.

**The shift is what replaces "unassign them".** The old way of easing the load on a
tired person was to take them off what they were doing, which cost the agency the
person, their place on the team, and their output all at once. **A person on light
duties recovers while they are still on the team** — that is what a shift buys and
what an unassignment cannot.

**The shift is an order, and it lands at the next hour.** A shift change is a
rostering decision and not a project. It costs no money and no goodwill, because the
player must be able to make a hundred of them.

**The shift is per person, and that is not the whole answer at three hundred people.**
The per-person control is the surgical tool, and it scales because it is one
keystroke on a screen the player is already looking at. The bulk answer — a shift
policy for the whole agency — is the standing instruction below.

**The shift policy [D]:** the agency has one standing instruction about shifts, and
it says two things — **who it covers** and **what they work**.

- **With no threshold, everybody works the shift.** "The agency is on half shifts
  over the winter."
- **With a threshold, only the tired are moved.** "Anybody over 60% fatigue goes to
  half shifts." The people who are fresher than the threshold keep their own shift.

**The threshold makes it a thermostat [D].** An eased-off person recovers, comes back up
to the threshold, and is eased off again, and the cycle is slow. The point is that the
agency never has anybody work themselves to pieces, and the player never has to watch
eighty fatigue bars to know whether it is doing that.

**And the thermostat is a latch and not a comparison [D].** A person is eased when their
fatigue reaches the threshold, and stays eased until their fatigue has come down by ten
points. **A comparison is not enough.** A person whose fatigue settles exactly on the
threshold crosses it every few hours, and the agency gets half of neither shift: the
shift flips on and off, the output flickers, and the performance model sees a team
that nobody has done anything to. The margin is the whole difference between a
thermostat and a buzzer.

**A person the manager has moved individually is moved off the policy**, and the staff
screen says so — a person on a light shift because the policy eased them off is a
different thing from a person the manager put on light duties, and the player has to
be able to tell them apart. **The player can also put somebody back on the policy**,
which is how they say "she works flat out until she is tired, and then she eases off".

**The policy is an order, and it lands after a week of bureaucracy.** It costs five
goodwill, because a standing instruction about how the whole agency works is a public
act. The per-person shift stays free and immediate: a player must be able to make a
hundred of them.

**Why it is standing and not a bulk write.** A bulk write would have to be repeated,
and the "tired ones" version would only ever catch the tired ones *at that moment*.
A standing instruction keeps working as people tire.

### 5.2.2 Finding people [D]

**Every list is filterable and sortable**, because an agency with three hundred people
has a list that cannot be read top to bottom.

- **The filter is a mode**, not a side effect: the keys go into the filter and nowhere
  else, so a filter can contain a `q` without the game quitting.
- **The filter takes conditions and not only names [D].** "Who is in trouble" is not a
  name, and a filter that can only search for names makes the player read three
  hundred rows to answer it. The staff list takes `fatigue>80`, `morale<-30`,
  `role:scientist` and `unassigned`; every term has to hold, so
  `fatigue>80 unassigned` means *tired and on nothing*.
- **The fields belong to the list.** Each list says which fields it has, and a term
  that names a field the list does not have is a word to look for rather than an
  error — so a filter typed on one screen does not become a wall of nothing on
  another.
- **The fields are named on the screen**, under the filter line, and in the manual.
  A player who does not know the filter takes conditions has no reason to type one.
- **The staff list sorts by name, age, role, fatigue, attrition risk, and shift.**
- **The attrition risk is the same number the simulation rolls against** — the chance
  the person leaves this month — and a person about to leave is a person the player
  can still save.

### 5.3 Performance model [D]

Performance on a task is **derived**, never set directly. The player manipulates the
*inputs* (who, where, with what, how hard) and reads the *outputs* (progress, quality,
confidence). This is what makes the People pillar a game rather than a spreadsheet.

**Settled: the C+B hybrid.** The underlying arithmetic is Option C's smooth
diminishing-returns model; the player-facing explanation is Option B's decomposed
modifier breakdown.

#### The model [D]

```
skill_fit = attributes × role_fit              # does this person fit this work?
drive     = skill_fit × motivation              # morale, personal stakes, interest
condition = drive × (1 - fatigue) × health      # what state are they in?
effective = condition × support × tools × creativity
```

**Clamping [D]:** every factor is floored above zero. A person at total exhaustion is
*very bad*, not *non-existent*. This removes the cliff where a single factor multiplies
a team to nothing.

**Saturation [D]:** every factor saturates toward its maximum with diminishing returns.
Maxing one lever stops paying, so the player is pushed to spread effort across several
levers rather than to solve staffing with a single input. This is the property that
stops the model from having a degenerate optimal strategy.

**Accumulation [D]:** performance is a **rate**, and it accrues continuously. Over an
elapsed duration a team accrues:

```
progress += effective × task_complexity × facility_quality × team_size_factor
```

Progress is visible and cumulative, and a setback need not erase a year of work.

#### What the player sees [D]

The TUI **must** decompose the result the way Option B does, even though the arithmetic
is Option C's. Every team, project, and role shows:

- effective performance as a percentage of a good team (100% is a competent, rested team),
- the **binding factor** — the single constraint costing the most, named plainly:
  *"held back by fatigue"*, *"held back by tool quality"*, *"held back by role fit"*,
- the modifier breakdown, each in the range −1..+1: skill, role fit, morale/motivation,
  support, tools, fatigue, creativity.

The binding factor is what makes the model teachable. The player should learn to read
the screen and know which lever to pull **without ever summing the numbers themselves**.
If a player can work out the answer in their head, the screen has failed.

#### Considered and rejected

The four candidates, and why the hybrid won:

---

#### Option A — Multiplicative cascade *(rejected)*

```
effective = skill × role_fit × morale × (1 - fatigue) × tool_quality
            × team_cohesion × (1 - overwork_penalty) × noise
```

Every factor is a multiplier, so a single factor near zero annihilates the team.

- **For:** one line of maths; every factor is obviously important; tuning is easy.
- **Against:** **cliff behaviour**. A team at 4% fatigue is at 96% output, but a team at
  100% fatigue is at 0% — and the player will learn to binary-search the cliff rather than
  manage a spectrum. Small multiplicative penalties compound into nonsense (six factors
  at 0.9 each is 53% output, which feels arbitrary). The player's question is always
  "which factor is near zero?", and the answer is often "several of them".

#### Option B — Additive competency with thresholds (Football Manager style) *(rejected as the model, adopted as the explanation layer)*

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

The task progresses when the **sum** crosses a threshold; the band around the threshold
determines quality, overrun, or shortfall.

- **For:** **maximally explainable** — "Marsha has 62 propulsion against the 55 this
  needs, but her morale is −1 and her fatigue is −1, so the team lands at 60: marginal".
  Scales cleanly to 300 people, because each person is evaluated independently. Each
  modifier is a **separate lever** the player can push.
- **Against:** the threshold is a cliff of a different kind — a team at 54 and a team at
  56 are near-identical, but 55 and 56 are not. Needs per-task tuning, and forty task
  types means forty threshold sets. Degenerate stacking is possible (many small +1s).

#### Option C — Diminishing-returns dimensional *(adopted as the model)*

One continuous performance number per person per activity, built from clamped factors
with diminishing returns:

```
skill_fit   = attributes × role_fit                     (match to the work)
drive       = skill_fit × motivation                     (morale, personal stakes, interest)
condition   = drive × (1 - fatigue) × health
effective   = condition × support × tools × creativity
```

with every factor **floored above zero** and **saturating** toward its maximum.

- **For:** **no cliffs and no degenerate stacking** — a saturated factor stops helping, so
  the player is forced to spread effort across several levers instead of maxing one.
  Smooth, so the UI can show a clear "this team is at 62% of a good team, held back mainly
  by fatigue and tool quality". Tunes in one place rather than per task.
- **Against:** the least self-explanatory of the four, and it **demands a good UI** to
  stay legible — the player must be told *which* factor is binding. Also needs the
  saturation curves designed carefully or everything converges to the average.

#### Option D — Full individual event simulation *(rejected)*

Each staff member takes their own actions each month, interacting with their colleagues,
their projects, and their circumstances. Performance, relationships, and events all
emerge from that.

- **For:** maximum emergence. Relationships acquire genuine causal power, and the
  emergent stories the design doc promises are *actually* emergent rather than
  hand-authored consequences.
- **Against:** the most expensive option by a wide margin, and at 150-300 staff it
  generates noise faster than meaning — the player gets three hundred personality
  vignettes and no legible causal chain. Balance is close to intractable, and it makes
  the "why did that fail?" question unanswerable. Overkill for a management fantasy.

---

**Decision [D]: the C+B hybrid**, as specified above. C supplies the arithmetic — no
cliffs, no degenerate strategies, tuned in one place. B supplies the legibility — every
modifier visible, the binding factor named. The combination concentrates the hard part,
a good UI, where the hard part belongs.

The structural point that applies to all four, and to the hybrid in particular: **the
model must never let the player compute the answer by inspection.** A staffing decision
should always require a judgement about people under pressure, not a sum the player can
do in their head.

### 5.4 Lifecycle

Applicants → screening → offer → onboarding → assignment → development → promotion or
exit. Exits are varied and consequential: **resignation** (overwork, low morale, better
offer), **retirement**, **injury**, **illness**, **death**, **scandal**, **poached by a
rival agency**, or **promotion out of the agency into the state apparatus**.

**Hiring takes 2-6 weeks and the applicant is not yours until they arrive.** The player
can offer someone in March and have them start in May, and the person they want may
have taken another offer in the meantime. A hiring spree is therefore a race, and it
costs goodwill for every order in it (§3.5).

### 5.5 Development

Training courses, mentoring pairs, research sabbaticals, rotational postings, special
assignments, and stretch duties. Each costs money and a slot on someone's calendar, and
each has a payback time. The player will constantly have to choose between
*developing* a person and *deploying* them. **A course takes 3-9 months**, so
development is a decision whose payoff is years away and whose cost is immediate.

### 5.6 Astronauts

Astronauts carry the public relationship of the agency and carry disproportionate risk.
They accumulate flight hours, EVA hours, injury records, and fame. They age out. They
become national heroes, or national liabilities, or both. An astronaut dying on a
successful-but-costly mission is a completely different political event from one dying in
an avoidable bureaucratic failure — the sim should be able to tell those apart.

### 5.7 Relationships

A relationship graph over staff and politicians. It is the raw material for emergent
history: two rivals on the same team, a mentor whose student is now the rival, a
relationship that keeps a division from self-destructing, a friendship that turns into a
cover-up. Relationships should be **first-class state**, not flavour text, because the
design pillar is emergent stories.

---

## 6. Programmes

**Programmes are the strategic layer** — the unit the state funds, the public understands,
and the player is judged on. The design pillar is explicit: *the game revolves around
programmes rather than missions*.

### 6.1 Anatomy of a programme

- **Title, category, sponsor** (the political patron funding it), **status**.
- **Status [D]:** Proposed → Funded → Active → (Paused → Resumed) → Completed | Failed |
  Cancelled.
- **Budget envelope** and a burn rate per day, with a stated ceiling that overruns breach.
- **Staffing plan** — named leads, assigned teams, vacancies.
- **Objectives** — primary (mission-critical) and secondary (bonus), each with a
  **success criterion** and a **deadline**.
- **Milestones** — dated checkpoints the player sets and can move. Moving a date has a
  cost: political capital, staff morale, or credibility.
- **Risk register** — technical, personnel, schedule, political, budgetary. Risk is
  **visible, priced, and reducible** for money, time, or redundancy.
- **Visibility** — how much the public and the state are watching.

### 6.2 Programme categories

Orbital Exploration, Lunar Landing, Deep Space Probe, Earth Observation, Military
Reconnaissance, Commercial Launch, Space Station, Resource Extraction, Interplanetary
Transport, Megastructure, Interstellar. Categories carry different political, military,
and commercial weight — running a military programme draws a different kind of state
attention than a commercial one.

### 6.3 Failure is content

A failed programme must have an **aftermath**: a budget hole, a morale crash, an
investigation, a cancelled follow-on, a political enemy, a rival agency gleeful in the
press. Failure is not a loss screen; it is a branch in the story. The player should be
able to run a failed flagship and still be playing a decade later.

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
| Power | Batteries vs. fuel cells vs. radioisotope (long-duration vs. heavy) |
| Payload | What you carry vs. what you can afford to build |
| Life Support | Duration vs. complexity vs. mass |
| Avionics & Autonomy | Capability vs. complexity (more complexity, more failure modes) |
| Heat Shield | Reentry capability vs. cost and mass |
| Redundancy | Safety margin vs. cost and schedule |

Each slider has a floor imposed by **available technology** — a player cannot buy a
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
is the number that matters — the player sits and watches it happen, at whatever dilation
they have chosen.

| Phase | Duration | What happens |
|---|---|---|
| Preparation | 3 days to 3 weeks | Crew training, loadout, final checks, and the paperwork that is the real enemy |
| Launch and staging | 0-5 minutes | Ignition, ascent, and staging |
| Ascent | 5-20 minutes | The climb, and the first place something can go wrong |
| Coast to orbital velocity | 20-60 minutes | The long, quiet, dangerous part |
| Circularisation | 60-90 minutes | Insertion, and the moment the vehicle either arrives or does not |

**Preparation is where the game is played and the flight is where it is felt.** A launch
cannot be improvised: the vehicle must exist, the crew must be trained and rested, the
ground support must be operational, and the paperwork must be in order. The 90 minutes
of flight are the visible consequence of decisions made weeks earlier.

**The mission's phases are scheduled events**, not monthly ticks, and the player can
watch any of them happen at the base rate.

### 7.5 Outcomes

Failures should mostly be *partial* rather than cinematic: degraded performance, lost
data, delay, damage, injury, missed objectives, discovery of something unexpected.
Catastrophic loss is possible and probabilistic, and the player is warned - the risk
number is on screen before the launch is approved. **The player should always know what
they were risking.**

### 7.6 Industrial constraint

Vehicles cannot be conjured. Builds consume **industrial capacity** (personnel, factories,
materials, time). Ambition is throttled by what the agency can actually manufacture, and
the industrial base is itself a thing the player must build and maintain.

---

## 8. Research & Technology

**Technology emerges from investment and expertise** — this is a design pillar, so the
research model must not be a tech tree with purchases.

### 8.1 Disciplines

Propulsion, Structures & Materials, Life Support, Power, Avionics & AI, Robotics,
Communications, Medical, Planetary Science, Astrophysics, Computing, Manufacturing.

### 8.2 Research projects

A project has a principal investigator, a discipline, a target capability, funding,
assigned staff, a duration, and progress. Projects compete for the same people as
missions do — a brilliant scientist is either on the probe or in the lab, not both.

### 8.3 The central research tension [D]

> **Capability vs. reliability.**

- **Research** produces *new capability* — it unlocks better sliders, new destinations,
  new vehicle classes.
- **Testing** produces *reliability* — it reduces the risk of what the agency already
  knows how to build, and produces hard evidence.

Both cost money, time, and the same scarce staff. A player who only researches ships
shaky hardware; a player who only tests never escapes the current era. Every launch
decision is a bet on which side of this line the agency is on.

### 8.4 Emergent capability

Progress accumulates as **knowledge points per discipline**, plus **evidence** from
successful missions and tests. Capabilities unlock at thresholds. The player perceives a
rising ladder — atmospheric, orbital, lunar, planetary, stations, interplanetary,
megastructural, interstellar — but the underlying mechanism is continuous and emergent,
not a list of purchases.

### 8.5 Knowledge and discovery

Missions return data. Data unlocks research directions, reveals the solar system, and
occasionally reveals something about the *civilisation itself* — the setting is a
procedurally generated alien civilisation, and its own artifacts and history are
discoverable. Discovery is a first-class reward loop.

### 8.6 Knowledge output [D]

Every research result and every successful mission's data is a **live choice** the player
makes, and the same result can be handled differently each time it comes up:

| Choice | Immediate effect | Strategic effect |
|---|---|---|
| **Publish openly** | Reputation up, the world advances | No exclusivity, no income — and rivals get faster too |
| **Patent and license** | Income begins, exclusivity held | Slower cooperation, possible patent disputes |
| **License to a rival** | Immediate cash now | The rival advances, possibly past the player, but the agency is funded |

The choice is made **per result**, and it interacts with the rival race: publishing
speeds everyone up, patenting slows the world down, and licensing a rival is a way to
fund a programme by giving away a lead. Publishing or licensing while the agency is
broke is a real, and occasionally correct, decision — which is exactly why it has to be
a live choice rather than a policy set once at the start.

---

## 9. Infrastructure & Industry

**All infrastructure is built. Nothing is abstract capacity.** Every lab, stand, plant,
complex, and depot exists because the player put it there, on a site, with a
specification. There is no "you have three units of lab capacity" abstraction, and no
facility exists that the player did not build.

### 9.1 A facility

| Field | Notes |
|---|---|
| Name & type | Laboratory, test stand, fabrication plant, launch complex, mission control, tracking station, training centre, observatory, depot, refinery, headquarters |
| Site | Named location. Affects logistics, security, political exposure, environmental risk |
| Specification | The result of its construction choices (below) — output, reliability, staffing need, upkeep |
| Condition | Degrades over time; affected by maintenance spend and staff attention |
| Staffing | Named staff assigned; understaffed facilities run below specification |
| Output | Feeds research, builds, missions, training, or data |
| Upkeep | A daily cost, rising with age and with material quality |

### 9.2 Construction choices

A facility's final specification is **not a preset**. The player chooses how to build it,
and the build method determines how it behaves for the next forty years. The option set
is small and **reusable across facility types**, so the player learns a handful of
decisions rather than dozens of bespoke sliders:

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

Two labs built with the same money and different methods will behave differently for the
rest of the campaign. This is where the "you built it, you live with it" feeling comes
from, and it is why early construction decisions are the ones that shape the late game.

**Each construction choice must therefore buy something and cost something.** *Robust*
builds are slow and expensive but do not fail catastrophically. *Experimental* builds are
cheap and fast and may unlock capability nobody else has — or may be unusable.
*Fully redundant* facilities cost double and free the agency from a whole class of
disaster.

### 9.3 Construction realities

- Construction is **routinely late**. The build takes as long as it takes, and the
  schedule is a forecast, not a promise.
- Construction draws **political scrutiny**, scaled by cost and by how visible the site is.
- Construction consumes **staff** — a build team is people the player does not have on
  programmes, so a big build in the same year as a big launch is a genuine conflict.
- Facilities **degrade**. Neglect produces accidents that get blamed on people, which is
  exactly the kind of unfair, emergent story the design wants.

### 9.4 Industrial capacity

Total industrial capacity is the ceiling on how much the agency can build per year, and
it is the sum of the player's own facilities plus whatever it can contract from industry.
Ambition is throttled by what can actually be manufactured, and the industrial base is
itself something the player must build and maintain.

### 9.5 External supply

Some capacity can be bought from contractors and commercial partners rather than owned.
Rented capacity is faster and cheaper up front, and carries exclusivity clauses,
delivery penalties, and dependencies on suppliers the player does not control.

---

## 10. Money

The budget is not a number the state hands out. It is the **output of several
interacting parameters**, which is what makes it a management problem rather than a
slider.

### 10.1 Revenue parameters

| Source | Behaviour |
|---|---|
| **Civilisation base** | The floor. The state can only fund what the civilisation's wealth and industrial maturity can support. Rises across the campaign as the civilisation grows; falls in a depression or a war. |
| **Political standing** | A multiplier on the base: approval, faction support, and the state's current priorities (military vs. civil vs. commercial) all move the appropriation. |
| **Investors** | Private capital committed to specific things, in exchange for a revenue share, IP rights, delivery penalties, or exclusivity. A source of cash and a source of obligation. |
| **Discoveries** | Major discoveries create new value: resource deposits, biotech applications, sellable data, salvage, naming rights, patent income. One good discovery can fund a decade. |
| **Commercial activity** | Launch services, hosted payloads, subscription science data, industrial contracts. |
| **Strategic grants** | One-off state funding for prestige objectives, usually attached to a milestone and a date. |
| **Debt** | Emergency borrowing, repaid with interest, available only when the base allocation has already failed. |

The exact parameterisation and weights are balance work (`BALANCE.md`), but the **list of
parameters is settled** — every source above must be able to move independently, so the
player can always trace *why* the budget is what it is.

### 10.2 Outgoings

Salaries, programme spend, construction, contracts, research, maintenance, debt service,
and the political cost of a budget that is too large.

### 10.3 The budget crisis ladder [D]

Running the treasury dry is not a game over; it is a **defined escalation ladder**:

1. **Warning** — a funding shortfall is projected. The player sees it coming in the
   forecast horizon and can act.
2. **Austerity** — the state imposes a cut. Unfunded programmes are throttled.
3. **Borrowing** — emergency loans at punitive rates, with public and political cost.
4. **Attrition** — unpaid staff leave. Senior staff leave first, and the ones who leave
   take knowledge with them.
5. **Cancellations** — programmes are terminated. A flagship is the obvious candidate,
   and the cost is measured in milestones that will now never be met.
6. **Collapse** — the agency cannot meet its obligations. The campaign ends, but the
   history is kept.

The design doc promises "a budget crisis cancels a flagship mission". The ladder is how
that promise is kept mechanically, and how it stays a drama instead of a coin flip.

### 10.4 Escalation

Costs inflate across a multi-decade campaign, and the state does **not** automatically
keep pace. Whether the appropriation tracks inflation is itself a political question the
player may have to fight.

---

## 11. Politics & the State

The agency answers to someone. The state is an NPC with its own personalities, factions,
and memory.

### 11.1 The political body

- **Ministers and factions**, each with an agenda: militarisation, pure science,
  commercial expansion, austerity, settlement, exploration at any cost.
- **Approval / trust** with the state, distinct from public approval. The state controls
  the appropriation; the public controls legitimacy. A player can have one and not the
  other, and that is a distinct and interesting position.
- **Political capital** is a spendable currency: lobbying, favours, patronage, and
  choosing which promises to keep.
- **Terms and elections.** A change of government can cancel a programme, defund a
  division, or unlock a previously forbidden one.

### 11.2 Promises

The player will be asked for things: a launch date, a milestone, a demonstration, a
headline. A **promise tracker** makes them visible and accountable. Broken promises are
remembered by the specific people who heard them.

### 11.3 Military tension

"Military Reconnaissance Programme" is named in the design doc, so militarisation is a real
axis. A military programme may unlock state support and prestige while alienating the
public, and it may attract oversight that civil programmes do not.

---

## 12. Public & Media

- **Public approval**, **prestige**, **national pride**, and **fear** (how alarmed the
  public is by anomalies, disasters, or alien threats).
- **Media outlets** with their own stances and stories. A hostile outlet is a persistent
  pressure; a friendly one is a tool.
- **Heroes and villains.** Named individuals become symbols — the first to orbit, the
  survivor, the whistleblower. Fame is a stat; so is disgrace.
- **Scandals and cover-ups.** The player may choose to bury a failure. It works, for a
  while. Exposure later is a real event with a real cost.
- **Ceremonies** — state funerals, parades, first landings — that let the player spend
  accumulated prestige.

---

## 13. The Wider World

### 13.1 Scope: microbial alien life only [D]

There are **no sentient alien species, no alien artefacts, no ruins of other
civilisations, and no first contact.** The setting's "alien civilisation" is the
*player's own* civilisation — exotic to any outside observer — and the game's
extraterrestrial biology is **bacterial**: single-celled life on other worlds.

This is a large scope reduction, and it buys something valuable. Microbial life makes
**planetary protection** a first-class mechanic:

- **Sample return** brings alien biology back. A contaminated return is a catastrophe
  with no undo button — and the risk is knowable but never zero.
- **Astromicrobial research** is among the most valuable research directions in the
  game, and the ethical and political pressure around it is permanent.
- **Contamination incidents** can end a programme, trigger a political crisis, and
  reshape the public's view of the agency for decades.

Every destination in the game is a rock, a ball of gas, an ice, or a world with bacteria
in it. That is enough.

### 13.2 Rival agencies [D]

Rivals exist, they are not enemies, and they can be **turned off**.

- **What they compete for:** the state funding pool, commercial contracts, and
  **milestone primacy** — being first to orbit, first to land, first at the outer
  planets. A milestone won by a rival is worth a great deal to them and noticeably less
  to the player, which is a clean, non-violent reason to be fast.
- **What they do not do:** no sabotage, no espionage war, no hostility of any kind.
  They are colleagues with the same goals and a smaller budget.
- **How they interact:** joint missions, competition for talent, competition for
  suppliers, and the occasional licence deal. The player may also ignore them entirely.
- **The toggle:** a game option. Rival agencies can be disabled entirely for a
  single-agency sandbox, where milestones still grant bonuses but nobody is racing the
  player for them. The option may also set how many rivals exist.
- **Implementation note [?]:** whether rivals run on the same full simulation as the
  player, or on a lighter progression model, is an architectural decision with a large
  performance impact. See the open decisions table.

### 13.3 Suppliers and partners

Industry relationships the agency can lean on or be held hostage by. Private investors
arriving with money also arrive with expectations.

---

## 14. Randomness & Events

**Event categories:** technical failure, personnel incident, discovery, political,
financial, environmental, public, external.

**Events happen at instants, not on a schedule [D]:** the simulation schedules an *event
check* at a future instant, draws at that instant, and schedules the next check. The base
interval is **one simulated week**, and most checks produce nothing — the agency is
usually quiet. A stress or crisis period tightens the interval, so a run of bad luck
becomes a run of bad luck rather than one event a week.

**Context weighting [D]:** events are not uniformly random. They are drawn against the
current state of the agency - a tired crew, a stressed project, a neglected facility, a
thin treasury, a hostile press, an overextended calendar. Bad conditions raise the odds
of bad events. The sim is fair: outcomes follow from the player's own management.

**Determinism [D]:** the entire simulation runs on a seeded RNG. Given the same seed and
the same decisions *at the same simulated instants*, the campaign plays out identically.
This is required for testing, for sharing scenarios, and for a replayable "the road not
taken". **The dilation is not part of this** — a campaign played at the base rate and the same
campaign played at the top rate are the same campaign.

### 14.1 Event visibility [D]

The player sees **both** directions in time, and they are deliberately not symmetric.

**History - the chronicle.** A complete, permanent, queryable record of everything that
has happened: launches, accidents, discoveries, promotions, arguments, marriages,
resignations, political falls. Filterable by year, category, programme, and person.

This is not a log. It is one of the game's most valuable surfaces, because it is where
the emergent history becomes *readable* - the player can go back to the day their best
engineer quit and see what it cost them over the following five years. Major events also
become permanent entries in the agency's **legend**, and the player builds a history that
reads like a history.

**Forecast - the timeline.** A rolling horizon of roughly the next few weeks of
simulated time, showing everything the player *knows is coming*:

- milestones and committed instants falling due
- launches authorised, and their windows
- research projects due to complete
- construction due to finish
- orders the player has scheduled but which have not yet taken effect
- contracts expiring or renewing
- political windows, budget reviews, elections
- staff returning from assignments, training, or sabbatical
- the known risk exposure on each of the above

**The forecast is honest but not omniscient.** It shows what is *scheduled*. It does not
show what will go wrong, and it does not show the outcome of any risky action — only its
known risk figure. The player knows a launch is on the calendar and that it carries a
14% failure risk; the player does not know that the third stage will fail in March.

This is the line that keeps the forecast useful without removing tension: **the player
always knows their obligations, never their fate.**

---

## 15. Derived, not managed

Computed by the simulation and shown to the player, but not directly set:

Mission success probability · failure severity and shape · time-to-completion ·
knowledge gain rate · technology unlock thresholds · morale and loyalty drift ·
staff performance · health outcomes · public and political opinion change · escalation of
interest from rivals and rivals' progress · industrial throughput · event draw weights ·
difficulty pacing · the feasibility of a promise.

---

## 16. Deliberate non-goals

Scope control, stated explicitly so it is not quietly undone later:

- **No orbital mechanics.** No trajectories, thrust profiles, or fuel physics.
- **No turn-based structure.** The game is not played in rounds. There is no month to
  resolve, no "end of month" button, and no fixed turn. The player acts whenever they
  like, and the simulation runs on.
- **No component-by-component engineering.** Sliders only.
- **No sentient alien life.** No other minds, no artefacts, no ruins, no first contact.
  Extraterrestrial biology is bacterial, and that is the whole of it.
- **No conflict with rivals.** Rival agencies compete for funding and primacy, and stop
  there. No sabotage, no war, no hostile action.
- **No precise real-world solar system geography.** Destinations are evocative and
  roughly ordered by difficulty, not surveyed.
- **No real-time combat.** The game is about programmes, not firefights.
- **No sub-individual micromanagement.** Individual work is managed at the team and
  programme level; individual lives matter only where they affect the work.
- **No spreadsheet simulator.** Budgets are strategic allocations, not a 500-row ledger.
- **No single objectively correct strategy.** Every pillar has a real cost.
- **No order cap.** The player may issue as many orders as they like. The cost of
  spamming is money and goodwill, not a rationing meter (§3.5).

---

## 17. Open decisions

Six questions remain from revision 2, and revision 3 added four of its own. **Four
of the ten have since been answered**, and they are listed at the bottom:

| # | Question | Why it matters | Blocks |
|---|---|---|---|
| 1 | Do rivals run the full player simulation, or a lighter progression model? | A full rival sim is expensive and raises the question of whether the sim is multi-agent by design | Architecture, performance |
| 2 | What obligations do private investors impose? (revenue share, IP rights, delivery penalties, exclusivity, board seats) | The investor revenue source is useless without a cost attached to it | Economy, politics |
| 3 | How does planetary-protection contamination resolve - instant campaign-ending, or a survivable crisis with lasting scars? | Sets the stakes of the microbial-life pillar | Events, risk model |
| 4 | Is "license to a rival" a permanent third option, or a one-off desperation mechanic? | Affects the knowledge economy and how rivals are modelled | Research, world |
| 5 | Exact revenue parameterisation: how much does each source in §10.1 weigh? | Balance across a 40-year campaign | Balance |
| 6 | Does the state adjust appropriations automatically to match inflation? | Decides whether escalation is a political fight or just noise | Economy |
| 7 | **Is goodwill a single meter, or separate ones for the state, the public, and the staff?** | One meter is legible; three are truer and much harder to read. Low goodwill is the anti-spamming mechanism, so its shape matters | Orders, politics |
| 8 | **How does a scheduled order behave if its target has changed by the time it lands?** — the project was cancelled, the person left, the vehicle was scrapped | Scheduled orders are a new mechanic and this is its main failure mode | Orders |
| 9 | **Does a launch window have to be scheduled in advance, or can the player authorise at the last moment?** | Determines whether the player plans a mission months ahead or manages it in the moment | Missions |
| 10 | **What is the smallest useful dilation step for a player who wants to watch a ninety-minute flight?** | At the base rate it is ninety seconds of real time, which is right; a slower step would be a lifetime | UI, time model |

**Closed by the implementation, since revision 3:**

- **Question 7: goodwill is a single meter.** One number, from −100 to +100, on the
  dashboard. Three meters would be truer and much harder to read, and the anti-spamming
  job is done by the scale of the effect rather than by the number of meters.
- **Question 8: an order is validated twice** — when it is issued and when it lands.
  One that can no longer be honoured is refused at landing, the cost is not charged,
  and the player is told why. `SIMULATION.md` §4.1.
- **Question 9: a launch is authorised at the moment the player chooses.** The
  authorisation order carries the flight time as its lead time, so the window is
  the flight itself rather than a date the player has to pick. `SIMULATION.md` §4.2.
- **Question 10: sixty times is the slowest rate there is.** One second in life is one
  minute in game, so a ninety-minute flight is ninety seconds of real time at the base
  rate, and there is nothing slower to get in the way of a player who wants to watch it.
  `MANAGEABLE_PARTS.md` §3.2.

**Closed in revision 3:** continuous time with player-chosen dilation (§3.2) · real-time
interaction with a pause and scheduled orders (§3.3) · orders take time to implement
(§3.4) · orders cost money and goodwill, with no cap (§3.5) · a launch to a stable orbit
takes 60-90 minutes (§7.4) · events fire at instants on a weekly check (§14).

**Closed in revision 2:** hard *and* soft deadlines (audience-based, §3.1) · both historic
and forecast event visibility (§14.1) · all infrastructure built, with construction
choices shaping specifications (§9) · budget driven by multiple parameters - civilisation
base, investors, discoveries, and more (§10.1) · publish/patent/license as a live
per-result choice (§8.6) · bacterial alien life only, no first contact (§13.1) · rival
agencies that compete for funding and milestones and can be switched off (§13.2) · the
staff performance model — the C+B hybrid, with its arithmetic and its UI contract
specified in §5.3.

---

## 18. Related documents

- `DEVELOPER_INTRO.md` — the original vision and pillars this document expands.
- `PLAYER_INTRO.md` — the in-fiction framing, and the source of the "Director of the
  Stellar Expansion Authority" premise.
- `README.md` — index of all documentation, and the plan for what comes next.
