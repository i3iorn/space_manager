//! Phase 6 — finance.
//!
//! The monthly allocation arrives, the budgets are charged, and the treasury is
//! checked against the crisis ladder.
//!
//! **The outgoings are a rate, not a monthly settlement.** The agency spends
//! money all the time, so the treasury drains continuously and the rate lives
//! in `crate::rates`. What is left here is the part that genuinely happens
//! once a month: the allocation from the state, the report of what the month
//! cost, and the crisis ladder.
//!
//! **The reference agency runs a deficit.** That is the design: a competent
//! player must find revenue, or cut costs, or ask for a larger appropriation,
//! and every one of those is a political act.

use domain::finance::{Budget, CrisisStage, IncomeSource};
use domain::money::Money;

use crate::balance;
use crate::derived;
use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The finance phase.
pub struct FinancePhase;

impl Phase for FinancePhase {
    const NAME: &'static str = "finance";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        let income = appropriation(context);
        let outgoings = derived::monthly_outgoings_detail(context.state);

        context.state.treasury.income.clone_from(&income);
        context.state.treasury.outgoings = outgoings;

        // Only the allocation moves the balance here. The outgoings have
        // already been taken, continuously, by the rate.
        let balance = context
            .state
            .treasury
            .balance
            .checked_add(income.iter().fold(Money::ZERO, |total, (_, amount)| {
                total.checked_add(*amount).unwrap_or(total)
            }))
            .unwrap_or(context.state.treasury.balance);
        context.state.treasury.balance = balance;

        update_costs(context);
        check_crisis(context);

        Ok(())
    }
}

/// What the state allocates, adjusted for political standing.
///
/// **The allocation is a derived value, and there is one of it.** The
/// interface shows the player where the money comes from, and the two must not
/// disagree.
fn appropriation(context: &mut MonthContext<'_>) -> Vec<(IncomeSource, Money)> {
    derived::monthly_income(context.state)
}
/// Charges each project and campaign its monthly cost against its budget.
fn update_costs(context: &mut MonthContext<'_>) {
    let project_ids: Vec<_> = context.state.projects.keys().copied().collect();
    for id in project_ids {
        let cost = balance::research_cost_per_month(context.state.projects[&id].team.len());
        let project = context
            .state
            .projects
            .get_mut(&id)
            .expect("the project exists");
        project.funding.spent = project
            .funding
            .spent
            .checked_add(cost)
            .unwrap_or(project.funding.spent);
    }

    let campaign_ids: Vec<_> = context.state.test_campaigns.keys().copied().collect();
    for id in campaign_ids {
        let cost = balance::test_cost_per_month(context.state.test_campaigns[&id].team.len());
        let campaign = context
            .state
            .test_campaigns
            .get_mut(&id)
            .expect("the campaign exists");
        campaign.funding.spent = campaign
            .funding
            .spent
            .checked_add(cost)
            .unwrap_or(campaign.funding.spent);
    }
}

/// Moves the agency down the crisis ladder.
///
/// The ladder takes six months to reach the bottom, and the warning appears
/// two months before the austerity. A crisis should be the consequence of a run
/// of bad decisions, not a surprise.
///
/// **The ladder goes back up as well as down.** A crisis is a run of bad
/// decisions and not a sentence: a warning that never lifts is a lie, and a cut
/// that can never be undone is a wound that never closes. The agency climbs one
/// step a month once the condition that caused the step has cleared.
fn check_crisis(context: &mut MonthContext<'_>) {
    let runway = context.state.treasury.runway_months();
    let current = context.state.treasury.crisis_stage;

    let next = if runway < 1 {
        // **The agency cannot cover a month.** It falls one step, and the
        // steps below the warning are the state's answer to that.
        current.next()
    } else if current != CrisisStage::None && runway >= current.recovers_at() {
        // **The crisis has cleared.** The stage lifts by one step, and the
        // austerity cut lifts with it.
        current.previous()
    } else if current == CrisisStage::None && runway < 3 {
        CrisisStage::Warning
    } else {
        current
    };

    if next != current {
        context.state.treasury.crisis_stage = next;
        context.warnings.push(message_for(next));
        // **The ladder is part of the campaign's history.** A crisis the player
        // cannot find in the chronicle afterwards is a crisis that did not
        // happen, as far as the record is concerned.
        context.events.push(
            domain::chronicle::ChronicleEntry::new(
                context.state.next_chronicle_id(),
                context.month,
                domain::chronicle::Category::Financial,
                headline_for(next),
                severity_for(next),
            )
            .with_body(message_for(next)),
        );
    } else if current == CrisisStage::Warning {
        context.warnings.push(
            "The treasury is still low. The state is watching to see what you do.".to_owned(),
        );
    }
}

