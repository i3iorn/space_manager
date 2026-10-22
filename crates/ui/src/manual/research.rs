//! Research, testing, and facilities.
//!
//! **The central tension is capability against reliability.** A research project
//! makes the agency able to do things it could not do before. A test campaign
//! makes the things it can do work. They compete for the same scarce staff,
//! and there is never enough of both.
//!
//! The topics here are written so that the player can do the arithmetic in
//! their head: what a thing costs, how long it takes, and what it does to the
//! numbers either side of it.

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
         costs 1,200 credits a head every month plus 3,000 of overhead.\n\n\
         **The cost is on top of the salary.** The people on the project are on the \
         payroll either way; the project budget is what it costs to *do* the work - the \
         equipment, the materials, the running of the facility. A four-person project \
         therefore costs 7,800 a month, on top of what those four people are paid.\n\n\
         **A finished project is worth 120 units of knowledge**, and it has a one in \
         three chance of a breakthrough, which is what unlocks a capability and lets \
         the agency do something it could not do before. Everything else the project \
         does is progress towards the capability, not the capability itself."
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
         | Team | Months at full performance |\n|---|---|\n\
         | 1 | 24 |\n\
         | 2 | 12 |\n\
         | 4 | 6 |\n\
         | 8 | 3 |\n\n\
         The complication is that performance is not the same for everybody. A person's \
         performance is a product of eight factors, and the binding factor - the worst of \
         them - multiplies into everything else. A brilliant addition to a team held back \
         by fatigue adds nothing, because the product is held down by the fatigue no \
         matter who is in the room.\n\n\
         **This is why the project estimate on the screen is worth reading.** It assumes \
         full performance. A team at seventy percent fatigue will take a third longer than \
         the estimate, and a player who believes the estimate is in for a surprise."
    ),
    topic!(
        "research.test",
        Section::Research,
        "Test campaigns",
        "Proving that what you built works. They cost less than research and compete for the same people.",
        "A test campaign is the reliability side of the work. A research project makes the \
         agency able to do something; a test campaign finds out whether it can actually be \
         done reliably.\n\n\
         A campaign costs 600 credits a head - less than a research project - and \
         takes 7 days to start. But it needs a capability to test, so the order is not \
         offered until the agency has developed one.\n\n\
         **The arithmetic is smaller and quicker than a project's.** A campaign needs 100 \
         points, and every member of the team adds 5 a month, so four people finish it \
         in five months for 19,000. A research project of the same four people takes six \
         months and costs 31,200.\n\n\
         **What it buys is fifteen points of risk.** A completed campaign takes fifteen \
         percentage points off the chance of failure of every mission to that \
         destination, and the memory of the testing **outlives the campaign** - the people \
         who did it can leave, and the risk they took out stays taken out. A second \
         campaign on the same capability takes out fifteen more, up to forty.\n\n\
         Testing is where the game bites. A capability that has never been tested is a \
         capability that does not work, and the difference between the two is the \
         difference between a programme and a post-mortem."
    ),
    topic!(
        "research.tension",
        Section::Research,
        "Capability against reliability",
        "The two things the agency needs, and the one pool of people they both draw from.",
        "**This is the central research tension, and it is a staffing tension before it \
         is a technical one.**\n\n\
         Research and testing want the same people. The most able scientists in the agency \
         are the ones who can do either, and the player has to choose every month which \
         of the two they are on. Putting somebody on a project makes the project finish \
         sooner and makes the capability less tested. Putting them on a campaign does the \
         opposite.\n\n\
         The two are not comparable on money either. A person on a project costs 1,200 a \
         month and produces points towards a capability; a person on a campaign costs \
         600 a month and produces five points of certainty. **A capability that has \
         never been tested still does not work**, and the missions that depend on it are \
         the most expensive things the agency does.\n\n\
         There is no answer that is right. The design is that capability and reliability pull \
         against each other and against a third thing - safety, prestige, exploration, and \
         profit all pull too - so that every choice costs something.\n\n\
         The usual way through is not to choose. It is to notice that a test campaign \
         takes five months and a project takes six, and that a person who is good at both \
         is worth more on the thing the agency has already paid for."
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
         existing technology is tested; a fabrication plant, where hardware is built; and \
         a launch complex, where vehicles leave the ground.\n\n\
         | Kind | Cost | Build time | Upkeep a month |\n|---|---|---|---|\n\
         | Laboratory | 350,000 | 15 months | 4,000 |\n\
         | Test stand | 600,000 | 21 months | 6,000 |\n\
         | Fabrication plant | 800,000 | 24 months | 8,000 |\n\
         | Launch complex | 1,200,000 | 30 months | 14,000 |\n\n\
         A facility is a long-term decision. The specification is chosen at the moment of \
         the order and it stands for the rest of the campaign, so a launch complex built \
         fast and cheap is a launch complex that is unreliable for thirty years.\n\n\
         **The upkeep is the smaller half of the bill and the larger half of the regret.** \
         Fourteen thousand a month for thirty years is most of a million credits for a \
         building that never has to be replaced - and a building that is not staffed wears \
         out anyway."
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
         A facility takes months to build - fifteen for a laboratory, thirty for a launch \
         complex - and the choice determines its specification for the rest of the \
         campaign. **Thirty months is two and a half years of the campaign gone before the \
         first thing happens in it**, which is a long time in a game where the player is \
         being judged on what they have done lately."
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
         agency pays whether or not the facility is being used. **Staffing is not a detail \
         of the facility; it is a running cost with a name.** A laboratory that wants four \
         people and has none is a laboratory that is losing 4% of its condition \
         a month and delivering worse tools to everybody who works in it.\n\n\
         The four properties feed three of the eight performance factors - tools, support, \
         and the capacity that limits how many people can work at once - which is why the \
         facility the agency built is still affecting the project it built it for."
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
         riskier, and if it goes wrong the cost is not the mission that was lost.\n\n\
         **The reason is the biology.** Everything out there is bacterial, which means it \
         is small, it is everywhere, and it does not negotiate. The only alien life in this \
         game is life that spreads, and the rules are what stop the agency bringing it \
         home."
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
         assumption is that everything after it is done under stricter rules than before.\n\n\
         **This is the mechanic that makes exploration cost something beyond the money.** \
         A mission that brings nothing back is cheap and dull; a mission that brings \
         something back is expensive, and the expense continues. The player is deciding, \
         every time, how much of the world's risk to take home."
    ),
    topic!(
        "research.condition",
        Section::Research,
        "The condition of a facility",
        "How well it is maintained. A neglected facility wears out, and a worn one produces less.",
        "**A facility is a thing the agency looks after, or it is not.** The upkeep \
         is the money and the staff are the maintenance, and a facility that is short \
         of staff **loses 4% of its condition a month** while a fully staffed \
         one holds what it has.\n\n\
         The condition is what the specification is worth today. A laboratory built to \
         a hundred percent that has been ignored for a year is not a laboratory any \
         more, and **its output falls with it** - the facility's capacity is what it \
         was built to be, and the condition is what it is worth.\n\n\
         This is the difference between the upkeep and the staff: paying for one and \
         not the other is how an agency ends up with expensive sheds. A laboratory that \
         costs four thousand a month in upkeep and has nobody running it is the most \
         expensive kind of useless.\n\n\
         **A worn facility is also a source of events.** The weighting that decides how \
         often things go wrong reads the condition directly, so a neglected building is \
         not merely less productive - it is more likely to fail, and a failure in the \
         world is more expensive than any amount of lost output."
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
         once, and the construction choices - the method, the scale, the site and the \
         financing - decide what it is.\n\n\
         **The trap is growth.** Hiring a hundred people into a laboratory built for \
         twenty does not make the laboratory bigger; it makes the hundred people worse, \
         by a factor that is invisible on the payroll. Capacity is the constraint that \
         makes buildings matter, and buildings are the most permanent thing in the game."
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
         rare on purpose - the game would be a punishment otherwise.\n\n\
         The findings go to the research that asked for them, because a mission's \
         destination names a capability and a capability has a discipline. The outcome \
         is drawn at a scheduled instant, on the mission's own stream, so nothing else \
         in the simulation can change it.\n\n\
         The chance of success is the vehicle's reliability, and it is visible before the \
         launch. The base chance is 95%, so a vehicle of ninety reliability gives eighty-five per \
         cent, and every point of testing takes one more off the risk. A mission on a \
         vehicle that is still being built is refused.\n\n\
         **And two failures in three are partial ones.** The share of the failure that is \
         partial is fixed, and the size of the failure follows from the vehicle: a shaky \
         rocket loses more of itself than a careful one, and a total loss writes the \
         vehicle off.\n\n\
         **A mission is also the fastest way to make the agency famous** - and fame is \
         what brings the good people to apply for the next one."
    ),
    topic!(
        "research.disciplines",
        Section::Research,
        "Disciplines and capabilities",
        "Three fields of science, five capabilities, and what each one unlocks.",
        "**The agency researches in three disciplines**: propulsion, structures and \
         materials, and life support. Each is adjacent to the other two, which is why a \
         narrow portfolio costs something - the knowledge a project produces spills a fifth \
         into the neighbouring fields, but nothing spills into a field the agency does not \
         work in at all.\n\n\
         **Five capabilities come out of that research**, and each is a thing the agency can \
         do afterwards:\n\n\
         | Capability | Field | Knowledge needed | What it unlocks |\n|---|---|---|---|\n\
         | Atmospheric sounding | propulsion | 0 | nothing - the agency always had it |\n\
         | Orbital propulsion | propulsion | 240 | the Low orbit destination, and the propulsion slider |\n\
         | Basic re-entry | structures and materials | 300 | the heat-shield slider |\n\
         | Closed-loop life support | life support | 380 | the Long duration destination, and the life-support slider |\n\
         | Orbital launch | propulsion | 420 | the Orbital destination, and the avionics slider |\n\n\
         **A capability is a key, and the keys are what the campaign is for.** Without \
         basic re-entry the agency cannot put a heat shield on a vehicle however much it \
         spends; without orbital propulsion there is nowhere to fly. The sliders a player \
         can turn are pinned at zero until the capability behind them has been developed, \
         and that is why a research project is not a way of earning points - it is the way \
         of getting the right to build anything."
    ),
    topic!(
        "research.knowledge",
        Section::Research,
        "Knowledge and breakthroughs",
        "What a finished project leaves behind, and the one-in-three chance of a leap.",
        "**A finished project is worth 120 units of knowledge**, and the knowledge goes to \
         the capability of the project's own discipline. A fifth of it spills into the \
         neighbouring disciplines, so a project in propulsion also teaches a little \
         structures and a little life support.\n\n\
         **35% of the projects yield a breakthrough**: a large gain in knowledge in a \
         single instant, and sometimes a capability that the agency was not going to reach \
         by arithmetic. The draw is on the project's own start month and its identity, so \
         the same campaign always has the same breakthroughs in the same places - the \
         player is never cheated by a dice roll they could not have seen coming.\n\n\
         The thresholds are the numbers that matter: 240 units of knowledge unlocks orbital \
         propulsion, 300 unlocks basic re-entry, 380 closed-loop life support, and 420 \
         orbital launch. **A project gives 120**, so a capability takes two or three \
         projects and a breakthrough can save one of them.\n\n\
         **This is the research half of the central tension, and it is why the portfolio \
         matters.** A player who researches one field exclusively will unlock that field's \
         capabilities and no others, and the capabilities are what the rest of the game \
         needs."
    ),
    topic!(
        "research.archetypes",
        Section::Research,
        "The vehicle archetypes",
        "Three kinds of machine, and what each of them is for.",
        "**A vehicle is an archetype and eight sliders.** The archetype says what kind of \
         machine it is, and it bounds what the sliders may do:\n\n\
         | Archetype | What it is |\n|---|---|\n\
         | Sounding rocket | an uncrewed probe that goes up and comes down |\n\
         | Orbital launcher | the thing that puts a payload into orbit |\n\
         | Orbital capsule | the thing that comes back |\n\n\
         **The archetype is a constraint, not a suggestion.** A sounding rocket cannot be \
         crewed, because the life-support slider it would need is not one it has. A player \
         who wants a crewed capsule has to design one, and an orbital launcher has to be \
         able to fly, which means the propulsion slider is not optional.\n\n\
         Everything else about the vehicle follows from the sliders - the cost, the \
         reliability, the payload, the delta-v - and the archetype only says what kind of \
         thing the numbers are allowed to describe. **The design is the player's decision, \
         and the derived numbers are computed from it every time they are shown.**"
    ),
];
