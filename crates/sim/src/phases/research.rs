//! Phase 3 — research and development.
//!
//! The projects that start, the projects that finish, breakthroughs, and
//! capability unlocks.
//!
//! Progress is a **rate**: it accrues from the team's effective performance, so
//! a tired team genuinely goes slower. That is the whole point of the
//! performance model, and it is what makes staffing decisions matter. **The
//! rate lives in `crate::rates`**, and this phase is what happens when a
//! project reaches its target.
//!
//! **The completion draws, and a draw may only happen at a scheduled
//! instant.** The progress crosses the target inside the integration, where
//! there is no random source to draw from; the crossing schedules a
//! `ProjectComplete` item, and the completion happens when that item fires.

use domain::chronicle::{Category, ChronicleEntry, Severity, SubjectRef};
use domain::ids::{PersonId, ResearchProjectId};
use domain::order::OrderKind;
use domain::person::Assignment;
use domain::research::{CAPABILITIES, CapabilityId, Progress};

use crate::balance;
use crate::error::SimError;
use crate::rng::Domain;
use crate::tick::{MonthContext, Phase};

/// The research phase.
pub struct ResearchPhase;

impl Phase for ResearchPhase {
    const NAME: &'static str = "research";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        start_projects(context);
        start_test_campaigns(context);
        complete_finished_projects(context);
        advance_test_campaigns(context);
        Ok(())
    }
}

/// Starts the research projects the player asked for.
fn start_projects(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);

    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Starts the test campaigns the player asked for.
fn start_test_campaigns(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);

    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Applies one research order, and reports whether it was one.
///
/// **The handlers are per-kind [D].** An order lands at its own instant, and
/// the phase that owns it is the one that applies it.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::StartProject {
            discipline,
            principal_investigator,
            title,
        } => {
            let id = context.state.id_counters.next_research_project();
            let team = vec![*principal_investigator];
            let project = domain::research::ResearchProject {
                id,
                title: title.clone(),
                discipline: *discipline,
                principal_investigator: *principal_investigator,
                team: team.clone(),
                funding: domain::finance::Budget::empty(),
                progress: Progress::empty(balance::PROJECT_POINTS_TARGET),
                target_capability: None,
                started: context.month,
                expected_completion: Some(context.month),
                remainders: domain::scale::Remainders::new(),
            };
            context.state.projects.insert(id, project);

            // **The lead joins the project, and leaves whatever they were on.**
            // The same rule as an assignment: being the lead and being on the
            // team are the same thing, and a person on two teams works for
            // both.
            crate::phases::personnel::assign(
                context,
                *principal_investigator,
                Assignment::Project { project: id },
            );

            context.programme_lines.push(
                domain::report::ReportLine::new(format!("Research started: {title}."))
                    .with_severity(Severity::Minor),
            );
            true
        }
        OrderKind::StartTestCampaign { capability } => {
            let id = context.state.id_counters.next_test_campaign();
            let campaign = domain::research::TestCampaign {
                id,
                target_capability: *capability,
                team: Vec::new(),
                funding: domain::finance::Budget::empty(),
                progress: Progress::empty(100),
                risk_reduction: 5,
            };
            context.state.test_campaigns.insert(id, campaign);
            let name = domain::research::capability_name(*capability);
            context.programme_lines.push(
                domain::report::ReportLine::new(format!("A test campaign on {name} began."))
                    .with_severity(Severity::Minor),
            );
            true
        }
        _ => false,
    }
}

/// Completes every project that has reached its target.
///
/// The progress itself is a rate, and it is integrated in `crate::rates`. By
/// the time this runs, a finished project is waiting for its scheduled
/// instant, and this is where the knowledge and the breakthrough happen.
fn complete_finished_projects(context: &mut MonthContext<'_>) {
    let finished: Vec<ResearchProjectId> = context
        .state
        .projects
        .values()
        .filter(|project| project.progress.is_complete())
        .map(|project| project.id)
        .collect();

    for id in finished {
        complete_project(context, id);
    }
}

