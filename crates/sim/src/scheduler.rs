//! The scheduler.
//!
//! Everything that happens at a definite time is on this queue. The simulation
//! advances by popping the items that are due, integrating the continuous work up
//! to each one, and applying it.
//!
//! **The ordering is a total order** — `(at, priority, seq)` — and that is what
//! makes the resolution sequence deterministic. Two items at the same instant
//! with the same priority are separated by their sequence number, and the
//! sequence number is never reused.
//!
//! See `docs/SIMULATION.md` §3.1.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use domain::ids::{CommitmentId, MissionId, OrderId, PersonId, ResearchProjectId};
use domain::mission::MissionPhase;
use domain::time::Instant;

/// The priority of a month boundary. The month's work comes first.
pub const PRIORITY_MONTH: u8 = 10;

/// The priority of a commitment falling due.
pub const PRIORITY_COMMITMENT: u8 = 40;

/// The priority of an order landing.
pub const PRIORITY_ORDER: u8 = 50;

/// The priority of a mission changing phase.
pub const PRIORITY_MISSION: u8 = 60;

/// The priority of a research project reaching its target.
pub const PRIORITY_PROJECT: u8 = 65;

/// The priority of an attrition check.
pub const PRIORITY_ATTRITION: u8 = 70;

/// The priority of a decision expiring.
///
/// **A decision's window closes after the world's work for the instant**, so
/// the expiry applies to the state the work produced — a decision about money
/// expires against the money the month actually has.
pub const PRIORITY_DECISION: u8 = 75;

/// The priority of an event check.
///
/// **Events resolve last within an instant**, so that they react to the work that
/// has just happened rather than to the state before it.
pub const PRIORITY_EVENT: u8 = 90;

/// Something that happens at a definite instant.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ScheduledItem {
    /// When it happens.
    pub at: Instant,
    /// Lower resolves first, for items at the same instant.
    pub priority: u8,
    /// The final tie-break. Never reused.
    pub seq: u64,
    /// What it is.
    pub kind: ScheduledKind,
}

impl ScheduledItem {
    /// The ordering key, which is the whole of the determinism guarantee.
    fn key(&self) -> (Instant, u8, u64) {
        (self.at, self.priority, self.seq)
    }
}

impl Ord for ScheduledItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for ScheduledItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// What is scheduled.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ScheduledKind {
    /// The month's work: the continuous accrual, the periodic triggers, and the
    /// month's decisions.
    MonthBoundary,
    /// The moment a random event is drawn.
    EventCheck,
    /// The moment an order lands and takes effect.
    OrderLands {
        /// The order.
        order: OrderId,
    },
    /// The moment a vehicle finishes being built.
    ///
    /// **The build completes at an instant**, the same way a project does: the
    /// progress crosses the target inside the integration, where there is no
    /// random source to draw from.
    VehicleComplete {
        /// The vehicle.
        vehicle: domain::ids::VehicleId,
    },
    /// The moment a follow-up event fires.
    EventFollowUp {
        /// The event.
        event: domain::ids::EventId,
    },
    /// The moment a mission changes phase.
    MissionPhase {
        /// The mission.
        mission: MissionId,
        /// The phase it is entering.
        phase: MissionPhase,
    },
    /// The moment a research project reaches its target.
    ///
    /// The progress is a rate and it crosses the target inside the
    /// integration, where there is no random source to draw from. The
    /// crossing schedules one of these, and the completion — which draws — is
    /// what happens when it fires.
    ProjectComplete {
        /// The project.
        project: ResearchProjectId,
    },
    /// The moment a dated commitment falls due.
    CommitmentDue {
        /// The commitment.
        commitment: CommitmentId,
    },
    /// The moment a person's fortune is rolled.
    AttritionCheck {
        /// The person.
        person: PersonId,
    },
    /// The moment a decision's answer is due.
    ///
    /// **A decision has a window, and the window is scheduled.** If the player
    /// has answered by the time this fires, the item is a no-op; if they have
    /// not, the expiry applies and the agency's silence has its cost.
    DecisionExpiry {
        /// The pending decision whose window has closed.
        decision: u64,
    },
}

impl ScheduledKind {
    /// The kind's priority. Lower resolves first.
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::MonthBoundary => PRIORITY_MONTH,
            Self::CommitmentDue { .. } => PRIORITY_COMMITMENT,
            Self::OrderLands { .. } => PRIORITY_ORDER,
            Self::MissionPhase { .. } => PRIORITY_MISSION,
            Self::ProjectComplete { .. } | Self::VehicleComplete { .. } => PRIORITY_PROJECT,
            Self::AttritionCheck { .. } => PRIORITY_ATTRITION,
            Self::DecisionExpiry { .. } => PRIORITY_DECISION,
            Self::EventCheck | Self::EventFollowUp { .. } => PRIORITY_EVENT,
        }
    }
}

