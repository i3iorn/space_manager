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
        "screen.flow",
        Section::Screens,
        "How the screens fit together",
        "The loop the player goes round, and which screen answers which question.",
        "**The screens are one loop, and the loop is: look, decide, order, watch.**\n\n\
         The **dashboard** is the only screen that answers *what should I do about \
         this?* - it holds the due-soon panel, the finance panel, and the count of \
         things that need attention. Everything else is a detail of one of those three \
         answers.\n\n\
         The **staff screen** is the answer to *why is nothing getting done?* It names the \
         binding factor for the selected person, and it is where the player changes a \
         shift or an assignment. Almost every order the player gives starts here.\n\n\
         The **programme screen** is the answer to *what have I promised?* It holds the \
         objectives, the targets, the commitments, and the slack between them. It is \
         where the player sets a deadline and where the player sees a deadline slipping.\n\n\
         The **research screen** is the answer to *what is being built?* The projects on \
         the left, the facilities on the right - the capability side of the central \
         tension and the plant that supports it.\n\n\
         The **vehicles screen** is the answer to *what is actually going out there?* It \
         is where the preparation is tested against the universe, and it is the only \
         screen where the agency can lose something outright.\n\n\
         The **chronicle** is the answer to *what has happened, and what is coming?* The \
         history on the left, the forecast of the next twelve months on the right.\n\n\
         The **branches** is the answer to *what would have happened if?* The player's own \
         decisions, and a fork at any of them.\n\n\
         **The order palette is not a screen.** It is a modal over all of them, opened \
         with `o`, and it is where the player acts on whatever the other screens told \
         them. The rule is that the player never changes the state directly - they issue \
         an order, and the order lands later."
    ),
    topic!(
        "screen.dashboard",
        Section::Screens,
        "The dashboard",
        "Where do I stand right now. The screen the player starts on and returns to.",
        "The dashboard answers three questions: what is the agency doing, what is it \
         about to have to answer for, and how long does the money last.\n\n\
         It has two halves. **The left half is the agenda** - the headline, what is \
         due, and the agency's standing - and **the right half is the money**, because \
         a player who cannot see where the money is going is a player who is guessing. \
         The panels are explained below.\n\n\
         **It is also the screen the player returns to after every month report**, because \
         the report says what happened and the dashboard says what to do about it. The \
         due-soon panel is the agenda; the budget panel is the ledger; and the count of \
         things needing attention is the one number to look at when the player has a \
         minute."
    ),
    topic!(
        "screen.dashboard.headline",
        Section::Screens,
        "Headline",
        "The one line that says what the agency is for.",
        "The headline is the agency's own statement of its purpose. It is set at the start \
         and it does not change, and it is there so that the player never has to remember \
         what they are running.\n\n\
         It is a small thing, and it is worth more than it looks: a player who has been \
         running an agency for twenty years of game time should be able to glance at the \
         top of the screen and know what it is for."
    ),
    topic!(
        "screen.dashboard.banner",
        Section::Screens,
        "What now",
        "One piece of advice, read from the campaign as it stands.",
        "The banner at the top of the dashboard says **what the agency should be doing \
         next**, and it works that out from the campaign as it stands.\n\n\
         **It is not a tutorial.** A tutorial says \"now do this\", and a player who came \
         here to do something else has to ignore it. The banner says nothing of the sort: \
         it looks at what is wrong or missing right now, and it says what to do about it in \
         one sentence. A player whose agency needs a rocket is told about the rocket; a \
         player whose agency is running out of money is told about the money.\n\n\
         **The advice is read from the campaign every time the screen is drawn, and it is \
         never stored.** So the banner always matches the campaign the player is actually \
         in, and doing the thing the banner said changes what it says next.\n\n\
         **Press `n` for another.** There is usually more than one thing worth saying, and \
         the player can browse them. The advice that is showing is the most urgent one, and \
         the rest are behind it."
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
         every month report.\n\n\
         **The slack is the thing to read here, and it is read in order of the date.** A \
         commitment with slack is a commitment the agency can survive a bad month on; a \
         commitment without one is a bet on everything going right. The player who can \
         only look at one line of this panel should look at the one with no slack."
    ),
    topic!(
        "screen.dashboard.standing",
        Section::Screens,
        "Standing",
        "The public's opinion, the agency's prestige, and how close the crisis is.",
        "The standing panel carries the three things the budget does not: what the \
         public thinks of the agency, how much prestige it has built, and how far down \
         the crisis ladder it is.\n\n\
         **The public's opinion is here because the world reads it.** The weighting that \
         decides which events fire looks at the public's opinion, so a number the \
         player cannot see is a number the player cannot act on. The state's approval \
         is a different thing, and it is in the top bar.\n\n\
         **The crisis stage is the road from solvent to collapse.** It only moves \
         downwards, and the warning appears two months before the austerity, so a \
         player who reads this panel has time to do something about it."
    ),
    topic!(
        "screen.dashboard.budget",
        Section::Screens,
        "Budget",
        "Where the money comes from, where it goes, and what has already been promised.",
        "The budget panel is the answer to **where is the money going**, and it is four \
         things in the order the player needs them.\n\n\
         **1. The position.** The balance, what comes in, what goes out, the net of \
         the two, and the runway in months. The net is a monthly figure here; the top \
         bar carries the daily one, because the game runs in real time and the player \
         thinks in days.\n\n\
         **2. Where it goes.** Every outgoing line, with a bar showing its share of the \
         total. The share is the point: payroll is the largest line and the least \
         changeable, research and testing are the ones the player chooses, and upkeep \
         is the price of the buildings. **A number without a share does not answer \
         the question.**\n\n\
         **3. Where it comes from.** The income by source. The state allocation is \
         the only source in the MVP, and it is 1,900 a head with a floor of 12,000, \
         scaled by the state's approval by up to half in either direction.\n\n\
         **4. What is committed.** The orders in flight and what they will cost when \
         they land. **The cost is charged when the order lands, not when it is \
         given**, so the money is still in the treasury and the commitment is \
         already a fact.\n\n\
         The programme budgets are under it: the money set aside for each programme \
         and how much of it is unspent. **Pledged money is not free money**, and a \
         programme with nothing left in it is a programme that cannot start \
         anything."
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
         game is about improving it.\n\n\
         **The screen is the argument for the shift.** The arrows on the selected person \
         change how hard they work, and the panel shows what the change does to their \
         output and their fatigue before it is made. It is the only control in the game \
         that changes a person's work in flight rather than replacing them, and it is the \
         difference between a team recovering and a team being rebuilt."
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
         the left and right arrows change the selected person's shift.\n\n\
         **The risk column is also the early-warning system.** At 0.5% a month almost \
         nobody leaves; at 25% a month a quarter of the team is going. A list where a \
         dozen names are red is a list telling the player that something is wrong, and it \
         is usually fatigue."
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
         The **shift** is the throttle, and the arrows change it. The panel shows what the \
         shift does - the output it produces and the fatigue it costs - and a change \
         that is in flight is shown as `100% -> 110% (pending)`, because the order lands \
         at the next hour and not now.\n\n\
         **The attributes and the specialisms are the second thing to read.** A technical \
         attribute of 82 is a skill factor of +32; two matching specialisms are a role-fit \
         factor of +30; and no specialism at all is -25. The player who is about to hire \
         should look at these before the salary."
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
         the player cannot decompose is a number the player cannot act on.\n\n\
         **What the numbers look like.** A good team reads 100%. The same team with one \
         member at seventy percent fatigue reads thirty percent; with the worst tools in \
         the world, seventy; with nobody in the right field, sixty-eight; and with no \
         lead, eighty-six. So the panel is a ranking of the player's problems, and the \
         top of it is the thing to fix this month."
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
         the target, the commitment, and the slack between them.\n\n\
         **Every objective says how it is done.** An objective that reads \"Reach orbit\" \
         tells the player nothing on its own, and the line under it - **done by** - names \
         what finishes it: a successful launch to Low orbit, or the programme's own staff \
         getting on with it. **The guessing is the whole of the first hour of the game, and \
         it does not have to happen.**\n\n\
         **This is the screen where the political layer is visible.** The audience of an \
         objective decides what missing it costs - nothing for the agency's own dates, \
         eighteen points of approval for the state's - and the screen shows the audience \
         beside the date. A player who cannot see who holds a deadline cannot price it."
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
         than its members' ability suggests.\n\n\
         **The objectives are the promises, and they are the thing the state reads.** The \
         panel shows the target, the commitment, and the slack, so the player can see at a \
         glance which of them are comfortable and which are a bet."
    ),
    topic!(
        "screen.research",
        Section::Screens,
        "The research screen",
        "Projects on one side, facilities on the other. The working screen.",
        "The research screen has the projects on the left and the facilities on the right. \
         It is where the player watches the two halves of the central tension: the \
         capability being built, and the plant that builds it.\n\n\
         **The two halves are connected, and the connection is the tools factor.** A team \
         working in a laboratory that has no room for them has worse tools, and worse \
         tools multiply down into everything else they have. A player who reads only the \
         left-hand side of this screen will not know why a project is slow."
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
         the reason is.\n\n\
         The estimate shown beside it assumes a team at full performance, so the player \
         should read it as a floor: a tired team will take a third longer, and the screen \
         will say so when the projection and the estimate disagree."
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
         the campaign.\n\n\
         **The people a facility needs are on this panel, and they are a cost.** A \
         facility that wants four people and has none loses four percent of its condition \
         a month, and the conditions of the facilities are what the world reads when it \
         decides whether something is going to go wrong."
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
         where the preparation is tested against the universe.\n\n\
         **The risk is the vehicle's reliability multiplied by ninety-five per cent, less the \
         points of testing the agency has done on that destination.** So a vehicle of \
         ninety reliability goes out at eighty-five per cent, and fifteen points of testing \
         takes it to seventy. Every point of testing is five months and 19,000 of \
         somebody else's time, and it is the only way to move that number."
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
         see the future at all, and it is a future the player has already committed to.\n\n\
         **The forecast never shows what will go wrong.** It shows what is scheduled and \
         what it is costing in risk; whether it is met is another matter. A game that \
         forecast the future would not be a game about managing risk."
    ),
    topic!(
        "screen.branches",
        Section::Screens,
        "The branches screen",
        "The decisions the player has made, and the fork at the selected one.",
        "The branches screen lists the points at which the player made a decision, and \
         `Enter` forks the campaign at the selected one.\n\n\
         A fork replays the order log, so the two branches are the same campaign up to the \
         fork point and differ only because of the decision. That is what makes the \
         comparison worth making.\n\n\
         **The list is the player's own history of decisions**, and it is the one place \
         where the campaign's quality is visible as a list: a campaign with a hundred \
         decisions in it is a campaign where something was always being done."
    ),
    topic!(
        "screen.chronicle.filter",
        Section::Screens,
        "Filtering the history",
        "Three axes - the year, the category, and the text - and the line that says which is in force.",
        "**The history is long and the player is looking for one thing in it**, so the \
         filter is three axes: the year, the category, and the text. The line above the \
         history always says which of them is in force, because a filtered list with a \
         no visible filter is a list the player cannot trust.\n\n\
         **The forecast is the next twelve months and no further.** The player is planning \
         *this* year, and a schedule that runs to the end of the campaign is not a plan. \
         Each entry carries its risk figure: the slack in a commitment - and a commitment \
         with no slack left is in red - and the chance of working for a mission.\n\n\
         **The forecast never shows what will go wrong.** It shows what is scheduled: the \
         promises, the commitments, and the missions. Whether they are met is another \
         matter."
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
            "screen.dashboard.standing",
            "screen.dashboard.budget",
            "screen.flow",
            "interface.firsthour",
            "interface.quitting",
            "state.capital",
            "state.public",
            "state.mandate",
        ],
        Screen::Staff => &[
            "screen.staff",
            "screen.staff.list",
            "screen.staff.detail",
            "screen.staff.performance",
            "agency.factors",
            "agency.binding",
            "agency.traits",
            "agency.specialisms",
            "order.shift",
        ],
        Screen::Programme => &[
            "screen.programme",
            "screen.programme.list",
            "programme.milestone",
            "programme.categories",
            "programme.slack",
        ],
        Screen::Research => &[
            "screen.research",
            "screen.research.projects",
            "screen.research.facilities",
            "research.tension",
            "research.disciplines",
            "research.knowledge",
        ],
        Screen::Vehicles => &[
            "screen.vehicles",
            "research.mission",
            "research.archetypes",
            "order.design_vehicle",
        ],
        Screen::Chronicle => &[
            "screen.chronicle",
            "screen.chronicle.filter",
            "world.events",
            "world.contexts",
        ],
        Screen::Branches => &["screen.branches"],
    }
}
