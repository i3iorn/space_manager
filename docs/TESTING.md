# TESTING.md

> **How we know the game still works.**
>
> **Status:** Design intent, revision 1. Extends `SIMULATION.md` §11 to the whole
> codebase.

---

## 1. Principles

1. **Every test must be able to fail.** A test that cannot fail is documentation
   pretending to be a check.
2. **Every rule has two tests: one that it holds, and one that it catches.** A
   rule with only the positive case is a rule nobody has checked, and a rule
   with only the negative case is a rule that rejects everything.
3. **Determinism is tested first and tested always.** It is the property everything else
   is built on.
4. **Behaviour changes are diffed, not silently absorbed.** The golden test turns every
   behaviour change into a reviewed diff.
5. **Test the invariants, not the implementation.** A test that asserts on an internal
   function's shape breaks on every refactor and catches nothing.
6. **Fast tests run often; slow tests run nightly.** A test suite nobody runs is a
   liability.

---

## 2. The layers

| Layer | Lives in | Speed | When |
|---|---|---|---|
| **Unit** | `crates/*/src/**` `#[cfg(test)]` modules | Milliseconds | Every change |
| **Integration** | `crates/*/tests/` | Seconds | Every change |
| **Golden** | `crates/sim/tests/golden.rs` | Seconds | Every change |
| **Property** | `crates/domain/tests/properties.rs` | Seconds to minutes | Every change |
| **Cross-system** | `crates/sim/tests/invariants.rs` | Seconds | Every change |
| **Fuzz** | `crates/sim/tests/fuzz.rs` | Minutes | Nightly |
| **Balance** | `crates/sim/tests/balance.rs` | Minutes | Nightly, and on demand |
| **Performance** | `crates/sim/tests/performance.rs` | Seconds | Every change |
| **UI snapshot** | `crates/ui/tests/` | Seconds | Every change |

**The dependency-direction check** is a test too — it lives in CI and fails if `domain`
grows a dependency on `sim`.

### 2.1 Measuring the coverage

**`cargo-llvm-cov` is the tool.** It uses the `llvm-tools` component, which
`rustup component add llvm-tools` installs, and it is the only one of the three
that gives per-line Rust coverage without a build script.

```
cargo install cargo-llvm-cov --locked
cargo llvm-cov --summary-only --workspace      # the table
cargo llvm-cov --workspace -- html            # the report
```

**It is an on-demand tool, not a CI gate.** The number moves as the code moves,
and a gate on a percentage is a gate that gets disabled. What it is for is
**finding the branches no test reaches**.

The current figure is **89.2%** of the lines and **90.5%** of the regions, over
sixteen thousand lines. **The number is not the point** — the point is that the
tool finds the branches no test reaches, and it has found two of them. The first
was the slack-meter widget, which was written, documented as done, and never
rendered by anything: a whole feature that the roadmap claimed and the game did
not have. The second is the order palette, which is a thousand lines and is under
four fifths covered, because the interesting cases are the ones a player has to
be clever to reach.

### 2.2 The suites that exist

