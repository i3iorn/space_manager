//! Recruitment: generating the people the agency hires.
//!
//! Everything here is **deterministic**. A hired person's name and abilities
//! come from the campaign's random source, so the same seed and the same
//! decisions always produce the same recruits. There is no `thread_rng` and no
//! system randomness anywhere in this module.

use domain::ids::PersonId;
use domain::money::Money;
use domain::person::{Attributes, Origin, Person, Role, Specialism};
use rand_chacha::ChaCha12Rng;

use crate::rng::draw_below;

/// The specialisms a recruit can come with.
const SPECIALISMS: [Specialism; 10] = Specialism::ALL;

/// A homeworld a recruit comes from.
const HOMEWORLDS: [&str; 8] = [
    "the Core",
    "Tarsis",
    "the Low Countries",
    "Oro",
    "Vesta",
    "the Winter Line",
    "Halcyon",
    "the Belt",
];

/// Generates a recruit.
///
/// The recruit's ability is drawn around the role's normal band, and the
/// specialisms follow the role: an engineer comes with engineering
/// specialisms, and a scientist with scientific ones.
///
/// **The name is drawn from one culture** and never carries a family name
/// anybody in the agency already has, so a campaign of three hundred people
/// has three hundred people the player can tell apart.
///
/// The family names already in use are passed in, because the generator cannot
/// see the campaign and the campaign cannot be in two places at once.
#[must_use]
pub fn recruit(
    rng: &mut ChaCha12Rng,
    id: PersonId,
    role: Role,
    salary: Money,
    taken: &[String],
) -> Person {
    let (name, _culture) = crate::names::name(rng, taken);
    let homeworld = HOMEWORLDS[draw_below(rng, HOMEWORLDS.len() as u32) as usize];

    let mut person = Person::new(id, name, role, salary);
    person.origin = Origin {
        homeworld: homeworld.to_owned(),
        species: "Human".to_owned(),
    };
    person.age = 24 + draw_below(rng, 18);
    person.attributes = attributes(rng, role);
    person.specialisms = specialisms(rng, role);
    person.traits = traits(rng);
    person
}

/// The traits a new recruit comes with.
///
/// **Two or three of them, and never the same one twice.** A recruit is a
/// person, and a person is not made entirely of numbers.
pub fn traits(rng: &mut ChaCha12Rng) -> Vec<domain::person::TraitId> {
    let catalogue = &domain::person::TRAITS;
    if catalogue.is_empty() {
        return Vec::new();
    }
    let count = 2 + usize::try_from(draw_below(rng, 2)).unwrap_or(0);
    let mut chosen: Vec<domain::person::TraitId> = Vec::new();
    let mut attempts = 0_u32;
    while chosen.len() < count && attempts < 64 {
        attempts += 1;
        let candidate = catalogue[draw_below(rng, catalogue.len() as u32) as usize].id;
        if !chosen.contains(&candidate) {
            chosen.push(candidate);
        }
    }
    chosen
}

/// A recruit's abilities, drawn around the role's band.
///
/// A competent recruit lands at 35-75 in their primary attribute, and 20-60 in
/// the rest. Stars are rare, and they are not generated — they arrive as
/// applicants the player must notice.
fn attributes(rng: &mut ChaCha12Rng, role: Role) -> Attributes {
    let primary = 35 + u8::try_from(draw_below(rng, 41)).unwrap_or(50);
    let secondary = 20 + u8::try_from(draw_below(rng, 41)).unwrap_or(40);
    let mut attributes = Attributes {
        technical: secondary,
        creative: secondary,
        leadership: secondary,
        physical: secondary,
        diplomatic: secondary,
    };

    match role {
        Role::Scientist => attributes.creative = primary,
        Role::Engineer => attributes.technical = primary,
        Role::Astronaut => attributes.physical = primary,
        Role::Administrator => attributes.diplomatic = primary,
        Role::Manager => attributes.leadership = primary,
    }
    attributes
}

/// The specialisms a recruit comes with.
///
/// One is always the role's natural field; the second is a chance of something
/// useful from elsewhere.
fn specialisms(rng: &mut ChaCha12Rng, role: Role) -> Vec<Specialism> {
    let natural: &[Specialism] = match role {
        Role::Scientist => &[
            Specialism::Computing,
            Specialism::Geology,
            Specialism::Medicine,
        ],
        Role::Engineer => &[
            Specialism::Propulsion,
            Specialism::Structures,
            Specialism::Avionics,
            Specialism::Power,
            Specialism::Manufacturing,
        ],
        Role::Astronaut => &[Specialism::LifeSupport, Specialism::Avionics],
        Role::Administrator | Role::Manager => {
            &[Specialism::PublicRelations, Specialism::Computing]
        }
    };

    let mut result = vec![natural[draw_below(rng, natural.len() as u32) as usize]];
    if draw_below(rng, 2) == 0 {
        let other = SPECIALISMS[draw_below(rng, SPECIALISMS.len() as u32) as usize];
        if !result.contains(&other) {
            result.push(other);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn a_recruit_is_the_same_for_the_same_stream() {
        let mut first = Rng::new(42).stream(crate::rng::Domain::Personnel, 0, 0);
        let mut second = Rng::new(42).stream(crate::rng::Domain::Personnel, 0, 0);

        let a = recruit(
            &mut first,
            PersonId(0),
            Role::Scientist,
            Money::from_credits(1_200),
            &[],
        );
        let b = recruit(
            &mut second,
            PersonId(0),
            Role::Scientist,
            Money::from_credits(1_200),
            &[],
        );
        assert_eq!(a, b);
    }

    #[test]
    fn a_recruit_is_competent_but_not_a_star() {
        let mut rng = Rng::new(7).stream(crate::rng::Domain::Personnel, 0, 0);
        for _ in 0..20 {
            let person = recruit(&mut rng, PersonId(0), Role::Engineer, Money::ZERO, &[]);
            assert!(person.attributes.technical >= 35);
            assert!(person.attributes.technical <= 75);
        }
    }

    #[test]
    fn a_recruit_comes_with_a_specialism() {
        let mut rng = Rng::new(9).stream(crate::rng::Domain::Personnel, 0, 0);
        let person = recruit(&mut rng, PersonId(0), Role::Scientist, Money::ZERO, &[]);
        assert!(!person.specialisms.is_empty());
    }
}
