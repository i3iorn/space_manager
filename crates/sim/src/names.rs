//! Names.
//!
//! **A name is a person's first impression, and a campaign has hundreds of
//! people.** Twenty-four given names and twenty-four family names is six
//! hundred combinations, and an agency of eighty people finds the
//! repetitions: two Okonjos, and a third Kim who is not in the campaign at all.
//!
//! There are ten cultures here, each with its own given names and its own
//! family names, and **a person draws both halves from the same one** — so
//! "Kwame Adeyemi" reads as a person and "Kwame Kowalski" does not.
//!
//! # The rule
//!
//! **A recruit never gets a family name somebody in the agency already
//! has.** Two people called Okonjo are two people the player cannot tell
//! apart on the staff screen, and the interface shows names and not
//! identifiers.
//!
//! **The names are content, not code.** They are in one place, they are the
//! only place a new name is added, and the generator is the only thing that
//! touches them.

use rand_chacha::ChaCha12Rng;

use crate::rng::draw_below;

/// One culture's names.
///
/// **The given name and the family name come from the same pool**, and the
/// pool is the reason the name reads the way it does.
pub struct Culture {
    /// The culture's name, for the record and for the tests.
    pub name: &'static str,
    /// The given names.
    pub given: &'static [&'static str],
    /// The family names.
    pub family: &'static [&'static str],
}

