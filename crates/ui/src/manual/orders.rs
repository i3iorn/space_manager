//! The orders.
//!
//! **Every kind of order, and what it does.** The palette shows the price of
//! an order - how long, how much, what it does to goodwill - and this is where
//! it says what the order is *for*.
//!
//! The theme running through all of them: **an order is a claim about the
//! future.** It lands at a definite instant, it is charged then and not now,
//! and the world moves on in the meantime.

use crate::manual::{Section, Topic, topic};

/// The topics about the orders.
pub const TOPICS: &[Topic] = &[
    topic!(
        "order.hire",
        Section::Orders,
        "Hire",
        "Bring somebody new onto the payroll. It takes two to six weeks, and it costs goodwill.",
        "A hire is an order, and it takes between 14 and 42 days to land. The \
         lead time is drawn from the campaign rather than from a random number, so it is \
         the same every time the same campaign is played.\n\n\
         The salary is chosen from the role and the seniority, and the player chooses. The \
         candidate comes from whoever applies, and who applies depends on the agency's \
         fame.\n\n\
         **Hiring is a long-term bet.** The salary is paid every month from the day they \
         arrive, whether or not they are on anything, and the lead time means the money \
         starts being spent before the work does.\n\n\
         The order costs one point of goodwill, which is the cheapest thing the agency does. \
         **What is not cheap is the person.** A hire at the top of the band costs 2,600 a \
         month against 1,200 at the bottom, and the extra 1,400 is paid every month for the \
         rest of the campaign - which is a better lead to spend a fortune on than a building."
    ),
    topic!(
        "order.fire",
        Section::Orders,
        "Fire",
        "Let somebody go. It saves a salary and it costs goodwill.",
        "A firing ends a contract. It saves the salary every month after, and it draws on \
         the state's goodwill - four points, which is the second most expensive thing the \
         agency does routinely - because a dismissal is a thing a state notices.\n\n\
         **It is a last resort, not a management tool.** The person was doing something, \
         and now they are not, and whatever they were working on loses a member of its \
         team. The binding factor on their project may change as a result, and it is \
         worth looking before pulling the trigger.\n\n\
         **The arithmetic that usually decides it:** a person at their limit is leaving \
         anyway at a quarter a month, and they are costing a salary while they do it. \
         Letting them go converts a certain loss into a certain one. The case for firing is \
         the case of somebody who is idle, or whose work is finished, or who is being made \
         miserable by a shift the player set."
    ),
    topic!(
        "order.assign",
        Section::Orders,
        "Assign",
        "Put somebody on something. It takes a day or three, and it costs almost nothing.",
        "An assignment puts a person on a programme, a project, a test campaign, a \
         facility, or a division. It takes 1 to 3 days to land, and it is the \
         cheapest order there is - two points of goodwill, and no money.\n\n\
         **An assignment is a move.** Somebody who is working in the propulsion \
         laboratory can be put on the combustion study instead, and they come off \
         the laboratory's staff list when the order lands - which may leave the \
         laboratory short of hands, and short laboratories wear out. The order \
         palette says whose team somebody is being taken off, before the order \
         is confirmed.\n\n\
         **Any number of people can be on one project**, and the more there are \
         the faster it finishes: each of them contributes their own performance \
         as points, and the points are summed. A project with one person on it \
         takes about twice as long as one with two, and the target does not \
         move.\n\n\
         **This is the order the game is played with.** Every performance number in the \
         agency comes from somebody being on something, and the difference between a \
         project that finishes and a project that does not is very often a decision made \
         here.\n\n\
         It is also the order that moves the whole set of numbers at once: a person on a \
         project joins its team and starts contributing points, starts accruing fatigue, \
         and starts costing the project's budget twelve thousand a month. **One keystroke, \
         three consequences, all of them visible on the screen the player is looking at.**"
    ),
    topic!(
        "order.unassign",
        Section::Orders,
        "Unassign",
        "Take somebody off something. They cost salary and earn nothing.",
        "An unassignment takes a person off whatever they were on. It is free and it is \
         instant in its effect - and it leaves the person on the payroll with nothing to \
         do.\n\n\
         **An unassigned person is a cost and not a neutral state.** They draw their \
         salary every month and contribute nothing to anything, so an unassignment that is \
         not followed by another assignment is a month of salary for nothing.\n\n\
         **It is also the crude way to let somebody recover.** An unassigned person rests at \
         ten points of fatigue a month, which is the fastest recovery in the game. The shift \
         does it better: a person on light duties recovers *while still on the team*, so \
         nothing is lost - no output, no place, no standing.\n\n\
         The unassignment is for when the work is finished, not for when the person is tired."
    ),
    topic!(
        "order.policy",
        Section::Orders,
        "Change the agency's shift policy",
        "A standing instruction about shifts, for everybody at once. The tool for an agency of three hundred people.",
        "**The per-person shift is the surgical tool, and this is the one for the whole \
         agency.** Setting somebody's shift is one keystroke on a screen the player is \
         already looking at, and it scales because it is one keystroke. **A player who \
         cannot make a hundred of them is a player who is not managing their staff at all.**\n\n\
         **The policy is a standing instruction and not a one-off.** It stays in force until \
         the player changes it, and it says two things: **who it covers** and **what they \
         work**. Either everybody works the shift, or — and this is the useful one — \
         **anybody who is tireder than a threshold is eased off**, and the people who are \
         fresher keep their own shift.\n\n\
         **The threshold makes it a thermostat.** A person who is eased off recovers, comes \
         back up to the threshold, and is eased off again, and the cycle is slow. The point \
         is that the agency never has anybody work themselves to pieces, and the player \
         never has to watch eighty fatigue bars to know whether the agency is doing that.\n\n\
         **A person the manager has moved individually is moved off the policy**, and the \
         staff screen says so — a person on a light shift because the policy eased them \
         off is a different thing from a person the manager put on light duties, and the \
         player has to be able to tell them apart.\n\n\
         **The policy is a week of bureaucracy and costs five goodwill**, because a standing \
         instruction about how the whole agency works is a public act. The per-person shift \
         stays free and immediate: a player must be able to make a hundred of them."
    ),
    topic!(
        "order.shift",
        Section::Orders,
        "Set a shift",
        "How hard somebody works. It lands at the next hour, and it costs nothing.",
        "The shift is the order that sets how hard a person works, and it is the cheapest \
         order in the game: it lands at the next hour, and it costs no money and no \
         goodwill.\n\n\
         It runs from nothing to twice a full load, in steps of ten percent, and the two \
         ends do different things. **Below a full load the person recovers while they are \
         still on the team** - they keep their place, their output falls in proportion, and \
         their fatigue goes down. **Above a full load the output rises more slowly than the \
         effort, and the fatigue and the misery rise faster.**\n\n\
         A hundred and fifty percent gives a hundred and thirty percent of the output, a \
         hundred and fifty percent of the fatigue, and seven and a half points of morale a \
         month. A hundred and sixty percent of the output for twice the wear is a bad trade, \
         and the interface shows the trade rather than hiding it.\n\n\
         **A shift given to one person is an override of the agency policy**, and the \
         option to put them back on it is the last one in the list. **That is how a player \
         says \"she works flat out until she is tired, and then she eases off\"** - push her \
         up while she is fresh, and put her back on the policy when the policy's threshold \
         should start to matter.\n\n\
         **This is the order that makes the other numbers move.** Fatigue is one of the \
         eight factors and the most expensive one, so a shift is a performance order in \
         its own right: a team held back by fatigue is held back by fatigue, and the \
         cheapest way to raise it is to give somebody a week at seventy percent."
    ),
    topic!(
        "order.promote",
        Section::Orders,
        "Promote",
        "Give somebody a step up. It takes until the next round, three months apart.",
        "A promotion raises a person's seniority: more salary, and a better support factor \
         for the team they lead.\n\n\
         The order lands and then waits for the next round, which comes every three months. \
         It draws on goodwill - the state likes seeing its people rise, and it costs three \
         points - and it is the cheapest way to improve a programme's support factor.\n\n\
         **The arithmetic is the best in the game.** A team with no lead is a -10 on the \
         support factor; a team with a lead is a +5. That is fifteen points on one of the \
         eight multipliers, for the price of a promotion and up to three months of waiting. \
         A promotion at the top of the band raises the salary by 1,400 a month, which is \
         worth paying when the team it leads is worth a third more."
    ),
    topic!(
        "order.train",
        Section::Orders,
        "Train",
        "Teach somebody a specialism they did not have. It takes six months.",
        "Training puts a person through a course and teaches them a specialism they did \
         not have, which raises their role fit - one of the eight factors.\n\n\
         It takes six months, and the person is still drawing their salary throughout, and \
         they are still not on the project that needs them. **It is the slow way to fix a \
         mismatch**, and it is worth it when the mismatch is a good person's.\n\n\
         **Six months is a long time.** In a campaign that runs thirty to sixty years it is \
         a rounding error; in the first year of a campaign it is most of the way to the \
         first deadline. The order is worth making early and for the people who are worth \
         keeping.\n\n\
         There are four courses - propulsion theory, orbital mechanics, life-support \
         systems, and public speaking - and the first three are worth more on a project \
         than the fourth, which is worth its keep on a programme whose manager has to be \
         believed."
    ),
    topic!(
        "order.start_project",
        Section::Orders,
        "Start a research project",
        "Begin a piece of research. It takes a week or two to land, and it costs by the head.",
        "Starting a project takes 7 to 14 days to land. It costs 1,200 \
         credits a head every month the project is running, plus 3,000 of \
         overhead.\n\n\
         **The target is fixed at 2,400 points and it never moves.** The project's team \
         contributes their performance as points and the points are summed, so adding \
         somebody makes it finish sooner.\n\n\
         A project with nobody on it is a project that will never finish, and a project \
         with a tired team is a project that will finish late. A four-person project at full \
         performance takes six months and costs 46,800; at seventy percent fatigue it \
         takes about nine and changes nothing about the cost.\n\n\
         **The order itself is cheap and the project is not.** One point of goodwill to \
         start, and then a commitment to 7,800 a month for as long as it runs. A player \
         who starts three projects to have the option of finishing one has committed to \
         three."
    ),
    topic!(
        "order.start_test_campaign",
        Section::Orders,
        "Start a test campaign",
        "Prove that what you built works. It costs by the head, and it competes for the same staff.",
        "A test campaign takes 7 days to start and costs 600 credits a head \
         every month - less than a research project.\n\n\
         **It needs a capability to test**, so the order is not on offer until the agency \
         has developed one.\n\n\
         This is the reliability half of the central tension. A capability that has never \
         been tested is a capability that does not work, and the people on the campaign \
         are people who could have been on a project.\n\n\
         **What it buys is fifteen points of risk, permanently.** A campaign of four people \
         takes five months and 19,000, and takes fifteen percentage points off every future \
         mission to that destination - for the rest of the campaign, and after the people \
         who did it have gone. It is the only investment in the game whose value does not \
         decay."
    ),
    topic!(
        "order.build_facility",
        Section::Orders,
        "Build a facility",
        "Build something. The method, the scale, the site, and the financing are all permanent.",
        "Building a facility is the most consequential order in the game, because \
         **the specification it is built to is fixed for the rest of the campaign**.\n\n\
         The method decides how it was built - fast and cheap is quick and less reliable; \
         robust is slow, expensive, and dependable; experimental is cheap and quick and \
         may unlock something nobody else has. The scale decides its capacity and its \
         upkeep. The site decides where it stands. The financing decides who paid, and \
         what they expect in return.\n\n\
         It takes months to build, and the cost is charged when the order lands.\n\n\
         **The numbers are the argument against doing this too early.** A laboratory costs \
         350,000 and fifteen months, and a launch complex 1,200,000 and thirty. On the \
         starting treasury of 190,000, a laboratory is a decision to be short of money for \
         a year and a half, and the campaign is thirty years long. A player who builds the \
         launch complex in the first year will be waiting for it in the fourth."
    ),
    topic!(
        "order.create_programme",
        Section::Orders,
        "Open a programme",
        "Start a body of work with its own budget, its own people, and its own objectives.",
        "A programme is the strategic layer: a piece of work with a name, a purpose, a \
         budget, and a set of objectives. People are assigned to programmes with a role - a \
         lead, a member - and the programme is run by the lead.\n\n\
         **A programme with no lead has a poor support factor**, and the support factor is \
         one of the eight multipliers, so everything the programme does is worth less than \
         it looks.\n\n\
         Opening one is immediate and costs a little goodwill. It is the order that gives \
         the player something to be responsible for.\n\n\
         **A programme is a container the state judges.** Its category - orbital, lunar, \
         deep space, earth observation, reconnaissance, commercial, station, or resource \
         extraction - tells the state what it is funding, and the state reads the portfolio \
         rather than one programme at a time. A player with three programmes and one lead \
         has three programmes with no lead."
    ),
    topic!(
        "order.set_objective",
        Section::Orders,
        "Set a deadline",
        "Add a dated objective to a programme. The target is half the horizon and the promise is the whole of it.",
        "This is the order that makes the game a management game. It puts a date on \
         something, and the date is a promise to somebody.\n\n\
         **The target is half the horizon and the commitment is the whole of it**, so the \
         player is given a slack rather than being asked to invent one - and the slack is \
         the buffer the rest of the campaign is spent against.\n\n\
         The audience is who the promise is made to, and it decides what breaking it costs. \
         Setting a deadline is immediate and costs a little goodwill.\n\n\
         **The horizon is the player's only real decision here**, and it is the decision \
         that determines how much trouble the player is in. Half of it is slack, and slack is \
         what absorbs a bad month, a sick scientist, or a building that takes longer than \
         it said. A player who promises the state the shortest date that could possibly work \
         has promised to do something with no margin at all, and the world knows it."
    ),
    topic!(
        "order.set_budget",
        Section::Orders,
        "Set a budget",
        "How much a programme gets each month. The money is the agency's, and the state's.",
        "A programme's budget is what it can afford, and it is a monthly allocation. It is \
         set immediately and it costs a little goodwill.\n\n\
         **The budget is not the same as the money.** A programme can have a budget and no \
         money, and a budget that is not being spent is not a budget - it is a plan to \
         spend money the agency does not have.\n\n\
         The budget is what the research projects and the test campaigns inside the \
         programme are paid out of, and it is set before they are started. **A player who \
         opens a programme with a generous budget and then starts four projects has not \
         budgeted for four projects.** The order that spends the money comes afterwards, \
         and the treasury is what has to cover it."
    ),
    topic!(
        "order.postpone_deadline",
        Section::Orders,
        "Push a deadline back",
        "Renegotiating costs political capital, and the state is the hardest to ask.",
        "Sometimes the work will not be done in time, and the only options are to miss the \
         deadline or to move it. **Moving it is cheaper than missing it, and it is not \
         free**: it costs political capital and goodwill, and how much depends on who holds \
         the deadline.\n\n\
         | Who holds it | Political capital | Goodwill |\n|---|---|---|\n\
         | the agency | 5 | -1 |\n\
         | a contractor | 20 | -4 |\n\
         | the state, the public, or the regulator | 40 | -8 |\n\n\
         **A renegotiation takes a week**, because a renegotiation is a conversation and \
         not a decision. And the promise moves: the new date is the one that falls due.\n\n\
         **Compare it with the alternative.** A missed political deadline costs eighteen \
         points of approval and eight of goodwill. A postponement costs forty political \
         capital and the same eight goodwill - and it saves the approval. The capital is \
         the currency for buying time, and it comes back at three a month, so a player who \
         postpones once a year is not spending a fortune.\n\n\
         A player who postpones every deadline has bought their way out of every promise \
         and has no promises left worth making."
    ),
    topic!(
        "order.design_vehicle",
        Section::Orders,
        "Design a vehicle",
        "A vehicle is a set of sliders, and the numbers come out of them. You cannot buy a capability you have not researched.",
        "**A vehicle is the player's decision made of parts.** Every slider can \
         be turned up and every slider costs, and the numbers the vehicle works out to \
         follow from the sliders and nothing else.\n\n\
         | Slider | What it buys |\n|---|---|\n\
         | Propulsion | How good it is - the abstract unit that is not real orbital mechanics |\n\
         | Payload | What it can carry |\n\
         | Life support | How long the crew can survive |\n\
         | Redundancy | How much can fail without sinking the vehicle |\n\
         | Power | How it makes and stores power |\n\
         | Structure | How strong it is against what it hits |\n\
         | Avionics | How good its guidance is |\n\
         | Heat shield | How well it survives re-entry |\n\n\
         **Two things bound a slider.** The archetype says what kind of machine it \
         is - a sounding rocket cannot be crewed and an orbital launcher must be able \
         to fly - and the capabilities say what the agency has developed. **A slider \
         the agency has not researched is pinned at zero**, whatever the archetype \
         would otherwise allow, and a player cannot buy a heat shield before the \
         agency has developed re-entry.\n\n\
         The design is the player's decision, and **the derived numbers are never \
         stored as an authority**: the cost, the reliability, the payload and the \
         delta-v are all computed from the sliders every time they are shown.\n\n\
         **And the build takes twenty months for every million credits it costs.** A \
         vehicle that costs four million takes a year to build, and the campaign is \
         running the whole time."
    ),
    topic!(
        "order.set_subsystem",
        Section::Orders,
        "Change a subsystem",
        "A vehicle is a set of sliders, and you can change any of them - for three days, and a small price.",
        "**A subsystem change is changing a piece of the drawing**, and it takes \
         the same three days a design does. The value is bounded exactly as the \
         design was: the archetype's band still applies, and **a slider the agency \
         has not researched is still pinned at zero** - editing it does not make the \
         technology exist.\n\n\
         **And the numbers move with it.** The cost, the reliability, the payload and \
         the delta-v are all recomputed from the sliders every time they are shown, so \
         the vehicle on the screen is always the vehicle the agency would build. A \
         change that makes the vehicle worse is refused by nothing at all: the engine \
         takes the order, the number falls, and the player sees what it cost.\n\n\
         **The price of the change is the price of the vehicle.** Turning a slider up \
         makes the machine more expensive as well as better, and a change to a vehicle \
         that is already being built does not change what is being built - it changes \
         what the agency pays for."
    ),
    topic!(
        "order.authorize_launch",
        Section::Orders,
        "Authorise a launch",
        "The risk figure is on the screen before you commit, and the flight takes ninety minutes.",
        "**A launch is authorised, and then it flies.** The order takes an hour and a \
         quarter to land - that is the ascent - and then the mission goes through \
         preparation, launch, transit, operations, return and archival, and each of \
         those takes as long as it takes.\n\n\
         **The engine refuses the impossible ones.** A vehicle that is not finished \
         being built cannot be launched. A destination the agency has not developed the \
         technology for is not somewhere it can fly. And a vehicle without the delta-v \
         to reach the destination cannot get there, whatever it is called.\n\n\
         **The risk figure is the vehicle's reliability**, less whatever the testing \
         has taken out of the destination's capability. A partial failure is the common \
         branch: the capsule comes back, the data is partial, and the agency has lost \
         something. A total failure is rarer, and it writes the vehicle off.\n\n\
         **And a launch is a political act.** A successful launch earns the state's \
         approval; a failure costs it, and costs the public's opinion as well. The \
         government is watching, and it is keeping a list."
    ),
];
