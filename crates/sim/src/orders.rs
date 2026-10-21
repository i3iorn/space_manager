//! The order registry, and its handlers.
//!
//! Each order kind has a handler that validates it and applies it. An order with
//! no registered handler is rejected, never ignored.
//!
//! **An order takes time to land.** The lead time, the direct cost, and the
//! goodwill charge are all here, because they are part of the order's cost and
//! the player must see them before they commit. See `docs/SIMULATION.md` §4.

use domain::ids::PersonId;
use domain::money::Money;
use domain::order::OrderKind;
use domain::political::Goodwill;
use domain::state::CampaignState;
use domain::time::{Duration, Instant, MILLIS_PER_MONTH};

use crate::balance;
use crate::error::OrderError;
use crate::tick::MonthContext;

/// A rocket takes between sixty and ninety minutes to reach a stable orbit.
pub const FLIGHT_LEAD_TIME_MINUTES: u64 = 60;

/// A rocket takes up to ninety minutes to reach a stable orbit.
pub const FLIGHT_LEAD_TIME_MAX_MINUTES: u64 = 90;

/// How long an order takes to implement.
///
/// **The lead time is part of the order's cost, and the player sees it before
/// they commit.** An order the player cannot see the price of is not a
/// decision.
///
/// **The state's goodwill is part of it [D].** An accommodating state approves
/// quickly; an obstructive one makes every order take longer to land. The
/// player sees the delay before they commit, so the political state is a lever
/// and not a surprise.
#[must_use]
pub fn lead_time(order: &OrderKind, state: &CampaignState, issued_at: Instant) -> Duration {
    let base = base_lead_time(order, issued_at);
    adjusted_lead_time(base, &state.political.goodwill)
}

/// The lead time before the state's goodwill is taken into account.
#[must_use]
pub fn base_lead_time(order: &OrderKind, issued_at: Instant) -> Duration {
    match order {
        OrderKind::Hire { role, salary } => hire_lead_time(*role, *salary),
        OrderKind::Assign { .. }
        | OrderKind::Unassign { .. }
        | OrderKind::AssignFacility { .. } => assign_lead_time(),
        // **A shift change lands at the next hour**, because a shift change is
        // a rostering decision and not a project.
        OrderKind::SetShift { .. } => Duration::from_hours(balance::SHIFT_LEAD_TIME_HOURS),
        OrderKind::Promote { .. } => next_promotion_round(issued_at),
        OrderKind::Train { .. } => Duration::from_months(balance::TRAIN_LEAD_TIME_MONTHS),
        // **A design is a drawing**, and a drawing takes an afternoon.
        // **A subsystem change is changing a piece of the drawing**, and it
        // takes the same time.
        OrderKind::DesignVehicle { .. } | OrderKind::SetSubsystem { .. } => {
            Duration::from_days(balance::DESIGN_LEAD_TIME_DAYS)
        }
        OrderKind::CreateProgramme { .. }
        | OrderKind::SetObjective { .. }
        | OrderKind::SetBudget { .. }
        | OrderKind::CancelMission { .. }
        | OrderKind::Fire { .. } => Duration::ZERO,

        OrderKind::StartProject { .. } => start_project_lead_time(),
        OrderKind::StartTestCampaign { .. } => {
            Duration::from_days(balance::START_TEST_CAMPAIGN_LEAD_TIME_DAYS)
        }
        OrderKind::BuildFacility { plan, .. } => {
            Duration::from_months(u64::from(plan.build_months))
        }
        OrderKind::AuthorizeLaunch { .. } => Duration::from_mins(
            balance::FLIGHT_LEAD_TIME_MINUTES.midpoint(balance::FLIGHT_LEAD_TIME_MAX_MINUTES),
        ),
        OrderKind::SetPolicy { .. } => Duration::from_days(balance::SET_POLICY_LEAD_TIME_DAYS),
        // **A deadline is renegotiated over a week**, because a renegotiation
        // is a conversation and not a decision.
        OrderKind::PostponeDeadline { .. } => Duration::from_days(balance::POSTPONE_LEAD_TIME_DAYS),
        OrderKind::PublishResult { .. } => Duration::from_days(balance::PUBLISH_LEAD_TIME_DAYS),
    }
}

