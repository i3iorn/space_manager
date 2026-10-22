//! The error-message pass.
//!
//! **Every message a player reads is a sentence to a player.** Not a
//! developer-facing string with a field name in it, and not a code: a refusal
//! that says "person already assigned" tells the player nothing they can act
//! on, and a refusal that says `SetPolicy` has leaked an implementation
//! detail into the interface.
//!
//! These tests are the pass. They are cheap, and they are the reason the
//! messages stay readable as the game grows.

use domain::order::OrderKind;
use domain::person::{Assignment, ProgrammeRole, Role};
use sim::Scenario;
use sim::Sim;
use sim::error::{OrderError, SimError};

/// Whether a message reads as a sentence to a player.
///
/// **A sentence starts with a capital letter and ends with a full stop**, and
/// it does not contain an identifier: a player cannot act on "ProgrammeId(0)".
fn reads_as_a_sentence(message: &str) -> bool {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return false;
    }
    let mut characters = trimmed.chars();
    let first = characters.next().unwrap_or(' ');
    let last = trimmed.chars().next_back().unwrap_or(' ');
    let capitalised = first.is_uppercase();
    let punctuated = matches!(last, '.' | '!' | '?');
    let no_identifier = !trimmed.contains("Id(") && !trimmed.contains("::");
    capitalised && punctuated && no_identifier
}

/// Every refusal the engine can make, and the message it makes.
fn refusals() -> Vec<String> {
    let mut scenario = Scenario::first_orbit();
    scenario
        .start_state
        .unlocked
        .insert(domain::research::CapabilityId(2));
    let mut sim = Sim::new(&scenario).expect("the campaign starts");

    let mut messages: Vec<String> = Vec::new();
    let mut record = |error: OrderError| messages.push(error.to_string());

    // A person who has left.
    record(
        sim.issue(OrderKind::Promote {
            person: domain::ids::PersonId(999),
        })
        .expect_err("the order is refused"),
    );
    // A person who is already where they have been put.
    let assigned = *sim.state().people.keys().next().expect("a person");
    let already = sim.state().people[&assigned].assignment.clone();
    record(
        sim.issue(OrderKind::Assign {
            person: assigned,
            to: already,
        })
        .expect_err("the order is refused"),
    );
    // A programme that does not exist.
    record(
        sim.issue(OrderKind::SetBudget {
            programme: domain::ids::ProgrammeId(99),
            allocation: domain::money::Money::ZERO,
        })
        .expect_err("the order is refused"),
    );
    // A vehicle that does not exist.
    record(
        sim.issue(OrderKind::SetSubsystem {
            vehicle: domain::ids::VehicleId(99),
            slider: domain::vehicle::Slider::Propulsion,
            value: 50,
        })
        .expect_err("the order is refused"),
    );
    // A capability the agency has not developed.
    record(
        sim.issue(OrderKind::StartTestCampaign {
            capability: domain::research::CapabilityId(3),
        })
        .expect_err("the order is refused"),
    );
    // An order with no handler.
    record(
        sim.issue(OrderKind::PublishResult {
            result: "The soil survey".to_owned(),
            mode: domain::order::KnowledgeMode::Open,
        })
        .expect_err("the order is refused"),
    );
    // A launch to somewhere the agency cannot fly to.
    record(
        sim.issue(OrderKind::AuthorizeLaunch {
            vehicle: domain::ids::VehicleId(0),
            destination: "Low orbit".to_owned(),
            crew: Vec::new(),
        })
        .expect_err("the order is refused"),
    );

    messages
}

#[test]
fn every_refusal_reads_as_a_sentence() {
    for message in refusals() {
        assert!(
            reads_as_a_sentence(&message),
            "the refusal does not read as a sentence: {message:?}"
        );
    }
}

#[test]
fn every_refusal_names_the_thing_it_is_about() {
    // **A refusal that says nothing actionable is a refusal that only
    // annoys.** Each of these has to say what the player should have done
    // instead.
    let messages = refusals();
    let joined = messages.join("\n");

    assert!(
        joined.contains("no longer with the agency"),
        "a refusal about a person who has left says so: {joined}"
    );
    assert!(
        joined.contains("already on"),
        "a refusal about an assignment says so: {joined}"
    );
    assert!(
        joined.contains("no programme called"),
        "a refusal about a programme says so: {joined}"
    );
    assert!(
        joined.contains("not yet developed"),
        "a refusal about a capability says so: {joined}"
    );
    assert!(
        joined.contains("no way to do that"),
        "a refusal about an order the agency cannot take says so: {joined}"
    );
}