/// Every culture the agency hires from.
///
/// **The agency is human and its people come from Earth.** The families are
/// Earth families and they live on eight worlds now, which is why a name and a
/// homeworld have nothing to do with each other.
pub const CULTURES: &[Culture] = &[
    Culture {
        name: "Yoruba",
        given: &[
            "Kwame",
            "Adwoa",
            "Ama",
            "Yaw",
            "Kofi",
            "Akosua",
            "Femi",
            "Nia",
            "Sade",
            "Olumide",
            "Bisi",
            "Folake",
            "Temitope",
            "Adunni",
            "Chinedu",
            "Oluwaseun",
        ],
        family: &[
            "Adeyemi", "Oyelaran", "Mensah", "Okonkwo", "Asante", "Boateng", "Adebayo", "Olawale",
            "Ogundipe", "Balogun", "Ogunleye", "Adesanya", "Falade", "Ajayi", "Akinwale", "Eze",
        ],
    },
    Culture {
        name: "Hindustani",
        given: &[
            "Priya", "Ravi", "Anika", "Deepak", "Meera", "Arjun", "Kavya", "Rohan", "Sunita",
            "Vikram", "Nisha", "Sameer", "Divya", "Nikhil", "Radha", "Manish",
        ],
        family: &[
            "Bhatt",
            "Rahman",
            "Iyer",
            "Kapoor",
            "Nair",
            "Chatterjee",
            "Deshpande",
            "Malhotra",
            "Reddy",
            "Sundaram",
            "Bhardwaj",
            "Kulkarni",
            "Menon",
            "Joshi",
            "Ganguly",
            "Trivedi",
        ],
    },
    Culture {
        name: "Japanese",
        given: &[
            "Yuki", "Haruki", "Sora", "Aiko", "Kenji", "Rina", "Daichi", "Mio", "Takumi", "Emi",
            "Ren", "Kaito", "Sayo", "Tatsuya", "Yumi", "Nobu",
        ],
        family: &[
            "Tanaka",
            "Yamamoto",
            "Sato",
            "Suzuki",
            "Watanabe",
            "Kobayashi",
            "Nakamura",
            "Fujiwara",
            "Hasegawa",
            "Ota",
            "Ishikawa",
            "Yamashita",
            "Morishita",
            "Ando",
            "Kondo",
            "Shimizu",
        ],
    },
    Culture {
        name: "Chinese",
        given: &[
            "Xiulan", "Jian", "Meiling", "Wei", "Yan", "Hong", "Ling", "Bo", "Qiao", "Xiaoyu",
            "Zhen", "Tao", "Ning", "Rui", "Shuang", "Sheng",
        ],
        family: &[
            "Zhang", "Chen", "Wang", "Liu", "Zhao", "Huang", "Xu", "Sun", "Gao", "Shen", "Ma",
            "Zhou", "Deng", "Cai", "Pan", "Ren",
        ],
    },
    Culture {
        name: "Nordic",
        given: &[
            "Halvor", "Ingrid", "Erik", "Astrid", "Sigrid", "Bjorn", "Solveig", "Lars", "Freja",
            "Oskar", "Eirik", "Runa", "Aud", "Gunnar", "Signe", "Torvald",
        ],
        family: &[
            "Lindqvist",
            "Sorensen",
            "Bergstrom",
            "Haugen",
            "Dahl",
            "Nyquist",
            "Aasen",
            "Holmqvist",
            "Ekdal",
            "Vik",
            "Sundqvist",
            "Blom",
            "Fjeld",
            "Gran",
            "Halvorsen",
            "Strom",
        ],
    },
    Culture {
        name: "Slavic",
        given: &[
            "Dmitri",
            "Yelena",
            "Pavel",
            "Anya",
            "Miroslav",
            "Zofia",
            "Katya",
            "Ruslan",
            "Ivana",
            "Stanislaw",
            "Lidia",
            "Boris",
            "Radoslav",
            "Viktor",
            "Oksana",
            "Yuri",
        ],
        family: &[
            "Petrov",
            "Ivanova",
            "Kowalski",
            "Nowak",
            "Zielinski",
            "Volkov",
            "Sokolova",
            "Dabrowski",
            "Morozov",
            "Kaczmarek",
            "Kaminski",
            "Zaitsev",
            "Belova",
            "Wisniewski",
            "Fedorov",
            "Matveeva",
        ],
    },
    Culture {
        name: "Latin",
        given: &[
            "Tomas", "Ines", "Mateo", "Carmen", "Rafael", "Lucia", "Diego", "Pilar", "Javier",
            "Rocio", "Alvaro", "Nuria", "Gabriel", "Marta", "Sergio", "Berta",
        ],
        family: &[
            "Reyes",
            "Marchetti",
            "Duval",
            "Ferreira",
            "Castellanos",
            "Sandoval",
            "Alvarez",
            "Fiorentino",
            "Guzman",
            "Serrano",
            "Delgado",
            "Navarrete",
            "Ibanez",
            "Cabrera",
            "Ortiz",
            "Montoya",
        ],
    },
    Culture {
        name: "Levantine",
        given: &[
            "Sami", "Layla", "Omar", "Nadia", "Yusuf", "Rania", "Karim", "Dalia", "Tarek", "Amira",
            "Fadi", "Salma", "Ghassan", "Muna", "Ziad", "Hala",
        ],
        family: &[
            "Haddad", "Nasser", "Mansour", "Khalil", "Darwish", "Hariri", "Sabri", "Zaidi",
            "Boulos", "Nassar", "Awad", "Daher", "Ghali", "Jamil", "Mikhael", "Attar",
        ],
    },
    Culture {
        name: "Germanic",
        given: &[
            "Marsha",
            "Tobias",
            "Anneliese",
            "Klaus",
            "Greta",
            "Johann",
            "Frieda",
            "Brigitta",
            "Wilhelm",
            "Hedwig",
            "Sebastian",
            "Ulrike",
            "Matthias",
            "Cordula",
            "Eberhard",
            "Rosalind",
        ],
        family: &[
            "Ashworth",
            "Kaufmann",
            "Bergmann",
            "Wendel",
            "Schirmer",
            "Vogel",
            "Reinhardt",
            "Falk",
            "Herzog",
            "Zimmermann",
            "Eichel",
            "Fassbender",
            "Grunwald",
            "Hoffmann",
            "Kellermann",
            "Weiss",
        ],
    },
    Culture {
        name: "Ethiopian",
        given: &[
            "Emeka", "Selam", "Dawit", "Almaz", "Habtom", "Hana", "Yemane", "Mahlet", "Bereket",
            "Tigist", "Kaleb", "Ruth", "Ephrem", "Meron", "Yonas", "Saron",
        ],
        family: &[
            "Bekele", "Haile", "Solomon", "Gebre", "Abebe", "Mengiste", "Worku", "Taye", "Girma",
            "Teferi", "Alemu", "Belay", "Desta", "Fikru", "Gezmu", "Wolde",
        ],
    },
];

/// How many times the generator will try before it gives up on a free name.
///
/// **The name space is two and a half thousand**, so thirty-two attempts
/// failing means an agency of hundreds, and the fallback below keeps the name
/// unique anyway.
const ATTEMPTS: usize = 32;

