//! The decisions an event puts to the player.
//!
//! **An event that asks nothing is a sentence; an event that asks something is
//! a decision.** The event catalogue is a list of things that happen; the
//! decision catalogue is the list of things that happen *and want an answer*.
//! When an event fires and it carries a decision here, the simulation holds a
//! pending decision instead of resolving the question itself, and the host —
//! the interface — stops the clock and shows it.
//!
//! The window is the whole of the design: **a decision that waits forever is a
//! message, and a message the player answers when they like is not a
//! decision.** Each decision has days to answer and an expiry — what happens
//! when the agency says nothing. Silence is an answer, and it is usually the
//! expensive one.
//!
//! The decision is data, like the event: options are deltas, and the
//! catalogue cannot name a person or a project that does not exist yet. The
//! same rule the effects follow, followed one more level down.

use domain::event::{Decision, DecisionOption, Effect};
use domain::ids::EventId;
use domain::money::Money;
use domain::time::Instant;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A decision the agency has been asked and has not answered.
///
/// It is held on the simulation, not in the campaign state, for the same
/// reason the scheduler is: it is in flight, not a fact about the world. It
/// is part of the save, because a campaign with a decision on the table must
/// come back with it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PendingDecision {
    /// The decision's identifier, monotonic across the campaign.
    pub id: u64,
    /// The event that raised it.
    pub event: EventId,
    /// The instant the answer is due.
    pub due: Instant,
}

impl PendingDecision {
    /// The decision as defined, from the catalogue.
    ///
    /// # Panics
    ///
    /// Panics if the catalogue has no decision for the event. A pending
    /// decision is only ever created from the catalogue, so this cannot fail
    /// in a campaign that came from a save that round-trips.
    #[must_use]
    pub fn definition(&self) -> &'static Decision {
        decision_for(self.event)
            .expect("a pending decision always has a definition in the catalogue")
    }
}

/// The decision that goes with an event, if it has one.
///
/// **The catalogue is the list of questions the world asks.** An event without
/// an entry here fires and is done — the world narrates, and nobody is
/// consulted. An event with an entry here raises a pending decision, and the
/// agency has to answer it.
#[must_use]
pub fn decision_for(event: EventId) -> Option<&'static Decision> {
    DECISIONS
        .iter()
        .find(|decision| decision.event == event)
        .map(|entry| &entry.decision)
}

/// One entry in the decision catalogue: the event it answers to, and the
/// question.
struct Entry {
    /// The event that raises it.
    event: EventId,
    /// The question.
    decision: Decision,
}

