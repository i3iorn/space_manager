//! The engine for `space_manager`.
//!
//! This crate turns a seed plus a sequence of player decisions into a forty-year
//! history. It is the game; the terminal interface is a window onto it.
//!
//! # Rules for this crate
//!
//! * **No wall-clock time, no system randomness, no `HashMap` iteration order.**
//!   See `docs/SIMULATION.md` §6.1.
//! * **No floating point in decision-making.** Comparisons that can change an
//!   outcome use integers or fixed point. Floating point is presentation-only.
//! * **No `unwrap`, no `expect`, no `panic!`** outside tests. A panic in the
//!   simulation corrupts a campaign; every fallible path returns a typed error.
//! * **The player never mutates state.** They issue *orders*, which are
//!   validated without side effects and then scheduled for their lead time. A
//!   rejected order is reported with a reason, never silently ignored.
//!
//! # Where to look
//!
//! | Module | Holds |
//! |---|---|
//! | [`rng`] | Named random streams and the pinned hash and seed expansion |
//! | [`tick`] | The ten-phase month driver |
//! | [`phases`] | One module per phase |
//! | [`orders`] | The order registry: the lead times, the costs, and the handlers |
//! | [`derived`] | Every derived value |
//! | [`performance`] | The C+B performance model |
//! | [`report`] | Building the month report |
//! | [`persist`] | Save, load, version checks, and the state hash |
//! | [`scenario`] | Scenario definitions |
//! | [`branch`] | Replay and forking |
//! | [`balance`] | Every tunable number in the game |
//! | [`events`] | The event catalogue |
//! | [`error`] | `SimError` and `OrderError` |
//!
//! The engine contract is `docs/SIMULATION.md`. The field-level contract for the
//! types it operates on is `docs/DOMAIN_MODEL.md`.

#![deny(missing_docs)]

pub mod balance;
pub mod branch;
pub mod deadlines;
pub mod derived;
pub mod error;
pub mod event_catalogue;
pub mod events;
pub mod orders;
pub mod performance;
pub mod persist;
pub mod phases;
pub mod rates;
pub mod recruitment;
pub mod report;
pub mod rng;
pub mod scenario;
pub mod scheduler;
pub mod shift;
pub mod tick;
pub mod version;

use std::collections::BTreeMap;

use domain::ids::OrderId;
use domain::money::Money;
use domain::order::OrderKind;
use domain::report::MonthReport;
use domain::state::CampaignState;
use domain::time::{Calendar, Duration, Instant};

pub use error::{OrderError, SimError};
pub use persist::{LoggedOrder, SaveEnvelope, state_hash};
pub use rng::{Domain, Rng};
pub use scenario::Scenario;
pub use scheduler::{ScheduledItem, ScheduledKind, Scheduler};

/// The most months one `advance_to` call will simulate.
///
/// A campaign that is left running for a very long time must not stall the
/// frame, so the remaining months are skipped rather than simulated.
pub const MAX_MONTHS_PER_ADVANCE: u32 = 4_096;

/// The simulation.
///
/// Owns the campaign state, the random source, the clock, the scheduler, and the
/// order log. It is the only thing that moves time forward.
///
/// **The clock lives here, not in the state [D].** The state holds no date at
/// all; the calendar is a projection of `now`, derived from the agency's
/// founding year. See `docs/DOMAIN_MODEL.md` §2.
///
/// **The dilation lives in the host, not here [D].** The simulation is only ever
/// told a duration, which is what makes dilation independence possible.
#[derive(Clone, Debug)]
pub struct Sim {
    state: CampaignState,
    /// The scenario the campaign started from.
    ///
    /// **The fork needs it.** Forking replays the order log from the
    /// beginning, and the beginning is the scenario's start state. Without it
    /// a campaign could not be forked, and forking is what makes "what if I had
    /// not cancelled that programme?" a question the player can answer.
    scenario: Scenario,
    rng: Rng,
    /// The current instant.
    now: Instant,
    /// The calendar the instant is projected onto.
    calendar: Calendar,
    /// Everything waiting to happen.
    scheduler: Scheduler,
    /// The simulated time that has been asked for but not yet used.
    ///
    /// The host asks for spans that are almost never a whole number of
    /// integration quanta, and the fraction is carried until it adds up to
    /// one. Nothing is lost.
    carry: Duration,
    /// The report lines produced since the last monthly report.
    ///
    /// **An order lands between month boundaries**, so its lines have to wait
    /// for the next report. The live feed (M1-13) replaces this.
    pending_lines: Vec<domain::report::ReportLine>,
    /// The orders that were refused since the last monthly report, and why.
    pending_rejections: Vec<OrderError>,
    log: Vec<LoggedOrder>,
    sequence: u64,
}

impl Sim {
    /// Starts a campaign from a scenario.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if the scenario cannot be started.
    pub fn new(scenario: &Scenario) -> Result<Self, SimError> {
        let calendar = Calendar::new(scenario.start_state.agency.founded);
        let mut sim = Self {
            state: scenario.start_state.clone(),
            scenario: scenario.clone(),
            rng: Rng::new(scenario.seed),
            now: Instant::ZERO,
            calendar,
            scheduler: Scheduler::new(),
            carry: Duration::ZERO,
            pending_lines: Vec::new(),
            pending_rejections: Vec::new(),
            log: Vec::new(),
            sequence: 0,
        };
        // The scenario's state is validated before the campaign begins, so a
        // broken scenario is caught at once rather than in the middle of a
        // month.
        domain::invariant::check(&sim.state).map_err(|violation| SimError::CorruptState {
            reason: violation.to_string(),
        })?;
        sim.schedule_periodic();
        sim.schedule_commitments();
        Ok(sim)
    }