/// The goodwill's effect, from −1000 (hostile) to +1000 (accommodating).
///
/// **The effect is zero between the thresholds** and grows to its maximum at
/// the extremes, which is what the table in `MANAGEABLE_PARTS.md` §3.5
/// describes: above +40 the state accommodates, below −40 it obstructs, and in
/// between it is the ordinary relationship.
#[must_use]
pub fn goodwill_effect(goodwill: &Goodwill) -> i64 {
    let percent = i64::from(goodwill.percent());
    let accommodating = i64::from(balance::GOODWILL_ACCOMMODATING);
    let obstructive = i64::from(balance::GOODWILL_OBSTRUCTING);
    if percent > accommodating {
        (percent - accommodating) * 1_000 / (100 - accommodating)
    } else if percent < obstructive {
        (percent - obstructive) * 1_000 / (100 + obstructive)
    } else {
        0
    }
}

/// A lead time, adjusted for the state's goodwill.
///
/// **A hostile state makes every order take half again as long; a friendly
/// one makes it take half the time.**
#[must_use]
pub fn adjusted_lead_time(base: Duration, goodwill: &Goodwill) -> Duration {
    let factor_bp =
        10_000 - goodwill_effect(goodwill) * balance::GOODWILL_LEAD_TIME_EFFECT_BP / 1_000;
    Duration::from_millis(base.millis().saturating_mul(factor_bp as u64) / 10_000)
}

/// A cost, adjusted for the state's goodwill.
///
/// **An unfriendly state charges a third more; a friendly one charges a third
/// less.**
#[must_use]
pub fn adjusted_cost(base: Money, goodwill: &Goodwill) -> Money {
    let factor_bp = 10_000 - goodwill_effect(goodwill) * balance::GOODWILL_COST_EFFECT_BP / 1_000;
    base.multiply_fraction(factor_bp, 10_000)
}

/// A hire takes two weeks at the entry salary and six weeks at the top of the
/// band.
///
/// **The agency cannot buy a division head in a fortnight.**
#[must_use]
pub fn hire_lead_time(role: domain::person::Role, salary: Money) -> Duration {
    let entry = balance::starting_salary(role).minor();
    let senior = balance::senior_salary(role).minor();
    let asked = salary.minor();
    let span = (senior - entry).max(1);
    let fraction = (asked - entry).clamp(0, span);
    let days = i64::try_from(balance::HIRE_LEAD_TIME_MIN_DAYS).unwrap_or(i64::MAX)
        + (i64::try_from(balance::HIRE_LEAD_TIME_MAX_DAYS).unwrap_or(i64::MAX)
            - i64::try_from(balance::HIRE_LEAD_TIME_MIN_DAYS).unwrap_or(i64::MAX))
            * fraction
            / span;
    Duration::from_days(u64::try_from(days).unwrap_or(0))
}

/// Putting someone on something takes a couple of days.
#[must_use]
pub fn assign_lead_time() -> Duration {
    Duration::from_days(
        balance::ASSIGN_LEAD_TIME_MIN_DAYS.midpoint(balance::ASSIGN_LEAD_TIME_MAX_DAYS),
    )
}

/// Starting a research project takes a week or so of set-up.
#[must_use]
pub fn start_project_lead_time() -> Duration {
    Duration::from_days(
        balance::START_PROJECT_LEAD_TIME_MIN_DAYS
            .midpoint(balance::START_PROJECT_LEAD_TIME_MAX_DAYS),
    )
}

/// The next quarterly promotion round.
///
/// Promotions happen in rounds, so an order issued just after a round waits
/// almost three months for the next one.
#[must_use]
pub fn next_promotion_round(issued_at: Instant) -> Duration {
    let month = issued_at.millis() / MILLIS_PER_MONTH;
    let next = (month / balance::PROMOTION_ROUND_MONTHS + 1) * balance::PROMOTION_ROUND_MONTHS;
    Duration::from_months(next - month)
}

/// What an order costs, charged when it lands.
#[must_use]
pub fn cost(order: &OrderKind, state: &CampaignState) -> Money {
    match order {
        OrderKind::Hire { salary, .. } => *salary,
        OrderKind::Fire { person } => state
            .people
            .get(person)
            .map_or(Money::ZERO, |person| person.contract.salary.percent(300)),
        OrderKind::BuildFacility { plan, .. } => plan.total_cost,
        _ => Money::ZERO,
    }
}

