//! The screens show names and dates, never numbers.
//!
//! **A screen that shows `0` where it should show `Okonjo` is telling the
//! player nothing, and a screen that shows `1700000000` where it should show a
//! date is worse than nothing.** These are the two failures this file is
//! about, and they are the same failure: an internal identifier reaching the
//! screen.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use sim::Scenario;
use ui::App;
use ui::screen::Screen;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app() -> App {
    let mut app = App::new(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    // **The campaign is run forward a little**, so the screens have something
    // to show: a promise, a deadline, a project, a person.
    app.advance(domain::time::Duration::from_days(400));
    // **The month report is a modal**, and it takes the first key. It is
    // dismissed here so that the screen keys work in the tests below.
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
fn no_screen_shows_a_raw_millisecond_count() {
    // **An instant is a count of milliseconds, and it is not a date.** The
    // interface has to project it onto the civil calendar, and the test is
    // that every screen does.
    for screen in Screen::ALL {
        let mut app = app();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let text = rendered(&mut app);
        assert!(
            !has_a_long_number(&text),
            "the {:?} screen shows a raw number: {:?}",
            screen,
            the_long_number(&text)
        );
    }
}

/// Whether the text holds a run of nine or more digits.
fn has_a_long_number(text: &str) -> bool {
    the_long_number(text).is_some()
}

/// The first run of nine or more digits, if there is one.
fn the_long_number(text: &str) -> Option<String> {
    let mut run = String::new();
    for character in text.chars() {
        if character.is_ascii_digit() {
            run.push(character);
        } else {
            if run.len() >= 9 {
                return Some(run);
            }
            run.clear();
        }
    }
    if run.len() >= 9 { Some(run) } else { None }
}

#[test]
fn the_programme_screen_names_the_principal_investigator() {
    // **A person is a name.** The project panel used to show the
    // investigator's index, which told the player nothing at all.
    let mut app = app();
    app.on_key(press(KeyCode::Char('3')));
    let text = rendered(&mut app);
    assert!(
        text.contains("PI ") && !text.contains("PI 0"),
        "the investigator must be named, and the screen said: {text}"
    );
}

#[test]
fn the_programme_screen_shows_the_dates_as_dates() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('3')));
    let text = rendered(&mut app);
    assert!(
        text.contains("target") && text.contains("COMMITTED"),
        "both instants are shown"
    );
    assert!(!has_a_long_number(&text), "and they are shown as dates");
}