| Suite | Lives in | What it holds |
|---|---|---|
| **The refusals** | `crates/sim/tests/refusals.rs` | Every refusal the game can make, and that a refused order changes nothing, is not charged, and is reported |
| **The order lifecycle** | `crates/sim/tests/lifecycle.rs` | An order is given, it waits, it lands, and it is applied — and every kind the engine claims to handle really is |
| **The domain's edges** | `crates/domain/tests/edges.rs` | Every invariant row has a case that holds and a case that is caught, and the money, the time, the person and the commitment at their limits |
| **The commitments** | `crates/sim/tests/commitments.rs` | The slack, the deadline consequences, the projections, and the programme's progress |
| **The facilities** | `crates/sim/src/rates.rs` | The wear, the maintenance, and the capacity |
| **The screens** | `crates/ui/tests/screens.rs` | Every screen renders, says something, and renders on an empty campaign |
| **The names** | `crates/ui/tests/names.rs` | No screen shows an index or a raw millisecond count where a name or a date belongs |
| **The identifiers** | `crates/ui/tests/identifiers.rs` | No screen, no message, and no confirmation shows an identifier where a name belongs — and `Ctrl+C` quits from any modal |
| **A long run** | `crates/sim/tests/longrun.rs` | A five-year campaign with an order every week survives without a panic |
| **The manual's numbers** | `crates/ui/tests/manual_numbers.rs` | The manual quotes the live balance constants, and a change to one fails the test that guards it |
| **The cross-system invariants** | `crates/sim/tests/invariants.rs` | No orphaned references, the team lists and the assignments agree, every commitment is on the scheduler, and every mission has its next phase |
| **The fuzz test** | `crates/sim/tests/fuzz.rs` | Random order sequences never panic, never break an invariant, and always save and load |
| **The performance budgets** | `crates/sim/tests/performance.rs` | A full five-year campaign resolves in under a second, and so does a fork |
| **The balance harness** | `crates/sim/tests/balance.rs` | The "First Orbit" scenario can be won, the three reference strategies all survive, and they are different from one another |
| **Replay and branching** | `crates/sim/tests/replay.rs` | A replayed campaign is the same campaign, and a branch diverges only because the player made it diverge |
| **The messages** | `crates/sim/tests/messages.rs` | Every refusal the engine can make is a sentence to a player, and contains no identifier |
| **The save files** | `crates/sim/tests/persistence.rs` | A garbled, truncated, empty, or foreign-version file is refused with a reason and never crashes |
| **The onboarding** | `crates/ui/tests/onboarding.rs` | The game opens with the framing and the first goal, and the world waits while the player reads it |
| **The advice** | `crates/ui/tests/advice.rs` | The banner says what the campaign needs, it says it again when the situation changes, and the panel pictures are gone |
| **Staffing a project** | `crates/sim/tests/staffing.rs` and `crates/ui/tests/staffing.rs` | Any number of people can be on one project, an assignment moves a person off their old team, and the whole path works through the palette |
| **The feed** | `crates/sim/src/feed.rs` | The feed says each thing once, and its timestamps do not move with the clock |
| **The scaling** | `crates/sim/tests/scaling.rs` | Four times the staff costs less than eight times as much, so the integration stays linear in the size of the agency |
| **The report** | `crates/sim/tests/report.rs` | The plain-text report names the campaign, its seed, and what happened |
| **The traits** | `crates/sim/tests/traits.rs` | A trait lands on the factor it describes, and the catalogue is data |
| **The name generator** | `crates/sim/tests/names.rs` | A hire gets a name from one culture, and never a family name the agency already has |

**The negative cases are the ones that find things.** Every bug this project has
found in the last month of work was found by a test that expected a refusal: an
order for a capability the agency had not developed was accepted, a postponement
of a deadline that did not exist was accepted, and a dismissal was accepted, charged,
and quietly ignored. **A game that says no has to be tested saying no.**

---

## 3. The determinism test

The single most important test in the project.

```rust
#[test]
fn same_campaign_twice_is_identical() {
    let a = run_campaign(FIXTURE_SEED, FIXTURE_LOG);
    let b = run_campaign(FIXTURE_SEED, FIXTURE_LOG);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.chronicle_digest(), b.chronicle_digest());
}
```

**It runs in CI on every change [D].** It is fast, and it catches nearly every
determinism bug before anything else does — a `HashMap` iteration, a stray wall-clock
read, a float comparison, an unseeded RNG.

**It also runs across architectures where CI allows it [D].** If cross-architecture
determinism turns out to be unachievable — which the integer-arithmetic rules in
`SIMULATION.md` §3.1 are designed to prevent — the sim version must record the
architecture and refuse to replay across it, rather than quietly producing a different
campaign.

### The state hash

A single number derived from the whole state, computed by a **fixed, documented
procedure**: a hash over the serialised state in a canonical form, with map iteration
sorted by id and no floating-point values anywhere in it. It must be implemented once,
in one place, and shared by the golden tests, the determinism test, the dilation test,
and the replay test.

### 3.1 The dilation-independence test [D]

**The test the new time model exists to support.**

```rust
#[test]
fn the_same_campaign_is_the_same_at_every_dilation() {
    let slow = play_at(TimeScale::One, FIXTURE);
    let fast = play_at(TimeScale::TenThousand, FIXTURE);
    assert_eq!(slow.state_hash(), fast.state_hash());
}
```

The fixture is a set of orders with fixed **simulated** issue instants. The two runs
differ only in how much simulated time each host frame delivers, and therefore only in
how finely the integration subdivides. **They must produce the same campaign.**

This is the test that catches a random draw inside an integration sub-step, and it is
the one that would catch it most painfully if it were left to a playtest.

**It runs at every dilation step, not just the two extremes [D].** The nine steps are
cheap to test, and each one is a different sub-step size — a different approximation
of the same integral.

**The residual risk it guards against:** integration is a numerical approximation, and
the approximation's error depends on the sub-step size, which depends on the dilation.
The test is what proves the error is small enough not to matter
(`SIMULATION.md` §6.3).

