//! The names, not the identifiers.
//!
//! **The player must never be shown a number that means something to the
//! engine and nothing to them.** The domain holds the state's shape and the
//! state's content separately, so the domain cannot name a programme — but the
//! interface and the reports can, and they must.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use domain::person::Assignment;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// The whole interface, as text.
fn rendered(app: &mut App) -> String {
    let backend = TestBackend::new(110, 40);
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

/// The app, with a campaign that has a programme and a person in it.
fn campaign() -> App {
    let mut app = App::new(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    app.advance(domain::time::Duration::from_days(400));
    app.on_key(press(KeyCode::Char('x')));
    app
}

#[test]
fn the_staff_screen_names_the_programme() {
    // **The complaint this file exists for.** The staff screen used to say
    // "works on programme 0", and a player reading that has been shown an
    // implementation detail.
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('2')));
    let screen = rendered(&mut app);

    // Whatever the person is on, the text must not be a bare identifier.
    assert!(
        !screen.contains("programme 0"),
        "the screen shows an identifier: {screen}"
    );
    assert!(
        !screen.contains("Project 0") && !screen.contains("Facility 0"),
        "the screen shows an identifier: {screen}"
    );
    assert!(
        !screen.contains("Test campaign 0") && !screen.contains("Division 0"),
        "the screen shows an identifier: {screen}"
    );
}

#[test]
fn the_palette_names_the_target() {
    // **The order palette shows what the order will do**, and a target named
    // "programme 0" tells the player nothing about what they are choosing.
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Char('o')));
    let palette = rendered(&mut app);
    assert!(
        !palette.contains("programme 0"),
        "the palette shows an identifier: {palette}"
    );
}

#[test]
fn the_domain_description_has_no_identifier() {
    // **The domain is the last line of defence.** It does not know the names,
    // so it can only say what kind of thing the person is on — and it must
    // not fall back to the number.
    let description = Assignment::Programme {
        programme: domain::ids::ProgrammeId(7),
        role: domain::person::ProgrammeRole::Lead,
    }
    .description();
    assert_eq!(description, "leads a programme");
    assert!(
        !description.contains('7'),
        "the description has an identifier: {description}"
    );

    let facility = Assignment::Facility {
        facility: domain::ids::FacilityId(3),
    }
    .description();
    assert_eq!(facility, "a facility");
    assert!(!facility.contains('3'), "the description has an identifier");
}

#[test]
fn the_simulation_names_the_assignment() {
    // **The simulation has the state, so it can name the thing.**
    let mut sim = sim::Sim::new(&Scenario::first_year()).expect("the campaign starts");
    sim.advance_to(
        sim.effective_now()
            .saturating_add(domain::time::Duration::from_days(400)),
    )
    .expect("the campaign advances");
    let state = sim.state();

    let programme = *state
        .programmes
        .keys()
        .next()
        .expect("there is a programme");
    let title = state.programmes[&programme].title.clone();
    let description = sim::derived::assignment_description(
        state,
        &Assignment::Programme {
            programme,
            role: domain::person::ProgrammeRole::Member,
        },
    );
    assert_eq!(description, format!("works on the {title} programme"));
    assert!(
        !description.contains(&programme.0.to_string()),
        "the description has an identifier: {description}"
    );
}

#[test]
fn control_c_quits_from_a_modal() {
    // **A player who presses Ctrl+C means "stop".** No modal has the right to
    // swallow it: the report, the manual, the filter, and the order palette
    // all take the keys while they are open, and the one key that means
    // "leave" has to get through all of them.
    let mut app = campaign();
    // The month report is up.
    app.advance(domain::time::Duration::from_days(40));
    app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit(), "Ctrl+C quits the game");
}
