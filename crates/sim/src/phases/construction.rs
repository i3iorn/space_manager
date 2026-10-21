//! Phase 4 — construction and production.
//!
//! The facilities that start, and the facilities that open.
//!
//! Construction is **routinely late**. The build takes as long as it takes, and
//! the schedule is a forecast, not a promise.
//!
//! **The build progress is a rate, and it lives in `crate::rates`.** What is
//! left here is what happens when the build is done: the specification is
//! calculated from the construction choices, and then it is fixed for the rest
//! of the campaign.

use domain::chronicle::{Category, ChronicleEntry, Severity};
use domain::facility::{Construction, Facility, Specification};
use domain::ids::FacilityId;
use domain::order::OrderKind;
use domain::research::Progress;

use crate::balance;
use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The construction phase.
pub struct ConstructionPhase;

impl Phase for ConstructionPhase {
    const NAME: &'static str = "construction";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        start_construction(context);
        complete_finished_construction(context);
        Ok(())
    }
}

/// Starts the facilities the player asked for.
fn start_construction(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);

    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Applies one construction order, and reports whether it was one.
///
/// **The handlers are per-kind [D].** An order lands at its own instant, and
/// the phase that owns it is the one that applies it.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::BuildFacility { kind, plan } => {
            let id = context.state.id_counters.next_facility();
            let facility = Facility {
                id,
                name: format!("{} {}", kind.label(), context.state.facilities.len() + 1),
                kind: *kind,
                site: plan.site.clone(),
                specification: Specification::empty(),
                condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
                staff: Vec::new(),
                construction: Some(Construction {
                    plan: plan.clone(),
                    started: context.month,
                    // A hundred points, at 256 scaled units to the point.
                    progress: Progress::empty(100 * domain::scale::PERCENT_UNITS as u32),
                    expected_completion: Some(context.month),
                    remainders: domain::scale::Remainders::new(),
                }),
                upkeep: balance::facility_base_upkeep(*kind),
                monthly_output: 0,
                remainders: domain::scale::Remainders::new(),
            };
            context.state.facilities.insert(id, facility);

            context.programme_lines.push(
                domain::report::ReportLine::new(format!(
                    "Construction of {} began, using the {} method.",
                    kind.label(),
                    plan.method.label()
                ))
                .with_severity(Severity::Minor),
            );
            true
        }
        _ => false,
    }
}

/// Opens every facility whose build is done.
fn complete_finished_construction(context: &mut MonthContext<'_>) {
    let facility_ids: Vec<FacilityId> = context.state.facilities.keys().copied().collect();

    for id in facility_ids {
        let finished = context.state.facilities[&id]
            .construction
            .as_ref()
            .is_some_and(|construction| construction.progress.is_complete());

        if finished {
            complete_facility(context, id);
        }
    }
}

/// A finished facility gets its specification, computed from the construction
/// choices and then fixed.
fn complete_facility(context: &mut MonthContext<'_>, id: FacilityId) {
    let Some(construction) = context.state.facilities[&id].construction.clone() else {
        return;
    };
    let plan = &construction.plan;

    let base_capacity = match context.state.facilities[&id].kind {
        domain::facility::FacilityType::Laboratory => 4,
        domain::facility::FacilityType::TestStand
        | domain::facility::FacilityType::LaunchComplex => 2,
        domain::facility::FacilityType::FabricationPlant => 6,
    };

    let capacity = base_capacity * i64::from(plan.scale.capacity_multiplier_bp()) / 10_000
        * i64::from(plan.method.capacity_multiplier_bp())
        / 10_000;
    let capacity = capacity.max(1);

    let reliability = (60_i16 + plan.method.reliability_delta()).clamp(0, 100) as u8;
    let quality = (60_i16 + plan.method.reliability_delta() / 2).clamp(0, 100) as u8;
    let automation = match plan.scale {
        domain::facility::BuildScale::Small => 30,
        domain::facility::BuildScale::Standard => 50,
        domain::facility::BuildScale::Large => 70,
        domain::facility::BuildScale::Oversized => 85,
    };

    let upkeep = balance::facility_base_upkeep(context.state.facilities[&id].kind)
        .multiply_fraction(i64::from(plan.scale.upkeep_multiplier_bp()), 10_000)
        .multiply_fraction(i64::from(plan.method.upkeep_multiplier_bp()), 10_000);

    let name = context.state.facilities[&id].name.clone();
    let facility = context
        .state
        .facilities
        .get_mut(&id)
        .expect("the facility exists");
    facility.construction = None;
    facility.specification = Specification {
        capacity,
        reliability,
        automation,
        staff_needed: capacity,
        quality,
    };
    facility.upkeep = upkeep;
    facility.monthly_output = capacity;

    let entry = ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        Category::Technical,
        format!("{name} is open for business."),
        Severity::Minor,
    );
    context.events.push(entry);
    context
        .opportunities
        .push(format!("{name} is now operational."));
}

/// The condition of a facility, used by the interface.
#[cfg(test)]
mod condition_tests {
    /// A facility's condition, as a plain value.
    const fn condition_of(condition: u8) -> u8 {
        condition
    }

    #[test]
    fn a_new_facility_is_in_good_condition() {
        assert_eq!(condition_of(100), 100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::facility::{BuildMethod, BuildScale, ConstructionPlan, Financing, Site};
    use domain::ids::AgencyId;
    use domain::money::Money;
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    fn plan() -> ConstructionPlan {
        ConstructionPlan {
            method: BuildMethod::Standard,
            scale: BuildScale::Small,
            site: Site {
                name: "The Core".to_owned(),
                publicly_visible: false,
            },
            financing: Financing::StateBudget,
            total_cost: Money::from_credits(350_000),
            build_months: 15,
        }
    }

    #[test]
    fn a_facility_is_built_and_becomes_operational() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(3);
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            Instant::ZERO,
            vec![OrderKind::BuildFacility {
                kind: domain::facility::FacilityType::Laboratory,
                plan: plan(),
            }],
        );
        crate::phases::intake::IntakePhase::run(&mut context).expect("intake runs");
        // **The build progress is a rate now**, and it lives in `crate::rates`.
        // The phase is what happens when the build is done.
        for _ in 0..20 {
            crate::rates::integrate(context.state, domain::time::Duration::from_months(1));
            ConstructionPhase::run(&mut context).expect("the phase runs");
            context.month = context
                .month
                .saturating_add(domain::time::Duration::from_months(1));
        }
        let facility = context
            .state
            .facilities
            .values()
            .next()
            .expect("a facility");
        assert!(facility.is_operational());
        assert!(facility.specification.capacity > 0);
    }
}
