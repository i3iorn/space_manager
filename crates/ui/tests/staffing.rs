//! Staffing a research project, through the interface.
//!
//! **A project is a team.** The player has to be able to put a second
//! scientist on one, and the screen has to show how many are on it — or the
//! research model is a model the player cannot use.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::order::OrderKind;
use domain::person::Assignment;
use sim::Scenario;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// The whole interface, as text.
fn rendered(app: &mut ui::App) -> String {
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
        .collect()
}

/// A campaign with a project and two people who are free.
fn campaign() -> sim::Sim {
    let sim = sim::Sim::new(&Scenario::first_year()).expect("the campaign starts");
    // The project's own lead is on it; the other two scientists are free.
    let free: Vec<domain::ids::PersonId> = sim
        .state()
        .people
        .values()
        .filter(|person| !person.is_assigned())
        .map(|person| person.id)
        .collect();
    assert!(
        free.len() >= 2,
        "the scenario needs two people who are free"
    );
    sim
}

#[test]
fn the_research_screen_shows_how_many_are_on_a_project() {
    let mut sim = campaign();
    let project = *sim
        .state()
        .projects
        .keys()
        .next()
        .expect("there is a project");
    let people: Vec<domain::ids::PersonId> = sim
        .state()
        .people
        .values()
        .filter(|person| !person.is_assigned())
        .map(|person| person.id)
        .collect();
    for person in people.iter().take(2) {
        sim.issue(OrderKind::Assign {
            person: *person,
            to: Assignment::Project { project },
        })
        .expect("the assignment is accepted");
    }
    sim.advance_to(domain::time::Instant::from_days(5))
        .expect("the campaign advances");

    let mut app = ui::App::started(sim);
    app.on_key(press(KeyCode::Char('4')));
    let text = rendered(&mut app);
    assert!(
        text.contains("team 3"),
        "the research screen says how many are on the project:\n{text}"
    );
}

#[test]
fn the_order_palette_can_put_a_second_person_on_a_project() {
    // **The whole path through the interface.** The player opens the palette,
    // chooses the order, picks the person, picks the project, and confirms —
    // and the order that comes out has to be the assignment they made.
    let sim = campaign();
    let project = *sim
        .state()
        .projects
        .keys()
        .next()
        .expect("there is a project");
    let project_title = sim.state().projects[&project].title.clone();
    let free: Vec<domain::ids::PersonId> = sim
        .state()
        .people
        .values()
        .filter(|person| !person.is_assigned())
        .map(|person| person.id)
        .collect();
    let wanted = free[1];

    let mut app = ui::App::started(sim);
    app.on_key(press(KeyCode::Char('o')));
    assert!(app.palette().is_some(), "the palette is open");

    // **The Assign order.** It is the third in the list.
    app.on_key(press(KeyCode::Down));
    app.on_key(press(KeyCode::Down));
    app.on_key(press(KeyCode::Enter));
    assert!(
        app.palette().expect("the palette is open").is_filling(),
        "and the form is up"
    );

    // **The person.** The parameter is selected; the arrows change the value.
    let parameter = app.palette().expect("the palette is open").parameter();
    assert_eq!(parameter, 0, "the person is the first parameter");
    loop {
        let chosen = app
            .palette()
            .expect("the palette is open")
            .parameters()
            .get(parameter)
            .and_then(|p| p.chosen())
            .map(|choice| choice.label(app.sim().state()));
        if let Some(label) = chosen {
            if app.sim().state().people[&wanted].name == label {
                break;
            }
        }
        app.on_key(press(KeyCode::Right));
    }

    // **The target.** Down moves to the next parameter, and the arrows change
    // the value.
    app.on_key(press(KeyCode::Down));
    assert_eq!(
        app.palette().expect("the palette is open").parameter(),
        1,
        "down moves to the next field"
    );
    let wanted_target = format!("the {project_title} project");
    for _ in 0..40 {
        let chosen = app
            .palette()
            .expect("the palette is open")
            .parameters()
            .get(1)
            .and_then(|p| p.chosen())
            .map(|choice| choice.label(app.sim().state()));
        if chosen.as_deref() == Some(wanted_target.as_str()) {
            break;
        }
        app.on_key(press(KeyCode::Right));
    }

    // **And confirm.**
    app.on_key(press(KeyCode::Enter));
    assert!(app.palette().is_none(), "the palette is closed");

    let pending = app.sim().pending_orders();
    assert_eq!(pending.len(), 1, "the order is in flight");
    let kind = pending
        .values()
        .next()
        .expect("there is an order")
        .kind
        .clone();
    assert!(
        matches!(
            &kind,
            OrderKind::Assign { person, to: Assignment::Project { project: other } }
                if *person == wanted && *other == project
        ),
        "and it is the assignment the player made: {kind:?}"
    );
}
