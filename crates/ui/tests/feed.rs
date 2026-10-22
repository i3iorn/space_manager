//! The live feed and the legend.
//!
//! **A feed is not a list.** The difference is the order and the freshness:
//! the newest thing is at the top, and it appears the moment it happens,
//! without the player navigating anywhere. A feed the player has to go and
//! look at is a list, and the roadmap has been complaining about that since
//! Slice 1.

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

/// A campaign that has been running for a while.
fn app() -> App {
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_orbit()).expect("the campaign starts"));
    app.advance(Duration::from_days(200));
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
fn the_dashboard_has_a_live_feed() {
    // **The feed is on the screen the player is on**, because a feed the
    // player has to navigate to is a list.
    let mut app = app();
    let text = rendered(&mut app);
    assert!(text.contains("Just happened"), "the feed is there:\n{text}");
}

#[test]
fn the_feed_shows_what_has_happened() {
    // **The world reacts, and the player sees it.** A campaign that has run
    // for two hundred days has something to say.
    let mut app = app();
    let text = rendered(&mut app);
    let entries = app.sim().state().chronicle.len();
    assert!(entries > 0, "the campaign has a record to show");
    let mut shown = 0;
    for entry in app.sim().state().chronicle.iter().rev() {
        if text.contains(&entry.headline) {
            shown += 1;
        }
    }
    assert!(shown > 0, "and the feed shows the record:\n{text}");
}

#[test]
fn the_feed_shows_what_the_player_decided() {
    // **A campaign is the world and what the player did about it.** The
    // player's own decisions are in the feed.
    let mut app = app();
    app.issue(OrderKind::Hire {
        role: Role::Scientist,
        salary: domain::money::Money::from_credits(1_200),
    });
    let text = rendered(&mut app);
    assert!(
        text.contains("Hire"),
        "the feed carries the player's decisions:\n{text}"
    );
}

#[test]
fn the_chronicle_has_a_legend() {
    // **The legend is the key to the record.** A player who does not know
    // what "Financial" means is reading a record they cannot read.
    let mut app = app();
    app.on_key(press(KeyCode::Char('6')));
    let text = rendered(&mut app);
    assert!(text.contains("Legend"), "the legend is there:\n{text}");
    for category in domain::chronicle::Category::ALL {
        assert!(
            text.contains(category.label()),
            "and it names the {} category:\n{text}",
            category.label()
        );
    }
}

#[test]
fn the_feed_is_newest_first() {
    // **The newest thing is at the top**, or it is a list.
    let mut app = app();
    let chronicle = app.sim().state().chronicle.clone();
    let text = rendered(&mut app);
    let positions: Vec<usize> = chronicle
        .iter()
        .rev()
        .take(3)
        .filter_map(|entry| text.find(&entry.headline))
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] <= pair[1]),
        "the newest entry is the first one on the screen: {positions:?}"
    );
}