/// The queue of scheduled work.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheduler {
    queue: BTreeSet<ScheduledItem>,
    next_seq: u64,
}

impl Scheduler {
    /// An empty scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedules something, and returns the item that was placed.
    pub fn schedule(&mut self, at: Instant, kind: ScheduledKind) -> ScheduledItem {
        let item = ScheduledItem {
            at,
            priority: kind.priority(),
            seq: self.next_seq,
            kind,
        };
        self.next_seq += 1;
        self.queue.insert(item.clone());
        item
    }

    /// The next item, if there is one.
    #[must_use]
    pub fn peek(&self) -> Option<&ScheduledItem> {
        self.queue.iter().next()
    }

    /// Everything waiting, in the order it will happen.
    ///
    /// **The queue is a set, and this is its order.** The interface's forecast
    /// and the cross-system invariants both need to see what is waiting, and
    /// they must see it in the order the scheduler will pop it.
    #[must_use]
    pub fn items(&self) -> Vec<&ScheduledItem> {
        self.queue.iter().collect()
    }

    /// Removes and returns the next item, if it falls at or before `now`.
    pub fn pop_due(&mut self, now: Instant) -> Option<ScheduledItem> {
        let is_due = self.queue.iter().next().is_some_and(|item| item.at <= now);
        if is_due { self.queue.pop_first() } else { None }
    }

    /// Removes and returns the next item, whatever it is.
    pub fn pop_next(&mut self) -> Option<ScheduledItem> {
        self.queue.pop_first()
    }

    /// How many items are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// The next instant at which anything will happen.
    #[must_use]
    pub fn next_instant(&self) -> Option<Instant> {
        self.queue.iter().next().map(|item| item.at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_come_out_in_time_order() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::from_months(3), ScheduledKind::EventCheck);
        scheduler.schedule(Instant::from_months(1), ScheduledKind::EventCheck);
        scheduler.schedule(Instant::from_months(2), ScheduledKind::EventCheck);

        let first = scheduler.pop_next().expect("there is an item");
        assert_eq!(first.at, Instant::from_months(1));
    }

    #[test]
    fn items_at_the_same_instant_are_ordered_by_priority() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::from_months(1), ScheduledKind::EventCheck);
        scheduler.schedule(Instant::from_months(1), ScheduledKind::MonthBoundary);

        let first = scheduler.pop_next().expect("there is an item");
        assert_eq!(first.kind, ScheduledKind::MonthBoundary);
    }

    #[test]
    fn events_resolve_last_within_an_instant() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::from_months(1), ScheduledKind::EventCheck);
        scheduler.schedule(
            Instant::from_months(1),
            ScheduledKind::AttritionCheck {
                person: PersonId(0),
            },
        );
        scheduler.schedule(
            Instant::from_months(1),
            ScheduledKind::OrderLands { order: OrderId(0) },
        );
        scheduler.schedule(Instant::from_months(1), ScheduledKind::MonthBoundary);

        let mut kinds = Vec::new();
        while let Some(item) = scheduler.pop_next() {
            kinds.push(item.kind);
        }
        assert_eq!(kinds.first(), Some(&ScheduledKind::MonthBoundary));
        assert_eq!(kinds.last(), Some(&ScheduledKind::EventCheck));
    }

    #[test]
    fn only_due_items_are_popped() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::from_months(5), ScheduledKind::EventCheck);

        assert!(scheduler.pop_due(Instant::from_months(1)).is_none());
        assert!(scheduler.pop_due(Instant::from_months(5)).is_some());
    }

    #[test]
    fn the_sequence_number_is_never_reused() {
        let mut scheduler = Scheduler::new();
        let first = scheduler.schedule(Instant::ZERO, ScheduledKind::EventCheck);
        let second = scheduler.schedule(Instant::ZERO, ScheduledKind::EventCheck);
        assert_ne!(first.seq, second.seq);
    }

    #[test]
    fn two_identical_items_do_not_collide() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::ZERO, ScheduledKind::EventCheck);
        scheduler.schedule(Instant::ZERO, ScheduledKind::EventCheck);
        assert_eq!(scheduler.len(), 2);
    }

    #[test]
    fn the_next_instant_is_reported() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(Instant::from_months(4), ScheduledKind::EventCheck);
        assert_eq!(scheduler.next_instant(), Some(Instant::from_months(4)));
    }
}