/// The catalogue.
///
/// **The catalogue is data and it is built once**, the way the event catalogue
/// is: a lazy static, because a decision holds vectors, and a vector cannot
/// be a constant.
static DECISIONS: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    vec![
        // **The tired shift** (event 5). A long shift went badly, and the agency
        // can either pay for the rest now or pay for the drift later. The event's
        // own effects are the baseline — morale and loyalty already fell.
        Entry {
            event: EventId(5),
            decision: Decision {
                headline: "The shift is asking for a break.".to_owned(),
                body: Some(
                    "A long shift went badly, and the people are asking whether the pace holds."
                        .to_owned(),
                ),
                days_to_decide: 5,
                expiry_effects: vec![Effect::AdjustAllLoyalty { delta: -2 }],
                options: vec![
                    DecisionOption {
                        id: "rest".to_owned(),
                        label: "Give the shift the week off".to_owned(),
                        effects: vec![Effect::AdjustAllMorale { delta: 4 }],
                    },
                    DecisionOption {
                        id: "hold".to_owned(),
                        label: "Hold the pace for now".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The contractor** (event 20). The event's own effect is the early
        // payment itself; the decision is how the agency answers the ask. Paying
        // now costs money and buys goodwill with the state; stalling saves the
        // money and lets the answer drift.
        Entry {
            event: EventId(20),
            decision: Decision {
                headline: "A contractor is asking to be paid early.".to_owned(),
                body: None,
                days_to_decide: 10,
                expiry_effects: vec![Effect::AdjustPrestige { delta: -1 }],
                options: vec![
                    DecisionOption {
                        id: "pay".to_owned(),
                        label: "Pay early, and note it in the books".to_owned(),
                        effects: vec![Effect::AdjustTreasury {
                            delta: Money::from_credits(-5_000),
                        }],
                    },
                    DecisionOption {
                        id: "stall".to_owned(),
                        label: "Pay on the contracted date".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The audit** (event 22). The event's own effect is the prestige cost of
        // being audited at all; the decision is whether the agency cooperates.
        // Opening the books costs nothing but the work, and the state remembers.
        Entry {
            event: EventId(22),
            decision: Decision {
                headline: "The state wants the agency's books.".to_owned(),
                body: Some("The audit is a formality, unless it is not.".to_owned()),
                days_to_decide: 7,
                expiry_effects: vec![Effect::AdjustApproval { delta: -3 }],
                options: vec![
                    DecisionOption {
                        id: "open".to_owned(),
                        label: "Open the books to the audit".to_owned(),
                        effects: vec![Effect::AdjustApproval { delta: 2 }],
                    },
                    DecisionOption {
                        id: "decline".to_owned(),
                        label: "Decline, and say why".to_owned(),
                        effects: vec![Effect::AdjustPrestige { delta: 1 }],
                    },
                ],
            },
        },
        // **The interview** (event 28). The event's own effect is the opinion cost
        // of the awkward questions; the decision is whether the agency faces the
        // camera. A voice in the room is worth more than no voice at all.
        Entry {
            event: EventId(28),
            decision: Decision {
                headline: "A television programme wants an interview.".to_owned(),
                body: None,
                days_to_decide: 5,
                expiry_effects: vec![Effect::AdjustPublicOpinion { delta: -2 }],
                options: vec![
                    DecisionOption {
                        id: "answer".to_owned(),
                        label: "Send somebody to answer the questions".to_owned(),
                        effects: vec![Effect::AdjustPublicOpinion { delta: 4 }],
                    },
                    DecisionOption {
                        id: "silence".to_owned(),
                        label: "Say nothing, and let it pass".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The weekend** (event 6). The work got done, and somebody paid for it in
        // hours nobody asked for. The event's own effect is the fatigue; the decision
        // is whether the agency gives the time back. Time off costs the schedule and
        // buys goodwill with the people; a word costs nothing and reads as noticed.
        // Saying nothing costs nothing now, and reads as taken for granted later.
        Entry {
            event: EventId(6),
            decision: Decision {
                headline: "Somebody worked the weekend, and the agency should say something."
                    .to_owned(),
                body: None,
                days_to_decide: 5,
                expiry_effects: vec![Effect::AdjustAllLoyalty { delta: -2 }],
                options: vec![
                    DecisionOption {
                        id: "comp".to_owned(),
                        label: "Give the hours back as time off".to_owned(),
                        effects: vec![
                            Effect::AdjustAllFatigue { delta: -4 },
                            Effect::AdjustAllMorale { delta: 2 },
                        ],
                    },
                    DecisionOption {
                        id: "note".to_owned(),
                        label: "Note it, and let the pace hold".to_owned(),
                        effects: vec![Effect::AdjustAllMorale { delta: 1 }],
                    },
                ],
            },
        },
        // **The slipped programme** (event 16). The state has noticed the schedule
        // slipping, and the decision is whether the agency meets the question head
        // on or lets it pass. Asking for time costs pride and buys patience; saying
        // nothing saves the conversation and lets the suspicion grow.
        Entry {
            event: EventId(16),
            decision: Decision {
                headline: "The state has noticed a programme is behind.".to_owned(),
                body: Some("A letter has arrived, and it wants an answer.".to_owned()),
                days_to_decide: 10,
                expiry_effects: vec![Effect::AdjustApproval { delta: -2 }],
                options: vec![
                    DecisionOption {
                        id: "explain".to_owned(),
                        label: "Explain the slip, and ask for time".to_owned(),
                        effects: vec![Effect::AdjustApproval { delta: 2 }],
                    },
                    DecisionOption {
                        id: "quiet".to_owned(),
                        label: "Say nothing, and fix it quietly".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The bill** (event 19). The money has already gone; the decision is
        // whether the agency argues about it. Disputing it recovers part of the cost
        // and costs the agency's standing with the state; letting it stand is tidy
        // and quiet, and word gets around either way.
        Entry {
            event: EventId(19),
            decision: Decision {
                headline: "A bill nobody planned for has been paid.".to_owned(),
                body: Some(
                    "It had been outstanding for two years, and it is not the agency's fault."
                        .to_owned(),
                ),
                days_to_decide: 10,
                expiry_effects: vec![Effect::AdjustPrestige { delta: -1 }],
                options: vec![
                    DecisionOption {
                        id: "dispute".to_owned(),
                        label: "Dispute the charges, and ask for some back".to_owned(),
                        effects: vec![
                            Effect::AdjustTreasury {
                                delta: Money::from_credits(15_000),
                            },
                            Effect::AdjustPrestige { delta: -1 },
                        ],
                    },
                    DecisionOption {
                        id: "settle".to_owned(),
                        label: "Let it stand, and keep the peace".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The story** (event 27). The event's own effect is the opinion cost of
        // the story running; the decision is whether the agency answers it. A plain
        // answer costs nothing and closes the question; silence lets the story grow
        // on its own.
        Entry {
            event: EventId(27),
            decision: Decision {
                headline: "The press wants the agency's side of a story.".to_owned(),
                body: Some("It is not a serious matter, but it is in the papers.".to_owned()),
                days_to_decide: 5,
                expiry_effects: vec![Effect::AdjustPublicOpinion { delta: -2 }],
                options: vec![
                    DecisionOption {
                        id: "respond".to_owned(),
                        label: "Answer the story, plainly".to_owned(),
                        effects: vec![Effect::AdjustPublicOpinion { delta: 3 }],
                    },
                    DecisionOption {
                        id: "silence".to_owned(),
                        label: "Say nothing, and let it pass".to_owned(),
                        effects: vec![],
                    },
                ],
            },
        },
        // **The poached scientist** (event 32). The event's own effect is the
        // prestige cost of the poaching; the decision is whether the agency
        // counter-offers. Keeping the person costs money; losing them costs the
        // agency's nerve.
        Entry {
            event: EventId(32),
            decision: Decision {
                headline: "A rival has poached a scientist.".to_owned(),
                body: Some("The team is watching what the agency does about it.".to_owned()),
                days_to_decide: 7,
                expiry_effects: vec![Effect::AdjustAllLoyalty { delta: -3 }],
                options: vec![
                    DecisionOption {
                        id: "counter".to_owned(),
                        label: "Counter-offer, and keep them".to_owned(),
                        effects: vec![
                            Effect::AdjustTreasury {
                                delta: Money::from_credits(-15_000),
                            },
                            Effect::AdjustAllLoyalty { delta: 2 },
                        ],
                    },
                    DecisionOption {
                        id: "let_them_go".to_owned(),
                        label: "Let them go, and say so plainly".to_owned(),
                        effects: vec![Effect::AdjustAllMorale { delta: 1 }],
                    },
                ],
            },
        },
    ]
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_decision_belongs_to_a_real_event() {
        // **The catalogue is keyed to the event catalogue**, and an entry
        // pointing at an event that does not exist would raise a decision the
        // world never asked for.
        for entry in DECISIONS.iter() {
            let event = crate::event_catalogue::event(entry.event);
            assert!(
                event.is_some(),
                "decision {} is keyed to event {} which does not exist",
                entry.decision.headline,
                entry.event.0
            );
        }
    }

    #[test]
    fn every_decision_has_options_and_an_expiry() {
        // **The window is the design.** A decision without a window is a
        // message, and a decision without an expiry makes silence free.
        for entry in DECISIONS.iter() {
            let decision = &entry.decision;
            assert!(
                decision.days_to_decide > 0,
                "decision {} has a window",
                decision.headline
            );
            assert!(
                !decision.expiry_effects.is_empty(),
                "silence costs something: {}",
                decision.headline
            );
            assert!(
                decision.options.len() >= 2,
                "a decision with one option is not a decision: {}",
                decision.headline
            );
            for option in &decision.options {
                assert!(!option.id.is_empty());
                assert!(!option.label.is_empty());
            }
        }
    }

    #[test]
    fn option_ids_are_unique_within_a_decision() {
        // **The option id is how the player's answer is recorded**, and two
        // options with the same id would make one of them unreachable.
        for entry in DECISIONS.iter() {
            let mut ids: Vec<&str> = entry
                .decision
                .options
                .iter()
                .map(|o| o.id.as_str())
                .collect();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(
                ids.len(),
                entry.decision.options.len(),
                "the options of {} are distinct",
                entry.decision.headline
            );
        }
    }

    #[test]
    fn the_lookup_finds_a_decision_and_misses_an_event_without_one() {
        assert!(decision_for(EventId(5)).is_some());
        assert!(decision_for(EventId(32)).is_some());
        // **Most events are sentences, and that is correct** — the world
        // narrates far more often than it asks.
        assert!(decision_for(EventId(0)).is_none());
    }

    #[test]
    fn a_pending_decision_carries_a_window_that_expires() {
        // **The window is the whole of the design.** A decision raised now has
        // a due instant in the future, and the number of days matches the
        // catalogue.
        let pending = PendingDecision {
            id: 1,
            event: EventId(5),
            due: Instant::from_days(5),
        };
        assert_eq!(pending.definition().days_to_decide, 5);
    }
}