/// The chronicle's headline for a move on the ladder.
///
/// **The headline says what changed**, in the words the player reads, and the
/// body says what to do about it.
fn headline_for(stage: CrisisStage) -> String {
    match stage {
        CrisisStage::None => "The finances have recovered.".to_owned(),
        CrisisStage::Warning => "The treasury will not last the quarter.".to_owned(),
        CrisisStage::Austerity => "The state has cut the agency's funding.".to_owned(),
        CrisisStage::Borrowing => "Emergency loans have been taken out.".to_owned(),
        CrisisStage::Attrition => "Unpaid staff are leaving.".to_owned(),
        CrisisStage::Cancellations => "Programmes are being cancelled.".to_owned(),
        CrisisStage::Collapse => "The agency cannot meet its obligations.".to_owned(),
    }
}

/// How serious a move on the ladder is.
fn severity_for(stage: CrisisStage) -> domain::chronicle::Severity {
    use domain::chronicle::Severity;
    match stage {
        CrisisStage::None | CrisisStage::Warning => Severity::Info,
        CrisisStage::Austerity | CrisisStage::Borrowing => Severity::Major,
        CrisisStage::Attrition | CrisisStage::Cancellations | CrisisStage::Collapse => {
            Severity::Critical
        }
    }
}

/// What the state says when the ladder moves.
///
/// **The message says what has happened and what would undo it**, because a
/// crisis the player cannot act on is not a crisis but a mood.
fn message_for(stage: CrisisStage) -> String {
    match stage {
        CrisisStage::None => "The finances have recovered.".to_owned(),
        CrisisStage::Warning => "The treasury will not last the quarter. Act now.".to_owned(),
        CrisisStage::Austerity => format!(
            "The state has cut the agency's funding by {}. Get a month of runway back and the cut is lifted.",
            balance::AUSTERITY_CUT_PERCENT
        ),
        CrisisStage::Borrowing => "Emergency loans have been taken out.".to_owned(),
        CrisisStage::Attrition => "Unpaid staff are leaving.".to_owned(),
        CrisisStage::Cancellations => {
            "Programmes are being cancelled. The flagship is in danger.".to_owned()
        }
        CrisisStage::Collapse => "The agency cannot meet its obligations.".to_owned(),
    }
}

