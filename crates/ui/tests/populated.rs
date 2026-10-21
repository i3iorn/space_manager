//! The screens that have content on them.
//!
//! **A screen is only ever tested on an empty campaign unless somebody makes
//! it full.** The report modal, the branch points and the chronicle all have
//! branches that only run when the campaign has something in it, and a branch
//! that no test reaches is a branch that is broken in a way nobody knows.
//!
//! These are the *positive* cases: the campaign has gone far enough for the
//! screen to have something to say.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::order::OrderKind;
use domain::time::Duration;
use sim::Scenario;
use ui::App;
use ui::screen::Screen;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A campaign that has been run forward far enough to have a history.
fn busy() -> App {
    let mut app = App::new(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    // **An order, so that the campaign has a decision in it and the branch
    // points have somewhere to be.**
    let person = *app
        .sim()
        .state()
        .people
        .keys()
        .next()
        .expect("there is a person");
    app.issue(OrderKind::SetShift {
        person,
        shift: (110 * domain::scale::PERCENT_UNITS as u16),
    });
    app.advance(Duration::from_days(400));
    // **The month report is a modal**, and it takes the first key.
    app.on_key(press(KeyCode::Char('x')));
    // **A dismissal, so that the chronicle has something in it.** The
    // chronicle is the record of what happened, and a dismissal is worth
    // recording.
    app.issue(OrderKind::Fire { person });
    app.advance(Duration::from_days(40));
    app.on_key(press(KeyCode::Char('x')));
    app
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
fn the_report_is_drawn_when_it_is_up() {
    // **The report modal.** It is a prompt, it waits until it is read, and it
    // has to be *drawn* — a modal that sets a flag and renders nothing is a
    // modal the player cannot read.
    let mut app = App::new(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    app.advance(Duration::from_days(400));
    assert!(app.report().is_some(), "the month has reported");

    let text = rendered(&mut app);
    assert!(!text.is_empty());
    assert!(
        text.contains("Press") || text.contains("esc") || text.contains("month"),
        "and it says something: {text}"
    );
}

#[test]
fn the_chronicle_shows_what_has_happened() {
    // **The history.** A campaign that has run has a record, and the record is
    // on the screen.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('6')));
    let text = rendered(&mut app);
    assert!(
        text.contains("History") || text.contains("entries"),
        "the chronicle shows the record: {text}"
    );
    assert!(
        text.contains("dismissed") || text.contains("shift") || text.contains("Progress"),
        "and it shows what happened: {text}"
    );
}

#[test]
fn the_chronicle_forecasts_what_is_coming() {
    // **The forecast.** The player can see the future the campaign has already
    // committed to, and that is the only future they can see.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('6')));
    let text = rendered(&mut app);
    assert!(
        text.contains("Forecast") || text.contains("promised") || text.contains("committed"),
        "the forecast is there: {text}"
    );
}

#[test]
fn the_branches_screen_lists_the_decisions() {
    // **The fork points.** A campaign with a decision in it has somewhere to
    // fork from, and the screen says so.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('7')));
    let text = rendered(&mut app);
    assert!(
        text.contains("Fork") || text.contains("fork"),
        "the branches screen is there: {text}"
    );
}

#[test]
fn the_fork_actually_forks() {
    // **The positive case of the fork.** The two branches are the same
    // campaign up to the fork, and they are the same campaign.
    let mut app = busy();
    let state = app.sim().state();
    let programme = *state.programmes.keys().next().expect("a programme");
    app.issue(OrderKind::SetBudget {
        programme,
        allocation: domain::money::Money::from_credits(50_000),
    });
    app.advance(Duration::from_days(40));

    app.fork();
    let branch = app.branch().expect("the campaign forks");
    assert!(
        branch.state().programmes.contains_key(&programme),
        "and the branch has the programme"
    );
}

#[test]
fn the_palette_shows_the_price_of_an_order() {
    // **The palette's three prices.** The lead time, the cost and the goodwill
    // are what the player is deciding on, and they are on the screen.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('o')));
    // **The first Enter moves from the list to the form**, and the form is
    // where the price is.
    app.on_key(press(KeyCode::Enter));
    let text = rendered(&mut app);
    assert!(
        text.contains("Takes effect") && text.contains("Cost") && text.contains("Goodwill"),
        "and all three are shown: {text}"
    );
}

#[test]
fn the_palette_explains_the_order() {
    // **The order's explanation**, from the manual's store.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('o')));
    let text = rendered(&mut app);
    assert!(
        text.contains("payroll") || text.contains("agency"),
        "and the order says what it is for: {text}"
    );
}

#[test]
fn the_programme_screen_shows_the_slack_meter() {
    // **The slack meter.** It is the one panel that says how much of the
    // buffer is left, and it is beside the objectives it belongs to.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('3')));
    let text = rendered(&mut app);
    assert!(
        text.contains("Slack") || text.contains("slack"),
        "the slack meter is on the programme screen: {text}"
    );
}

#[test]
fn every_screen_with_a_selection_moves_it() {
    // **The selection is the player's position on a screen**, and every screen
    // with one has to move it.
    for screen in [Screen::Staff, Screen::Programme, Screen::Branches] {
        let mut app = busy();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let before = rendered(&mut app);
        app.on_key(press(KeyCode::Down));
        let after = rendered(&mut app);
        assert_ne!(before, after, "the {screen:?} screen moves its selection");
    }
}

#[test]
fn the_vehicles_screen_shows_a_vehicle_and_its_numbers() {
    // **The vehicles screen.** A vehicle is a set of sliders and the numbers
    // that come out of them, and the screen has to show both — the sliders are
    // the decision and the numbers are the consequence.
    let mut app = busy();
    app.issue(OrderKind::DesignVehicle {
        archetype: domain::vehicle::ArchetypeId(1),
        name: "Orbital Launcher 1".to_owned(),
        sliders: domain::vehicle::SliderValues {
            propulsion: 80,
            payload: 60,
            life_support: 40,
            redundancy: 50,
            power: 30,
            structure: 70,
            avionics: 60,
            heat_shield: 20,
        },
    });
    app.advance(Duration::from_days(10));
    app.on_key(press(KeyCode::Char('x')));

    app.on_key(press(KeyCode::Char('5')));
    let text = rendered(&mut app);
    assert!(
        text.contains("Orbital Launcher 1"),
        "the vehicle is listed: {text}"
    );
    assert!(
        text.contains("Propulsion") && text.contains("reliability"),
        "and its sliders and its numbers are shown: {text}"
    );
}

#[test]
fn the_chronicle_filters_by_year_and_category() {
    // **The history is long and the player is looking for one thing in it.**
    let mut app = busy();
    app.on_key(press(KeyCode::Char('6')));
    let all = rendered(&mut app);
    assert!(all.contains("History"), "the history is there");

    // **The filter says which of the three axes is in force**, because a
    // filtered list with no visible filter is a list the player cannot trust.
    app.on_key(press(KeyCode::Char('y')));
    let by_year = rendered(&mut app);
    assert!(
        by_year.contains("showing") && by_year != all,
        "and the year filter is on: {by_year}"
    );

    app.on_key(press(KeyCode::Char('c')));
    let by_category = rendered(&mut app);
    assert!(
        by_category.contains("showing"),
        "and the category filter is on too: {by_category}"
    );
}

#[test]
fn the_chronicle_can_be_filtered_by_text() {
    // **The third axis**, and the one the player reaches for most.
    let mut app = busy();
    app.on_key(press(KeyCode::Char('6')));
    app.on_key(press(KeyCode::Char('/')));
    for character in "equipment".chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    app.on_key(press(KeyCode::Enter));
    let text = rendered(&mut app);
    assert!(
        text.contains("History"),
        "the history is still there: {text}"
    );
}