/// What an order costs in goodwill, charged when it lands.
///
/// **Goodwill is the anti-spamming mechanism [D], and it is deliberately not
/// a cap.** The cost of a decision is financial and political.
#[must_use]
pub fn goodwill(order: &OrderKind) -> i16 {
    match order {
        OrderKind::Hire { .. } => balance::GOODWILL_HIRE,
        OrderKind::Fire { .. } => balance::GOODWILL_FIRE,
        OrderKind::Assign { .. }
        | OrderKind::Unassign { .. }
        | OrderKind::AssignFacility { .. } => balance::GOODWILL_ASSIGN,
        // **A shift change costs the state nothing.** It is the agency's own
        // business, and the player must be able to make a hundred of them
        // without the state's patience running out.
        OrderKind::SetShift { .. } => 0,
        OrderKind::Promote { .. } => balance::GOODWILL_PROMOTE,
        OrderKind::Train { .. } => balance::GOODWILL_TRAIN,
        OrderKind::CreateProgramme { .. } | OrderKind::CancelMission { .. } => {
            balance::GOODWILL_PROGRAMME
        }
        OrderKind::SetObjective { .. } => balance::GOODWILL_OBJECTIVE,
        OrderKind::DesignVehicle { .. } | OrderKind::SetSubsystem { .. } => {
            balance::GOODWILL_DESIGN
        }
        OrderKind::PostponeDeadline { .. } => balance::GOODWILL_POSTPONE_POLITICAL,
        OrderKind::SetBudget { .. } => balance::GOODWILL_BUDGET,

        OrderKind::StartProject { .. } => balance::GOODWILL_PROJECT,
        OrderKind::StartTestCampaign { .. } => balance::GOODWILL_TEST_CAMPAIGN,
        OrderKind::BuildFacility { plan, .. } => balance::facility_goodwill(plan.scale),
        OrderKind::AuthorizeLaunch { .. } => balance::GOODWILL_LAUNCH,
        OrderKind::SetPolicy { .. } => balance::GOODWILL_POLICY,
        OrderKind::PublishResult { .. } => balance::GOODWILL_PUBLISH,
    }
}

/// Whether an order has a handler.
///
/// **An order with no registered handler is rejected, never ignored.** The
/// kinds without handlers yet — the programme, policy, and publication
/// machinery — are the ones Slice 2 brings.
#[must_use]
pub fn has_handler(order: &OrderKind) -> bool {
    match order {
        OrderKind::Hire { .. }
        | OrderKind::Fire { .. }
        | OrderKind::Assign { .. }
        | OrderKind::Unassign { .. }
        | OrderKind::SetShift { .. }
        | OrderKind::Promote { .. }
        | OrderKind::Train { .. }
        | OrderKind::StartProject { .. }
        | OrderKind::StartTestCampaign { .. }
        | OrderKind::BuildFacility { .. }
        | OrderKind::DesignVehicle { .. }
        | OrderKind::SetSubsystem { .. }
        | OrderKind::CreateProgramme { .. }
        | OrderKind::SetObjective { .. }
        | OrderKind::SetBudget { .. }
        | OrderKind::PostponeDeadline { .. }
        | OrderKind::AuthorizeLaunch { .. } => true,
        OrderKind::SetPolicy { .. }
        | OrderKind::PublishResult { .. }
        | OrderKind::AssignFacility { .. }
        | OrderKind::CancelMission { .. } => false,
    }
}

/// Applies an order that has landed.
///
/// The phase that owns the order kind applies it, and the first one that owns
/// it wins.
pub fn apply(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    crate::phases::personnel::apply_one(context, order)
        || crate::phases::research::apply_one(context, order)
        || crate::phases::construction::apply_one(context, order)
        || crate::phases::missions::apply_one(context, order)
        || crate::phases::vehicles::apply_one(context, order)
        || crate::phases::programmes::apply_one(context, order)
}

