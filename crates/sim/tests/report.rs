//! The campaign report.
//!
//! **The report is what a playtester sends back.** It has to say what
//! happened, and it has to name the seed so the campaign can be reproduced.

use sim::{Scenario, Sim};

fn campaign() -> Sim {
    let mut scenario = Scenario::first_orbit();
    scenario.seed = 4_242;
    let mut sim = Sim::new(&scenario).expect("the scenario starts");
    sim.advance_to(domain::time::Instant::from_months(24))
        .expect("the campaign advances");
    sim
}

#[test]
fn the_report_names_the_campaign_and_how_to_reproduce_it() {
    // **A playtester who cannot reproduce the campaign has sent a report
    // about a campaign nobody else can see.** The scenario and the seed are
    // the first two lines.
    let sim = campaign();
    let report = sim::summary::summary(&sim);
    assert!(report.contains("seed 4242"), "the seed is in it:\n{report}");
    assert!(
        report.contains(&sim.state().agency.name),
        "the agency's name is in it:\n{report}"
    );
    assert!(
        report.contains(sim.scenario().name.as_str()),
        "the scenario is in it:\n{report}"
    );
}

#[test]
fn the_report_says_what_happened() {
    // **The report is a summary of the campaign and not a summary of the
    // code.** It has to name the money, the staff, the programmes, and the
    // missions.
    let sim = campaign();
    let report = sim::summary::summary(&sim);
    for section in [
        "THE MONEY",
        "THE AGENCY",
        "THE PROGRAMMES",
        "THE MISSIONS",
        "WHAT THE AGENCY KNOWS",
        "THE CHRONICLE",
        "THE DECISIONS",
    ] {
        assert!(
            report.contains(section),
            "the report has {section}:\n{report}"
        );
    }
    assert!(
        report.contains(&sim.state().treasury.balance.to_string()),
        "the balance is in it:\n{report}"
    );
}

#[test]
fn the_report_carries_the_state_hash() {
    // **Two reports of the same campaign can be compared without replaying
    // it**, and that is what the hash is for.
    let sim = campaign();
    let report = sim::summary::summary(&sim);
    assert!(
        report.contains(&format!("state hash {}", sim.state_hash())),
        "the hash is in it:\n{report}"
    );
}

#[test]
fn the_report_is_the_same_for_the_same_campaign() {
    // **A report that changes between two runs of the same campaign is a
    // report nobody can trust.**
    let first = sim::summary::summary(&campaign());
    let second = sim::summary::summary(&campaign());
    assert_eq!(first, second);
}