/// The budget a new programme would be given, for the interface.
#[must_use]
pub fn default_budget() -> Budget {
    Budget::of(Money::from_credits(250_000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{AgencyId, PersonId};
    use domain::money::Money;
    use domain::person::{Person, Role};
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    fn setup() -> (CampaignState, Rng) {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        state.people.insert(
            PersonId(0),
            Person::new(
                PersonId(0),
                "Marsha Okonjo".to_owned(),
                Role::Scientist,
                Money::from_credits(1_200),
            ),
        );
        (state, Rng::new(5))
    }

    #[test]
    fn payroll_is_an_outgoing() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        assert!(
            context
                .state
                .treasury
                .outgoings
                .iter()
                .any(|(name, _)| name == "Payroll")
        );
    }

    #[test]
    fn the_appropriation_scales_with_the_agency() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        let income = context.state.treasury.total_income();
        // Three people is below the floor, so the floor applies.
        assert_eq!(income, Money::from_credits(balance::APPROPRIATION_FLOOR));
    }

    #[test]
    fn a_larger_agency_is_funded_proportionally() {
        let (mut state, rng) = setup();
        for id in 1..40 {
            state.people.insert(
                domain::ids::PersonId(id),
                domain::person::Person::new(
                    domain::ids::PersonId(id),
                    format!("Person {id}"),
                    Role::Scientist,
                    Money::from_credits(1_200),
                ),
            );
        }
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        let income = context.state.treasury.total_income();
        assert!(income > Money::from_credits(balance::APPROPRIATION_FLOOR));
    }

    #[test]
    fn a_deficit_shortens_the_runway() {
        let (mut state, rng) = setup();
        state.treasury.balance = Money::from_credits(100_000);
        // **A project big enough that the agency spends more than it is
        // given.** The state funds the agency in proportion to its headcount,
        // so it takes a team of a dozen before the research outweighs the
        // allocation.
        for id in 1..12 {
            state.people.insert(
                PersonId(id),
                domain::person::Person::new(
                    PersonId(id),
                    format!("Person {id}"),
                    Role::Scientist,
                    Money::from_credits(1_200),
                ),
            );
        }
        let project_id = state.id_counters.next_research_project();
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                team: state.people.keys().copied().collect(),
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(100),
                target_capability: None,
                started: Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        assert!(context.state.treasury.net().is_negative());
        assert!(context.state.treasury.runway_months() < 24);
    }

    #[test]
    fn a_low_treasury_raises_the_warning() {
        let (mut state, rng) = setup();
        state.treasury.balance = Money::ZERO;
        // A project that costs more than the agency is given.
        for id in 1..12 {
            state.people.insert(
                PersonId(id),
                domain::person::Person::new(
                    PersonId(id),
                    format!("Person {id}"),
                    Role::Scientist,
                    Money::from_credits(1_200),
                ),
            );
        }
        let project_id = state.id_counters.next_research_project();
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                team: state.people.keys().copied().collect(),
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(100),
                target_capability: None,
                started: Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.treasury.crisis_stage, CrisisStage::Warning);
        assert!(!context.warnings.is_empty());
    }

    /// An agency that is spending more than it is given.
    ///
    /// **The project is staffed by people who are not on it**, so the cost is
    /// real and the payroll is not. What matters is that the burn is bigger
    /// than the allocation, which is the state of affairs the crisis ladder is
    /// for.
    fn in_deficit() -> (CampaignState, Rng) {
        let (mut state, rng) = setup();
        for id in 1..12 {
            state.people.insert(
                PersonId(id),
                domain::person::Person::new(
                    PersonId(id),
                    format!("Person {id}"),
                    Role::Scientist,
                    Money::from_credits(1_200),
                ),
            );
        }
        let project_id = state.id_counters.next_research_project();
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                // **A hundred and eighty hands on one project.** The team is
                // not the agency's staff; the cost is what the treasury feels.
                team: (0..180).map(PersonId).collect(),
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(100),
                target_capability: None,
                started: Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        (state, rng)
    }

    /// Runs one month of the finance phase with a given balance.
    ///
    /// **The month's chronicle entries come back with it**, because the record
    /// is part of what the phase did.
    fn month(
        state: &mut CampaignState,
        rng: Rng,
        balance: Money,
    ) -> Vec<domain::chronicle::ChronicleEntry> {
        state.treasury.balance = balance;
        let mut context = MonthContext::new(state, &rng, Instant::ZERO, Vec::new());
        FinancePhase::run(&mut context).expect("the phase runs");
        context.events.clone()
    }

    #[test]
    fn the_ladder_falls_a_step_a_month_while_the_treasury_is_empty() {
        let (mut state, rng) = in_deficit();
        let _ = month(&mut state, rng, Money::ZERO);
        assert_eq!(state.treasury.crisis_stage, CrisisStage::Warning);
        let _ = month(&mut state, rng, Money::ZERO);
        assert_eq!(state.treasury.crisis_stage, CrisisStage::Austerity);
        // **And it keeps falling**, because the treasury is still empty and
        // the state is not finished.
        let _ = month(&mut state, rng, Money::ZERO);
        assert_eq!(state.treasury.crisis_stage, CrisisStage::Borrowing);
    }

    #[test]
    fn the_austerity_cut_takes_a_fifth_off_the_allocation() {
        let (mut state, rng) = in_deficit();
        let _ = month(&mut state, rng, Money::ZERO);
        let _ = month(&mut state, rng, Money::ZERO);
        assert_eq!(state.treasury.crisis_stage, CrisisStage::Austerity);

        // **The cut is on the state's money**, so the whole allocation is
        // smaller and nothing else is.
        let cut = derived::monthly_income(&state)[0].1;
        state.treasury.crisis_stage = CrisisStage::None;
        let whole = derived::monthly_income(&state)[0].1;
        assert_eq!(cut, whole.percent(100 - balance::AUSTERITY_CUT_PERCENT));
        assert!(cut < whole);
    }

    #[test]
    fn the_cut_stays_in_force_below_the_austerity_rung() {
        // **Every rung below austerity is a worse version of the same thing**,
        // and the cut is the state's answer to all of them.
        for stage in [
            CrisisStage::Austerity,
            CrisisStage::Borrowing,
            CrisisStage::Attrition,
            CrisisStage::Cancellations,
            CrisisStage::Collapse,
        ] {
            assert!(stage.in_austerity(), "{stage:?} is an austerity");
        }
        assert!(!CrisisStage::Warning.in_austerity());
        assert!(!CrisisStage::None.in_austerity());
    }

    #[test]
    fn the_cut_lifts_when_the_treasury_recovers() {
        let (mut state, rng) = in_deficit();
        state.treasury.crisis_stage = CrisisStage::Austerity;
        // **A treasury with money in it.** The agency is no longer in the
        // hole, so the state lifts the cut.
        let _ = month(&mut state, rng, Money::from_credits(1_000_000));
        assert_eq!(state.treasury.crisis_stage, CrisisStage::Warning);
        // **And the warning lifts in turn.**
        let _ = month(&mut state, rng, Money::from_credits(1_000_000));
        assert_eq!(state.treasury.crisis_stage, CrisisStage::None);
    }

    #[test]
    fn the_warning_lifts_when_the_treasury_recovers() {
        let (mut state, rng) = in_deficit();
        state.treasury.crisis_stage = CrisisStage::Warning;
        let _ = month(&mut state, rng, Money::from_credits(1_000_000));
        assert_eq!(state.treasury.crisis_stage, CrisisStage::None);
    }

    #[test]
    fn a_crisis_is_in_the_chronicle() {
        let (mut state, rng) = in_deficit();
        let events = month(&mut state, rng, Money::ZERO);
        assert!(
            events
                .iter()
                .any(|event| event.headline.contains("treasury will not last")),
            "the warning is in the record: {:?}",
            events
                .iter()
                .map(|event| event.headline.clone())
                .collect::<Vec<_>>()
        );
    }
}