#[test]
fn no_refusal_contains_an_identifier() {
    // **An identifier in a message is an implementation detail in the
    // interface**, and the first playtest found four of them.
    for message in refusals() {
        assert!(
            !message.contains("Id("),
            "the refusal leaks an identifier: {message:?}"
        );
        assert!(
            !message.contains("ProgrammeId"),
            "the refusal leaks an identifier: {message:?}"
        );
    }
}

#[test]
fn the_version_mismatch_says_what_to_do() {
    // **A refusal the player cannot act on is a refusal that only annoys.**
    let error = SimError::VersionMismatch {
        expected: "the running version".to_owned(),
        found: "the saved version".to_owned(),
    };
    let message = error.to_string();
    assert!(
        reads_as_a_sentence(&message),
        "the message reads well: {message}"
    );
    assert!(
        message.contains("the running version") && message.contains("the saved version"),
        "and it says which is which: {message}"
    );
}

#[test]
fn a_corrupt_campaign_says_what_is_wrong() {
    let error = SimError::CorruptState {
        reason: "the file is not a campaign".to_owned(),
    };
    let message = error.to_string();
    assert!(
        reads_as_a_sentence(&message),
        "the message reads well: {message}"
    );
    assert!(
        message.contains("the file is not a campaign"),
        "and it says what is wrong: {message}"
    );
}

#[test]
fn the_order_error_variants_all_have_a_message() {
    // **Every variant of the error has to say something.** A variant with an
    // empty message is a variant that tells the player nothing.
    let errors = [
        OrderError::PersonGone {
            person: domain::ids::PersonId(0),
            name: "Marsha Okonjo".to_owned(),
        },
        OrderError::AlreadyAssigned {
            name: "Marsha Okonjo".to_owned(),
            current: "the Combustion stability project".to_owned(),
        },
        OrderError::CannotAssign {
            name: "Marsha Okonjo".to_owned(),
        },
        OrderError::NoSuchProgramme {
            name: "Orbital Survey".to_owned(),
        },
        OrderError::NoSuchFacility {
            name: "The Old Lab".to_owned(),
        },
        OrderError::NoSuchVehicle {
            name: "Orbital Launcher 1".to_owned(),
        },
        OrderError::WrongStatus {
            name: "Orbital Survey".to_owned(),
            status: "Cancelled".to_owned(),
        },
        OrderError::InsufficientFunds {
            cost: "350,000 credits".to_owned(),
            available: "12,000 credits".to_owned(),
        },
        OrderError::CapabilityUnavailable {
            capability: "Basic re-entry".to_owned(),
        },
        OrderError::Rejected {
            reason: "There is no way to do that yet.".to_owned(),
        },
    ];
    for error in errors {
        let message = error.to_string();
        assert!(
            reads_as_a_sentence(&message),
            "the refusal does not read as a sentence: {message:?}"
        );
    }
}

#[test]
fn the_hire_order_is_always_accepted() {
    // **A sanity check on the whole pass**: the common orders are not
    // refusals, and a refusal for one of them would be a bug.
    let scenario = Scenario::first_orbit();
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    assert!(
        sim.issue(OrderKind::Hire {
            role: Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        })
        .is_ok(),
        "a hire is always accepted"
    );
    let programme = *sim.state().programmes.keys().next().expect("a programme");
    // **Somebody with nothing to do.** A person who is already assigned
    // cannot be given a second assignment, and that is a different refusal.
    let free = sim
        .state()
        .people
        .values()
        .find(|person| !person.is_assigned())
        .expect("somebody with nothing to do");
    let person = free.id;
    assert!(
        sim.issue(OrderKind::Assign {
            person,
            to: Assignment::Programme {
                programme,
                role: ProgrammeRole::Member,
            },
        })
        .is_ok(),
        "and so is an assignment to a programme that exists"
    );
}

#[test]
fn a_refusal_changes_nothing() {
    // **A refused order is refused, not half-applied.** This is the rule the
    // whole order model rests on, and it is worth one test of its own.
    let scenario = Scenario::first_orbit();
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    let before = sim.state_hash();
    let money = sim.state().treasury.balance;
    let _ = sim.issue(OrderKind::Promote {
        person: domain::ids::PersonId(999),
    });
    assert_eq!(
        sim.state_hash(),
        before,
        "a refused order changes nothing at all"
    );
    assert_eq!(sim.state().treasury.balance, money, "and it costs nothing");
    assert!(
        sim.pending_orders().is_empty(),
        "and nothing is put in flight"
    );
    assert!(
        sim.state()
            .chronicle
            .iter()
            .all(|entry| !entry.headline.contains("refused")),
        "and the chronicle says nothing about it"
    );
}