/// Checks an order against the current state.
///
/// Validation is **side-effect free**. That is the only way to tell the player
/// "Inspector Okonjo is already assigned to the Lunar programme" without
/// having already moved her.
pub fn validate(kind: &OrderKind, state: &CampaignState) -> Result<(), OrderError> {
    match kind {
        OrderKind::Hire { .. }
        | OrderKind::CreateProgramme { .. }
        | OrderKind::SetPolicy { .. }
        | OrderKind::PublishResult { .. }
        | OrderKind::BuildFacility { .. }
        | OrderKind::DesignVehicle { .. } => Ok(()),

        OrderKind::Fire { person }
        | OrderKind::Unassign { person }
        | OrderKind::SetShift { person, .. }
        | OrderKind::Promote { person }
        | OrderKind::Train { person, .. } => person_must_exist(*person, state),

        OrderKind::Assign { person, to } => validate_assign(*person, to, state),

        OrderKind::SetObjective { programme, .. } | OrderKind::SetBudget { programme, .. } => {
            programme_must_exist(*programme, state)?;
            programme_must_take_work(*programme, state)
        }

        OrderKind::PostponeDeadline {
            programme,
            objective,
            ..
        } => {
            programme_must_exist(*programme, state)?;
            objective_must_exist(*programme, *objective, state)
        }

        OrderKind::SetSubsystem { vehicle, .. } => vehicle_must_exist(*vehicle, state),

        // **A mission needs a vehicle that is finished.**
        OrderKind::AuthorizeLaunch { vehicle, .. } => {
            vehicle_must_exist(*vehicle, state)?;
            let state_of = state.vehicles.get(vehicle).expect("the vehicle");
            if state_of.build_state != domain::vehicle::BuildState::Complete {
                return Err(OrderError::Rejected {
                    reason: format!("{} is not finished being built.", state_of.name),
                });
            }
            Ok(())
        }

        OrderKind::StartProject {
            principal_investigator,
            ..
        } => person_must_exist(*principal_investigator, state),

        OrderKind::StartTestCampaign { capability } => {
            // **The agency cannot test what it has not developed.** The
            // capability has to be one the agency has unlocked, and not merely
            // one that exists in the world: the palette only offers the
            // unlocked ones, and the engine has to agree with the palette.
            if state.unlocked.contains(capability) {
                Ok(())
            } else {
                Err(OrderError::CapabilityUnavailable {
                    capability: capability_label(*capability),
                })
            }
        }

        OrderKind::AssignFacility {
            facility,
            programme,
            ..
        } => {
            facility_must_exist(*facility, state)?;
            programme_must_exist(*programme, state)
        }

        OrderKind::CancelMission { mission } => {
            if state.missions.contains_key(mission) {
                Ok(())
            } else {
                Err(OrderError::Rejected {
                    reason: "There is no such mission.".to_owned(),
                })
            }
        }
    }
}

/// The person must exist, and must be free to take the assignment.
fn validate_assign(
    person: PersonId,
    to: &domain::person::Assignment,
    state: &CampaignState,
) -> Result<(), OrderError> {
    person_must_exist(person, state)?;
    let current = &state.people[&person];
    if current.is_assigned() {
        return Err(OrderError::AlreadyAssigned {
            name: current.name.clone(),
            current: crate::derived::assignment_description(state, &current.assignment),
        });
    }
    match to {
        domain::person::Assignment::Programme { programme, .. } => {
            programme_must_exist(*programme, state)
        }
        domain::person::Assignment::Facility { facility } => facility_must_exist(*facility, state),
        domain::person::Assignment::Project { project } => {
            if state.projects.contains_key(project) {
                Ok(())
            } else {
                Err(OrderError::Rejected {
                    reason: "There is no such project.".to_owned(),
                })
            }
        }
        domain::person::Assignment::TestCampaign { campaign } => {
            if state.test_campaigns.contains_key(campaign) {
                Ok(())
            } else {
                Err(OrderError::Rejected {
                    reason: "There is no such test campaign.".to_owned(),
                })
            }
        }
        domain::person::Assignment::Division(_) | domain::person::Assignment::Unassigned => Ok(()),
    }
}

