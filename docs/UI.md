# UI.md

> **The terminal interface.** What the player sees, what they can do, and — mostly —
> how anything is explained.
>
> **Status:** Design intent, revision 2. The interface is a **live console**, not a set of
> pages. The game runs; the player looks at it and acts.

---

## 1. Principles

1. **Legibility is the whole game.** A management sim's value is entirely in whether
   the player can *understand* the situation. A beautiful screen that hides a
   relationship is worse than a plain one that shows it.
2. **Always show the clock and the rate.** The date, the time, and the dilation are
   never more than a glance away. A player who does not know how fast time is passing
   does not know what a decision costs.
3. **Never make the player do arithmetic.** The screen computes what it can and names
   the constraint. The player should never add two numbers to find the answer.
4. **Explain with words, not just numbers.** A bar is not an explanation. "Held back by
   fatigue" is.
5. **Colour is a reinforcement, never the message.** Every colour-coded state also has a
   glyph or a word, so the interface survives a monochrome terminal and a colour-blind
   player.
6. **The forecast shows obligations, not fate.** The player knows what is coming; the
   player does not know what will go wrong.
7. **Every order shows its price before it is confirmed** — the lead time, the money,
   and the goodwill.

---

## 2. The shell

Every screen sits inside the same frame: a **top bar**, a **body**, a **hint line**
for the tutorial, and a **key ledger** at the very bottom.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ STELLAR EXPANSION AUTHORITY  2041-06-14 09:32 │   1000x  │ Treasury 349k  │
│ net −38k/day  approval +12  goodwill +18  3 due                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Top bar field | Meaning | When it changes colour |
|---|---|---|
| Agency name | The player's agency | Never |
| Date and time | The current instant, to the minute | Never |
| **Dilation** | The current rate, or `paused` | Never |
| Treasury | The balance | Red below one day of burn |
| Net | The daily burn rate, signed | Red when negative |
| Approval | The state's approval | Red when negative |
| Goodwill | The institution's patience | Red below -40 |
| Due | How many things need attention | Yellow when anything is at risk |

**The net is a daily rate, not a monthly one.** The game runs in real time, so the
player thinks in "per day" - and the number is small enough to read at a glance.

**The key ledger is always on screen [D].** One line, at the bottom, on every
screen, never changing height:

```
 o order  space pause  + - speed  1-7 screens  F1 keys  F2 manual  q quit
```

A player should not have to remember the keymap, and should not have to open it to
find the one key they were told about. The ledger lists the *common* keys only; the
`F1` modal is the full list. **The keys in the ledger are the ones a player reaches
for in the first ten minutes** — the order, the clock, the screens, the keymap, the
manual and the exit — and nothing else earns a place in it.

**The clock shows the campaign's effective time.** The simulation integrates its
continuous quantities on a fixed one-hour grid, and the grid is an implementation
detail - **the player's clock moves smoothly, and the minute is not always zero.**
What the top bar shows is where the campaign has actually reached.

**The dilation is in the top bar, always.** It is the control the player uses most, and
it must never be something they have to go looking for. When the clock is paused the
field reads `paused`, and it is bracketed so it is obvious at a glance.

| Top bar field | Meaning | When it changes colour |
|---|---|---|
| Agency name | The player's agency | Never |
| Date and time | The current instant, to the minute | Never |
| Pause indicator | Whether the clock is stopped | Toggled by the pause key |
| Dilation | The current rate, always visible | Never |
| Treasury | The balance | Red below one day of burn |
| Net | The daily burn rate, signed | Red when negative |
| Approval | The state's approval | Red when negative |
| Goodwill | The institution's patience | Red below −40 |
| Due | How many things need attention | Yellow when anything is at risk |

**The net is a daily rate, not a monthly one.** The game runs in real time, so the
player thinks in "per day" — and the number is small enough to read at a glance.

**The dilution is in the top bar, always.** It is the control the player uses most, and
it must never be something they have to go looking for.

---

## 3. Navigation and controls

