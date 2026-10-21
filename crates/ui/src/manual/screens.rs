//! The screens, and what is on them.
//!
//! **This is the information view.** Every screen names its own panels, and
//! every panel explains itself. The manual restricted to a screen is a guided
//! tour of that screen, and it is the fastest answer to "what am I looking at".
//!
//! The screen topics are also what the info line shows, so a player who is
//! wondering about a panel gets the answer without opening anything.

use crate::manual::{Section, Topic, topic};
use crate::screen::Screen;

/// The topics about the screens.
pub const TOPICS: &[Topic] = &[
    topic!(
        "screen.dashboard",
        Section::Screens,
        "The dashboard",
        "Where do I stand right now. The screen the player starts on and returns to.",
        "The dashboard answers three questions: what is the agency doing, what is it \
         about to have to answer for, and how long does the money last.\n\n\
         It is deliberately the least detailed screen in the game. Everything on it is a \
         pointer to somewhere else, and the panels are explained below."
    ),
    topic!(
        "screen.dashboard.headline",
        Section::Screens,
        "Headline",
        "The one line that says what the agency is for.",
        "The headline is the agency's own statement of its purpose. It is set at the start \
         and it does not change, and it is there so that the player never has to remember \
         what they are running."
    ),
    topic!(
        "screen.dashboard.due",
        Section::Screens,
        "Due soon",
        "Everything with a date on it: projects, objectives, and outstanding promises.",
        "The **due soon** panel is the agenda. It lists the research projects that are \
         running, the programme objectives that have dates, and the promises that are still \
         outstanding, with the date each is due.\n\n\
         For an objective it also shows the **slack** - the room between the target and the \
         commitment. An objective with no slack is one where the next thing that goes wrong \
         is a missed commitment, and that is the line the player should be reading this \
         panel for.\n\n\
         A campaign is won and lost on this panel, and it is the first place to look after \
         every month report."
    ),
    topic!(
        "screen.dashboard.finance",
        Section::Screens,
        "Finance",
        "Income, outgoings, net, runway, and the crisis stage.",
        "The finance panel carries the five numbers that decide whether the campaign \
         continues: the income, the outgoings, the net of the two, the runway in months, \
         and the crisis stage.\n\n\
         The **net** is a daily figure, not a monthly one. The **runway** is how many \
         months the money lasts at the current rate, and it is the number to watch - it is \
         a function of the net rather than of the balance, so a large balance with a bad \
         net has a short runway. The **crisis stage** is where the agency is on the road \
         from solvent to collapse."
    ),
    topic!(
        "screen.staff",
        Section::Screens,
        "The staff screen",
        "The people, and why they are slow. The most important screen in the game.",
        "The staff screen has two halves: the list of people, and the detail of the one \
         selected. The detail carries the **binding factor** - the reason this person is \
         slow, named in words.\n\n\
         If there is one screen to understand, it is this one. The binding factor is the \
         single number that decides whether the agency gets anything done, and the whole \
         game is about improving it."
    ),
    topic!(
        "screen.staff.list",
        Section::Screens,
        "The staff list",
        "Everyone the agency employs, with their age, their fatigue, and their risk of leaving.",
        "The list shows every member of staff: their name, their role, their age, their \
         fatigue, and the chance they leave this month.\n\n\
         **The risk is the column to sort by.** A person about to leave is a person the \
         agency can still save, and the risk turns red when it matters.\n\n\
         **The list filters and sorts.** Press `/` to filter it and `s` to change the \
         sort, which cycles through name, age, role, fatigue, risk, and shift. The filter \
         survives a trip to another screen and back.\n\n\
         Move the selection with the up and down arrows and the detail panel follows it; \
         the left and right arrows change the selected person's shift."
    ),
    topic!(
        "screen.staff.detail",
        Section::Screens,
        "The detail panel",
        "One person: their assignment, their condition, their shift, and their salary.",
        "The detail panel is everything about the selected person: what they are assigned \
         to, their morale, fatigue, health and fame, their four attributes, their \
         specialisms, their salary, their **shift**, and the chance they leave this \
         month.\n\n\
         The **assignment** is the one to check first. A person who is unassigned earns \
         nothing and still costs their salary, and the detail panel says so plainly.\n\n\
         The **shift** is the throttle, and the arrows change it. The panel shows what \
         the shift does - the output it produces and the fatigue it costs - and a change \
         that is in flight is shown as `100% → 110% (pending)`, because the order lands \
         at the next hour and not now."
    ),
    topic!(
        "screen.staff.performance",
        Section::Screens,
        "The performance panel",
        "The eight factors, and the one that binds. This is the most important panel in the game.",
        "The performance panel shows the eight factors - skill, role fit, morale, support, \
         tools, fatigue, creativity and health - as named modifiers, and then names the \
         **binding factor**: the one of them that is holding the person back.\n\n\
         The eight are multiplied together, so the smallest one decides the answer. A \
         panel showing a fine technician held back by fatigue is not a panel to be read \
         optimistically; it is a panel telling the player to send them home.\n\n\
         **Fix the binding factor first.** Every point of it is worth more than any other \
         improvement to the same team, because it multiplies into all the rest. The panel \
         is deliberately shown as named modifiers rather than as a number, because a number \
         the player cannot decompose is a number the player cannot act on."
    ),
    topic!(
        "screen.programme",
        Section::Screens,
        "The programme screen",
        "What the agency has committed to, and when.",
        "The programme screen lists the programmes and their objectives, with the dates. It \
         is the strategic view: the research screen is what is being built, and this is \
         what has been promised.\n\n\
         Move the selection to move between programmes, and the objectives underneath show \
         the target, the commitment, and the slack between them."
    ),
    topic!(
        "screen.programme.list",
        Section::Screens,
        "The programme list",
        "The programmes, their leads, and their objectives.",
        "Each programme is listed with its lead and its objectives, and each objective \
         carries its description and the date it is due.\n\n\
         **A programme with no lead has a poor support factor**, and the support factor \
         multiplies into everything the programme does. An unled programme is worth less \
         than its members' ability suggests."
    ),
    topic!(
        "screen.research",
        Section::Screens,
        "The research screen",
        "Projects on one side, facilities on the other. The working screen.",
        "The research screen has the projects on the left and the facilities on the right. \
         It is where the player watches the two halves of the central tension: the \
         capability being built, and the plant that builds it."
    ),
    topic!(
        "screen.research.projects",
        Section::Screens,
        "Research projects",
        "Every project, its team, and how far it is toward its target.",
        "Each project is listed with its title, its team, and the points it has earned \
         against the 2,400 it needs.\n\n\
         **The points are the rate of the team, made visible.** A project that has earned \
         nothing is a project whose team is not producing, and the staff screen is where \
         the reason is."
    ),
    topic!(
        "screen.research.facilities",
        Section::Screens,
        "Facilities",
        "What the agency has built, and what each one is for.",
        "Each facility is listed with its type, its specification, and how many people it \
         needs. A facility that is still being built says so.\n\n\
         **There is no abstract capacity.** Every facility was built by an order, on a \
         site, with a method and a scale, and its specification is fixed for the rest of \
         the campaign."
    ),
    topic!(
        "screen.vehicles",
        Section::Screens,
        "The vehicles screen",
        "Vehicles, missions, and the risk of launching them.",
        "The vehicles screen lists the vehicles the agency has and the missions they are \
         on, with the risk of each.\n\n\
         Vehicles are where the campaign stops being a spreadsheet and starts being a \
         thing that can be lost. Everything before this screen is preparation; this is \
         where the preparation is tested against the universe."
    ),
    topic!(
        "screen.chronicle",
        Section::Screens,
        "The chronicle screen",
        "The history on the left, the forecast on the right.",
        "The chronicle screen shows the campaign's record on the left and what is \
         scheduled to happen on the right.\n\n\
         The **history** is everything that has already happened, with the instant it \
         happened. The **forecast** is everything scheduled - orders in flight, project \
         completions, commitments falling due - which is the only place the player can \
         see the future at all, and it is a future the player has already committed to."
    ),
    topic!(
        "screen.branches",
        Section::Screens,
        "The branches screen",
        "The decisions the player has made, and the fork at the selected one.",
        "The branches screen lists the points at which the player made a decision, and \
         `Enter` forks the campaign at the selected one.\n\n\
         A fork replays the order log, so the two branches are the same campaign up to \
         the fork point and differ only because of the decision. That is what makes the \
         comparison worth making."
    ),
];

/// The topics that belong to each screen.
///
/// **This is the mapping that makes the information view work**: the manual
/// restricted to a screen is exactly these topics, in this order.
#[must_use]
pub fn topics_for(screen: Screen) -> &'static [&'static str] {
    match screen {
        Screen::Dashboard => &[
            "screen.dashboard",
            "screen.dashboard.headline",
            "screen.dashboard.due",
            "screen.dashboard.finance",
        ],
        Screen::Staff => &[
            "screen.staff",
            "screen.staff.list",
            "screen.staff.detail",
            "screen.staff.performance",
        ],
        Screen::Programme => &["screen.programme", "screen.programme.list"],
        Screen::Research => &[
            "screen.research",
            "screen.research.projects",
            "screen.research.facilities",
        ],
        Screen::Vehicles => &["screen.vehicles"],
        Screen::Chronicle => &["screen.chronicle"],
        Screen::Branches => &["screen.branches"],
    }
}