/// A finished project yields knowledge, and may yield a breakthrough.
pub fn complete_project(context: &mut MonthContext<'_>, id: ResearchProjectId) {
    let discipline = context.state.projects[&id].discipline;
    let title = context.state.projects[&id].title.clone();

    // Knowledge goes into the discipline's own capability, and 20% spills into
    // the adjacent ones. A narrow research portfolio therefore has a real
    // opportunity cost.
    let own = CAPABILITIES
        .iter()
        .find(|capability| capability.discipline == discipline)
        .map_or(CapabilityId(0), |capability| capability.id);

    let mut gains: Vec<(CapabilityId, u32)> = vec![(own, balance::KNOWLEDGE_PER_PROJECT)];
    let adjacent: &'static [domain::research::DisciplineId] = domain::research::DISCIPLINES
        .iter()
        .find(|known| known.id == discipline)
        .map_or(&[], |known| known.adjacent);
    for adjacent in adjacent {
        if let Some(capability) = CAPABILITIES
            .iter()
            .find(|capability| capability.discipline == *adjacent)
        {
            gains.push((capability.id, balance::KNOWLEDGE_PER_PROJECT / 5));
        }
    }

    for (capability, amount) in gains {
        let entry = context.state.knowledge.entry(capability).or_insert(0);
        *entry = entry.saturating_add(amount);
        check_unlock(context, capability);
    }

    // A breakthrough: a large gain in a single Instant.
    //
    // **The bucket is a stable identifier, not an instant.** The project's own
    // start month and its id: the same project always draws the same
    // breakthrough, however long it took and whatever month it finished in.
    let started = context.state.projects[&id].started.months();
    let mut rng = context.rng.stream(Domain::Research, started, id.0);
    let roll = crate::rng::draw_basis_points(&mut rng);
    if roll < balance::BREAKTHROUGH_CHANCE_BP {
        let entry = context.state.knowledge.entry(own).or_insert(0);
        *entry = entry.saturating_add(balance::KNOWLEDGE_PER_PROJECT);
        check_unlock(context, own);

        let entry = ChronicleEntry::new(
            context.state.next_chronicle_id(),
            context.month,
            Category::Discovery,
            format!("A breakthrough in {title}."),
            Severity::Major,
        )
        .about(SubjectRef::Person(
            context.state.projects[&id].principal_investigator,
        ));
        context.events.push(entry);
        context
            .opportunities
            .push(format!("A breakthrough in {title}."));
    }

    context.programme_lines.push(
        domain::report::ReportLine::new(format!("Research completed: {title}."))
            .with_severity(Severity::Minor),
    );

    // The chronicle records the completion as well as the report, because a
    // finished project is a thing that happened to the agency and not just a
    // line in a Instant's summary.
    let entry = ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        Category::Discovery,
        format!("Research completed: {title}."),
        Severity::Minor,
    )
    .about(SubjectRef::Person(
        context.state.projects[&id].principal_investigator,
    ));
    context.events.push(entry);

    // The project is finished; its people are released.
    let team = context.state.projects[&id].team.clone();
    for member in team {
        if let Some(person) = context.state.people.get_mut(&member) {
            person.assignment = Assignment::Unassigned;
        }
    }
    context.state.projects.remove(&id);
}

/// Checks whether enough knowledge has accumulated to unlock a capability.
fn check_unlock(context: &mut MonthContext<'_>, capability: CapabilityId) {
    if context.state.unlocked.contains(&capability) {
        return;
    }
    let Some(definition) = CAPABILITIES.iter().find(|known| known.id == capability) else {
        return;
    };
    let knowledge = context
        .state
        .knowledge
        .get(&capability)
        .copied()
        .unwrap_or(0);
    if knowledge < definition.threshold {
        return;
    }

    context.state.unlocked.insert(capability);

    // A capability with a threshold of zero was never *developed* — the agency
    // always had it. It is unlocked silently, without a chronicle entry.
    if definition.threshold == 0 {
        return;
    }

    let name = definition.name;
    let entry = ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        Category::Discovery,
        format!("The agency has developed {name}."),
        Severity::Major,
    );
    context.events.push(entry);
    context
        .opportunities
        .push(format!("{name} is now available."));
}