**The mechanism is already in place [D].** The integration runs on a **fixed grid** of
one hour, anchored at the start of the campaign, so the sub-step boundaries do not
depend on the host at all. The test that proves it is in
`crates/sim/tests/determinism.rs`:

```rust
determinism_the_campaign_does_not_depend_on_how_the_time_was_chopped_up()
determinism_the_campaign_does_not_depend_on_the_tempo()
```

The first advances the same campaign in one-hour pieces, in one-week pieces, and in
one-month pieces; the second advances two campaigns of the same orders at two very
different tempi. Both must produce the same state hash.

**What this catches.** If the rates were applied per *host chunk* rather than per
grid quantum, the rate would be sampled at different points in the two runs, and the
campaigns would diverge. That is the same failure mode as a draw inside a sub-step,
and the test catches both.

### 3.1.1 The test through the host [D]

`crates/ui/tests/dilation.rs` is the *whole* property, tested through the host's own
path — the real `App`, the real `Clock`, the real frame:

```rust
the_same_campaign_is_the_same_at_every_dilation()
the_dilations_really_do_use_different_sized_frames()
```

The first runs the same campaign at each of the nine dilations and asserts that the
state hash and the chronicle are identical. The second checks that the nine dilations
really do use very different frame sizes — so the first is not vacuous.

**Both campaigns must cover the same span [D].** A frame that carries the campaign
past the target leaves the fast campaigns covering *more* simulated time than the slow
ones, and they are then different campaigns — which is not what the test is about. The
test clamps its requests to the target, and the frames still differ by four orders of
magnitude in between.

### 3.2 The order log is the replay

A second determinism test: the campaign recorded in a save's order log, replayed from
the seed, must reproduce the save's state hash. This is the property that makes
branching possible, and it is a different property from the first — the first is about
the *engine*, this one is about the *save format*.

---

## 4. Golden tests

A golden test pins a **known campaign** and asserts the result.

```
crates/sim/tests/fixtures/
├─ ground_game/seed.txt
├─ ground_game/orders.json
├─ ground_game/expected_state_hash.txt
├─ ground_game/expected_chronicle.txt
└─ first_orbit/…
```

**The expected values live in files, not in the test code [D].** The test compares; it
does not define. When a behaviour change is intended, the developer regenerates the
fixture and **reviews the diff** — which is the entire point. A golden test that
asserts a literal buried in the test file is much easier to "fix" by accident.

**The fixture's orders carry simulated issue instants [D].** A golden fixture is
therefore a *script*, not a log of what happened: it says what the player did and when,
and the test re-issues those orders at those instants.

**The golden fixtures are now keyed by instant rather than by month [D].** The fixture
also records the instant at which the state was captured, so a fixture can be checked at
"the state after the launch reaches orbit" rather than only at the end of a year.

**The expected chronicle is stored as readable text [D].** When a golden test fails, the
developer should be able to diff two readable chronicles and see exactly what changed
in the world. A hash difference tells them nothing.

```rust
#[test]
fn first_year_campaign_matches() {
    let result = run_campaign(load_fixture("first_year"));
    assert_eq!(result.state_hash(), read_expected_hash("first_year"));
    assert_eq!(result.chronicle_text(), read_expected_chronicle("first_year"));
}
```

**Every behaviour change lands with a regenerated fixture** (`ROADMAP.md` X-04). The
diff of the chronicle *is* the review artefact.

---

## 5. Property tests

Property tests assert that a **rule holds for all inputs**, not for one case.

### Invariants

One test per row of the invariant catalogue in `DOMAIN_MODEL.md` §18, expressed as
properties over randomly generated states. For example:

```rust
proptest! {
    #[test]
    fn a_person_has_at_most_one_assignment(state in arb_state()) {
        let p = state.people.values().next().unwrap();
        // the invariant holds for every generated state
    }
}
```

**Generated states must be valid states.** Generating arbitrary bytes and asserting
invariants on them produces failures that are artefacts of the generator. The
generator has to build *plausible* campaigns.

### Cross-system properties

These are the ones that catch the interesting bugs:

| Property | Catches |
|---|---|
| Every `SubjectRef` in the chronicle resolves to a live entity | Dangling references after a deletion |
| Every committed date that has passed has a resolution | Deadlines silently ignored |
| The treasury never goes negative without the crisis ladder advancing | Money bugs |
| The sum of allocations equals the total budget | Rounding drift from the split helper |
| Every mission in flight has a vehicle and a crew | Half-built missions |
| The chronicle is append-only and monotonically dated | Reordering bugs |
| No person appears in two staffing lists | Assignment bookkeeping |
| The number of chronicle entries is non-decreasing across time | Events being dropped |

