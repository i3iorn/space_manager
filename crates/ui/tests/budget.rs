//! The budget panel.
//!
//! **The player asked for it**, and the reason is simple: the top bar has the
//! balance, the net and the runway, and none of those say *where the money is
//! going*. A management game where the player cannot see the breakdown is a
//! management game where the player is guessing.
//!
//! These tests are about what the panel says and what it must not say.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::order::OrderKind;
use domain::person::Role;
use domain::time::Duration;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A campaign with some money in it and something to spend it on.
fn app() -> App {
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_orbit()).expect("the campaign starts"));
    // **A month**, so that the finance phase has run and the treasury has a
    // month's outgoings to show.
    app.advance(Duration::from_days(40));
    // **The month report is a modal over the dashboard**, and the player reads
    // it before they look at the screen behind it. The tests are looking at
    // the dashboard, so the report is dismissed.
    app.on_key(press(KeyCode::Enter));
    app
}

/// An agency with nobody in it, and so nothing to spend.
fn empty() -> App {
    let mut scenario = Scenario::first_year();
    scenario.start_state.people.clear();
    scenario.start_state.projects.clear();
    scenario.start_state.facilities.clear();
    let mut app = App::started(sim::Sim::new(&scenario).expect("the campaign starts"));
    app.advance(Duration::from_days(40));
    app.on_key(press(KeyCode::Enter));
    app
}

/// The screen, as the player sees it.
fn rendered(app: &mut App) -> String {
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("the terminal starts");
    terminal
        .draw(|frame| app.render(frame))
        .expect("the interface draws");
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>()
}

#[test]
fn the_dashboard_has_a_budget_panel() {
    // **The panel is on the right half of screen one**, and it is there from
    // the first frame.
    let mut app = app();
    let text = rendered(&mut app);
    assert!(text.contains("Budget"), "the panel is there:\n{text}");
    assert!(
        text.contains("Where it goes"),
        "and it says where the money goes"
    );
}

#[test]
fn the_budget_panel_says_where_the_money_goes() {
    // **Every outgoing line, with a share of the total.** The share is the
    // point: a number without a share does not answer the question.
    let mut app = app();
    let text = rendered(&mut app);
    assert!(
        text.contains("Payroll"),
        "the biggest line is the payroll:\n{text}"
    );
    assert!(
        text.contains("Research"),
        "and the research is on it:\n{text}"
    );
    assert!(
        text.contains("Where it comes from"),
        "and the panel says where the money comes from too:\n{text}"
    );
    assert!(
        text.contains('%'),
        "and every line carries its share of the total:\n{text}"
    );
}

#[test]
fn the_budget_panel_shows_the_position() {
    // **The balance, the net and the runway** are the three numbers the player
    // is budgeting against, and they are all on the panel.
    let mut app = app();
    let text = rendered(&mut app);
    assert!(text.contains("balance"), "the balance is on it:\n{text}");
    assert!(text.contains("net"), "and the net");
    assert!(text.contains("runway"), "and the runway");
}

#[test]
fn the_budget_panel_shows_what_is_committed() {
    // **The money the agency has already promised** is money that has not left
    // the treasury yet, and a player who cannot see it does not know it.
    let mut app = app();
    app.issue(OrderKind::Hire {
        role: Role::Scientist,
        salary: domain::money::Money::from_credits(1_200),
    });
    let text = rendered(&mut app);
    assert!(
        text.contains("Committed"),
        "the panel says what is committed:\n{text}"
    );
    assert!(
        text.contains("in flight"),
        "and how many orders it is:\n{text}"
    );
}

#[test]
fn the_budget_panel_says_nothing_when_nothing_is_spent() {
    // **An agency with nobody in it spends nothing**, and the panel has to say
    // so rather than show a row of zeroes.
    let mut app = empty();
    let text = rendered(&mut app);
    assert!(text.contains("Budget"), "the panel is there:\n{text}");
    assert!(
        text.contains("Nothing is being spent"),
        "and it says that nothing is being spent:\n{text}"
    );
}