/// Advances every test campaign.
fn advance_test_campaigns(context: &mut MonthContext<'_>) {
    let campaign_ids: Vec<_> = context.state.test_campaigns.keys().copied().collect();
    for id in campaign_ids {
        let team_size = context.state.test_campaigns[&id].team.len();
        if team_size == 0 {
            continue;
        }
        let points = 5 * domain::scale::PERCENT_UNITS as u32;
        let campaign = context
            .state
            .test_campaigns
            .get_mut(&id)
            .expect("the campaign exists");
        campaign.progress.points = campaign.progress.points.saturating_add(points);
        if campaign.progress.is_complete() {
            let name = domain::research::capability_name(campaign.target_capability);
            context.programme_lines.push(
                domain::report::ReportLine::new(format!("A test campaign on {name} is complete."))
                    .with_severity(Severity::Minor),
            );
            context.state.test_campaigns.remove(&id);
        }
    }
}

/// The people a project has, for the interface.
#[must_use]
pub fn project_team(
    state: &domain::state::CampaignState,
    id: domain::ids::ResearchProjectId,
) -> Vec<PersonId> {
    state
        .projects
        .get(&id)
        .map(|project| project.team.clone())
        .unwrap_or_default()
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
        (state, Rng::new(11))
    }

    #[test]
    fn a_project_starts_with_its_lead_assigned() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            Instant::ZERO,
            vec![OrderKind::StartProject {
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                title: "Liquid upper stage".to_owned(),
            }],
        );
        crate::phases::intake::IntakePhase::run(&mut context).expect("intake runs");
        ResearchPhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.projects.len(), 1);
        assert!(matches!(
            context.state.people[&PersonId(0)].assignment,
            Assignment::Project { .. }
        ));
    }

    #[test]
    fn the_rates_accrue_a_projects_progress() {
        // **The progress is a rate now**, and it lives in `crate::rates`. The
        // phase is what happens when the project reaches its target.
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            Instant::ZERO,
            vec![OrderKind::StartProject {
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                title: "Liquid upper stage".to_owned(),
            }],
        );
        crate::phases::intake::IntakePhase::run(&mut context).expect("intake runs");
        ResearchPhase::run(&mut context).expect("the phase runs");
        crate::rates::integrate(context.state, domain::time::Duration::from_months(1));
        let project = context.state.projects.values().next().expect("a project");
        assert!(project.progress.points > 0);
    }

    #[test]
    fn a_project_that_reached_its_target_is_completed() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            Instant::ZERO,
            vec![OrderKind::StartProject {
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                title: "Liquid upper stage".to_owned(),
            }],
        );
        crate::phases::intake::IntakePhase::run(&mut context).expect("intake runs");
        ResearchPhase::run(&mut context).expect("the phase runs");
        let id = context
            .state
            .projects
            .keys()
            .next()
            .copied()
            .expect("a project");
        context
            .state
            .projects
            .get_mut(&id)
            .expect("a project")
            .progress
            .points = context.state.projects[&id].progress.target;
        ResearchPhase::run(&mut context).expect("the phase runs");
        assert!(context.state.projects.is_empty(), "the project is finished");
        assert!(!context.events.is_empty(), "and it is in the chronicle");
    }

    #[test]
    fn enough_knowledge_unlocks_a_capability() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        context.state.knowledge.insert(CapabilityId(2), 500);
        check_unlock(&mut context, CapabilityId(2));
        assert!(context.state.unlocked.contains(&CapabilityId(2)));
        assert!(!context.opportunities.is_empty());
    }
}
