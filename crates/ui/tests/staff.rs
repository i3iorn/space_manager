//! The staff list: the shift, the risk, the filter and the sort.
//!
//! **The staff list is the screen that has to work at three hundred people**,
//! and these tests are about the three things that make it work: the focus
//! must be visible, the list must be findable, and the shift must be a
//! control the player can reach while looking at the person.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::time::Duration;
use sim::Scenario;
use ui::App;
use ui::screen::Screen;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app() -> App {
    App::new(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"))
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
fn the_focus_is_visible() {
    // **The focus must be impossible to miss.** A player who cannot see where
    // the selection is cannot use the selection.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    let text = rendered(&mut app);
    assert!(
        text.contains('▸'),
        "the focused row must carry a marker as well as a colour"
    );
}

#[test]
fn the_focus_moves() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    let before = rendered(&mut app);
    app.on_key(press(KeyCode::Down));
    let after = rendered(&mut app);
    assert_ne!(before, after, "the selection must move");
}

#[test]
fn the_shift_is_changed_by_the_arrow_keys() {
    // **The throttle is a control the player reaches for while they are
    // looking at the person.**
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    assert!(
        app.sim().state().pending_orders.is_empty(),
        "nothing has been ordered yet"
    );
    app.on_key(press(KeyCode::Right));
    let orders = app.sim().state().pending_orders.len();
    assert_eq!(orders, 1, "the shift change is an order in flight");
}

#[test]
fn the_shift_change_stacks() {
    // **Pressing the key twice moves two steps**, because the second step is
    // on the shift that is already in flight and not on the one that landed.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Right));
    app.on_key(press(KeyCode::Right));
    let person = ui::screens::staff::rows(&app)[0].id;
    assert_eq!(
        app.pending_shift(person),
        Some(120 * domain::scale::PERCENT_UNITS as u16),
        "two presses is two steps"
    );
}

#[test]
fn the_shift_lands_and_takes_effect() {
    // **The order lands at the next hour**, and then it is real.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Right));
    let person = ui::screens::staff::rows(&app)[0].id;
    app.advance(Duration::from_hours(2));
    let state = app.sim().state();
    let person = state.people.get(&person).expect("the person exists");
    assert!(
        person.state.shift > (100 * domain::scale::PERCENT_UNITS as u16),
        "the shift has landed"
    );
}

#[test]
fn the_list_is_filtered() {
    // **A player who has to scroll to find one person has a screen that does
    // not do its job.**
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Char('/')));
    for character in "okonjo".chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    app.on_key(press(KeyCode::Enter));
    let text = rendered(&mut app);
    assert!(text.contains("Okonjo"), "the filter finds the person");
    assert!(!text.contains("Tomas Reyes"), "and it hides everybody else");
}

#[test]
fn the_filter_is_a_mode() {
    // **A filter can contain a `q` without the game quitting**, because
    // typing is a mode and not a side effect.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Char('/')));
    app.on_key(press(KeyCode::Char('q')));
    assert!(!app.should_quit(), "the game is still running");
    assert_eq!(app.list_control().filter(), "q", "and the filter has the q");
}

#[test]
fn the_escape_clears_the_filter() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Char('/')));
    app.on_key(press(KeyCode::Char('o')));
    app.on_key(press(KeyCode::Esc));
    assert_eq!(
        app.list_control().filter(),
        "",
        "escape abandons the filter"
    );
    assert!(!app.list_control().is_typing(), "and it stops the typing");
}

#[test]
fn the_list_is_sorted() {
    // **The list sorts by name by default**, and the player can ask for the
    // others.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    let by_name = rendered(&mut app);
    for _ in 0..4 {
        app.on_key(press(KeyCode::Char('s')));
    }
    let by_risk = rendered(&mut app);
    assert_ne!(by_name, by_risk, "the sort changes the list");
    assert!(
        by_risk.contains("sort risk"),
        "and the sort is named on the screen"
    );
}

#[test]
fn the_filter_survives_a_trip_elsewhere() {
    // **A player who has filtered the staff list and looked at a project
    // should find the filter still in force when they come back.**
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    app.on_key(press(KeyCode::Char('/')));
    app.on_key(press(KeyCode::Char('o')));
    app.on_key(press(KeyCode::Enter));
    app.on_key(press(KeyCode::Char('1')));
    app.on_key(press(KeyCode::Char('2')));
    assert_eq!(
        app.list_control().filter(),
        "o",
        "the filter is still there"
    );
}

#[test]
fn the_staff_screen_is_where_the_shift_lives() {
    // **The shift is on the staff screen**, because that is where the person
    // is.
    let mut app = app();
    app.on_key(press(KeyCode::Char('2')));
    assert_eq!(app.screen(), Screen::Staff);
    let text = rendered(&mut app);
    assert!(text.contains("shift"), "and the shift is on it");
    assert!(text.contains("leaves this month"), "and so is the risk");
}
