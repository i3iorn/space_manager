//! The live feed: what has just happened.
//!
//! **A feed is not a list.** A list is read from the top; a feed is watched.
//! The difference is the order and the freshness: the newest thing is at the
//! top, and it appears the moment it happens, without the player navigating
//! anywhere.
//!
//! The feed is the chronicle read live, plus the player's own decisions and
//! the lines that are not chronicle entries. **The chronicle is still the
//! durable record** — it is what a save keeps and what a fork replays — and
//! the feed is the view of it that moves.
//!
//! See `docs/MVP.md` §2 (the Chronicle row) and `docs/UI.md` §4.1.

use domain::chronicle::Severity;
use domain::time::Instant;

use crate::Sim;

/// One thing in the feed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedItem {
    /// Something the world did.
    Event {
        /// When it happened.
        at: Instant,
        /// How much it matters.
        severity: Severity,
        /// What happened.
        text: String,
    },
    /// Something the player decided.
    ///
    /// **The player's own decisions are in the feed**, because a campaign is
    /// the world *and* what the player did about it, and a record that only
    /// has one of the two is not a record of the campaign.
    Decision {
        /// When the decision was made.
        at: Instant,
        /// What was decided.
        text: String,
    },
}

impl FeedItem {
    /// When it happened.
    #[must_use]
    pub const fn at(&self) -> Instant {
        match self {
            Self::Event { at, .. } | Self::Decision { at, .. } => *at,
        }
    }

    /// What happened, in the player's language.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Event { text, .. } | Self::Decision { text, .. } => text,
        }
    }

    /// How much it matters.
    ///
    /// **A decision is not an event**, and it is shown in a different colour
    /// for that reason: the player can see at a glance what the world did and
    /// what they did about it.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::Event { severity, .. } => *severity,
            Self::Decision { .. } => Severity::Info,
        }
    }

    /// Whether the player made this decision.
    #[must_use]
    pub const fn is_decision(&self) -> bool {
        matches!(self, Self::Decision { .. })
    }
}

/// The feed, newest first.
///
/// **Everything that has happened, most recent first.** The chronicle's entries
/// are the bulk of it; the player's decisions and the live report lines are the
/// rest. The order is by instant, and within an instant the oldest comes
/// first, so the feed reads the way the campaign happened.
///
/// # Errors
///
/// None. A feed is a view of the campaign, and it is built from what is
/// already there.
#[must_use]
pub fn feed(sim: &Sim, limit: usize) -> Vec<FeedItem> {
    let state = sim.state();
    // **The candidates are gathered as references**, and only the ones the
    // feed actually shows are copied. The chronicle is the campaign's whole
    // history and it grows for ever, so copying every headline in it to show
    // twelve of them is a waste that gets worse every month.
    let mut candidates: Vec<(Instant, Severity, &str, bool)> = Vec::new();

    // The chronicle, oldest first.
    for entry in &state.chronicle {
        candidates.push((entry.date, entry.severity, &entry.headline, false));
    }

    // The player's decisions, oldest first.
    for order in sim.log() {
        candidates.push((order.issued_at, Severity::Info, &order.kind, true));
    }

    // The lines that are not chronicle entries: the things that have happened
    // since the last month report and would otherwise wait for it.
    //
    // **They carry the instant they happened**, so the feed's timestamps do
    // not move with the clock.
    for (at, line) in sim.pending_lines() {
        candidates.push((*at, line.severity, &line.text, false));
    }

    // **Newest first.** The sort is stable, so the order within an instant is
    // the order the campaign happened in.
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.0));
    candidates
        .into_iter()
        .take(limit)
        .map(|(at, severity, text, decision)| {
            if decision {
                FeedItem::Decision {
                    at,
                    text: text.to_owned(),
                }
            } else {
                FeedItem::Event {
                    at,
                    severity,
                    text: text.to_owned(),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scenario;

    #[test]
    fn the_feed_is_newest_first() {
        let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(2))
            .expect("the campaign advances");
        let items = feed(&sim, 50);
        assert!(!items.is_empty(), "there is something to report");
        for pair in items.windows(2) {
            assert!(
                pair[0].at() >= pair[1].at(),
                "the feed runs backwards: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn the_feed_carries_the_players_decisions() {
        let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        sim.issue(domain::order::OrderKind::Hire {
            role: domain::person::Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        })
        .expect("the order is accepted");
        let items = feed(&sim, 50);
        assert!(
            items.iter().any(FeedItem::is_decision),
            "the feed carries what the player did"
        );
    }

    #[test]
    fn the_feed_respects_its_limit() {
        let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(2))
            .expect("the campaign advances");
        assert!(feed(&sim, 3).len() <= 3);
    }

    #[test]
    fn the_feed_does_not_say_the_same_thing_twice() {
        // **The event's headline used to be in the chronicle and in the report
        // lines**, so the feed showed every event twice: once from the
        // chronicle and once from the report lines waiting for the month.
        //
        // **The same words at the same instant are the same thing twice.** The
        // same words at different instants are two different things that
        // happened to be alike, which happens all the time.
        let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(2))
            .expect("the campaign advances");
        let items = feed(&sim, 50);
        for (index, item) in items.iter().enumerate() {
            for other in &items[index + 1..] {
                if item.text() == other.text() {
                    assert_ne!(
                        item.at(),
                        other.at(),
                        "the feed says {:?} twice at the same instant",
                        item.text()
                    );
                }
            }
        }
    }

    #[test]
    fn the_feed_timestamps_do_not_move_with_the_clock() {
        // **A line stamped with "now" is a line whose timestamp moves**, and a
        // feed whose timestamps move is a feed the player cannot read.
        let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(1))
            .expect("the campaign advances");
        let before: Vec<(String, domain::time::Instant)> = feed(&sim, 50)
            .iter()
            .map(|item| (item.text().to_owned(), item.at()))
            .collect();
        // The clock moves on, and nothing new happens.
        sim.advance_to(
            domain::time::Instant::from_months(1)
                .saturating_add(domain::time::Duration::from_days(3)),
        )
        .expect("the campaign advances");
        let after: Vec<(String, domain::time::Instant)> = feed(&sim, 50)
            .iter()
            .map(|item| (item.text().to_owned(), item.at()))
            .collect();
        for (text, at) in &before {
            let found = after
                .iter()
                .find(|(other, _)| other == text)
                .map(|(_, other)| *other);
            if let Some(other) = found {
                assert_eq!(*at, other, "the timestamp of {text:?} moved");
            }
        }
    }
}
