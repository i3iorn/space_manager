//! Research, testing, and facilities.
//!
//! **The central tension is capability against reliability.** A research project
//! makes the agency able to do things it could not do before. A test campaign
//! makes the things it can do work. They compete for the same scarce staff,
//! and there is never enough of both.

use crate::manual::{Section, Topic, topic};

/// The topics about research and facilities.
pub const TOPICS: &[Topic] = &[
    topic!(
        "research.project",
        Section::Research,
        "A research project",
        "A fixed piece of work, with a fixed target, done by a team of people.",
        "A research project is the agency's unit of scientific work. It has a name, a \
         discipline, a team, and a target of 2,400 points.\n\n\
         Starting a project is an order, and it takes between 7 and 14 days to \
         land. Once it is running, it advances at the rate its team can sustain, and it \
         costs 12,000 credits a head every month plus 6,000 of overhead."
    ),
    topic!(
        "research.points",
        Section::Research,
        "Points and the target",
        "A project needs 2,400 points. Every member of the team contributes their performance, and the points are summed.",
        "**The target never moves.** Every research project needs 2,400 points, whatever \
         the discipline, whatever the size of the team, whatever the state of the world.\n\n\
         Each member of the team contributes their performance as points, and the points are \
         **summed**. So adding a person to a project makes it finish sooner and the target \
         does not move. That is the whole arithmetic, and it is deliberately simple enough \
         to hold in the head while making a staffing decision under time pressure.\n\n\
         The complication is that performance is not the same for everybody. A person's \
         performance is a product of eight factors, and the binding factor - the worst of \
         them - multiplies into everything else. A brilliant addition to a team held back \
         by fatigue adds nothing, because the product is held down by the fatigue no \
         matter who is in the room."
    ),
    topic!(
        "research.test",
        Section::Research,
        "Test campaigns",
        "Proving that what you built works. They cost less than research and compete for the same people.",
        "A test campaign is the reliability side of the work. A research project makes the \
         agency able to do something; a test campaign finds out whether it can actually be \
         done reliably.\n\n\
         A campaign costs 7,000 credits a head - less than a research project - and \
         takes 7 days to start. But it needs a capability to test, so the order is not \
         offered until the agency has developed one.\n\n\
         Testing is where the game bites. A capability that has never been tested is a \
         capability that does not work, and the difference between the two is the \
         difference between a programme and a post-mortem."
    ),
    topic!(
        "research.tension",
        Section::Research,
        "Capability against reliability",
        "The two things the agency needs, and the one pool of people they both draw from.",
        "**This is the central research tension, and it is a staffing tension before it is \
         is a technical one.**\n\n\
         Research and testing want the same people. The most able scientists in the agency \
         are the ones who can do either, and the player has to choose every month which \
         of the two they are on. Putting somebody on a project makes the project finish \
         sooner and makes the capability less tested. Putting them on a campaign does the \
         opposite.\n\n\
         There is no answer that is right. The design is that capability and reliability pull \
         against each other and against a third thing - safety, prestige, exploration, and \
         profit all pull too - so that every choice costs something."
    ),
    topic!(
        "research.facility",
        Section::Research,
        "Facilities",
        "Every facility is built, and its specification is fixed for the rest of the campaign.",
        "**There is no abstract capacity.** A facility is a thing the agency built, on a \
         site, with a method and a scale, and it has a specification that does not change: \
         how much work it can take on at once, how dependable it is, how automated it is, \
         how many people it needs, and how good the output is.\n\n\
         There are four kinds: a laboratory, where research happens; a test stand, where \
         existing technology is tested; a fabrication plant, where hardware is built; and a \
         launch complex, where vehicles leave the ground.\n\n\
         A facility is a long-term decision. The specification is chosen at the moment of \
         the order and it stands for the rest of the campaign, so a launch complex built \
         fast and cheap is a launch complex that is unreliable for thirty years."
    ),
    topic!(
        "research.construction",
        Section::Research,
        "How a facility is built",
        "Method, scale, site, and financing. The choice is made once and it is permanent.",
        "Building a facility is four choices, and each of them is permanent.\n\n\
         The **method** decides how it was built. Fast and cheap is quick and inexpensive and \
         less reliable. Standard is the ordinary way. Robust is slow, expensive, and \
         dependable. Experimental is cheap and quick and may unlock capability nobody else \
         has - at the risk of unlocking nothing at all.\n\n\
         The **scale** decides how big it is: a corner of the agency, an ordinary facility, a \
         large installation, or something the whole agency is built around. Bigger is more \
         capacity and more upkeep.\n\n\
         The **site** decides where it stands and whether the public can see it.\n\n\
         The **financing** decides who paid. The agency's own budget means no strings \
         attached. Private investment is someone else's money, with conditions. A commercial \
         partner is someone with its own claims on the output.\n\n\
         A facility takes months to build, and the choice determines its specification for \
         the rest of the campaign."
    ),
    topic!(
        "research.specification",
        Section::Research,
        "The four properties",
        "Capacity, reliability, automation and quality: what a finished facility actually is.",
        "A finished facility has four properties the player cares about.\n\n\
         **Capacity** is how much work it can take on at once, and it is what a project or a \
         campaign is limited by. **Reliability** is how dependable it is, from 0 to 100, and \
         it is what decides whether a test campaign ends in a result or in an incident.\n\n\
         **Automation** is how much of the work it does without people, and it trades \
         staffing cost against the quality of the output. **Quality** is how good the work \
         it does actually is.\n\n\
         A facility also needs a number of staff to run it, and those staff are salary the \
         agency pays whether or not the facility is being used."
    ),
    topic!(
        "research.protection",
        Section::Research,
        "Planetary protection",
        "The rules for not contaminating what you find, and not being contaminated by it.",
        "Planetary protection is a first-class mechanic, not flavour. Everything the agency \
         finds out there is bacterial, and the rules for going somewhere are as much a part \
         of the job as the science.\n\n\
         A mission that lands has to be able to justify itself under the protection \
         protocols, and the cost of doing so - in time, in money, in the mission profile - \
         is a real cost the player pays. A mission that skips the protocols is cheaper and \
         riskier, and if it goes wrong the cost is not the mission that was lost."
    ),
    topic!(
        "research.contamination",
        Section::Research,
        "Contamination and sample return",
        "What comes back, and what it brings with it.",
        "A sample returning to Earth is the most dangerous ordinary event in the campaign. \
         The sample has to be contained, the containment has to be verified, and the \
         verification takes time and money and people.\n\n\
         Contamination is not a failure state to be avoided by careful play; it is a \
         permanent property of what the agency has done. Once a sample has come back, the \
         agency has a containment problem for the rest of the campaign, and the safe \
         assumption is that everything after it is done under stricter rules than before."
    ),
    topic!(
        "research.condition",
        Section::Research,
        "The condition of a facility",
        "How well it is maintained. A neglected facility wears out, and a worn one produces less.",
        "**A facility is a thing the agency looks after, or it is not.** The upkeep \
         is the money and the staff are the maintenance, and a facility that is short \
         of staff **loses four percent of its condition a month** while a fully staffed \
         one holds what it has.\n\n\
         The condition is what the specification is worth today. A laboratory built to \
         a hundred percent that has been ignored for a year is not a laboratory any \
         more, and **its output falls with it** — the facility's capacity is what it \
         was built to be, and the condition is what it is worth.\n\n\
         This is the difference between the upkeep and the staff: paying for one and \
         not the other is how an agency ends up with expensive sheds."
    ),
    topic!(
        "research.capacity",
        Section::Research,
        "Industrial capacity",
        "How many people can work at once. Exceeding it is a real constraint, and the tools factor says so.",
        "**Every member of staff working at once needs a place to work**, and the \
         facilities are the places. The agency's industrial capacity is the sum of what \
         its facilities can hold, and the demand is how many people are working.\n\n\
         **Going over capacity is a real constraint**, and it shows where it should: \
         in the **tools** factor, which falls with the overload. A team working in a \
         laboratory with no room for them is working with worse tools, and every \
         other factor they have is multiplied down by it.\n\n\
         It is also the reason a bigger laboratory is worth building. A facility's \
         capacity is permanent and it is the ceiling on how much the agency can do at \
         once, and the construction choices — the method, the scale, the site and the \
         financing — decide what it is."
    ),
    topic!(
        "research.mission",
        Section::Research,
        "A mission",
        "Preparation, launch, transit, operations, return, archival. A partial failure is the common case, not the rare one.",
        "**A mission is a chain of scheduled instants, not a monthly step.** The \
         phases have real durations: three weeks of preparation, ninety minutes of \
         launch, four days of transit, a week of operations, four days of return, and \
         a fortnight of archival. The campaign keeps running while it does.\n\n\
         **Partial failure is the common branch.** A mission that goes wrong usually \
         goes partly wrong: the capsule comes back, the data is partial, and the agency \
         learns something and loses something. A total loss is the rare case, and it is \
         rare on purpose — the game would be a punishment otherwise.\n\n\
         The findings go to the research that asked for them, because a mission's \
         destination names a capability and a capability has a discipline. The outcome \
         is drawn at a scheduled instant, on the mission's own stream, so nothing else \
         in the simulation can change it.\n\n\
         The chance of success is the vehicle's reliability, and it is visible before the \
         launch. A mission on a vehicle that is still being built is refused."
    ),
];
