//! The world the agency works in.
//!
//! **The setting is a procedurally generated alien civilisation**, and the
//! generator is a first-class part of the game rather than fixture data. What
//! is *not* in the world matters as much: there is nobody to negotiate with.

use crate::manual::{Section, Topic, topic};

/// The topics about the world.
pub const TOPICS: &[Topic] = &[
    topic!(
        "world.civilisation",
        Section::World,
        "The alien civilisation",
        "A procedurally generated world. It is bacterial, and it is nobody's friend.",
        "The world the agency works in is generated, not written. A campaign's seed \
         determines the civilisation, the staff, and the programmes, and the same seed \
         always produces the same world - so a campaign is reproducible, and a branch of \
         it is the same world with a different decision in it.\n\n\
         What the world provides is background pressure: politics, budgets, and the long \
         arc the agency is trying to climb. It does not provide opponents."
    ),
    topic!(
        "world.bacteria",
        Section::World,
        "Bacterial life only",
        "There are no sentient species, no artefacts, and no first contact.",
        "**Everything alive out there is bacterial.** There are no intelligent species, no \
         ruins, no artefacts, and nothing that talks back.\n\n\
         This is a deliberate constraint, and it is the reason planetary protection and \
         sample-return contamination are first-class mechanics rather than flavour. If \
         there is nobody to negotiate with, then the only thing that can go wrong is the \
         thing the agency itself does - and the only relationship the agency has with the \
         world is care."
    ),
    topic!(
        "world.arc",
        Section::World,
        "The long arc",
        "Atmospheric research, then orbitals, then the moon, then planets, then everything else.",
        "The campaign runs thirty to sixty years, and the agency climbs a ladder:\n\n\
         atmospheric research first, then orbital capability, then lunar and planetary \
         exploration, then stations, then resource extraction, then interplanetary \
         transport, then megastructures, and finally interstellar missions.\n\n\
         Each rung needs the one below it. A facility built for orbital work does not \
         make a lunar mission possible, and a lunar mission does not make an interstellar \
         one possible. The ladder is the reason a campaign is long, and the reason the \
         early decisions - the first hire, the first laboratory, the first project - decide \
         what kind of agency the player ends up with."
    ),
    topic!(
        "world.arc.time",
        Section::World,
        "How long a campaign takes",
        "Thirty to sixty years of simulated time. At the top of the dilation, an evening.",
        "A campaign is thirty to sixty years long, and the dilation is what makes that \
         playable rather than a lifetime. The base rate is a minute of game time a second, \
         so a year is about ninety minutes of watching; the top rate is ten thousand \
         minutes a second, so a thirty-year campaign is about half an hour of real time.\n\n\
         **The dilation does not change the campaign.** A campaign played at the base rate \
         and the same campaign played at the top rate are the same campaign: the same \
         people, the same draws from the same random streams, the same history. The \
         dilation is a pacing control and nothing else, and the game is built to keep that \
         guarantee."
    ),
];
