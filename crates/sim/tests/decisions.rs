//! The decisions the world puts to the player.
//!
//! **An event that asks nothing is a sentence; an event that asks something is
//! a decision.** The event catalogue narrates; the decision catalogue asks.
//! These tests follow a question from the instant its event fires to the
//! instant the agency answers it — or lets it pass.
//!
//! The campaign is seed 4 of `first_year`, which is known to raise its first
//! decision ("The state has noticed a programme is behind") inside the first
//! month. The event's own effects are the baseline, so the tests measure the
//! *answer's* effect against a control that chose the other answer, rather
//! than against the state before the event.

use domain::time::Duration;
use sim::{Scenario, Sim, SimError};

/// A campaign whose first decision is known to arrive in the first month.
fn campaign() -> Sim {
    let mut scenario = Scenario::first_year();
    scenario.seed = 4;
    Sim::new(&scenario).expect("the campaign starts")
}

/// Advances a day, and returns whether a decision is now waiting.
fn step(sim: &mut Sim) -> bool {
    let target = sim.effective_now().saturating_add(Duration::from_days(1));
    sim.advance_to(target).expect("the day resolves");
    !sim.pending_decisions().is_empty()
}

/// Advances a day at a time until a decision is raised.
fn until_it_asks(sim: &mut Sim) {
    for _ in 0..31 {
        if step(sim) {
            return;
        }
    }
    panic!("seed 4 raises its first decision in the first month");
}

/// Advances a day at a time until the window has closed.
fn until_the_window_closes(sim: &mut Sim) {
    for _ in 0..11 {
        if !step(sim) {
            return;
        }
    }
    panic!("the window closes ten days after the question was raised");
}

#[test]
fn a_decision_arrives_with_a_window() {
    let mut sim = campaign();
    until_it_asks(&mut sim);

    let pending = &sim.pending_decisions()[0];
    assert_eq!(pending.id, 1, "the first question is numbered one");
    assert_eq!(
        pending.definition().days_to_decide,
        10,
        "the state's letter gives ten days"
    );
    assert!(
        pending.due > sim.effective_now(),
        "the window is still open when the question arrives"
    );
}

#[test]
fn the_answer_applies_its_effects_and_the_question_closes() {
    let mut explained = campaign();
    let mut quiet = campaign();
    until_it_asks(&mut explained);
    until_it_asks(&mut quiet);

    explained
        .decide(1, "explain")
        .expect("the answer is one of the choices");
    quiet
        .decide(1, "quiet")
        .expect("the answer is one of the choices");

    // **The two campaigns are the same campaign with one different answer**,
    // so the difference in the state's approval is the answer's effect and
    // nothing else.
    let difference =
        explained.state().political.approval.percent() - quiet.state().political.approval.percent();
    assert_eq!(difference, 2, "explaining buys two points of approval");

    assert!(
        explained.pending_decisions().is_empty(),
        "the question is off the table once it is answered"
    );
    assert!(
        explained
            .state()
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("answered")),
        "the chronicle records the answer: {:?}",
        explained
            .state()
            .chronicle
            .iter()
            .map(|entry| &entry.headline)
            .collect::<Vec<_>>()
    );
}

#[test]
fn silence_applies_the_expiry_effects() {
    let mut answered = campaign();
    let mut silent = campaign();
    until_it_asks(&mut answered);
    until_it_asks(&mut silent);

    answered
        .decide(1, "quiet")
        .expect("the answer is one of the choices");
    until_the_window_closes(&mut silent);

    let difference =
        answered.state().political.approval.percent() - silent.state().political.approval.percent();
    assert_eq!(difference, 2, "silence costs two points of approval");

    assert!(
        silent
            .state()
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("let it pass")),
        "the chronicle records the silence"
    );
}

#[test]
fn the_expiry_of_an_answered_decision_is_a_no_op() {
    let mut sim = campaign();
    until_it_asks(&mut sim);
    sim.decide(1, "explain")
        .expect("the answer is one of the choices");

    let let_it_pass = |sim: &Sim| {
        sim.state()
            .chronicle
            .iter()
            .filter(|entry| entry.headline.contains("let it pass"))
            .count()
    };
    let before = let_it_pass(&sim);

    // The expiry instant is ten days out; a fortnight covers it.
    for _ in 0..14 {
        step(&mut sim);
    }
    assert_eq!(let_it_pass(&sim), before, "an answered expiry does nothing");
    assert!(
        sim.pending_decisions().is_empty(),
        "the question stays answered"
    );
}

#[test]
fn a_saved_campaign_keeps_its_question() {
    let mut sim = campaign();
    until_it_asks(&mut sim);
    let expected = sim.pending_decisions().to_vec();

    let text = sim.save().expect("the campaign saves");
    let mut loaded = Sim::load(&text).expect("the campaign loads");
    assert_eq!(
        loaded.pending_decisions(),
        expected.as_slice(),
        "the question comes back with the campaign"
    );

    // **The window survives the save**, because a decision whose expiry was
    // lost would be a decision the player could sit on for ever.
    until_the_window_closes(&mut loaded);
    assert!(
        loaded
            .state()
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("let it pass")),
        "the expiry still fires after a save and a load"
    );
}

#[test]
fn a_wrong_answer_is_refused_and_the_question_stays() {
    let mut sim = campaign();
    until_it_asks(&mut sim);
    let id = sim.pending_decisions()[0].id;

    assert!(
        matches!(
            sim.decide(id + 1, "explain"),
            Err(SimError::NoDecision { .. })
        ),
        "there is no question by that number"
    );
    assert!(
        matches!(sim.decide(id, "bribe"), Err(SimError::NoSuchOption { .. })),
        "a bribe is not one of the choices"
    );
    assert_eq!(
        sim.pending_decisions().len(),
        1,
        "a refused answer leaves the question on the table"
    );
}