/// The vehicle must exist.
fn vehicle_must_exist(
    vehicle: domain::ids::VehicleId,
    state: &CampaignState,
) -> Result<(), OrderError> {
    if state.vehicles.contains_key(&vehicle) {
        Ok(())
    } else {
        Err(OrderError::NoSuchVehicle {
            name: "a vehicle that has been scrapped".to_owned(),
        })
    }
}

/// The programme must exist.
fn programme_must_exist(
    programme: domain::ids::ProgrammeId,
    state: &CampaignState,
) -> Result<(), OrderError> {
    if state.programmes.contains_key(&programme) {
        Ok(())
    } else {
        Err(OrderError::NoSuchProgramme {
            name: "a programme that has closed".to_owned(),
        })
    }
}

/// The objective must exist in the programme.
///
/// **A postponement names a deadline**, and a deadline that is not there
/// cannot be pushed back.
fn objective_must_exist(
    programme: domain::ids::ProgrammeId,
    objective: domain::ids::ObjectiveId,
    state: &CampaignState,
) -> Result<(), OrderError> {
    let Some(programme) = state.programmes.get(&programme) else {
        return programme_must_exist(programme, state);
    };
    if programme.objectives.iter().any(|o| o.id == objective) {
        Ok(())
    } else {
        Err(OrderError::Rejected {
            reason: format!("The programme {} has no such objective.", programme.title),
        })
    }
}

/// The programme must be one that can still take work or money.
///
/// **A cancelled programme is cancelled.** The status exists to mean
/// something, and an order that adds work to a finished programme has to be
/// refused with the status in the message.
fn programme_must_take_work(
    programme: domain::ids::ProgrammeId,
    state: &CampaignState,
) -> Result<(), OrderError> {
    let Some(programme) = state.programmes.get(&programme) else {
        return programme_must_exist(programme, state);
    };
    match programme.status {
        domain::programme::ProgrammeStatus::Cancelled
        | domain::programme::ProgrammeStatus::Failed
        | domain::programme::ProgrammeStatus::Completed => Err(OrderError::WrongStatus {
            name: programme.title.clone(),
            status: programme.status.label().to_owned(),
        }),
        _ => Ok(()),
    }
}
/// The facility must exist.
fn facility_must_exist(
    facility: domain::ids::FacilityId,
    state: &CampaignState,
) -> Result<(), OrderError> {
    if state.facilities.contains_key(&facility) {
        Ok(())
    } else {
        Err(OrderError::NoSuchFacility {
            name: "a facility that has been demolished".to_owned(),
        })
    }
}

/// The person must exist.
fn person_must_exist(person: PersonId, state: &CampaignState) -> Result<(), OrderError> {
    match state.people.get(&person) {
        Some(_found) => Ok(()),
        None => Err(OrderError::PersonGone {
            person,
            name: "someone who has left".to_owned(),
        }),
    }
}

