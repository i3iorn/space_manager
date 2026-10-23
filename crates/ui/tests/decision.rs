//! The decision dialog, end to end.
//!
//! **A decision is the game's own pause button.** The world stops while the
//! question is on the table, the player answers or sets it aside, and the
//! window keeps running either way. These tests walk that flow with real keys:
//! the question arrives on its own, `Esc` puts it aside, `D` brings it back,
//! and `Enter` answers it at the price the dialog showed.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use domain::time::Duration;
use sim::{Scenario, Sim};
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A campaign whose first decision is known to arrive in the first month.
fn app() -> App {
    let mut scenario = Scenario::first_year();
    scenario.seed = 4;
    App::started(Sim::new(&scenario).expect("the campaign starts"))
}

/// Advances a day at a time until the question is on the table.
fn until_it_asks(app: &mut App) {
    for _ in 0..31 {
        if app.decision().is_some() {
            return;
        }
        app.advance(Duration::from_days(1));
    }
    panic!("seed 4 raises its first decision in the first month");
}

#[test]
fn the_question_comes_up_on_its_own() {
    let mut app = app();
    assert!(app.decision().is_none(), "nothing waits at the start");
    until_it_asks(&mut app);
    assert!(app.decision().is_some(), "the dialog opens by itself");
}

#[test]
fn the_world_stops_while_the_question_is_open() {
    let mut app = app();
    until_it_asks(&mut app);
    let now = app.sim().effective_now();

    app.advance(Duration::from_months(2));
    assert_eq!(
        app.sim().effective_now(),
        now,
        "the world waits while the player reads the question"
    );
}

#[test]
fn the_question_can_be_put_aside_and_brought_back() {
    let mut app = app();
    until_it_asks(&mut app);

    app.on_key(press(KeyCode::Esc));
    assert!(app.decision().is_none(), "Esc puts it aside");

    // **The clock runs while the question is set aside** — that is what
    // putting it aside means, and the window keeps counting.
    let now = app.sim().effective_now();
    app.advance(Duration::from_days(1));
    assert!(
        app.sim().effective_now() > now,
        "the clock runs while the question is aside"
    );

    app.on_key(press(KeyCode::Char('d')));
    assert!(app.decision().is_some(), "`D` brings it back");
}

#[test]
fn the_advice_points_at_the_question() {
    let mut app = app();
    until_it_asks(&mut app);
    app.on_key(press(KeyCode::Esc));

    let advice: Vec<String> = (0..app.advice_count())
        .filter_map(|index| app.advice_at(index))
        .collect();
    assert!(
        advice
            .iter()
            .any(|line| line.contains("The window closes on") && line.contains('D')),
        "the advice names the question and the key: {advice:?}"
    );
}

#[test]
fn answering_applies_the_price_the_dialog_showed() {
    let mut app = app();
    until_it_asks(&mut app);
    let before = app.sim().state().political.approval.percent();

    // The first answer is "Explain the slip, and ask for time": two points.
    app.on_key(press(KeyCode::Enter));
    assert!(
        app.decision().is_none(),
        "the question is answered and gone"
    );
    assert!(
        app.sim().pending_decisions().is_empty(),
        "the simulation holds no question"
    );
    assert_eq!(
        app.sim().state().political.approval.percent() - before,
        2,
        "explaining buys the two points the dialog promised"
    );
    assert!(
        app.sim()
            .state()
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("answered")),
        "the chronicle records the answer"
    );

    // The world resumes.
    let now = app.sim().effective_now();
    app.advance(Duration::from_days(1));
    assert!(
        app.sim().effective_now() > now,
        "the clock runs again once the question is answered"
    );
}

#[test]
fn the_second_answer_is_the_quiet_one_and_it_costs_nothing() {
    let mut app = app();
    until_it_asks(&mut app);
    let before = app.sim().state().political.approval.percent();

    app.on_key(press(KeyCode::Down));
    app.on_key(press(KeyCode::Enter));
    assert!(
        app.decision().is_none(),
        "the question is answered and gone"
    );
    assert_eq!(
        app.sim().state().political.approval.percent() - before,
        0,
        "fixing it quietly changes nothing but the record"
    );
}

#[test]
fn nothing_opens_when_nothing_waits() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('d')));
    assert!(
        app.decision().is_none(),
        "`D` does nothing when there is no question"
    );
}
