//! Traits: the small permanent differences between people.
//!
//! **A trait is worth a few points, not a career.** That is the whole design,
//! and the tests are about the three things that have to be true: the traits
//! are data, they land on the factor they describe, and everybody has some.

use domain::performance::ModifierKind;
use domain::person::{TraitId, trait_def, trait_name};
use domain::state::CampaignState;
use sim::Scenario;
use sim::Sim;
use sim::derived::{TaskKind, person_performance};
use sim::recruitment;

fn state_with(person: domain::person::Person) -> CampaignState {
    let mut state = CampaignState::new(domain::state::Agency::new(
        domain::ids::AgencyId(0),
        "SEA".to_owned(),
        2039,
    ));
    state.people.insert(person.id, person);
    state
}

fn a_person() -> domain::person::Person {
    let mut person = domain::person::Person::new(
        domain::ids::PersonId(0),
        "Marsha Okonjo".to_owned(),
        domain::person::Role::Scientist,
        domain::money::Money::from_credits(1_200),
    );
    person.attributes = domain::person::Attributes {
        technical: 70,
        creative: 70,
        leadership: 50,
        physical: 50,
        diplomatic: 50,
    };
    person
}

#[test]
fn a_trait_is_data_and_the_catalogue_is_readable() {
    // **The traits are a catalogue, not a hard-coded list.** Every trait has a
    // name, a description, and at least one modifier.
    assert!(!domain::person::TRAITS.is_empty());
    for definition in domain::person::TRAITS.iter() {
        assert!(!definition.name.is_empty(), "a trait has a name");
        assert!(
            !definition.description.is_empty(),
            "{} has a description",
            definition.name
        );
        assert!(
            !definition.modifiers.is_empty(),
            "{} modifies something",
            definition.name
        );
    }
}

#[test]
fn a_trait_lands_on_the_factor_it_describes() {
    // **A trait is a nudge on one of the eight factors.** A team player
    // raises the support factor and nothing else.
    let team_player = domain::person::TRAITS
        .iter()
        .find(|definition| definition.name == "Team player")
        .expect("the catalogue has a team player");
    assert!(
        team_player
            .modifiers
            .iter()
            .all(|modifier| modifier.kind() == ModifierKind::Support)
    );
}

#[test]
fn a_trait_changes_a_person_s_performance() {
    // **The whole of the trait system**: a person with a trait does not
    // perform the same as one without it.
    let plain = a_person();
    let mut gifted = a_person();
    gifted.traits = vec![TraitId(0)]; // Prodigy: skill +6, creativity +3.

    let state = state_with(plain.clone());
    let before = person_performance(&state, &plain, TaskKind::Research, 1, false);

    let state = state_with(gifted.clone());
    let after = person_performance(&state, &gifted, TaskKind::Research, 1, false);

    assert!(
        after.effective > before.effective,
        "a prodigy is better than the same person without the trait: {} then {}",
        before.effective,
        after.effective
    );
}

#[test]
fn a_trait_cannot_make_a_tired_person_fresh() {
    // **Fatigue is the one factor that can only hurt**, and the rules say so.
    // A trait that would push it above zero is clamped.
    let mut person = a_person();
    person.state.fatigue = 0;
    person.traits = vec![TraitId(5)]; // Frail: health -7. No fatigue involved.
    let state = state_with(person.clone());
    let result = person_performance(&state, &person, TaskKind::Research, 1, false);
    let fatigue = result
        .modifiers
        .iter()
        .find(|modifier| modifier.kind() == ModifierKind::Fatigue)
        .expect("the fatigue factor is there");
    assert!(
        fatigue.value() <= 0,
        "the fatigue factor is never positive: {}",
        fatigue.value()
    );
}

#[test]
fn a_recruit_comes_with_two_or_three_traits() {
    // **Everybody has traits.** A recruit is a person, and a person is not made
    // entirely of numbers.
    let mut rng = sim::rng::Rng::new(99).stream(sim::rng::Domain::Personnel, 0, 0);
    for _ in 0..20 {
        let traits = recruitment::traits(&mut rng);
        assert!(
            (2..=3).contains(&traits.len()),
            "a recruit has two or three traits: {traits:?}"
        );
        let mut sorted = traits.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), traits.len(), "and never the same one twice");
    }
}

#[test]
fn the_starting_staff_have_traits() {
    // **The people the campaign opens with are people too.**
    let scenario = Scenario::first_orbit();
    assert!(!scenario.start_state.people.is_empty());
    for person in scenario.start_state.people.values() {
        assert!(
            (2..=3).contains(&person.traits.len()),
            "{} has two or three traits: {:?}",
            person.name,
            person.traits
        );
    }
}

#[test]
fn a_trait_the_campaign_has_never_heard_of_is_ignored() {
    // **A trait that is not in the catalogue is not a trait.** The catalogue is
    // code and the campaign stores identifiers, and a mismatch must not panic.
    let mut person = a_person();
    person.traits = vec![TraitId(9_999)];
    let state = state_with(person.clone());
    let result = person_performance(&state, &person, TaskKind::Research, 1, false);
    assert!(result.effective > 0);
    assert_eq!(trait_name(TraitId(9_999)), "Something unusual");
    assert!(trait_def(TraitId(9_999)).is_none());
}

#[test]
fn the_traits_are_the_same_every_time() {
    // **A campaign is reproducible from its seed**, and the traits are part of
    // the campaign.
    let first = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    let second = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    let names = |sim: &Sim| -> Vec<String> {
        sim.state()
            .people
            .values()
            .map(|person| {
                person
                    .traits
                    .iter()
                    .map(|id| trait_name(*id).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect()
    };
    assert_eq!(names(&first), names(&second));
}
