//! The filters.
//!
//! **A name is not the question a player is asking.** "Who is in trouble" is
//! not a name, and a filter that can only search for names makes the player
//! read three hundred rows to answer it. These tests are about the conditions:
//! `fatigue>80`, `unassigned`, `role:scientist`.
//!
//! **And they are about the lists the filter was not wired to.** The staff
//! list had a filter and the other three had nothing, and a list the player
//! cannot search is a list the player has to read.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use domain::time::Duration;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Types a filter into the current screen.
///
/// **The filter is a mode and it keeps what was in it**, so a second filter
/// starts from an empty one — which is what a player does, and what the
/// second key of the pair is for.
fn filter(app: &mut App, text: &str) {
    app.on_key(press(KeyCode::Char('/')));
    app.on_key(press(KeyCode::Esc));
    app.on_key(press(KeyCode::Char('/')));
    for character in text.chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    app.on_key(press(KeyCode::Enter));
}

/// A campaign with a month of play in it.
fn played() -> App {
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_orbit()).expect("the campaign starts"));
    app.advance(Duration::from_days(40));
    app
}

#[test]
fn the_staff_list_is_filtered_by_a_condition() {
    // **Nobody is a hundred and ten percent tired**, so a filter that says so
    // shows nobody, and one that says "at least zero" shows everybody.
    let mut app = played();
    app.on_key(press(KeyCode::Char('2')));
    let everyone = ui::screens::staff::rows(&app).len();
    assert!(everyone > 0, "the agency has people");

    filter(&mut app, "fatigue>110");
    assert_eq!(
        ui::screens::staff::rows(&app).len(),
        0,
        "nobody is that tired"
    );

    filter(&mut app, "fatigue>=0");
    assert_eq!(
        ui::screens::staff::rows(&app).len(),
        everyone,
        "and everybody is at least that tired"
    );
}

#[test]
fn the_staff_list_is_filtered_by_a_flag() {
    // **"Unassigned" is the question a player asks about a person who is on
    // nothing**, and it is a question a name cannot answer.
    let mut app = played();
    app.on_key(press(KeyCode::Char('2')));
    let everyone = ui::screens::staff::rows(&app).len();
    filter(&mut app, "unassigned");
    let free = ui::screens::staff::rows(&app).len();
    assert!(free <= everyone, "the filter cannot add anybody");
    for row in ui::screens::staff::rows(&app) {
        assert!(row.unassigned, "{} is on nothing", row.name);
    }
}

#[test]
fn the_staff_list_is_filtered_by_a_role() {
    let mut app = played();
    app.on_key(press(KeyCode::Char('2')));
    filter(&mut app, "role:engineer");
    for row in ui::screens::staff::rows(&app) {
        assert_eq!(row.role.label(), "Engineer", "{} is an engineer", row.name);
    }
}

#[test]
fn a_filter_with_two_terms_needs_both() {
    // **"Tired and on nothing"** is a different question from "tired" and from
    // "on nothing", and every term has to hold.
    let mut app = played();
    app.on_key(press(KeyCode::Char('2')));
    filter(&mut app, "fatigue>=0 unassigned");
    for row in ui::screens::staff::rows(&app) {
        assert!(row.unassigned, "{} is on nothing", row.name);
    }
}

#[test]
fn the_research_list_is_filtered() {
    // **The research list had no filter at all**, and an agency with a dozen
    // projects and a dozen test campaigns is a list the player has to read.
    let mut app = played();
    app.on_key(press(KeyCode::Char('4')));
    let everything = ui::screens::research::items(&app).len();
    assert!(everything > 0, "the agency is doing something");

    filter(&mut app, "progress>110");
    assert_eq!(
        ui::screens::research::items(&app).len(),
        0,
        "nothing is that finished"
    );

    filter(&mut app, "team>=1");
    assert_eq!(
        ui::screens::research::items(&app).len(),
        everything,
        "and everything has somebody on it"
    );
}

#[test]
fn the_programme_list_is_filtered() {
    // **The programme list is a list of objectives**, and the player's
    // question about it is "which of them is in trouble".
    let mut app = played();
    app.on_key(press(KeyCode::Char('3')));
    let everything = ui::screens::programme::objective_count(&app);
    assert!(everything > 0, "the agency has objectives");

    filter(&mut app, "progress>110");
    assert_eq!(
        ui::screens::programme::objective_count(&app),
        0,
        "no objective is that finished"
    );

    filter(&mut app, "progress>=0");
    assert_eq!(
        ui::screens::programme::objective_count(&app),
        everything,
        "and every objective is at least that far"
    );
}

#[test]
fn the_branches_list_is_filtered() {
    // **The decisions are the campaign's own history**, and a player who wants
    // to see the hires wants the hires.
    let mut app = played();
    app.issue(domain::order::OrderKind::Hire {
        role: domain::person::Role::Scientist,
        salary: domain::money::Money::from_credits(1_200),
    });
    app.on_key(press(KeyCode::Char('7')));
    let everything = app.decisions().len();
    assert!(everything > 0, "the campaign has decisions");

    filter(&mut app, "kind:nothing");
    assert_eq!(app.decisions().len(), 0, "no decision is of that kind");

    filter(&mut app, "kind:hire");
    assert_eq!(app.decisions().len(), 1, "and one of them is a hire");
}

#[test]
fn the_chronicle_is_filtered_by_a_condition() {
    // **The chronicle's filter was a text search**, and "show me the bad things"
    // is not a text search.
    let mut app = played();
    app.on_key(press(KeyCode::Char('6')));
    let before = app.list_control().filter().to_owned();
    assert!(before.is_empty());

    filter(&mut app, "year:1980");
    // **A year nobody lived in**, so the filter is in force and finds nothing.
    assert_eq!(app.list_control().filter(), "year:1980");
}

#[test]
fn a_filter_survives_a_trip_to_another_screen() {
    // **A player who has filtered the staff list down to the people in
    // trouble, and who goes to look at a project and comes back, should find
    // the filter still in force.**
    let mut app = played();
    app.on_key(press(KeyCode::Char('2')));
    filter(&mut app, "fatigue>=0");
    app.on_key(press(KeyCode::Char('3')));
    app.on_key(press(KeyCode::Char('2')));
    assert_eq!(
        app.list_control().filter(),
        "fatigue>=0",
        "the filter is still there"
    );
}
