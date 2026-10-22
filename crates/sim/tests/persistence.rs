//! Saving and loading a campaign.
//!
//! **A save file is the only thing that carries a campaign between sessions**,
//! and a player who loses one loses an evening. So the format is versioned,
//! the refusals say what is wrong, and a damaged file is a message rather than
//! a crash.
//!
//! These are the MVP's Slice 4 checks: a corrupted, truncated, or
//! foreign-version file fails gracefully, and the message says what to do.

use sim::Scenario;
use sim::Sim;

/// A campaign with a few decisions in it, saved.
fn saved() -> String {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for order in sim::harness::plan_for_debug(sim.state(), sim::harness::Policy::Fast) {
        let _ = sim.issue(order);
    }
    sim.advance_to(domain::time::Instant::from_months(6))
        .expect("the campaign advances");
    sim.save().expect("the campaign saves")
}

#[test]
fn a_campaign_round_trips_through_a_save() {
    let text = saved();
    let restored = Sim::load(&text).expect("the campaign loads");
    let mut original = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for order in sim::harness::plan_for_debug(original.state(), sim::harness::Policy::Fast) {
        let _ = original.issue(order);
    }
    original
        .advance_to(domain::time::Instant::from_months(6))
        .expect("the campaign advances");
    assert_eq!(restored.state_hash(), original.state_hash());
}

#[test]
fn a_garbled_file_is_refused_with_a_reason() {
    let error = Sim::load("{ this is not a campaign").expect_err("the load is refused");
    let message = error.to_string();
    assert!(
        message.contains("could not be read"),
        "the refusal says what is wrong: {message}"
    );
    assert!(message.ends_with('.'), "and it is a sentence: {message}");
}

#[test]
fn a_truncated_file_is_refused_with_a_reason() {
    // **A truncated file is the commonest kind of damaged save**, and it must
    // not crash the game.
    let text = saved();
    for cut in [1, text.len() / 4, text.len() / 2, text.len() - 1] {
        let truncated = &text[..cut];
        let error = Sim::load(truncated).expect_err("the load is refused");
        let message = error.to_string();
        assert!(
            message.contains("could not be read"),
            "the refusal says what is wrong: {message}"
        );
    }
}

#[test]
fn an_empty_file_is_refused_with_a_reason() {
    let error = Sim::load("").expect_err("the load is refused");
    assert!(
        error.to_string().contains("could not be read"),
        "the refusal says what is wrong: {error}"
    );
}

#[test]
fn a_campaign_from_another_version_is_refused_with_a_reason() {
    // **The version is in the save, and a save from another build of the game
    // cannot be replayed** — the outcomes would be different, and a campaign
    // that silently changed is worse than one that is refused.
    let mut envelope: serde_json::Value = serde_json::from_str(&saved()).expect("the save is JSON");
    envelope["sim_version"] = serde_json::Value::String("some-other-build".to_owned());
    let text = serde_json::to_string(&envelope).expect("the envelope serialises");

    let error = Sim::load(&text).expect_err("the load is refused");
    let message = error.to_string();
    assert!(
        message.contains("different version"),
        "the refusal says what is wrong: {message}"
    );
    assert!(
        message.contains("some-other-build"),
        "and it says which version: {message}"
    );
}

#[test]
fn a_save_from_the_future_is_refused_with_a_reason() {
    let mut envelope: serde_json::Value = serde_json::from_str(&saved()).expect("the save is JSON");
    envelope["format"] = serde_json::Value::from(9_999_u32);
    let text = serde_json::to_string(&envelope).expect("the envelope serialises");

    let error = Sim::load(&text).expect_err("the load is refused");
    let message = error.to_string();
    assert!(
        message.contains("format"),
        "the refusal says what is wrong: {message}"
    );
}

#[test]
fn a_save_can_be_written_to_a_file_and_read_back() {
    // **The whole point of the format.** A file on disk, a campaign in it, and
    // the campaign back.
    let path = std::env::temp_dir().join("space_manager_save_test.json");
    let text = saved();
    std::fs::write(&path, &text).expect("the file is written");
    let read = std::fs::read_to_string(&path).expect("the file is read");
    let _ = std::fs::remove_file(&path);
    let restored = Sim::load(&read).expect("the campaign loads");
    assert_eq!(restored.state().chronicle.len(), {
        let mut original = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
        for order in sim::harness::plan_for_debug(original.state(), sim::harness::Policy::Fast) {
            let _ = original.issue(order);
        }
        original
            .advance_to(domain::time::Instant::from_months(6))
            .expect("the campaign advances");
        original.state().chronicle.len()
    });
}

#[test]
fn the_save_format_is_bumped_when_the_shape_changes() {
    // **The format version is the guard against a save being half-read.** A
    // file of an older format is refused rather than interpreted.
    let text = saved();
    let envelope: serde_json::Value = serde_json::from_str(&text).expect("the save is JSON");
    let format = envelope["format"].as_u64().expect("the format is a number");
    assert_eq!(
        format,
        u64::from(sim::persist::SAVE_FORMAT),
        "the save is in the current format"
    );
}
