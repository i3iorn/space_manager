//! A stress test: a long campaign with orders being given.
//!
//! **The headless smoke test runs a
//! long campaign but gives no orders, and the short tests give orders but run
//! for a few months. A playtest does both, so this does both: a campaign of
//! five years, with an order every week, and the state keeps changing
//! underneath the orders.

use domain::order::OrderKind;
use domain::person::Assignment;
use sim::{Scenario, Sim};

#[test]
fn a_long_campaign_with_orders_does_not_panic() {
    let mut sim = Sim::new(&Scenario::first_year()).expect("the campaign starts");
    let scenario = Scenario::first_year();

    for week in 0..220u32 {
        let target = sim
            .effective_now()
            .saturating_add(domain::time::Duration::from_days(7));
        sim.advance_to(target).expect("the campaign advances");

        // An order every week, alternating between the kinds that touch the
        // most state.
        let order = match week % 6 {
            0 => OrderKind::Hire {
                role: domain::person::Role::Scientist,
                salary: domain::money::Money::from_credits(1_200),
            },
            1 => OrderKind::CreateProgramme {
                title: format!("Programme {week}"),
                category: domain::programme::ProgrammeCategory::Orbital,
            },
            2 => OrderKind::SetShift {
                person: domain::ids::PersonId(0),
                shift: Some(110),
            },
            3 => OrderKind::Promote {
                person: domain::ids::PersonId(0),
            },
            4 => OrderKind::Unassign {
                person: domain::ids::PersonId(0),
            },
            _ => OrderKind::Assign {
                person: domain::ids::PersonId(0),
                to: Assignment::Unassigned,
            },
        };
        // **A refusal is a fine outcome.** The point is that whatever the
        // order, the engine survives it.
        let _ = sim.issue(order);
        let _ = &scenario;
    }
}