| Key | Action |
|---|---|
| `1` … `7` | Jump to a screen |
| `Tab` / `Shift-Tab` | Cycle screens |
| `↑` `↓` | Move the selection, or change the chosen value |
| `←` `→` | Move between the fields of a form |
| `Enter` | Open the selected thing, confirm an order, or fork at the selected instant |
| `Esc` | Back, or close what is open |
| **`Space`** | **Pause and resume the clock** |
| **`+` / `-`** | **One dilation step faster or slower** |
| **`Ctrl+1` … `Ctrl+9`** | **Set the dilation directly** |
| `o` | **Open the order palette** |
| `f` | Jump to the feed — the live feed, Slice 1-37's successor, not yet |
| `w` | Jump to "what needs attention" — the live feed, Slice 1-37's successor, not yet |
| **`F1`** | **The keymap, in the game** |
| `?` or shift-`/` | The keymap, on layouts that have them |
| **`F2`** | **The manual — everything the game explains** |
| **`I`** | **The manual, for this screen only** |
| `/` | **Filter the list** |
| `s` | **Change how the list is sorted** |
| `←` `→` | The shift, on the staff screen; the fields of a form, elsewhere |
| `q` | Quit (with a save prompt) |

**The keymap is on `F1`, and `F1` is the same key on every layout [D].** A
function key is reported by the terminal from the hardware, not from a character
table, so it cannot be in a different place on someone else's keyboard. `?` and
`/` are also accepted, because on a Nordic layout the `?` is a shifted `/` and a
terminal may report either the shifted character or the unshifted one with the
shift modifier — but they are the *fallback*, not the primary.

**A binding must never depend on a shifted character [D].** Every layout puts
`?` behind Shift, and Shift is a second key that can arrive as its own event.
Where a key has to work everywhere, it is a function key, a control key, or a
punctuation key that is unshifted in the common layouts.

**Only a press is an action [D].** On Windows the terminal reports *both* the press
and the release of every key, and acting on both makes every key fire twice — which
a player sees as "it jumps two steps", and which makes the keymap flicker open and
shut again. The release is ignored.

**A modal that is not drawn is not a modal [D].** The keymap once set a flag and
nothing rendered it, and the only test checked the flag — so the keymap was
"working" and invisible at the same time, and the player's key did nothing. A
modal has to be tested by *rendering* it and reading the text back, and a modal
that is sized to a percentage of the screen has to be sized to its content
instead, or it is clipped in a small terminal.

**There is no month to end.** The old model had an `Enter` key that resolved a
month; the continuous model has no month to resolve. The clock runs the campaign
whether or not the player is touching anything, and `Space` is how the player stops
to think.

**Two of these keys are not there yet [D].** The live feed and the "what needs
attention" screen are Slice 1-37's successors. The top bar already carries the count
of what needs attention, so the player is not left without the information — but the
keys are promised in the design and are not in the build, and this document says so
rather than implying otherwise.

### 3.1 The tutorial [D]

The tutorial is a **set of contextual hints, not a sequence**. Each hint belongs to a
screen and appears the first time the player is on that screen, and never again. A
player who goes straight to the research screen sees the research hint first — the
tutorial does not insist on an order, because a tutorial that insists on an order is
a tutorial that gets in the way.

**A hint stays until the player acts [D].** The hint is marked as read when the player
presses a key, not when the screen is first drawn — a hint that is marked as seen
when it is first asked for is on the screen for one frame, sixteen milliseconds, and
a hint nobody read is not a hint.

**The hints live in the footer**, which never changes height, so the body never
jumps. **When there is no hint waiting, the footer carries the info line** — the
one-line explanation of the screen the player is on. The tutorial introduces a
screen; the info line then explains it for the rest of the campaign.

| Screen | The hint |
|---|---|
| Dashboard | "Three scientists, one project, and a budget that will not last. Press 2 to look at them." |
| Staff | "This is why your team is slow. The panel names the thing that is holding them back — and it is not always the same thing." |
| Dashboard | "Press o to give an order. You will see what it costs before you commit to it." |
| Research | "Your project advances at the rate your team can sustain. A tired team finishes late." |
| Dashboard | "Time passes whether you are ready or not. Press Space to pause the clock, and Ctrl+1 to Ctrl+9 to set how fast it runs." |
| Chronicle | "Everything that happens is recorded here. The agency keeps a history, and so does the state." |
| Branches | "You can fork the campaign at any decision you have made, and watch the two branches play out. Press Enter to fork at the selected instant." |

**The staff hint is the one that matters.** The binding-factor widget is the
highest-risk component in the interface — it is the one panel the player has to
understand to understand the game — and a panel nobody looks at teaches nothing.

**`Space` pauses. This is not a convenience** — a management game in which you cannot
stop to think is a management game you cannot play, and a game in which you cannot
stop is a game in which you are losing to the clock rather than to the decisions.

**The clock controls are global**, because the clock belongs to the host and not to any
screen. They work the same on every screen.