    /// Puts the commitments in the starting state on the queue.
    ///
    /// **A commitment that is not on the scheduler never falls due.** The
    /// scenario may open with a deadline the player has to answer, and a
    /// deadline the game does not know about is not a deadline.
    fn schedule_commitments(&mut self) {
        let commitments: Vec<(Instant, domain::ids::CommitmentId)> = self
            .state
            .programmes
            .values()
            .flat_map(|programme| programme.objectives.iter())
            .filter_map(|objective| {
                objective
                    .commitment
                    .committed
                    .map(|due| (due, objective.commitment.id))
            })
            .collect();
        for (due, commitment) in commitments {
            self.scheduler
                .schedule(due, ScheduledKind::CommitmentDue { commitment });
        }
    }
    /// Puts the recurring work on the queue.
    fn schedule_periodic(&mut self) {
        // The first month boundary; each boundary schedules the next.
        self.scheduler
            .schedule(Instant::ZERO, ScheduledKind::MonthBoundary);
        // The first event check.
        self.scheduler.schedule(
            Instant::from_days(balance::EVENT_CHECK_INTERVAL_DAYS),
            ScheduledKind::EventCheck,
        );
    }

    /// The campaign state.
    #[must_use]
    pub const fn state(&self) -> &CampaignState {
        &self.state
    }

    /// The campaign's random source.
    #[must_use]
    pub const fn rng(&self) -> Rng {
        self.rng
    }

    /// The calendar the campaign's dates are projected onto.
    #[must_use]
    pub const fn calendar(&self) -> Calendar {
        self.calendar
    }

    /// The scheduler.
    #[must_use]
    pub const fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// The order log - every decision the player has made.
    #[must_use]
    pub fn log(&self) -> &[LoggedOrder] {
        &self.log
    }

    /// The instant the campaign is in.
    #[must_use]
    pub const fn now(&self) -> Instant {
        self.now
    }

    /// The orders the player has issued that have not landed yet.
    ///
    /// **This is the agenda.** The player can see what the agency is about to
    /// do, and when each decision takes effect.
    #[must_use]
    pub fn pending_orders(&self) -> &BTreeMap<OrderId, domain::order::Order> {
        &self.state.pending_orders
    }

    /// Issues orders, and resolves the month they belong to.
    ///
    /// **The orders are issued now and land at their own lead times.** An order
    /// that cannot be honoured is refused at once, with a reason, and costs
    /// nothing.
    ///
    /// The month is resolved on a **working copy** of the state. Only if every
    /// phase completes is the copy committed, so a month either happens
    /// entirely or not at all.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if the month could not be resolved. The campaign
    /// state is left untouched.
    pub fn step(&mut self, orders: Vec<OrderKind>) -> Result<MonthReport, SimError> {
        let now = self.now;

        for order in orders {
            if let Err(reason) = self.issue_at(order, now) {
                // A refused order is information, not a failure: the month
                // still resolves, and the player is told why.
                self.pending_rejections.push(reason);
            }
        }

        // A step is a month, and it goes through the scheduler like everything
        // else. There is exactly one way to move time.
        let target = now.saturating_add(Duration::from_months(1));
        let reports = self.advance_to(target)?;

        Ok(reports
            .last()
            .cloned()
            .unwrap_or_else(|| report::empty(now, self.calendar().date(now).to_string())))
    }

    /// Issues an order.
    ///
    /// **The order is validated at once.** The player must be told immediately
    /// that an order is impossible, not three weeks later. A refused order
    /// costs nothing, changes nothing, and is never scheduled.
    ///
    /// The order lands at its lead time, and it is charged then — not now.
    ///
    /// # Errors
    ///
    /// Returns the [`OrderError`] describing why the order was refused.
    pub fn issue(&mut self, order: OrderKind) -> Result<OrderId, OrderError> {
        self.issue_at(order, self.effective_now())
    }

    /// Issues an order at a given instant.
    ///
    /// The instant is the campaign's **effective time** — the grid point plus
    /// the carried fraction — and not the grid point itself. The order log
    /// records it, and the fork replays the log to those instants, so the
    /// carried fraction has to be reproduced exactly or the branch and the
    /// original would drift apart.
    fn issue_at(&mut self, order: OrderKind, issued_at: Instant) -> Result<OrderId, OrderError> {
        // **An order with no registered handler is rejected, never ignored.**
        if !crate::orders::has_handler(&order) {
            return Err(OrderError::Rejected {
                reason: format!("There is no way to {} yet.", order.kind_name()),
            });
        }
        // **The first validation.** The player is told now, not at the landing.
        crate::orders::validate(&order, &self.state)?;

        let lead_time = crate::orders::lead_time(&order, &self.state, issued_at);
        let cost = crate::orders::adjusted_cost(
            crate::orders::cost(&order, &self.state),
            &self.state.political.goodwill,
        );
        let goodwill = crate::orders::goodwill(&order);
        let id = self.state.id_counters.next_order();
        let order = domain::order::Order {
            id,
            issued_at,
            effective_at: issued_at.saturating_add(lead_time),
            kind: order,
            cost,
            goodwill,
        };
        self.state.pending_orders.insert(id, order.clone());
        self.scheduler
            .schedule(order.effective_at, ScheduledKind::OrderLands { order: id });

        self.sequence += 1;
        self.log.push(LoggedOrder {
            issued_at,
            seq: self.sequence,
            kind: order.kind.kind_name().to_owned(),
            payload: serde_json::to_value(&order.kind).unwrap_or(serde_json::Value::Null),
        });
        Ok(id)
    }

    /// The instant the campaign has actually reached.
    ///
    /// **The simulation's clock lands on the integration grid**, so `now` is
    /// the last point on the grid and the fraction beyond it is carried. This
    /// is the campaign's real time, and it is what the host asks to advance
    /// from.
    #[must_use]
    pub fn effective_now(&self) -> Instant {
        self.now.saturating_add(self.carry)
    }

