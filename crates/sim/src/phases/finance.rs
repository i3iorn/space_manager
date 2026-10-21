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
/// The allocation scales with the number of people the agency employs, with a
/// floor for a young agency, and is then multiplied by the state's political
/// standing.
fn appropriation(context: &mut MonthContext<'_>) -> Vec<(IncomeSource, Money)> {
    let headcount = i64::try_from(context.state.people.len()).unwrap_or(i64::MAX);
    let base = (balance::APPROPRIATION_PER_HEAD * headcount).max(balance::APPROPRIATION_FLOOR);

    let approval = i64::from(context.state.political.approval.value);
    let multiplier = 10_000 + approval * balance::APPROPRIATION_POLITICAL_SCALE_BP;
    let amount = Money::from_credits(base).multiply_fraction(multiplier.max(0), 10_000);

    vec![(IncomeSource::CivilisationBase, amount)]
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
fn check_crisis(context: &mut MonthContext<'_>) {
    let runway = context.state.treasury.runway_months();
    let current = context.state.treasury.crisis_stage;

    let next = if runway < 1 {
        current.next()
    } else if runway < 3 && current == CrisisStage::None {
        CrisisStage::Warning
    } else {
        current
    };

    if next != current {
        context.state.treasury.crisis_stage = next;
        let message = match next {
            CrisisStage::Warning => "The treasury will not last the quarter. Act now.".to_owned(),
            CrisisStage::Austerity => {
                "The state has imposed a cut. Programmes are being throttled.".to_owned()
            }
            CrisisStage::Borrowing => "Emergency loans have been taken out.".to_owned(),
            CrisisStage::Attrition => "Unpaid staff are leaving.".to_owned(),
            CrisisStage::Cancellations => {
                "Programmes are being cancelled. The flagship is in danger.".to_owned()
            }
            CrisisStage::Collapse => "The agency cannot meet its obligations.".to_owned(),
            CrisisStage::None => "The finances have recovered.".to_owned(),
        };
        context.warnings.push(message);
    } else if current == CrisisStage::Warning {
        context
            .warnings
            .push("The treasury is still low. The state will not renew the cut.".to_owned());
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
        // A project big enough that the agency spends more than it is given.
        let project_id = state.id_counters.next_research_project();
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                team: vec![PersonId(0), PersonId(0), PersonId(0)],
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
        assert!(context.state.treasury.runway_months() < 12);
    }

    #[test]
    fn a_low_treasury_raises_the_warning() {
        let (mut state, rng) = setup();
        state.treasury.balance = Money::ZERO;
        // A project that costs more than the agency is given.
        let project_id = state.id_counters.next_research_project();
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                team: vec![PersonId(0), PersonId(0), PersonId(0)],
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
}
