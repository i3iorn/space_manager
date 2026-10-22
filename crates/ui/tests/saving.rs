//! Saving and loading from inside the game.
//!
//! **A player must be able to save without having planned to.** The campaign
//! they have spent an evening on is not saved unless they say so, and they
//! should not have to restart the game to say it.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use domain::time::Duration;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn fresh_app() -> App {
    App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"))
}

/// A file name of the app's own, so two tests do not fight over one file.
fn scratch(name: &str) -> String {
    let path = std::env::temp_dir().join(format!("space_manager_ui_{name}.json"));
    path.to_string_lossy().into_owned()
}

#[test]
fn the_game_saves_itself() {
    // **The player must not have to think about saving.** The game writes the
    // campaign at every month boundary, so the worst a player can lose by
    // closing the terminal is a month.
    let mut app = fresh_app();
    let path = scratch("autosave");
    app.set_autosave_path(path.clone());
    app.advance(Duration::from_months(2));
    let written = std::path::Path::new(&path).exists();
    let _ = std::fs::remove_file(&path);

    assert!(written, "the campaign was written to {path}");
}

#[test]
fn the_automatic_save_is_the_campaign() {
    // **A save that is not the campaign is worse than no save**, and the
    // automatic one is the one the player trusts most, because they never
    // asked for it.
    let mut app = fresh_app();
    let path = scratch("autosave_contents");
    app.set_autosave_path(path.clone());
    app.advance(Duration::from_months(6));
    let expected = app.sim().state_hash();
    let text = std::fs::read_to_string(&path).expect("the file is there");
    let _ = std::fs::remove_file(&path);

    let loaded = sim::Sim::load(&text).expect("the campaign loads");
    assert_eq!(loaded.state_hash(), expected);
}

#[test]
fn the_campaign_can_be_saved_from_inside_the_game() {
    // **The whole point of the key.** A player who never thought about saving
    // presses one key and the campaign is on disk.
    let mut app = fresh_app();
    let path = scratch("save");
    app.set_save_path(path.clone());
    app.on_key(press(KeyCode::F(3)));
    let written = std::path::Path::new(&path).exists();
    let _ = std::fs::remove_file(&path);

    assert!(written, "the campaign was written to {path}");
}

#[test]
fn the_saved_campaign_is_the_campaign() {
    // **A save that is not the campaign is worse than no save**, because the
    // player trusts it.
    let mut app = fresh_app();
    let path = scratch("save_contents");
    app.set_save_path(path.clone());
    app.advance(Duration::from_months(6));
    let expected = app.sim().state_hash();
    app.save();
    let text = std::fs::read_to_string(&path).expect("the file is there");
    let _ = std::fs::remove_file(&path);

    let loaded = sim::Sim::load(&text).expect("the campaign loads");
    assert_eq!(loaded.state_hash(), expected);
}

#[test]
fn the_campaign_can_be_loaded_from_inside_the_game() {
    // **The pair of the save.** A player who saves, quits and comes back
    // picks up where they left off without leaving the terminal.
    let mut app = fresh_app();
    let path = scratch("load");
    app.set_save_path(path.clone());
    app.advance(Duration::from_months(6));
    app.save();
    let expected = app.sim().state_hash();

    // A second campaign, and then the first one loaded back over it.
    let mut second = fresh_app();
    second.set_save_path(path.clone());
    second.on_key(press(KeyCode::F(4)));
    // The load is a modal, because it throws the campaign away.
    second.on_key(press(KeyCode::Right));
    second.on_key(press(KeyCode::Enter));
    let _ = std::fs::remove_file(&path);

    assert_eq!(second.sim().state_hash(), expected);
}

#[test]
fn loading_a_campaign_that_is_not_there_changes_nothing() {
    // **A file that cannot be read is a message and not a crash**, and the
    // campaign the player is playing is still there.
    let mut app = fresh_app();
    app.set_save_path(scratch("missing"));
    let before = app.sim().state_hash();
    app.on_key(press(KeyCode::F(4)));
    app.on_key(press(KeyCode::Right));
    app.on_key(press(KeyCode::Enter));

    assert_eq!(
        app.sim().state_hash(),
        before,
        "the campaign is still there"
    );
    assert!(
        app.notice()
            .is_some_and(|notice| notice.contains("could not be read")),
        "and the player is told why: {:?}",
        app.notice()
    );
}

#[test]
fn the_save_path_defaults_to_a_file_the_player_can_find() {
    // **A save the player cannot find is a save they will not trust**, so
    // there is a default and it is named.
    let app = fresh_app();
    assert_eq!(app.save_path(), "campaign.json");
}