**`Ctrl+digit` is used rather than `Shift+digit`** because the shifted character is
layout-dependent and the control modifier is not. `+` and `-` are the everyday case;
`Ctrl+digit` is for when the player wants a specific rate at once.

**Screen numbers are fixed [D]:** staff is always `2`, programmes always `3`. A player
who learns one screen's number keeps it.

---

## 4. The screens

The screens are **live views**. Nothing is a snapshot; every panel shows the state as of
the last simulation instant, with rates and projections beside the values.

### 4.1 Dashboard (`1`)

The screen that answers "where do I stand right now", and the one the player looks at
constantly.

```
┌─ Dashboard ──────────────────────────────────────────────────────────────────┐
│ HEADLINE   The Orbital Launch is 4 hours from its committed instant.         │
│                                                                             │
│ NEEDS ATTENTION                                                           │
│  ⚠  Orbital Launch — committed in 4h          slack 2d  risk 14%             │
│  ⚠  Treasury — 4 days of burn remaining                                     │
│  ○  Vance returns from the test campaign in 3 days                          │
│  ○  Life Support project — 62% done, 4 months at the current rate           │
│                                                                             │
│ IN FLIGHT                                                                  │
│  ●  Orbital Launcher 2 — ascent, 41 minutes remaining                       │
│  ●  Propulsion Lab — building, 62% complete, due 2041-08-14                   │
│                                                                             │
│ FINANCE   income 4.0k/day   outgoings 4.6k/day   net −0.6k/day   runway 4d  │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **The headline** is one sentence, generated from the most pressing state, and it is
  the first thing the player reads. It is *the thing that needs deciding*.
- **Needs attention** is the panel that replaces the old month report. It is sorted by
  urgency, and every line links to the thing it is about.
- **In flight** shows the work that is happening right now — the missions climbing, the
  buildings rising, the projects advancing. At the base rate this is the panel the player watches.
- **The runway** is the number that turns a spreadsheet into a survival clock.

### 4.2 Staff (`2`)

```
┌─ Staff ─────────────────────────────────────┬─ Detail ──────────────────────┐
│▸ Bhatt, Anika      Scientist      29    0   │ Anika Bhatt — Scientist       │
│  Okonjo, Marsha    Scientist      41    0   │ Unassigned                     │
│  Reyes, Tomas      Engineer       35    0   │ morale 20  fatigue 0  health… │
│                                              │ technical 64  creative 71     │
│ filter               sort name  s to change  │ physical 48  age 29           │
│                                              │ specialisms: Life support      │
│                                              │ salary 1,200.00               │
│                                              │ shift 100%  ← → to change     │
│                                              │ output 100%  fatigue 6%/month │
│                                              │ leaves this month: 0.25%      │
│                                              ├─ Performance ─────────────────┤
│                                              │ SKILL       ▇▇░░░░░░░░   +21  │
│                                              │ ROLE FIT    ▇▇░░░░░░░░   -25  │
└──────────────────────────────────────────────┴──────────────────────────────┘
```

**The right panel is the most important thing in the game** (`MVP.md` §6). It has four
jobs:

1. **State** — morale, fatigue, health, fame, in bars with numbers.
2. **Performance, decomposed** — the modifier breakdown.
3. **The binding factor, named in words** — *"Held back by FATIGUE"*.
4. **The projection** — what the work they are on will take at the current rate.

**The columns are the ones the player decides with** — name, role, age, fatigue, and
**the chance they leave this month**. The risk is in the warning colour when it
matters, so a person about to quit is findable without reading the number.

#### 4.2.1 The shift [D]

**The shift is the player's throttle on a person.** A hundred percent is a full load.
Below it the person contributes less and tires less; above it they contribute more
and tire much more, and the extra effort is worth less than the extra fatigue it
costs.

| Shift | Output | Fatigue | Morale |
|---|---|---|---|
| 0% | nothing | recovers, as if unassigned | unaffected |
| 50% | half | recovers, while still on the team | unaffected |
| 100% | full | the ordinary rate | unaffected |
| 150% | 130% | one and a half times the ordinary rate | −7.5% |
| 200% | 160% | twice the ordinary rate | −15% |

**The old way of easing off was to unassign somebody**, which costs the agency the
person, their place on the team, and their output all at once. The shift costs none
of that, and it is a single keystroke on the screen the player is already looking
at.

**The shift is an order, like everything else the player does.** It lands at the next
hour — a shift change is a rostering decision, not a project — and it costs no money
and no goodwill, because the player must be able to make a hundred of them. The
detail panel shows the change in flight as `100% → 110% (pending)`, so a player
waiting for it can see it coming.

**The arrows move the selected person's shift**, and they stack: the second press is
on the shift already in flight, not on the one that landed.

#### 4.2.2 Filtering and sorting [D]

**Every list is filterable and sortable.** An agency with three hundred people has a
staff list that cannot be read top to bottom, and a player who has to scroll to find
the one person about to resign has a screen that does not do its job.

- **`/` starts filtering.** It is a *mode*, not a side effect: while the player is
  typing, the keys go into the filter and nowhere else, so a filter can contain a
  `q` without the game quitting and a `1` without the screen changing. `Enter` keeps
  the filter and stops the typing; `Esc` abandons it.
- **`s` changes the sort**, cycling rather than opening a menu, because a menu to
  choose a sort is a menu to dismiss.
- **The sort key is named on the screen**, and the filter is shown next to it, so a
  player who cannot see that a filter is in force cannot tell why the list is short.
- **The staff list sorts by name, age, role, fatigue, attrition risk, and shift.**
  The risk is the one that matters: a person about to leave is a person the player
  can still save.
- **The filter is per screen and it survives a trip elsewhere.** A player who has
  filtered the staff list down to the people in trouble, and who goes to look at a
  project and comes back, should find the filter still in force.

### 4.3 Programme (`3`)

**The programme screen is the strategic view, and it is where the slack is.**
Each programme shows its objectives with both instants side by side, the slack
between them, the progress, and **the projection** — when the work would finish at
the rate the programme is running at now.

**The slack is a duration, and it is the number the player manages.** It is
consumed by things going slowly, and the player buys it back with money and time.
The slack meter shows what is left of the gap between the plan and the promise, and
it is the one number on the screen that gets worse on its own.

```
● Reach orbit.
     target    2040-07-01          the agency's own plan, in dim type
     COMMITTED 2041-07-01          what the state was promised, in bold red
     progress 41%   12 months of slack   promised to the state
     at this rate it finishes 2041-02-14
