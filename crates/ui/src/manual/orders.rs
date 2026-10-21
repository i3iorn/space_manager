//! The orders.
//!
//! **Every kind of order, and what it does.** The palette shows the price of
//! an order - how long, how much, what it does to goodwill - and this is where
//! it says what the order is *for*.

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
         starts being spent before the work does."
    ),
    topic!(
        "order.fire",
        Section::Orders,
        "Fire",
        "Let somebody go. It saves a salary and it costs goodwill.",
        "A firing ends a contract. It saves the salary every month after, and it draws on \
         the state's goodwill - a dismissal is a thing a state notices.\n\n\
         **It is a last resort, not a management tool.** The person was doing something, \
         and now they are not, and whatever they were working on loses a member of its \
         team. The binding factor on their project may change as a result, and it is \
         worth looking before pulling the trigger."
    ),
    topic!(
        "order.assign",
        Section::Orders,
        "Assign",
        "Put somebody on something. It takes a day or three, and it costs almost nothing.",
        "An assignment puts a person on a programme, a project, a test campaign, a \
         facility, or a division. It takes 1 to 3 days to land, and it is the \
         cheapest order there is - a little goodwill, and no money.\n\n\
         **This is the order the game is played with.** Every performance number in the \
         agency comes from somebody being on something, and the difference between a \
         project that finishes and a project that does not is very often a decision made \
         here."
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
         not followed by another assignment is a month of salary for nothing."
    ),
    topic!(
        "order.promote",
        Section::Orders,
        "Promote",
        "Give somebody a step up. It takes until the next round, three months apart.",
        "A promotion raises a person's seniority: more salary, and a better support factor \
         for the team they lead.\n\n\
         The order lands and then waits for the next round, which comes every three months. \
         It draws on goodwill - the state likes seeing its people rise - and it is the \
         cheapest way to improve a programme's support factor."
    ),
    topic!(
        "order.train",
        Section::Orders,
        "Train",
        "Teach somebody a specialism they do not have. It takes six months.",
        "Training puts a person through a course and teaches them a specialism they did \
         not have, which raises their role fit - one of the eight factors.\n\n\
         It takes six months, and the person is still drawing their salary throughout, and \
         they are still not on the project that needs them. **It is the slow way to fix a \
         mismatch**, and it is worth it when the mismatch is a good person's."
    ),
    topic!(
        "order.start_project",
        Section::Orders,
        "Start a research project",
        "Begin a piece of research. It takes a week or two to land, and it costs by the head.",
        "Starting a project takes 7 to 14 days to land. It costs 12,000 \
         credits a head every month the project is running, plus 6,000 of \
         overhead.\n\n\
         **The target is fixed at 2,400 points and it never moves.** The project's team \
         contributes their performance as points and the points are summed, so adding \
         somebody makes it finish sooner.\n\n\
         A project with nobody on it is a project that will never finish, and a project \
         with a tired team is a project that will finish late."
    ),
    topic!(
        "order.start_test_campaign",
        Section::Orders,
        "Start a test campaign",
        "Prove that what you built works. It costs by the head, and it competes for the same staff.",
        "A test campaign takes 7 days to start and costs 7,000 credits a head \
         every month - less than a research project.\n\n\
         **It needs a capability to test**, so the order is not on offer until the agency \
         has developed one.\n\n\
         This is the reliability half of the central tension. A capability that has never \
         been tested is a capability that does not work, and the people on the campaign \
         are people who could have been on a project."
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
         It takes months to build, and the cost is charged when the order lands."
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
         the player something to be responsible for."
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
         Setting a deadline is immediate and costs a little goodwill."
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
         spend money the agency does not have."
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
         | the agency | 5 | -1 |\n| a contractor | 20 | -4 |\n| the state, the public, or the regulator | 40 | -8 |\n\n\
         **A renegotiation takes a week**, because a renegotiation is a conversation and \
         not a decision. And the promise moves: the new date is the one that falls due."
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
         | Propulsion | How good it is — the abstract unit that is not real orbital mechanics |\n\
         | Payload | What it can carry |\n\
         | Life support | How long the crew can survive |\n\
         | Redundancy | How much can fail without sinking the vehicle |\n\
         | Power | How it makes and stores power |\n\
         | Structure | How strong it is against what it hits |\n\
         | Avionics | How good its guidance is |\n\
         | Heat shield | How well it survives re-entry |\n\n\
         **Two things bound a slider.** The archetype says what kind of machine it \
         is — a sounding rocket cannot be crewed and an orbital launcher must be able \
         to fly — and the capabilities say what the agency has developed. **A slider \
         the agency has not researched is pinned at zero**, whatever the archetype \
         would otherwise allow, and a player cannot buy a heat shield before the \
         agency has developed re-entry.\n\n\
         The design is the player's decision, and **the derived numbers are never \
         stored as an authority**: the cost, the reliability, the payload and the \
         delta-v are all computed from the sliders every time they are shown."
    ),
];
