//! Replay and forking.
//!
//! Forking replays the log to a given Instant, snapshots the state, and continues
//! with a new log. This is what makes "what if I had not cancelled that
//! programme?" a question the player can answer.
//!
//! **The fork is exact [D].** The branch is the *same campaign* up to the fork
//! point: the same state, the same chronicle, the same recruits. That is what
//! makes a comparison meaningful — if the branch were a different campaign,
//! every difference would be noise.
//!
//! The replay re-issues the logged orders at their logged **simulated issue
//! instants**, and advances the campaign to each of them in turn. The order log
//! records the *effective* time of the issue, so the carried fraction is
//! reproduced exactly and the two campaigns stay on the same grid.

use domain::state::CampaignState;

/// One thing the two branches disagree about.
///
/// The player should be able to see not just *that* the branches differ but
/// *when* and *why*, and that means naming the thing that differs rather than
/// showing two columns of numbers.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    /// What differs.
    pub subject: String,
    /// The first branch's value.
    pub left: String,
    /// The second branch's value.
    pub right: String,
}

/// Where two branches differ.
///
/// The comparison is by *value* and not by hash: the player wants to know that
/// the treasury is forty thousand apart, not that two numbers are not equal.
#[must_use]
pub fn divergence(left: &CampaignState, right: &CampaignState) -> Vec<Divergence> {
    let mut differences = Vec::new();

    let mut compare = |subject: &str, left: String, right: String| {
        if left != right {
            differences.push(Divergence {
                subject: subject.to_owned(),
                left,
                right,
            });
        }
    };

    compare(
        "Treasury",
        left.treasury.balance.credits().to_string(),
        right.treasury.balance.credits().to_string(),
    );
    compare(
        "Daily net",
        left.treasury.daily_net().credits().to_string(),
        right.treasury.daily_net().credits().to_string(),
    );
    compare(
        "Approval",
        left.political.approval.percent().to_string(),
        right.political.approval.percent().to_string(),
    );
    compare(
        "Goodwill",
        left.political.goodwill.percent().to_string(),
        right.political.goodwill.percent().to_string(),
    );
    compare(
        "Staff",
        left.people.len().to_string(),
        right.people.len().to_string(),
    );
    compare(
        "Projects",
        left.projects.len().to_string(),
        right.projects.len().to_string(),
    );
    compare(
        "Facilities",
        left.facilities.len().to_string(),
        right.facilities.len().to_string(),
    );
    compare(
        "Missions",
        left.missions.len().to_string(),
        right.missions.len().to_string(),
    );
    compare(
        "Knowledge",
        left.knowledge.values().sum::<u32>().to_string(),
        right.knowledge.values().sum::<u32>().to_string(),
    );
    compare(
        "Chronicle",
        left.chronicle.len().to_string(),
        right.chronicle.len().to_string(),
    );

    differences
}

/// The chronicle entries the two branches disagree about.
///
/// The entries are matched by their headline, so an entry that only one branch
/// has is reported as the difference it is.
#[must_use]
pub fn chronicle_differences(left: &CampaignState, right: &CampaignState) -> Vec<(String, bool)> {
    let right_headlines: Vec<&str> = right
        .chronicle
        .iter()
        .map(|entry| entry.headline.as_str())
        .collect();

    let mut differences = Vec::new();
    for entry in &left.chronicle {
        if !right_headlines.contains(&entry.headline.as_str()) {
            differences.push((entry.headline.clone(), true));
        }
    }
    for entry in &right.chronicle {
        let in_left = left
            .chronicle
            .iter()
            .any(|other| other.headline == entry.headline);
        if !in_left {
            differences.push((entry.headline.clone(), false));
        }
    }
    differences
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::state::Agency;

    fn state() -> CampaignState {
        CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039))
    }

    #[test]
    fn two_identical_campaigns_have_no_differences() {
        let campaign = state();
        assert!(divergence(&campaign, &campaign).is_empty());
        assert!(chronicle_differences(&campaign, &campaign).is_empty());
    }

    #[test]
    fn a_difference_is_named() {
        let mut left = state();
        let mut right = state();
        left.treasury.balance = domain::money::Money::from_credits(1_000);
        right.treasury.balance = domain::money::Money::from_credits(2_000);
        let differences = divergence(&left, &right);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].subject, "Treasury");
        assert_eq!(differences[0].left, "1000");
        assert_eq!(differences[0].right, "2000");
    }

    #[test]
    fn a_chronicle_entry_only_one_branch_has_is_reported() {
        let mut left = state();
        let right = state();
        left.chronicle.push(domain::chronicle::ChronicleEntry::new(
            domain::ids::ChronicleId(0),
            domain::time::Instant::ZERO,
            domain::chronicle::Category::Technical,
            "A thing happened.".to_owned(),
            domain::chronicle::Severity::Minor,
        ));
        let differences = chronicle_differences(&left, &right);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].0, "A thing happened.");
        assert!(differences[0].1, "the entry is only in the first branch");
    }
}
