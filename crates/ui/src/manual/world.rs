//! The world the agency works in.
//!
//! **The setting is a procedurally generated alien civilisation**, and the
//! generator is a first-class part of the game rather than fixture data. What
//! is *not* in the world matters as much: there is nobody to negotiate with.
//!
//! The world is also the referee. It does not act against the agency on its
//! own; it reacts to what the agency has done, and every reaction it has
//! traces back to a decision the player made.

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
         arc the agency is trying to climb. It does not provide opponents.\n\n\
         **The seed is the campaign.** Two players with the same seed meet the same \
         people, the same programmes, and the same world, and make different decisions \
         about them. That is what makes the branches screen meaningful: the fork is not \
         a counterfactual, it is a second play of the same world."
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
         world is care.\n\n\
         It also means the other half of the game's tension is safe. There is no first \
         contact to negotiate, no artefact to decipher, and no faction to befriend. Every \
         difficult decision the player faces is a decision about people, money, and \
         time - the ones they are actually good at making."
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
         what kind of agency the player ends up with.\n\n\
         **The world gets busier as the agency gets bigger.** The expected number of \
         events a month rises from about one in the early years to one and a half in the \
         middle and two in the late ones. The player is not fighting a harder world; they \
         are living in a bigger one, and the noise comes from the size of what they have \
         built."
    ),
    topic!(
        "world.arc.time",
        Section::World,
        "How long a campaign takes",
        "Thirty to sixty years of simulated time. At the top of the dilation, an evening.",
        "A campaign is thirty to sixty years long, and the dilation is what makes that \
         playable rather than a lifetime. The base rate is a minute of game time a second, \
         and the top rate is ten thousand minutes a second.\n\n\
         **How long a thirty-year campaign takes to watch:**\n\n\
         | Dilation | Real time |\n|---|---|\n\
         | 60x | about six months |\n\
         | 300x | about five weeks |\n\
         | 600x | about two and a half weeks |\n\
         | 3,000x | about three and a half days |\n\
         | 6,000x | about forty-five hours |\n\
         | 30,000x | about nine hours |\n\
         | 60,000x | about four and a half hours |\n\
         | **300,000x** | **about fifty-five minutes** |\n\
         | **600,000x** | **about half an hour** |\n\n\
         The two fastest rates are the ones a player actually uses; the slower ones are \
         for watching a project land, or for a campaign played across a long evening. The \
         base rate is the slowest there is, and it is there for the first hour of a \
         campaign, when the player wants to watch the world rather than outrun it.\n\n\
         **The dilation does not change the campaign.** A campaign played at the base rate \
         and the same campaign played at the top rate are the same campaign: the same \
         people, the same draws from the same random streams, the same history. The \
         dilation is a pacing control and nothing else, and the game is built to keep that \
         guarantee.\n\n\
         **And stopping does not release a backlog.** The clock consumes the real time it \
         passed whether or not it was paused, so an hour spent away is not an hour of \
         campaign."
    ),
    topic!(
        "world.events",
        Section::World,
        "Events",
        "The world reacts. Bad conditions raise the odds of bad events, and most weeks produce nothing at all.",
        "**Once a week the world looks at the agency and something may happen.** An \
         event is data - a weight, some conditions, some effects, and the words it uses - \
         and the weight is the base weight multiplied by the modifiers that apply.\n\n\
         **Bad conditions raise the odds of bad events**, and good conditions raise the odds \
         of good ones. A laboratory that nobody maintains is more likely to fail; an agency \
         the state approves of is more likely to be praised. The player should be able to \
         look back at a disaster and say *my fault - I was running everyone at eighty percent \
         fatigue and skipped testing*.\n\n\
         **Most weeks produce nothing.** The weights are in basis points and they add up to \
         the chance that a check produces anything at all, so a quiet stretch is a normal \
         outcome and not a sign that the game has stopped.\n\n\
         The pressure is not constant: it is scaled to the era, so the world produces about \
         one event a month in the early years and about two in the late ones. The modifiers \
         decide **which** event the pressure lands on rather than how much of it there is, \
         which is why a fatigued agency gets more equipment failures and not merely more \
         of everything.\n\n\
         An event cannot break the campaign, because an event is a list of deltas and cannot \
         express a mutation that would. Some events cause others: a scientist who leaves \
         takes a team's loyalty with them three months later."
    ),
    topic!(
        "world.contexts",
        Section::World,
        "The eight weighting contexts",
        "The eight things about the agency that make the world pay attention. Every one is the player's own doing.",
        "**Every event the world fires is traceable to one of eight things about the \
         agency.** The rule is the fairness rule: an event that cannot be traced to a \
         context is an event the player had no way to prevent, and an event they could not \
         have prevented is not a consequence, it is noise.\n\n\
         | The condition | What it makes more likely |\n|---|---|\n\
         | The average fatigue of the agency is above 70 | personnel incidents |\n\
         | A facility's condition is below 60 | technical failures at that facility |\n\
         | The treasury has under two months of runway | financial crises, political pressure |\n\
         | The public's opinion is below -30 | public events and hostile media |\n\
         | A programme has no slack left | programme-level events and deadline events |\n\
         | A milestone falls due within the month | deadline and pressure events |\n\
         | A launch carries more than 20% risk | technical failure and investigation |\n\
         | A facility has nobody working in it | technical failure and staff incidents |\n\n\
         **Read down the left-hand column and the game's advice is written out.** Fatigue \
         is the one the player causes most easily and can fix most cheaply, and it is the \
         one the world reads most carefully. The rest follow: keep the buildings staffed, \
         keep a buffer on the commitments, and do not launch something that is more than \
         one chance in five of failing.\n\n\
         The modification is large - a context can triple the weight of the events it \
         governs - and it is why the last month of a badly-run campaign is so much worse \
         than the first. The player is not being unlucky. They are being read."
    ),
];