```

**The projection is a plan, not a promise.** It is what happens if nothing changes,
and it is computed from the same rate the simulation applies, so the projection
and the progress cannot disagree.

### 4.3.1 The programme orders [D]

| Order | What it does |
|---|---|
| **Open a programme** | A body of work with its own budget, its own people, and its own objectives |
| **Set a deadline** | A dated objective on a programme. The target is half the horizon and the promise is the whole of it, so the player is given a slack rather than being asked to invent one |
| **Set a budget** | What the programme gets each month |
| **Push a deadline back** | Renegotiating, which costs political capital and goodwill — and the state is the hardest to ask |

**Every parameter is a choice from a list**, because the palette has no free text
and no date picker. A deadline is a horizon, and the audience is who it was
promised to.

**Who holds the deadline decides what breaking it costs.** Missing a political
deadline costs 18 points of approval; missing an internal one costs nothing
outside the agency.


```
┌─ Orbital Launch Programme ──────────────────────────────────────────────────┐
│ status: Active   lead: Okonjo   visibility 68   sponsor: The State Council  │
│                                                                             │
│ OBJECTIVES                                                                  │
│  ●  Primary   Reach orbit and return a capsule intact                       │
│               target 2041-07-02   COMMITTED 2041-07-05  slack 3 days        │
│               71% done · 4 months left at the current rate                   │
│  ○  Secondary Demonstrate a re-entry heat shield                             │
│               target 2042-01-01   COMMITTED 2042-06-01  slack 5 months      │
│               12% done · 17 months left at the current rate                  │
│                                                                             │
│ BUDGET   2.40M allocated   1.86M spent   0.31M committed   0.23M remaining  │
│ STAFF    9 assigned   2 vacant   morale 61  ⚠ 2 fatigued                    │
│ RISK     14% launch failure   ↑ 4% this month                                │
│                                                                             │
│ MILESTONES                                                                  │
│  2041-05-02  Vehicle complete        ✓ met                                    │
│  2041-07-02  Ground test             ○ due                                    │
│  2041-07-05  Launch                  ○ committed                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**The two dates are always shown together and never on the same row** — the target
above, the commitment below, with the slack between them. This is the game's central
idea, and the layout has to make it impossible to misread.

**`COMMITTED` is uppercase and coloured**; the target is dim. A commitment is a
promise; a target is a plan. The typography carries the distinction.

**Every date is a full date and time**, because a commitment four hours away and one
four months away need to be told apart at a glance.