    /// Advances the simulation to an instant.
    ///
    /// This is **the only way time moves** once the continuous model lands, and
    /// it is the API the host's clock calls.
    ///
    /// The loop is:
    ///
    /// 1. Work out where the campaign's clock lands: the last point on the
    ///    **integration grid** at or before the target. The grid is one hour
    ///    long and anchored at the start of the campaign.
    /// 2. Walk the grid, one quantum at a time, applying the continuous
    ///    **rates** over each one.
    /// 3. At every point on the grid, apply the scheduled items that fall due.
    ///
    /// **The grid is what makes dilation independence hold.** A rate is often a
    /// function of the state — a project's progress depends on how tired its
    /// team is — so the result of an integration depends on how finely it was
    /// done. If the host's frame rate decided that, a campaign played at 1x
    /// and the same campaign played at 10,000x would diverge in the third
    /// month, with nothing on screen to explain it.
    ///
    /// **The leftover is carried, not lost.** The host asks for a span of
    /// simulated time that is almost never a whole number of hours, and the
    /// fraction is kept until it adds up to an hour. A player who stops for an
    /// hour does not come back to a campaign that skipped an hour.
    ///
    /// **The campaign never overshoots the target [D].** The target is measured
    /// from the campaign's *effective* time, and the grid point it lands on is
    /// the last one at or before it. This is what makes two campaigns at
    /// different dilations reach the same grid point: without it, the faster
    /// one would carry its fraction past the target and land a quantum further
    /// along, and the two campaigns would end in different months.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if an item could not be applied. The campaign
    /// state is left untouched.
    pub fn advance_to(&mut self, target: Instant) -> Result<Vec<MonthReport>, SimError> {
        let effective = self.effective_now();
        if target <= effective {
            return Ok(Vec::new());
        }

        // The grid end: the last whole quantum at or before the target, plus
        // whatever the last call did not use.
        let asked = target.millis().saturating_sub(effective.millis());
        let total = asked.saturating_add(self.carry.millis());
        let whole = total - (total % rates::QUANTUM.millis());
        let end = self.now.saturating_add(Duration::from_millis(whole));

        let mut reports = Vec::new();
        let mut months = 0_u32;

        while self.now < end {
            // Everything due at this instant, in the scheduler's total order.
            // This happens *before* the rates for the instant are applied, so
            // the month's work at a boundary sees the world it woke up to, and
            // an event check at the same instant sees the month's work.
            while let Some(item) = self.scheduler.peek() {
                if item.at > self.now {
                    break;
                }
                let item = self.scheduler.pop_next().expect("there is an item");
                if let Some(report) = self.apply_item(&item)? {
                    reports.push(report);
                    months += 1;
                }
                if months >= MAX_MONTHS_PER_ADVANCE {
                    break;
                }
            }

            if months >= MAX_MONTHS_PER_ADVANCE {
                break;
            }

            // One quantum of continuous work.
            let step = self.now.saturating_add(rates::QUANTUM).min(end);
            let completed = rates::integrate(
                &mut self.state,
                Duration::from_millis(step.millis() - self.now.millis()),
            );
            self.now = step;

            // A project that reached its target inside the integration cannot
            // be completed there: the completion draws, and a draw may only
            // happen at a scheduled instant. It becomes one.
            //
            // **The same is true of a vehicle that finished being built.**
            for project in &completed.projects {
                self.scheduler.schedule(
                    self.now,
                    ScheduledKind::ProjectComplete { project: *project },
                );
            }
            for vehicle in &completed.vehicles {
                self.scheduler.schedule(
                    self.now,
                    ScheduledKind::VehicleComplete { vehicle: *vehicle },
                );
            }
        }

        self.carry = Duration::from_millis(total - whole);
        Ok(reports)
    }

    /// Applies one scheduled item.
    fn apply_item(&mut self, item: &ScheduledItem) -> Result<Option<MonthReport>, SimError> {
        // The work is dated at the item's own instant, not at the start of the
        // advance. An event check in the middle of a month is dated in the
        // middle of that month.
        let at = item.at;
        match &item.kind {
            ScheduledKind::MonthBoundary => {
                let report = self.resolve_month(at)?;
                // The next boundary.
                self.scheduler.schedule(
                    at.saturating_add(Duration::from_months(1)),
                    ScheduledKind::MonthBoundary,
                );
                Ok(Some(report))
            }
            ScheduledKind::EventCheck => {
                // The next check. The interval tightens under stress, so a run
                // of bad luck becomes a run of bad luck.
                let interval = Duration::from_days(self.event_check_interval_days());
                self.scheduler
                    .schedule(at.saturating_add(interval), ScheduledKind::EventCheck);
                // **The draw is here, at a scheduled instant**, and it is on the
                // check's own stream, so the n-th check of a campaign always
                // draws the same numbers.
                self.draw_event(at);
                // **The event check's index is its bucket**, and it is a stable
                // identifier: the n-th check of a campaign always draws the
                // same numbers, whether or not it happens at the same instant —
                // and the interval tightens under stress, so the instants move
                // and the index does not. The counter is in the state, so a
                // save and load does not restart it.
                self.resolve_events(at)?;
                self.state.event_checks = self.state.event_checks.saturating_add(1);
                Ok(None)
            }
            ScheduledKind::EventFollowUp { event } => {
                self.fire_follow_up(at, *event);
                Ok(None)
            }
            ScheduledKind::VehicleComplete { vehicle } => {
                self.complete_vehicle(at, *vehicle);
                Ok(None)
            }
            ScheduledKind::ProjectComplete { project } => {
                self.complete_project(at, *project);
                Ok(None)
            }
            ScheduledKind::OrderLands { order } => {
                self.land_order(at, *order);
                Ok(None)
            }
            ScheduledKind::CommitmentDue { commitment } => {
                self.resolve_commitment(at, *commitment);
                Ok(None)
            }
            ScheduledKind::MissionPhase { mission, phase } => {
                self.enter_mission_phase(at, *mission, phase);
                Ok(None)
            }
            ScheduledKind::AttritionCheck { .. } => {
                // These land with the mission, commitment, and attrition
                // systems, which arrive in the later milestones. Until then they
                // are carried and do nothing.
                Ok(None)
            }
        }
    }