/// The name of a capability, for messages.
#[must_use]
pub fn capability_label(capability: domain::research::CapabilityId) -> String {
    domain::research::capability_name(capability).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{AgencyId, PersonId};
    use domain::money::Money;
    use domain::person::{Assignment, Person, Role};
    use domain::state::Agency;

    fn state() -> CampaignState {
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
        state
    }

    #[test]
    fn hiring_is_always_valid() {
        let kind = OrderKind::Hire {
            role: Role::Scientist,
            salary: Money::from_credits(1_200),
        };
        assert!(validate(&kind, &state()).is_ok());
    }

    #[test]
    fn assigning_an_unassigned_person_is_valid() {
        let kind = OrderKind::Assign {
            person: PersonId(0),
            to: Assignment::Unassigned,
        };
        assert!(validate(&kind, &state()).is_ok());
    }

    #[test]
    fn assigning_an_already_assigned_person_is_refused_with_a_reason() {
        let mut campaign = state();
        campaign
            .people
            .get_mut(&PersonId(0))
            .expect("the person exists")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));

        let kind = OrderKind::Assign {
            person: PersonId(0),
            to: Assignment::Unassigned,
        };
        let error = validate(&kind, &campaign).expect_err("the order is refused");
        assert!(matches!(error, OrderError::AlreadyAssigned { .. }));
        assert!(error.to_string().contains("Marsha Okonjo"));
    }

    #[test]
    fn an_order_about_someone_who_does_not_exist_is_refused() {
        let kind = OrderKind::Promote {
            person: PersonId(99),
        };
        assert!(matches!(
            validate(&kind, &state()),
            Err(OrderError::PersonGone { .. })
        ));
    }

    #[test]
    fn validation_does_not_change_the_state() {
        let campaign = state();
        let before = campaign.clone();
        let kind = OrderKind::Assign {
            person: PersonId(0),
            to: Assignment::Division(domain::ids::DivisionId(0)),
        };
        let _ = validate(&kind, &campaign);
        assert_eq!(campaign, before);
    }

    // ---- The lead times, the costs, and the goodwill ----

    #[test]
    fn a_hire_takes_longer_the_more_senior_it_is() {
        // **The agency cannot buy a division head in a fortnight.**
        let entry = hire_lead_time(Role::Scientist, Money::from_credits(1_200));
        let senior = hire_lead_time(Role::Scientist, Money::from_credits(2_600));
        assert!(senior > entry, "a senior hire takes longer to find");
        assert_eq!(entry, Duration::from_days(balance::HIRE_LEAD_TIME_MIN_DAYS));
        assert_eq!(
            senior,
            Duration::from_days(balance::HIRE_LEAD_TIME_MAX_DAYS)
        );
    }

    #[test]
    fn a_hire_never_takes_longer_than_six_weeks() {
        let absurd = hire_lead_time(Role::Scientist, Money::from_credits(1_000_000));
        assert!(
            absurd <= Duration::from_days(balance::HIRE_LEAD_TIME_MAX_DAYS),
            "a hire does not take more than six weeks, however senior"
        );
    }

    #[test]
    fn a_promotion_waits_for_the_next_quarterly_round() {
        // Just after a round, the wait is almost three months. Just before one,
        // it is almost none.
        let after = next_promotion_round(Instant::from_months(3));
        let before = next_promotion_round(Instant::from_months(2));
        assert_eq!(after, Duration::from_months(3));
        assert_eq!(before, Duration::from_months(1));
    }

    #[test]
    fn an_immediate_order_lands_at_once() {
        let kind = OrderKind::SetBudget {
            programme: domain::ids::ProgrammeId(0),
            allocation: Money::from_credits(1_000),
        };
        assert_eq!(base_lead_time(&kind, Instant::ZERO), Duration::ZERO);
    }

    #[test]
    fn a_facility_takes_as_long_as_its_plan_says() {
        let plan = domain::facility::ConstructionPlan {
            method: domain::facility::BuildMethod::Standard,
            scale: domain::facility::BuildScale::Small,
            site: domain::facility::Site {
                name: "The Core".to_owned(),
                publicly_visible: false,
            },
            financing: domain::facility::Financing::StateBudget,
            total_cost: Money::from_credits(350_000),
            build_months: 15,
        };
        let kind = OrderKind::BuildFacility {
            kind: domain::facility::FacilityType::Laboratory,
            plan,
        };
        assert_eq!(
            base_lead_time(&kind, Instant::ZERO),
            Duration::from_months(15)
        );
        assert_eq!(cost(&kind, &state()), Money::from_credits(350_000));
    }

    #[test]
    fn the_goodwill_is_proportional_to_how_disruptive_the_order_is() {
        // **Goodwill is the anti-spamming mechanism, and it is deliberately
        // not a cap.**
        let policy = OrderKind::SetPolicy {
            policy: "Publicity".to_owned(),
            enabled: true,
        };
        let hire = OrderKind::Hire {
            role: Role::Scientist,
            salary: Money::from_credits(1_200),
        };
        assert!(goodwill(&policy) > goodwill(&hire));
        assert_eq!(goodwill(&hire), balance::GOODWILL_HIRE);
    }

    #[test]
    fn an_order_with_no_handler_is_not_accepted() {
        // **A policy still has no handler**, and the test is that an order
        // with no handler is refused at issue rather than accepted and
        // quietly doing nothing.
        let kind = OrderKind::SetPolicy {
            policy: "Open the launch manifest".to_owned(),
            enabled: true,
        };
        assert!(!has_handler(&kind));
        let hire = OrderKind::Hire {
            role: Role::Scientist,
            salary: Money::from_credits(1_200),
        };
        assert!(has_handler(&hire));
    }
}