### 4.4 Research and Facilities (`4`)

**The facilities panel is on the research screen**, and it is the screen where
the agency looks after what it has built.

```
✓ Propulsion Lab
    Laboratory   condition 84%   upkeep 4,000.00   staffed 0/2
    output 3   quality 65   reliability 70   places 4
  capacity 1 working, 4 places   room to spare
```

**The condition is what the upkeep and the staff are for.** A facility short of
staff loses four percent of its condition a month, and a worn facility produces
less — the output shown is the output at the current condition, not the
specification it was built to.

**The capacity line is the reason a bigger laboratory is worth building.** Every
member of staff working at once needs a place to work, and going over capacity
falls in the **tools** factor, which multiplies everything else down.

Two panes side by side.

```
┌─ Research ───────────────────────┬─ Facilities ──────────────────────────────┐
│  Propulsion                        │  Launch Complex North        ✓ online  │
│  ● Liquid upper stage    71%       │    method Robust · scale Large        │
│    PI Okonjo · 4 staff · 1.9k/day │    output 2/yr · reliability 82        │
│    4 months left at current rate  │    upkeep 41k/mo · staff 12 · cond 94  │
│  ○ Heat shield plating    22%      │                                       │
│    PI Reyes · 2 staff · 0.8k/day  │  Propulsion Lab 2        ⚠ building    │
│    9 months left at current rate  │    method Standard · scale Small      │
│                                   │    progress 62% · due 2041-08-14     │
│  Life Support                      │                                       │
│  ● Closed-loop systems   48%       │  Test Stand West          ✓ online     │
│    PI Bhatt · 3 staff · 1.4k/day  │    output 1/yr · reliability 71        │
│    6 months left at current rate  │                                       │
│                                   │                                       │
│  TESTING                           │                                       │
│  ● Heat shield — ground campaign  │                                       │
│    3 staff · 0.65k/day · risk −6% │                                       │
│    ⓘ 1 test would cut this launch's failure risk by 5 points                │
└───────────────────────────────────┴────────────────────────────────────────┘
```

**The ⓘ line is the capability-versus-reliability decision, surfaced.** It tells the
player what a test would cost and what it would buy, so the choice is informed rather
than mysterious.

**A building's method and scale are shown on the facility card** — the player sees their
own decision from three years ago, every time they look at the building.

### 4.5 Vehicles and Missions (`5`)

**A vehicle is the player's decision made of parts.** The screen shows the sliders
and the numbers that come out of them, because the sliders are the decision and
the numbers are the consequence.

```
  Orbital Launcher 1 - Orbital launcher
    Propulsion     ▇▇▇▇▇▇▇░░░   80
    Payload        ▇▇▇▇▇▇░░░░   60
    ...
    costs 1,240,000.00   reliability 71%   capacity 2400   delta-v 1240
    needs Basic re-entry
```

**Two things bound a slider.** The archetype says what kind of machine it is — a
sounding rocket cannot be crewed, and an orbital launcher must be able to fly —
and the capabilities say what the agency has developed. **A slider the agency has
not researched is pinned at zero**, whatever the archetype would otherwise allow:
you cannot buy a heat shield before the agency has developed re-entry.

**The derived numbers are never stored as an authority.** The cost, the
reliability, the payload and the delta-v are computed from the sliders every time
they are shown, so they cannot drift from the design that produced them.

**A vehicle is built, and the build takes real time** — the time is its cost, so
a cheap sounding rocket is built in a month and an orbital launcher takes a year.
The industrial capacity slows the build the same way it slows the work.

**A mission is a chain of scheduled instants**, and the screen shows where it is
in it: preparation, launch, transit, operations, return, archival. The chance of
success is the vehicle's reliability, and it is visible before the launch. **A
mission on a vehicle that is still being built is refused.**

```
┌─ Orbital Launcher 2 ────────────────────────────────────────────────────────┐
│  PROPULSION   ▇▇▇▇▇▇░░░  68      PAYLOAD   ▇▇▇▇▇▇▇░░  74                │
│  LIFE SUPPORT ▇▇▇▇░░░░░  41      REDUNDANCY ▇▇▇▇▇▇▇▇░  82                │
│                                                                             │
│  DERIVED   payload 2.4t   Δv 940   reliability 79   build 12mo   1.9M      │
│  REQUIRES  Orbital propulsion · basic re-entry                             │
│                                                                             │
│  RISK      14% failure      31% partial      3% catastrophic                │
│            ⓘ Testing "re-entry" would cut failure risk by 5 points          │
│                                                                             │
│  MISSION   destination: LEO · window 2041-07-05 · crew 3 · risk 14%          │
│            [ Authorise launch ]   lands in 1 hour 34 minutes                │
└─────────────────────────────────────────────────────────────────────────────┘
```