    /// Lands an order.
    ///
    /// **This is the second validation [D].** The world has moved on since the
    /// order was issued: the project may have been cancelled, the person may
    /// have left, the vehicle may have been scrapped. An order that can no
    /// longer be honoured is **refused at landing**, the cost is not charged,
    /// and the player is told why.
    fn land_order(&mut self, at: Instant, id: OrderId) {
        let Some(order) = self.state.pending_orders.remove(&id) else {
            return;
        };

        let mut working = self.state.clone();
        {
            let mut context =
                tick::MonthContext::new(&mut working, &self.rng, at, vec![order.kind.clone()]);
            // **The second validation.** The world has moved on since the order
            // was issued.
            match crate::orders::validate(&order.kind, context.state) {
                Ok(()) => {
                    // **Landed.** The order is applied, the cost is charged,
                    // and the goodwill the order drew on is spent.
                    crate::orders::apply(&mut context, &order.kind);
                    let balance = context
                        .state
                        .treasury
                        .balance
                        .minor()
                        .saturating_sub(order.cost.minor());
                    context.state.treasury.balance = domain::money::Money::from_minor(balance);
                    context.state.political.goodwill =
                        context.state.political.goodwill.charged(order.goodwill);
                }
                Err(reason) => {
                    // **Refused.** The cost is not charged, and the player is
                    // told why — in the report, and in the chronicle, which is
                    // the durable record.
                    self.pending_rejections.push(reason.clone());
                    let entry = domain::chronicle::ChronicleEntry::new(
                        context.state.next_chronicle_id(),
                        at,
                        domain::chronicle::Category::Political,
                        format!("An order was refused: {reason}"),
                        domain::chronicle::Severity::Major,
                    );
                    context.state.chronicle(entry);
                }
            }

            // **The items the phase asked for go on the scheduler.** A
            // phase cannot reach the scheduler itself, so it leaves them here.
            for (at, item) in std::mem::take(&mut context.schedule) {
                self.scheduler.schedule(at, item);
            }

            // The report lines wait for the next monthly report.
            self.pending_lines.extend(context.personnel_lines);
            self.pending_lines.extend(context.programme_lines);
            for entry in context.events {
                working.chronicle(entry);
            }
        }
        self.state = working;
    }