### The fuzz test

Random **order** sequences, generated and applied, must never panic and must always
resolve to a valid state. This is the test that finds the "the player did something
weird and the game crashed" bugs, which are the ones that end campaigns.

```rust
proptest! {
    #[test]
    fn arbitrary_orders_never_panic(orders in arb_order_seq(200)) {
        let mut sim = Sim::new(scenario());
        let _ = sim.issue_orders(orders);  // must not panic
    }
}
```

**Run it nightly with a large case count.** A thousand cases in CI is enough to catch
the common shapes; a hundred thousand overnight is enough to find the rare ones.

---

## 6. Replay and save tests

| Test | Asserts |
|---|---|
| **Round trip** | Save → load → the state hash is unchanged |
| **Replay** | A seed plus a log reproduces the original state hash |
| **Replay after a change** | A log recorded under an older `sim_version` is refused with a clear message |
| **Corruption** | A truncated, garbled, or empty file fails gracefully with a readable message and never crashes |
| **Foreign version** | A file with a newer `format` is refused cleanly |
| **Fork** | Forking at instant *t* produces a state identical to the original at *t* |
| **Branch divergence** | Two branches from the same fork point produce different states when given different orders |

**The fork test is the one that proves the branching feature** and it is easy to write
incorrectly: fork at an instant, then assert the fork's state equals the original's state
at that instant. If that holds, the replay machinery is sound.

---

## 7. Performance tests

Performance budgets from `ARCHITECTURE.md` §10, asserted in tests so they cannot
silently regress.

| Benchmark | Budget | Where |
|---|---|---|
| A full five-year campaign (60 months) | < 1 s release, < 15 s debug | `crates/sim/tests/performance.rs` |
| A fork at any instant | The same budget — a fork replays the log | `crates/sim/tests/performance.rs` |
| One hour of simulation (one grid quantum) | < 20 µs release, < 400 µs debug | `crates/sim/tests/performance.rs` |
| Four times the staff costs less than eight times as much | A ratio, not a time | `crates/sim/tests/scaling.rs` |
| A two-year campaign at three hundred people | < 30 s debug, < 4 s release | `crates/sim/tests/scaling.rs` |

**These are tests, not benchmarks [D].** A benchmark reports a number and is run by
whoever thinks to run it; a test fails the build. A generous margin is fine — the point
is to catch an order-of-magnitude regression, not to tune microseconds.

**The budget is the criterion in the shipped build.** A debug build is five to
twenty times slower, and a test that failed in CI and passed in a release build
would be a test nobody trusted. So the release number is the line, the debug number
is a loose ceiling, and the CI runs the release check as a job of its own.

**The full-campaign test is the important one [D].** It is the guard on the property
that makes branching, fast-forward, and scrubbing affordable. If it ever fails, the
architecture has been broken and the cause needs finding, not the budget raising.

**The scaling test is the second one, and it guards a different thing.** The
absolute budget says "fast enough at the size we ship"; the ratio says "the cost
grows with the size of the agency rather than with its square", which is the
property that will still hold when the game is played with eight hundred people
rather than three hundred. **A clock alone would not catch it** — an implementation
that is quadratic at every size passes a generous ceiling for a long time and then
does not.

---

## 8. Balance testing

Balance is not decided by feel alone. The harness in `crates/sim/src/harness.rs`
runs many campaigns under scripted reference policies and reports the outcome
distribution, and `crates/sim/examples/harness.rs` is it from the command line:

```
cargo run -p sim --example harness -- first_orbit 20 60
```
### Reference policies

Three scripted players, each making reasonable decisions:

| Policy | Behaviour |
|---|---|
| **Cautious** | Test before launching, keep a large budget buffer, favour redundancy, accept slow progress |
| **Fast** | Launch early, accept risk, prioritise schedule over capability |
| **Science-first** | Maximise research, delay launches, build capability before using it |

**All three must be viable [D].** This is the MVP's most important balance criterion
(`MVP.md` §5 criterion 3). If one policy collapses, the trade-offs are fake and the game
is a puzzle with a single answer.

### What the harness reports

For each policy, over N seeds:

- the primary outcome: how many campaigns reached orbit, and how many met the
  deadline