**The risk breakdown is three numbers, not one.** A 14% chance of something is
meaningless without knowing what that 14% is made of. Partial failure is the most likely
outcome, so it gets its own line.

**"Lands in 1 hour 34 minutes" is the line that makes the whole model legible.** The
player knows exactly how long they will be watching.

### 4.6 Chronicle (`6`)

**The history is long and the player is looking for one thing in it**, so the
filter is three axes — the year, the category, and the text — and the line above
the history always says which of them is in force. A filtered list with no visible
filter is a list the player cannot trust.

| Key | What it does |
|---|---|
| `y` | The next year, or back to every year |
| `c` | The next category, or back to everything |
| `/` | Type into the text filter |

**The forecast is the next twelve months and no further.** The player is planning
*this* year, and a schedule that runs to the end of the campaign is not a plan.
Each entry carries its risk figure: the slack in a commitment — and a commitment
with no slack left is in red — and the chance of working for a mission.

```
┌─ Chronicle ─────────────────────────────┬────────────────────────────────────┐
│ [ Feed ] [ Timeline ]                     │ 2041-06-11 09:40 · technical  ⚠   │
│ filter: category [all]   subject:         │                                    │
│                                        │ Stage 2 of the Orbital Launch      │
│  09:40  Stage 2 failed to ignite   ⚠   │ failed to ignite. Three crew were   │
│  08:15  Ground test complete           │ injured. The programme was delayed  │
│  06:02  Okonjo promoted to Senior      │ by four days.                       │
│  02:30  Breakthrough: Heat shielding    │                                    │
│  2041-05-02  Vehicle complete     ✓     │ subjects: Orbital Launch, Okonjo    │
│                                        │                                    │
│  [3 of 47 entries]                      │                                    │
└────────────────────────────────────────┴────────────────────────────────────┘
```

**Two tabs: Feed and Timeline.**

- **The Feed** is what has just happened, newest first, with the time. It is the live
  replacement for the month report.
- **The Timeline** is what is coming: scheduled obligations, orders in flight, mission
  windows, contracts expiring, and the known risk on each.

**A subject filter is the key feature.** "Everything that ever happened to the Orbital
Launch Programme" is a single keystroke away, and it is how a player reads their own
history as a story.

### 4.7 Branches (`7`)

```
┌─ Branches ───────────────────────────────────────────────────────────────────┐
│  Fork the campaign and watch the two branches diverge.                        │
│                                                                              │
│  Fork at:                                                                   │
│  ▸ 2041-06-11 09:40     the launch was authorised                            │
│    2041-05-02 16:10     Okonjo was promoted                                 │
│    2041-03-14 11:02     the heat-shield project started                     │
│                                                                              │
│  The branch is at 2041-06-11 09:40 — the original is at 2041-07-02 08:00.    │
│                                                                              │
│  Diverged         original     branch                                        │
│  Treasury             349k         412k                                       │
│  Approval                12            3                                       │
│  Staff                    34           33                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Forking is the answer to "what if I had not cancelled that programme?"** The
player looks back at a decision, forks the campaign at that instant, and watches the
two campaigns play out side by side.

**The fork points are the instants at which the player gave an order** — the
decisions worth asking "what if I had not?". The selection is the most recent one by
default, because that is the decision the player is usually thinking about.

**The branch is the same campaign up to the fork point** — the same state, the same
chronicle, the same recruits. The fork replays the order log, so it is exact. If it
were not, every difference would be noise, and the comparison would be useless.

**The branch advances with the clock.** It is a second campaign, and a comparison
between two campaigns at different instants says nothing.

**The divergence is named, not just shown.** The player wants to know that the
treasury is sixty thousand apart, not that two numbers are not equal.

---

## 5. The order palette

**The order palette is how the player acts.** It is a modal, opened with `o` or with
`Enter` on something that can be ordered.
```
┌─ Order ──────────────────────────────────────────────────────────────────────┐
│  Order: Assign                                                               │
│                                                                              │
│  ▸ Person      Dr. Tomas Reyes                                               │
│                (1 of 4 — ← → to change)                                       │
│    To          Orbital Launch project                                        │
│                (2 of 7 — ← → to change)                                       │
│                                                                              │
│  Takes effect  3 days from now                                               │
│  Cost         —                                                              │
│  Goodwill     -2                                                              │
│                                                                              │
│  ⓘ Reyes is already assigned to the Orbital Launch project.                  │
│                                                                              │
│  [ Enter to confirm ]   [ Esc to cancel ]                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