    /// Moves a mission into its next phase.
    ///
    /// **The phase change is a scheduled item**, and the outcome is resolved
    /// when the mission comes home. The draw happens here, at a scheduled
    /// instant, and not inside the integration.
    fn enter_mission_phase(
        &mut self,
        at: Instant,
        mission: domain::ids::MissionId,
        phase: &domain::mission::MissionPhase,
    ) {
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            crate::phases::missions::enter_phase(&mut context, mission, phase);
            for (at, item) in std::mem::take(&mut context.schedule) {
                self.scheduler.schedule(at, item);
            }
            self.pending_lines.extend(context.programme_lines);
            for entry in context.events {
                working.chronicle(entry);
            }
        }
        self.state = working;
    }
    /// Draws an event at an event check, and applies it.
    ///
    /// **The events phase runs outside the month's commit**, so it writes its
    /// own chronicle entries. Without that, an event would fire and never be
    /// recorded.
    fn draw_event(&mut self, at: Instant) {
        let check = self.state.event_checks;
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            crate::phases::events::draw_and_apply(&mut context, check);
            for (at, item) in std::mem::take(&mut context.schedule) {
                self.scheduler.schedule(at, item);
            }
            self.pending_lines.extend(context.programme_lines);
            for warning in context.warnings {
                self.pending_lines
                    .push(domain::report::ReportLine::new(warning));
            }
            for entry in context.events {
                working.chronicle(entry);
            }
        }
        self.state = working;
    }

    /// Fires a follow-up event.
    fn fire_follow_up(&mut self, at: Instant, event: domain::ids::EventId) {
        let Some(definition) = crate::event_catalogue::event(event) else {
            return;
        };
        let definition = definition.clone();
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            crate::phases::events::apply_event(&mut context, &definition);
            for (at, item) in std::mem::take(&mut context.schedule) {
                self.scheduler.schedule(at, item);
            }
            self.pending_lines.extend(context.programme_lines);
            for warning in context.warnings {
                self.pending_lines
                    .push(domain::report::ReportLine::new(warning));
            }
            for entry in context.events {
                working.chronicle(entry);
            }
        }
        self.state = working;
    }
    /// Completes a vehicle that finished being built.
    ///
    /// **The build completes at a scheduled instant**, the same way a project
    /// does, and the vehicle is ready to fly from that moment.
    fn complete_vehicle(&mut self, _at: Instant, vehicle: domain::ids::VehicleId) {
        let Some(state) = self.state.vehicles.get_mut(&vehicle) else {
            return;
        };
        if state.build_state != domain::vehicle::BuildState::Complete {
            state.build_state = domain::vehicle::BuildState::Complete;
            state.build_progress = 100 * domain::scale::PERCENT_UNITS as u32;
            let name = state.name.clone();
            self.pending_lines.push(
                domain::report::ReportLine::new(format!("{name} is finished and ready to fly."))
                    .with_severity(domain::chronicle::Severity::Info),
            );
        }
    }
    /// Resolves a commitment that has fallen due.
    ///
    /// **The commitment is resolved against the work, not against the clock.**
    /// An objective that is finished when the date arrives is met, and one
    /// that is not is missed, and the audience decides what that costs.
    fn resolve_commitment(&mut self, at: Instant, commitment: domain::ids::CommitmentId) {
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            crate::phases::programmes::resolve(&mut context, commitment);
            for (at, item) in std::mem::take(&mut context.schedule) {
                self.scheduler.schedule(at, item);
            }
            self.pending_lines.extend(context.programme_lines);
            for entry in context.events {
                working.chronicle(entry);
            }
        }
        self.state = working;
    }
    /// Completes a research project that reached its target.
    ///
    /// **This is where the draw happens**, and it happens at a scheduled
    /// instant: the project's own completion item. The progress crossed the
    /// target inside the integration, where there is no random source to draw
    /// from.
    ///
    /// **The chronicle entries are written here.** The completion runs outside
    /// the month's commit, so it does not share the month's working copy with
    /// the phase that writes the chronicle. Without this the project would
    /// finish and nobody would ever hear about it.
    fn complete_project(&mut self, at: Instant, project: domain::ids::ResearchProjectId) {
        if !self
            .state
            .projects
            .get(&project)
            .is_some_and(|project| project.progress.is_complete())
        {
            return;
        }
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            crate::phases::research::complete_project(&mut context, project);
            for entry in context.events {
                working.chronicle(entry);
            }
            self.state = working;
        }
    }

    /// Resolves the month's work.
    fn resolve_month(&mut self, at: Instant) -> Result<MonthReport, SimError> {
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            // The orders that landed since the last report, and the ones that
            // were refused, belong to this report.
            context
                .personnel_lines
                .extend(std::mem::take(&mut self.pending_lines));
            context
                .rejections
                .extend(std::mem::take(&mut self.pending_rejections));
            tick::resolve_month(&mut context)?;
            let report = report::build(&context);
            self.state = working;
            Ok(report)
        }
    }

    /// Draws the month's random events.
    ///
    /// **The events phase runs outside the month's commit**, so its chronicle
    /// entries are written here rather than by the commit phase. Without this an
    /// event would fire and never be recorded.
    fn resolve_events(&mut self, at: Instant) -> Result<(), SimError> {
        let mut working = self.state.clone();
        {
            let mut context = tick::MonthContext::new(&mut working, &self.rng, at, Vec::new());
            tick::resolve_events(&mut context)?;
            for entry in context.events {
                working.chronicle(entry);
            }
            self.state = working;
        }
        Ok(())
    }

    /// How many days between event checks.
    ///
    /// The interval tightens when the agency is under strain, so that a run of
    /// bad luck becomes a run of bad luck rather than one event a week.
    fn event_check_interval_days(&self) -> u64 {
        let state = &self.state;
        let strained = state.treasury.balance <= Money::ZERO
            || state
                .people
                .values()
                .any(|person| person.state.fatigue_percent() >= 80)
            || state
                .facilities
                .values()
                .any(|facility| facility.condition < 50);
        if strained {
            balance::EVENT_CHECK_INTERVAL_DAYS / 2
        } else {
            balance::EVENT_CHECK_INTERVAL_DAYS
        }
    }
    /// Serialises the campaign to a save.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if the campaign cannot be written.
    pub fn save(&self) -> Result<String, SimError> {
        let envelope = SaveEnvelope::new(
            self.now,
            self.rng.root_seed(),
            self.state.clone(),
            self.scenario.clone(),
            self.log.clone(),
            self.scheduler.clone(),
        );
        envelope.to_json()
    }

    /// Reads a campaign from a save.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] describing the first problem found.
    pub fn load(text: &str) -> Result<Self, SimError> {
        let envelope = SaveEnvelope::from_json(text)?;
        domain::invariant::check(&envelope.state).map_err(|violation| SimError::CorruptState {
            reason: violation.to_string(),
        })?;
        let sequence = envelope.log.last().map_or(0, |entry| entry.seq);
        let calendar = Calendar::new(envelope.state.agency.founded);
        // The scheduler comes back with the campaign. Re-scheduling the
        // recurring work here would re-fire every month boundary that already
        // happened, and the campaign would silently change.
        Ok(Self {
            state: envelope.state,
            scenario: envelope.scenario,
            rng: Rng::new(envelope.root_seed),
            now: envelope.now,
            calendar,
            scheduler: envelope.scheduler,
            carry: Duration::ZERO,
            pending_lines: Vec::new(),
            pending_rejections: Vec::new(),
            log: envelope.log,
            sequence,
        })
    }

    /// A hash over the whole campaign state.
    #[must_use]
    pub fn state_hash(&self) -> u64 {
        state_hash(&self.state)
    }

    /// The scenario the campaign started from.
    #[must_use]
    pub const fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// Forks the campaign at an instant.
    ///
    /// **The fork replays the order log**, so the branch is the *same campaign*
    /// up to the fork point — the same state, the same chronicle, the same
    /// recruits — and the original log is preserved. That is what makes "what
    /// if I had not cancelled that programme?" a question the player can
    /// answer.
    ///
    /// The replay re-issues the logged orders at their logged **simulated**
    /// instants, and advances the campaign to each of them in turn. The branch
    /// then continues from the fork point with a **new** log.
    ///
    /// **The fork point is usually in the past [D].** That is the point: the
    /// player looks back at a decision and asks what would have happened
    /// otherwise. The branch is the campaign as it was at that instant, and it
    /// plays on from there while the original carries on.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if the log cannot be replayed.
    pub fn fork(&self, at: Instant) -> Result<Self, SimError> {
        // The branch starts from the scenario, with the campaign's own seed.
        let mut scenario = self.scenario.clone();
        scenario.seed = self.rng.root_seed();
        let mut branch = Self::new(&scenario)?;

        for entry in &self.log {
            if entry.issued_at > at {
                break;
            }
            // The campaign reaches the instant the order was given, so the
            // branch is in the same state the original was in.
            branch.advance_to(entry.issued_at)?;
            let kind: OrderKind =
                serde_json::from_value(entry.payload.clone()).map_err(|error| {
                    SimError::CorruptState {
                        reason: format!("The order log could not be read: {error}"),
                    }
                })?;
            // An order that was accepted once is accepted again: the branch is
            // in the same state the original was in. If it is not, the log and
            // the campaign have drifted apart, and that is a bug worth hearing
            // about rather than a branch to hide.
            branch
                .issue_at(kind, entry.issued_at)
                .map_err(|error| SimError::CorruptState {
                    reason: format!("The order log could not be replayed: {error}"),
                })?;
        }

        // And to the fork point itself.
        branch.advance_to(at)?;
        Ok(branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::PersonId;

    #[test]
    fn a_new_campaign_starts_at_the_first_instant() {
        let sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        assert_eq!(sim.now(), Instant::ZERO);
    }

    #[test]
    fn a_month_advances_the_clock() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(Vec::new()).expect("the month resolves");
        assert_eq!(sim.now(), Instant::from_months(1));
    }

    #[test]
    fn a_new_campaign_has_a_month_boundary_and_an_event_check_scheduled() {
        let sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        assert!(!sim.scheduler().is_empty());
        assert_eq!(sim.scheduler().next_instant(), Some(Instant::ZERO));
    }

    #[test]
    fn advancing_fires_the_scheduled_items() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        // Twelve months crosses twelve month boundaries.
        let reports = sim
            .advance_to(Instant::from_months(12))
            .expect("the campaign advances");
        assert_eq!(reports.len(), 12);
        assert_eq!(sim.now(), Instant::from_months(12));
    }

    #[test]
    fn advancing_to_the_same_instant_twice_does_nothing() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.advance_to(Instant::from_months(6)).expect("advances");
        let hash = sim.state_hash();
        sim.advance_to(Instant::from_months(6)).expect("advances");
        assert_eq!(sim.state_hash(), hash);
    }

    #[test]
    fn advancing_backwards_does_nothing() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.advance_to(Instant::from_months(6)).expect("advances");
        let hash = sim.state_hash();
        sim.advance_to(Instant::from_months(2)).expect("advances");
        assert_eq!(sim.state_hash(), hash);
        assert_eq!(sim.now(), Instant::from_months(6));
    }

    #[test]
    fn the_month_boundary_schedules_the_next_one() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.advance_to(Instant::from_months(1)).expect("advances");
        // The boundary that fired scheduled its successor, and the event check
        // scheduled itself.
        let instants: Vec<Instant> = {
            let mut items = Vec::new();
            let mut scheduler = sim.scheduler().clone();
            while let Some(item) = scheduler.pop_next() {
                items.push(item.at);
            }
            items
        };
        assert!(instants.contains(&Instant::from_months(1)));
        assert!(instants.iter().any(|at| *at > Instant::from_days(7)));
    }

    #[test]
    fn the_event_check_fires_on_its_own_schedule() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        // A week is well inside a month, so the check fires before the boundary.
        sim.advance_to(Instant::from_days(8)).expect("advances");
        // The event check scheduled its successor.
        let next = sim.scheduler().next_instant();
        assert!(next.is_some());
    }

    #[test]
    fn the_clock_advances_in_whole_quarters_of_an_hour() {
        // The clock lands on the integration grid, which is what makes the
        // campaign independent of the host's frame rate.
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.advance_to(Instant::ZERO.saturating_add(Duration::from_millis(1_234_567)))
            .expect("advances");
        assert_eq!(sim.now().millis() % crate::rates::QUANTUM.millis(), 0);
    }

    #[test]
    fn the_leftover_is_carried_and_not_lost() {
        // A host that asks for a fraction of an hour does not lose it: the
        // fraction is kept until it adds up to a whole hour.
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        let quarter = Duration::from_millis(crate::rates::QUANTUM.millis() / 4);
        for _ in 0..4 {
            sim.advance_to(sim.effective_now().saturating_add(quarter))
                .expect("advances");
        }
        // Four quarters of an hour is one whole hour, and the clock has moved
        // by exactly that.
        assert_eq!(sim.now().millis(), crate::rates::QUANTUM.millis());
    }

    // ---- The order model ----

    /// A hire, at the entry salary for a scientist.
    fn hire() -> OrderKind {
        OrderKind::Hire {
            role: domain::person::Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        }
    }

    #[test]
    fn an_order_takes_time_to_land() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        let before = sim.state().people.len();
        sim.issue(hire()).expect("the order is accepted");

        // The order is in flight, and the recruit has not arrived.
        assert_eq!(sim.pending_orders().len(), 1);
        assert_eq!(sim.state().people.len(), before);

        // A month is longer than a hire's lead time.
        sim.advance_to(Instant::from_months(1)).expect("advances");
        assert_eq!(
            sim.state().people.len(),
            before + 1,
            "the recruit should have arrived"
        );
        assert!(sim.pending_orders().is_empty());
    }

    #[test]
    fn the_lead_time_is_visible_before_the_order_is_confirmed() {
        let lead = crate::orders::base_lead_time(&hire(), Instant::ZERO);
        assert!(lead > Duration::ZERO, "a hire is not instant");
        assert!(
            lead <= Duration::from_days(42),
            "a hire does not take more than six weeks"
        );
    }

    #[test]
    fn an_order_that_cannot_be_honoured_is_refused_at_once() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        let error = sim
            .issue(OrderKind::Promote {
                person: PersonId(99),
            })
            .expect_err("the order is refused");
        assert!(error.to_string().contains("no longer with the agency"));
        // Nothing was scheduled, and nothing was charged.
        assert!(sim.pending_orders().is_empty());
    }

    #[test]
    fn an_order_with_no_handler_is_refused_rather_than_ignored() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        let error = sim
            .issue(OrderKind::SetPolicy {
                policy: "Open the launch manifest".to_owned(),
                enabled: true,
            })
            .expect_err("the order is refused");
        assert!(error.to_string().contains("no way to"));
        assert!(sim.pending_orders().is_empty());
    }

    #[test]
    fn an_order_is_charged_when_it_lands_and_not_before() {
        // A campaign with nobody in it and nothing to maintain, so the only
        // money that moves is the order's cost.
        let mut scenario = Scenario::first_year();
        scenario.start_state.people.clear();
        scenario.start_state.facilities.clear();
        let mut sim = Sim::new(&scenario).expect("the scenario starts");

        let salary = domain::money::Money::from_credits(1_200);
        let before = sim.state().treasury.balance.minor();
        sim.issue(OrderKind::Hire {
            role: domain::person::Role::Scientist,
            salary,
        })
        .expect("the order is accepted");

        // Nothing is charged yet: the cost lands with the order.
        assert_eq!(sim.state().treasury.balance.minor(), before);

        // The instant the order lands, and the instant just before it.
        let lands = sim
            .pending_orders()
            .values()
            .next()
            .expect("an order")
            .effective_at;

        // **A control campaign that never hires**, from the same scenario. The
        // two drain the same treasury at the same rate, so the difference
        // between them is the order and nothing else.
        let mut control = Sim::new(&scenario).expect("the scenario starts");
        control
            .advance_to(lands.saturating_sub(Duration::from_hours(1)))
            .expect("advances");
        let control_before = control.state().treasury.balance.minor();

        sim.advance_to(lands.saturating_sub(Duration::from_hours(1)))
            .expect("advances");
        let just_before = sim.state().treasury.balance.minor();

        // **Nothing is charged yet.** The cost lands with the order.
        assert_eq!(
            just_before - control_before,
            0,
            "the signing fee is not charged when the order is given"
        );

        // **The order lands when the clock reaches it**, so the advance has to
        // carry on past it.
        control
            .advance_to(lands.saturating_add(Duration::from_hours(1)))
            .expect("advances");
        sim.advance_to(lands.saturating_add(Duration::from_hours(1)))
            .expect("advances");

        // **And now it is.** The drop is the fee, plus the hour the recruit
        // spends being paid.
        let drop = just_before - sim.state().treasury.balance.minor();
        let control_drop = control_before - control.state().treasury.balance.minor();
        let hour_of_pay = salary.minor() / 720;
        assert!(
            drop - control_drop >= salary.minor(),
            "the signing fee is charged when the hire lands: {drop}"
        );
        assert!(
            drop - control_drop < salary.minor() + hour_of_pay + 10,
            "and the rest is the hour the recruit is paid for: {drop}"
        );
        assert_eq!(sim.state().people.len(), 1, "the recruit has arrived");
    }
    #[test]
    fn an_order_that_can_no_longer_be_honoured_is_refused_at_landing() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        // The order is valid now.
        sim.issue(OrderKind::Assign {
            person: PersonId(1),
            to: domain::person::Assignment::Division(domain::ids::DivisionId(0)),
        })
        .expect("the order is accepted");

        // And then the world moves on: the person is given something else.
        {
            let person = sim
                .state
                .people
                .get_mut(&PersonId(1))
                .expect("the person exists");
            person.assignment = domain::person::Assignment::Division(domain::ids::DivisionId(1));
        }

        let reports = sim
            .advance_to(Instant::from_months(2))
            .expect("advances")
            .into_iter()
            .collect::<Vec<_>>();
        let last = reports.last().expect("a report");
        assert!(
            last.rejections
                .iter()
                .any(|reason| reason.contains("already assigned")),
            "the order should have been refused at landing: {:?}",
            last.rejections
        );
        assert!(sim.pending_orders().is_empty());
        // The refusal is in the chronicle too, which is the durable record.
        assert!(
            sim.state()
                .chronicle
                .iter()
                .any(|entry| entry.headline.contains("refused")),
            "the refusal should be in the chronicle"
        );
    }

    #[test]
    fn the_orders_in_flight_survive_a_save() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.issue(hire()).expect("the order is accepted");
        let text = sim.save().expect("the campaign saves");
        let restored = Sim::load(&text).expect("the campaign loads");
        assert_eq!(restored.pending_orders().len(), 1);
        assert_eq!(
            restored
                .pending_orders()
                .values()
                .next()
                .expect("an order")
                .kind,
            hire()
        );
    }

    // ---- The fork ----

    #[test]
    fn a_fork_is_the_same_campaign_up_to_the_fork_point() {
        // **The fork is exact.** The branch is the same campaign up to the fork
        // point: the same state, the same chronicle, the same recruits. If it
        // were not, every difference would be noise.
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![hire()]).expect("the month resolves");
        sim.step(Vec::new()).expect("the month resolves");
        sim.step(vec![OrderKind::StartProject {
            discipline: domain::research::DisciplineId(0),
            principal_investigator: PersonId(0),
            title: "Liquid upper stage".to_owned(),
        }])
        .expect("the month resolves");

        let branch = sim.fork(sim.effective_now()).expect("the campaign forks");

        assert_eq!(branch.now(), sim.effective_now());
        assert_eq!(
            branch.state_hash(),
            sim.state_hash(),
            "the branch must be the same campaign up to the fork point"
        );
    }

    #[test]
    fn a_fork_in_the_past_replays_the_log_to_that_point() {
        // **The fork point is usually in the past.** That is the point: the
        // player looks back at a decision and asks what would have happened
        // otherwise.
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![hire()]).expect("the month resolves");
        sim.step(Vec::new()).expect("the month resolves");
        sim.step(vec![OrderKind::StartProject {
            discipline: domain::research::DisciplineId(0),
            principal_investigator: PersonId(0),
            title: "Liquid upper stage".to_owned(),
        }])
        .expect("the month resolves");

        let branch = sim
            .fork(Instant::from_months(1))
            .expect("the campaign forks");
        assert_eq!(
            branch.now(),
            Instant::from_months(1),
            "the branch is the campaign as it was at the fork point"
        );
        assert!(
            branch.log().len() < sim.log().len(),
            "the branch's log stops at the fork point"
        );
    }

    #[test]
    fn a_fork_keeps_the_original_log() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![hire()]).expect("the month resolves");
        let before = sim.log().len();
        let branch = sim.fork(sim.effective_now()).expect("the campaign forks");
        assert_eq!(sim.log().len(), before, "the original log is preserved");
        assert_eq!(branch.log().len(), before, "and the branch has its own");
    }

    #[test]
    fn a_branch_and_the_original_can_be_compared() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![hire()]).expect("the month resolves");
        let mut branch = sim
            .fork(Instant::from_months(1))
            .expect("the campaign forks");
        // The branch does something the original did not.
        branch
            .issue(OrderKind::Hire {
                role: domain::person::Role::Scientist,
                salary: domain::money::Money::from_credits(1_200),
            })
            .expect("the order is accepted");
        branch
            .advance_to(Instant::from_months(2))
            .expect("the campaign advances");

        let differences = crate::branch::divergence(sim.state(), branch.state());
        assert!(
            differences
                .iter()
                .any(|difference| difference.subject == "Staff"),
            "the branches should differ in staff: {differences:?}"
        );
    }

    #[test]
    fn a_forked_campaign_can_be_saved_and_loaded() {
        // **A campaign has to be forkable after it is loaded**, and forking
        // needs the scenario's start state.
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![hire()]).expect("the month resolves");
        let text = sim.save().expect("the campaign saves");
        let restored = Sim::load(&text).expect("the campaign loads");
        let branch = restored
            .fork(Instant::from_months(1))
            .expect("the campaign forks");
        assert_eq!(branch.log().len(), restored.log().len());
    }

    // ---- The goodwill ----

    #[test]
    fn a_landed_order_charges_the_goodwill() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        assert_eq!(sim.state().political.goodwill.percent(), 0);

        sim.issue(hire()).expect("the order is accepted");
        // The goodwill is spent when the order *lands*, not when it is issued.
        assert_eq!(sim.state().political.goodwill.percent(), 0);

        // The instant the order lands, and the instant just before it.
        let lands = sim
            .pending_orders()
            .values()
            .next()
            .expect("an order")
            .effective_at;
        sim.advance_to(lands.saturating_sub(Duration::from_hours(1)))
            .expect("advances");
        let just_before = sim.state().political.goodwill.percent();
        sim.advance_to(lands.saturating_add(Duration::from_hours(1)))
            .expect("advances");

        assert_eq!(
            sim.state().political.goodwill.percent(),
            just_before - crate::balance::GOODWILL_HIRE,
            "a hire costs a point of goodwill"
        );
    }

    #[test]
    fn a_refused_order_costs_no_goodwill() {
        // Two campaigns: one that issues an order that cannot be honoured, and
        // one that does not. They must end up with the same goodwill.
        let mut refused = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        refused
            .issue(OrderKind::Assign {
                person: PersonId(1),
                to: domain::person::Assignment::Division(domain::ids::DivisionId(0)),
            })
            .expect("the order is accepted");
        {
            let person = refused
                .state
                .people
                .get_mut(&PersonId(1))
                .expect("the person exists");
            person.assignment = domain::person::Assignment::Division(domain::ids::DivisionId(1));
        }

        let mut quiet = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        refused
            .advance_to(Instant::from_months(2))
            .expect("advances");
        quiet.advance_to(Instant::from_months(2)).expect("advances");

        assert_eq!(
            refused.state().political.goodwill.percent(),
            quiet.state().political.goodwill.percent(),
            "an order that never lands costs nothing"
        );
    }

    #[test]
    fn the_goodwill_comes_back_slowly() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.issue(hire()).expect("the order is accepted");
        sim.advance_to(Instant::from_months(1)).expect("advances");
        let after_the_order = sim.state().political.goodwill.percent();

        // A year of doing nothing, and the goodwill is most of the way back.
        sim.advance_to(Instant::from_months(13)).expect("advances");
        assert!(
            sim.state().political.goodwill.percent() > after_the_order,
            "the goodwill regenerates"
        );
    }

    #[test]
    fn a_hostile_state_makes_the_orders_take_longer() {
        // **Low goodwill is the anti-spamming mechanism** - not a cap, but a
        // consequence. The player sees the delay before they commit.
        let base = crate::orders::base_lead_time(&hire(), Instant::ZERO);
        let hostile = domain::political::Goodwill {
            value: domain::scale::signed_percent_to_units(-100),
        };
        let friendly = domain::political::Goodwill {
            value: domain::scale::signed_percent_to_units(100),
        };
        let hostile_lead = crate::orders::adjusted_lead_time(base, &hostile);
        let friendly_lead = crate::orders::adjusted_lead_time(base, &friendly);
        assert!(hostile_lead > base, "a hostile state is slow");
        assert!(friendly_lead < base, "a friendly state is quick");
    }

    #[test]
    fn a_hostile_state_charges_more() {
        let base = domain::money::Money::from_credits(1_000);
        let hostile = domain::political::Goodwill {
            value: domain::scale::signed_percent_to_units(-100),
        };
        let friendly = domain::political::Goodwill {
            value: domain::scale::signed_percent_to_units(100),
        };
        assert!(
            crate::orders::adjusted_cost(base, &hostile) > base,
            "an unfriendly state charges a third more"
        );
        assert!(
            crate::orders::adjusted_cost(base, &friendly) < base,
            "a friendly state charges a third less"
        );
    }

    #[test]
    fn a_campaign_advanced_in_one_go_matches_one_advanced_in_pieces() {
        // The same campaign, reached the same way and the same instant, must
        // be the same campaign however the host chopped up the time.
        let scenario = Scenario::first_year();

        let mut whole = Sim::new(&scenario).expect("the scenario starts");
        whole.step(Vec::new()).expect("the month resolves");
        whole
            .advance_to(Instant::from_months(12))
            .expect("the campaign advances");

        let mut pieces = Sim::new(&scenario).expect("the scenario starts");
        pieces.step(Vec::new()).expect("the month resolves");
        while pieces.now() < Instant::from_months(12) {
            let next = pieces
                .now()
                .saturating_add(Duration::from_hours(37))
                .min(Instant::from_months(12));
            pieces.advance_to(next).expect("the campaign advances");
        }

        assert_eq!(whole.state_hash(), pieces.state_hash());
    }

    #[test]
    fn the_same_campaign_twice_produces_the_same_state() {
        let scenario = Scenario::first_year();
        let mut first = Sim::new(&scenario).expect("the scenario starts");
        let mut second = Sim::new(&scenario).expect("the scenario starts");
        for _ in 0..6 {
            first.step(Vec::new()).expect("the month resolves");
            second.step(Vec::new()).expect("the month resolves");
        }
        assert_eq!(first.state_hash(), second.state_hash());
    }

    #[test]
    fn a_campaign_round_trips_through_a_save() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        for _ in 0..3 {
            sim.step(Vec::new()).expect("the month resolves");
        }
        let text = sim.save().expect("the campaign saves");
        let restored = Sim::load(&text).expect("the campaign loads");
        assert_eq!(restored.state_hash(), sim.state_hash());
        assert_eq!(restored.now(), sim.now());
    }

    #[test]
    fn orders_are_recorded_in_the_log() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        sim.step(vec![OrderKind::Hire {
            role: domain::person::Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        }])
        .expect("the month resolves");
        assert_eq!(sim.log().len(), 1);
        assert_eq!(sim.log()[0].kind, "Hire");
        assert_eq!(sim.log()[0].issued_at, Instant::ZERO);
    }

    #[test]
    fn a_refused_order_changes_nothing() {
        let mut sim = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        let report = sim
            .step(vec![OrderKind::Promote {
                person: PersonId(99),
            }])
            .expect("the month resolves");
        assert_eq!(report.rejections.len(), 1);
        // The refusal is reported, and the state is otherwise the same as a
        // month in which nothing was asked for.
        let mut quiet = Sim::new(&Scenario::first_year()).expect("the scenario starts");
        quiet.step(Vec::new()).expect("the month resolves");
        assert_eq!(sim.state_hash(), quiet.state_hash());
    }

    #[test]
    fn a_month_is_a_thirtieth_of_a_year() {
        assert_eq!(Duration::from_months(12), Duration::from_years(1));
    }
}