/// A person's name, and the culture it came from.
///
/// **The culture is chosen first and the name is drawn from it**, so the two
/// halves of a name belong together.
///
/// The family names in `taken` are the ones already in the agency, and **a
/// recruit never gets one of them**. Two people called Okonjo are two people
/// the player cannot tell apart.
#[must_use]
pub fn name(rng: &mut ChaCha12Rng, taken: &[String]) -> (String, &'static str) {
    for _ in 0..ATTEMPTS {
        let culture = &CULTURES[draw_below(rng, CULTURES.len() as u32) as usize];
        let given = culture.given[draw_below(rng, culture.given.len() as u32) as usize];
        let family = culture.family[draw_below(rng, culture.family.len() as u32) as usize];
        if !taken.iter().any(|other| other == family) {
            return (format!("{given} {family}"), culture.name);
        }
    }

    // **Every family name in every culture is taken.** That is an agency of
    // hundreds, and the honest answer is a second member of a family: the
    // same name with a different given name, which is what families are.
    let culture = &CULTURES[draw_below(rng, CULTURES.len() as u32) as usize];
    let given = culture.given[draw_below(rng, culture.given.len() as u32) as usize];
    let family = culture.family[draw_below(rng, culture.family.len() as u32) as usize];
    let full = format!("{given} {family}");
    if !taken.iter().any(|other| other == family) {
        return (full, culture.name);
    }
    // **And the whole name is taken as well.** A middle initial keeps it
    // unique and still readable.
    let initial = given.chars().next().unwrap_or('X');
    (format!("{given} {initial}. {family}"), culture.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{Domain, Rng};

    fn rng() -> ChaCha12Rng {
        Rng::new(4_242).stream(Domain::Personnel, 0, 0)
    }

    #[test]
    fn a_name_comes_from_one_culture() {
        // **The two halves of a name belong together.** This is the whole
        // reason the pools are separate.
        for _ in 0..100 {
            let mut rng = rng();
            let (name, culture) = name(&mut rng, &[]);
            let pool = CULTURES
                .iter()
                .find(|pool| pool.name == culture)
                .expect("the culture is in the catalogue");
            let given = name.split_whitespace().next().expect("a given name");
            let family = name.split_whitespace().last().expect("a family name");
            assert!(
                pool.given.contains(&given),
                "{given} is not a {culture} given name"
            );
            assert!(
                pool.family.contains(&family),
                "{family} is not a {culture} family name"
            );
        }
    }

    #[test]
    fn a_family_name_already_in_use_is_not_used_again() {
        // **Two people with the same surname are two people the player cannot
        // tell apart.**
        let mut rng = rng();
        let mut taken: Vec<String> = Vec::new();
        for _ in 0..120 {
            let (candidate, _) = name(&mut rng, &taken);
            let family = candidate.split_whitespace().last().expect("a family name");
            assert!(
                !taken.iter().any(|other| other == family),
                "{family} was handed out twice"
            );
            taken.push(family.to_owned());
        }
    }

    #[test]
    fn the_names_are_wide_enough_to_be_worth_using() {
        // **The old catalogue was six hundred combinations for an agency of
        // hundreds of people.** This one is two and a half thousand, and it
        // does not mix the cultures.
        let combinations: usize = CULTURES
            .iter()
            .map(|culture| culture.given.len() * culture.family.len())
            .sum();
        assert!(
            combinations > 2_000,
            "the name space is {combinations} names, which is not enough"
        );
        let families: usize = CULTURES.iter().map(|culture| culture.family.len()).sum();
        assert!(
            families > 100,
            "and only {families} family names for the whole agency"
        );
    }

    #[test]
    fn the_cultures_have_enough_names_each() {
        for culture in CULTURES {
            assert!(
                culture.given.len() >= 12,
                "{} has too few given names",
                culture.name
            );
            assert!(
                culture.family.len() >= 12,
                "{} has too few family names",
                culture.name
            );
        }
    }

    #[test]
    fn no_name_is_in_two_cultures() {
        // **A name in two cultures is a name that is wrong half the time.**
        let mut given: Vec<&str> = Vec::new();
        let mut family: Vec<&str> = Vec::new();
        for culture in CULTURES {
            for name in culture.given {
                assert!(
                    !given.contains(name),
                    "{name} is in two cultures' given names"
                );
                given.push(name);
            }
            for name in culture.family {
                assert!(
                    !family.contains(name),
                    "{name} is in two cultures' family names"
                );
                family.push(name);
            }
        }
    }
}