**The price is shown before the order is confirmed, and it is always the same three
lines**: how long it takes, what it costs, and what it does to goodwill.

**The ⓘ line says why the order might be a bad idea.** It is a factual statement about
the state, not advice about the game. In the build it is the order's own validation
result — "Reyes is already assigned to the Orbital Launch project" — which is the
truth and is the most useful thing it could say.

**Every parameter is a choice from a list [D].** The interface has no text field, so
a parameter is always a selection: a person, a role, a salary, a target, a field of
research, a capability, a course, a build method, a build scale, a kind of facility.
**A project's title is generated from its discipline**, because there is no way to
type one; the title is display-only, and this is recorded as a limitation of the MVP
rather than hidden.

**The palette can schedule for later.** Choosing a future instant turns the header into
a date picker, and the order joins the timeline. *Not implemented in the MVP build.*

**An order that is refused is refused here, at the moment it is given**, with a
reason. The player is never told three weeks later that something was impossible.
---

## 6. The binding-factor widget

Called out separately because it is the highest-risk component in the UI.

### What it must do

1. Show the **effective performance as a percentage of a good team**. 100% means a
   competent, rested team with adequate tools; a tired team reads around 50%.
2. Show the **modifier breakdown**, each as a bar and a signed number.
3. **Name the binding factor in words** — the single largest cost.
4. Say what would **fix it**, where the game knows.

### What it must never do

- Show the raw formula. A player should not be able to reconstruct the arithmetic.
- Show a number without a name. A bar with no label is decoration.
- Show only one factor. "Held back by fatigue" without the rest of the breakdown is not
  an explanation — the player needs to see what else is contributing, or they cannot
  weigh whether to fix fatigue *or* tools.

### The wording

The binding factor is a **short, concrete phrase**: `FATIGUE`, `TOOL QUALITY`,
`ROLE FIT`, `SUPPORT`, `CREATIVITY`, `MORALE`, `SKILL`, `HEALTH`. Uppercase for the
name, not a sentence. The surrounding sentence in the detail panel turns it into one:
*"Held back by FATIGUE"*.

**Why uppercase and not a full sentence:** the phrase appears in several places — the
staff list, the programme screen, the feed — and it must read identically every time
so the player learns to recognise it.

### The MVP risk

If the model cannot be explained here, the fix is to the model, not to the widget.
`ROADMAP.md` names the binding-factor widget as the riskiest single task for this
reason, and the mitigation is that it is developed alongside the model rather than
after it.

---

## 7. Colour and glyphs

| Meaning | Colour | Glyph | Used for |
|---|---|---|---|
| Good, safe, complete | green | `✓` | Met objectives, online facilities, healthy staff |
| Attention, at risk | yellow | `⚠` | Shrinking slack, a near-deadline, low condition |
| Bad, failed, over budget | red | `✗` | Failures, overruns, negative approval |
| Informational, future | cyan | `○` | Timeline entries, pending work |
| Historical, inactive | dim | `—` | Past events, closed programmes |

**Every coloured element also carries a glyph** so the interface is fully usable in a
monochrome terminal, over SSH, or with a colour-blind player. Colour is the second
signal, never the only one.

**An `--ascii` flag** replaces the box-drawing and the glyphs with plain ASCII. It is
cheap, and it is the difference between working and not working on an unusual terminal.

---

## 8. Layout and size

- **Minimum supported size: 100 × 30.** Below that, the side-by-side layouts collapse
  to stacked ones.
- **Maximum: fill the terminal.** Panels expand; the layout is never centred in a box
  with dead space around it.
- **Selection is always visible, and always obvious.** The selected row is in
  **reverse video and bold**, and it carries a `▸` marker in the margin. The reverse
  video is the signal; the marker is the second signal, so the focus is findable in
  a monochrome terminal as well as a colourful one. **A player who cannot see where
  the selection is cannot use the selection**, and a management game with three
  hundred rows on a screen is unusable without it.
- **The top bar never changes height**, whatever is in it, so the body never jumps.

---

## 9. The manual [D]

**Everything the game can explain is in one place, and it is reached three ways.**