- the time to orbit: the earliest, the median, and the latest
- the launches, and how many of them worked, partly worked, and were lost
- the treasury left, the staff at the end, and the knowledge the agency holds

**And a verdict.** The harness exits with a non-zero status if any policy has
collapsed, because a balance pass that leaves the game with one answer has
not passed.
### What the harness must not do

**It must not optimise.** The harness measures and reports; it does not search for the
best policy. An optimiser would find the dominant strategy and destroy the game's
balance by making it true.

---

## 9. UI tests

### Snapshot tests

Each screen renders to a ratatui `Buffer` from a fixed state, and the result is compared
against a stored snapshot.

```rust
#[test]
fn staff_detail_shows_the_binding_factor() {
    let buffer = render_staff_detail(FIXTURE_STATE);
    assert_snapshot!(buffer);
}
```

**This is how the performance model's legibility is protected [D].** A snapshot of the
binding-factor panel is a test that the panel still names the constraint, still shows
the breakdown, and has not been quietly simplified into a number.

Snapshots are reviewed as text diffs, so the change is visible.

### Interaction tests

The key handling is pure enough to test without a terminal: a key press plus a state
in, a new app state out.

```rust
#[test]
fn pressing_enter_on_a_person_opens_their_detail() { ... }
#[test]
fn a_rejected_order_produces_a_message_and_no_state_change() { ... }
```

**The second one is worth a test in every crate** — it is the guarantee from
`SIMULATION.md` §4.1 that a rejected order changes nothing at all.

---

## 10. Fixtures and test data

| Kind | Where | Notes |
|---|---|---|
| Scenarios | `crates/sim/tests/fixtures/scenarios/` | Small, hand-built, each exercising one thing |
| Order scripts | `crates/sim/tests/fixtures/orders/` | The input to golden tests |
| Expected output | `crates/sim/tests/fixtures/expected/` | Hashes and readable chronicles |
| UI states | `crates/ui/tests/fixtures/` | A state per interesting screen |
| Generated states | `crates/sim/tests/fixtures/generated/` | From the property-test generators, for debugging failures |

**Generated states should be committed [D]:** when a property test fails, the
generator's output is written to a file so the failure is reproducible. A property test
that fails only in CI, with the input lost, is a test that gets ignored.

---

## 11. CI

```yaml
# Every push, on Linux and on Windows
- fmt:      cargo fmt --all --check
- clippy:   cargo clippy --workspace --all-targets -- -D warnings
- test:     cargo test --workspace
- deps:     the dependency-direction check, which is one of those tests
- golden:   included in cargo test, but reported separately

# Every push, on Linux
- perf:     cargo test --release -p sim --test performance

# Every push, on Linux and on Windows
- determ:   cargo test --workspace determinism

# Nightly, on a schedule
- fuzz:     cargo test --release -p sim --test fuzz -- --ignored
- balance:  cargo run --release -p sim --example harness -- first_orbit 40 60
```

**The determinism test runs on two operating systems on its own.** A stream that
differed between Windows and Linux would invalidate every replay made on the other
one, so it is worth a job of its own rather than riding along with the rest.

**The performance budgets are checked in a release build**, because a debug build
is five to twenty times slower and a budget that only holds in release is a budget
that has to be checked in release.

**Golden and determinism failures block the build. The balance harness does not [D]** —
it is a report, and a failing balance assertion should prompt a decision, not a retry.

**A failing golden test is a change, not an error [D].** The build fails, the developer
reads the chronicle diff, and either the change was intended (regenerate the fixture and
say so in the commit) or it was a bug (fix it). The point is that it can never happen
quietly.

---

## 12. What is not worth testing

| Not worth it | Why |
|---|---|
| Serde round-trips of every domain type | Covered incidentally by the save round-trip test |
| Every `OrderKind` variant's happy path in isolation | Covered by the integration tests that use them |
| Exact wording of narrative strings | A wording change should not fail a build. Chronicle *content* is golden; *prose* is not |
| The UI's colour values | Snapshot tests cover the visible result |
| Speculative "what if" cases | Write a test when the behaviour is specified, not before |

**The rule: test what the design says must be true, and nothing else.** A test suite
that is mostly noise gets ignored, and an ignored suite protects nothing.

---

## 13. Related documents

- `SIMULATION.md` — the determinism contract these tests defend.
- `DOMAIN_MODEL.md` — the invariant catalogue that becomes property tests.
- `PERSISTENCE.md` — the save, replay, and fork formats the tests exercise.
- `BALANCE.md` — the numbers the harness measures.
- `ROADMAP.md` — the tasks that build the suite.
