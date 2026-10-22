//! The advice banner: what the game tells the player to do next.
//!
//! **The advice is a situation and not a script.** These tests are about the
//! two things that make it worth having: it says the right thing about the
//! campaign the player is actually in, and it never says a thing the player
//! cannot act on.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app(scenario: &Scenario) -> App {
    App::started(sim::Sim::new(scenario).expect("the campaign starts"))
}

/// The whole interface, as text.
fn rendered(app: &mut App) -> String {
    let backend = TestBackend::new(120, 40);
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
fn the_dashboard_has_a_banner() {
    // **The banner is the point of the feature**, and it is on the home
    // screen, where the player is every time they come back.
    let mut app = app(&Scenario::first_orbit());
    let text = rendered(&mut app);
    assert!(text.contains("What now"), "the banner is on the dashboard");
}

#[test]
fn the_banner_says_the_agency_needs_a_rocket() {
    // **With the research done, the blocker is the rocket.** The player has
    // to design a vehicle, and then build it, and then fly it.
    let mut scenario = Scenario::first_orbit();
    scenario
        .start_state
        .unlocked
        .insert(domain::research::CapabilityId(2));
    let app = app(&scenario);
    let advice = app.advice().expect("there is advice");
    assert!(
        advice.contains("rocket"),
        "and it says what is missing: {advice}"
    );
    assert!(
        advice.contains("Press 5"),
        "and it says where to go: {advice}"
    );
}

#[test]
fn the_banner_says_the_agency_needs_the_research() {
    // **The real blocker, and nothing on the screen says so.** The launch
    // screen's list of destinations does not have "Low orbit" in it and the
    // propulsion slider is pinned at zero, so a player who cannot tell that
    // the answer is research will spend the first hour of the game guessing.
    let app = app(&Scenario::first_orbit());
    let advice = app.advice().expect("there is advice");
    assert!(
        advice.contains("Orbital propulsion"),
        "and it says what the agency has to develop: {advice}"
    );
    assert!(
        advice.contains("points of research short"),
        "and how far away it is: {advice}"
    );
    assert!(
        advice.contains("Combustion stability"),
        "and it names the work that does it: {advice}"
    );
}

#[test]
fn the_advice_never_names_an_identifier() {
    // **A player can act on a name and cannot act on a number.** The message
    // test in the simulation says the same thing about its messages, and the
    // advice is a message the player reads.
    let app = app(&Scenario::first_orbit());
    for index in 0..app.advice_count() {
        let advice = app.advice_at(index).expect("there is advice");
        assert!(
            !advice.contains("Id(") && !advice.contains("PersonId"),
            "the advice names no identifier: {advice}"
        );
        assert!(
            !advice.contains("the science-first authority"),
            "and it is about what to do, not about the agency's slogan: {advice}"
        );
    }
}

#[test]
fn the_advice_changes_with_the_campaign() {
    // **The advice is read from the campaign and is never stored.** A player
    // who has given somebody work must not still be told they have none.
    let mut app = app(&Scenario::first_orbit());
    let idle = app
        .sim()
        .state()
        .people
        .values()
        .find(|person| !person.is_assigned())
        .expect("somebody has nothing to do");
    let name = idle.name.clone();
    let idle_id = idle.id;
    let mentions = |app: &App| {
        (0..app.advice_count())
            .filter_map(|index| app.advice_at(index))
            .any(|advice| advice.contains(&name))
    };
    assert!(mentions(&app), "the advice names them: {name}");

    // Put them on the propulsion project.
    let project = app
        .sim()
        .state()
        .projects
        .keys()
        .copied()
        .next()
        .expect("a project");
    app.sim_mut()
        .issue(domain::order::OrderKind::Assign {
            person: idle_id,
            to: domain::person::Assignment::Project { project },
        })
        .expect("the assignment is accepted");
    app.advance(domain::time::Duration::from_days(30));

    assert!(
        !mentions(&app),
        "and the advice stopped telling the player to do it"
    );
}

#[test]
fn the_player_can_ask_for_another_tip() {
    // **One tip is not advice, it is a lecture.** The player has to be able
    // to see the others.
    let mut app = app(&Scenario::first_orbit());
    let first = app.advice().expect("there is advice");
    app.on_key(press(KeyCode::Char('n')));
    let second = app.advice().expect("there is advice");
    assert_ne!(first, second, "the banner moved on");
}

#[test]
fn the_tip_cycle_wraps() {
    // **A player who keeps asking must not run out.**
    let mut app = app(&Scenario::first_orbit());
    let count = app.advice_count();
    assert!(count > 1, "there is more than one piece of advice");
    let first = app.advice().expect("there is advice");
    for _ in 0..count {
        app.on_key(press(KeyCode::Char('n')));
    }
    assert_eq!(app.advice().expect("there is advice"), first);
}

#[test]
fn the_banner_never_blocks_the_screen() {
    // **The banner is a panel and not a modal.** The player has to be able to
    // reach the screens underneath it.
    let mut app = app(&Scenario::first_orbit());
    app.on_key(press(KeyCode::Char('2')));
    assert_eq!(app.screen(), ui::screen::Screen::Staff);
}

#[test]
fn the_programme_says_how_the_objective_is_done() {
    // **"Reach orbit" is not a clue.** A player who is told the agency has
    // to reach orbit, and not told that a launch is what reaches orbit, has
    // to guess — and the guessing is the whole of the first hour.
    let mut app = app(&Scenario::first_orbit());
    app.on_key(press(KeyCode::Char('3')));
    let text = rendered(&mut app);
    assert!(
        text.contains("done by"),
        "the objective says what finishes it:\n{text}"
    );
    assert!(
        text.contains("a successful launch to Low orbit"),
        "and it names the launch:\n{text}"
    );
}