The player is managing an agency that runs on a vocabulary — goodwill, slack, the
binding factor, appropriation — and none of it is obvious from looking at it. **A
number the player does not understand is a number they cannot make a decision about**,
so the interface has to be able to say what every number means, what every option
does, and what every panel is for.

### 9.1 One store, three ways in

The explanations are **a store of topics, not a set of screens**. A topic is written
once and reached three ways:

| Route | Key | What it is |
|---|---|---|
| **The manual** | `F2` | Every topic, grouped by section, narrowed by typing |
| **The screen's information** | `I` | The same browser, filtered to the screen the player is on |
| **The info line** | — | The one-line summary of the screen, always on screen |

**One store means one answer.** If the binding factor is explained in the staff
screen's information and in the manual, the two will disagree within a month. There
is one binding-factor topic and both routes reach it.

`I` is the player's own idea — *every screen has an information view, and you can
navigate around to all the different elements on the screen* — and it is the same
browser with the list filtered, rather than a second implementation of the same idea.

### 9.2 What the manual contains

| Section | What it explains |
|---|---|
| Using the agency | The clock, the orders, the report, the chronicle, forking |
| The people | Morale, fatigue, health, fame, the attributes, the roles, the binding factor, attrition |
| Money and the state | The treasury, the net, the runway, the crisis stages, the appropriation, goodwill |
| Research and facilities | Projects, points, test campaigns, how a facility is built, planetary protection |
| Programmes and promises | Objectives, the target and the committed instant, slack, promises |
| The orders | Every order in the palette, and what it does |
| The world | The alien civilisation, the long arc, how long a campaign takes |
| The screens | Every screen and every panel on it |

### 9.3 The rules

- **The filter matches the title, the summary, and the body.** A player who
  remembers that a thing is called "slack" finds it, and a player who only
  remembers that it is about "time" finds it too.
- **A topic named by the search comes first.** A player who types "binding" wants
  the binding factor, not the first topic that happens to mention it in its body.
- **Typing narrows; the arrow keys move.** The list scrolls, and the selection is
  never the thing that is off the screen.
- **The explanation scrolls too.** The longest one is over a thousand characters,
  and a detail pane in a small terminal cannot show it all. `PgUp` and `PgDn` scroll
  it, the title says how far down the player is, and a new topic starts at the top.
- **The filter survives closing the manual**, so the player does not find the same
  topic twice.
- **A filter that matches nothing says so.** An empty screen with no explanation is
  a bug the player cannot diagnose.
- **Typing a letter in the manual is not a command.** `q` in a filter is a letter,
  not the quit key.
- **The manual opens over the order palette.** A player who wants to know what an
  order is while they are filling one in should not have to cancel the order.

### 9.4 The order palette says what the order is for

The palette already shows the price of an order — how long, how much, what it does
to goodwill. **It also says what the order is for**, from the same store, because
the list of what is possible is not the same as knowing what any of it means.

### 9.5 The numbers in the manual are the numbers in the game [D]

The prose is written by hand and the balance is written in code, so the two can
drift. **A manual that quotes a figure the simulation does not use is worse than
no manual at all** — the player believes it, and it is wrong.

There is a guard: `crates/ui/tests/manual_numbers.rs` checks that the manual quotes
the live `sim::balance` constants for the appropriation, the research and test
costs, the project target, the hire lead time, the goodwill thresholds and effects,
and the attrition rates. **If a balance constant moves, the test fails and the
failure names the number that needs rewriting.**

The numbers are written with thousands separators, the way the interface writes
them, and the guard matches whole numbers rather than substrings — "12,000"
contains "2,000", and a guard that thinks it does has no guard in it.

---

## 10. What the interface does not do

| Not | Why |
|---|---|
| **A modal per month** | There is no month. The feed and "needs attention" replace it |
| **A "commit" button** | Orders are issued one at a time, and they take effect on their own schedule |
| **A calendar of months to click through** | Time is continuous. The player schedules *instants*, not months |
| **A turn order** | There is no turn. The player acts whenever |
| **A separate help screen per concept** | One store, reached three ways. Two answers to one question is one too many |

---

## 11. Related documents

- `DOMAIN_MODEL.md` — the types the screens render, and the fixed-point scales.
- `MANAGEABLE_PARTS.md` — the management surface the screens expose, and §3 for the
  time model.
- `SIMULATION.md` — the engine, and §4 for the order model.
- `ARCHITECTURE.md` — the module layout of the `ui` crate.
- `ROADMAP.md` — the tasks that build each screen.
