//! Programmes, objectives, and promises.
//!
//! **The strategic layer.** A programme is what the agency has committed to
//! doing, and an objective is a dated piece of that. The gap between when the
//! agency would like to finish and when it has promised to finish is the thing
//! the player manages.
//!
//! The vocabulary here is the hardest in the game, because it is the part the
//! state judges the agency on rather than the part the agency controls.

use crate::manual::{Section, Topic, topic};

/// The topics about programmes and promises.
pub const TOPICS: &[Topic] = &[
    topic!(
        "programme.programme",
        Section::Programmes,
        "A programme",
        "A body of work with its own budget, its own people, and its own objectives.",
        "A programme is the strategic layer: a piece of work with a name, a purpose, a \
         budget, and a set of objectives. People are assigned to programmes with a role \
         - a lead, a member - and the programme is run by the lead.\n\n\
         Programmes are what the state funds and what the state judges. The programme \
         screen is where the player sees what has been promised, when it is due, and how \
         much room there is.\n\n\
         **A programme is a container, and everything the player cares about is inside \
         one.** The budget, the people, the objectives, the milestones, and the risks are \
         all attached to a programme rather than to the agency as a whole. A player who \
         cannot say which programme a commitment belongs to cannot say which budget is \
         going to be short.\n\n\
         The category matters too: an orbital programme and a lunar one are judged by the \
         state on different terms, and the state reads the whole portfolio rather than one \
         programme at a time."
    ),
    topic!(
        "programme.objective",
        Section::Programmes,
        "An objective",
        "A dated piece of a programme, with a target and a commitment.",
        "An objective is a thing the programme must achieve, with a date on it. Every \
         dated objective carries **two** instants, and the difference between them is the \
         whole of the player's problem.\n\n\
         The target is the soft one: when the agency expects to finish, if everything goes \
         well. The commitment is the hard one: when the agency has told the state it will \
         finish, and will be judged against.\n\n\
         **An objective also has a kind** - whether it is a thing that must be done or a \
         thing that must be true - and an **audience**, which is who the promise was made \
         to. The audience is what decides what missing it costs, and it is the most \
         consequential thing on the objective.\n\n\
         An objective also has a target of 600 points, which is the work the programme \
         has to get done behind it. A programme with three objectives and no people on it \
         has three targets and nothing to hit them with."
    ),
    topic!(
        "programme.target",
        Section::Programmes,
        "The target instant",
        "When the agency expects to finish. It is soft, and nobody is judged against it.",
        "The target instant is the agency's own expectation - when the work will be done if \
         nothing goes wrong. It is soft: the state does not judge the agency against it, \
         and missing it costs nothing directly.\n\n\
         A target that keeps moving backwards is a sign that the plan is wrong. The player \
         does not have to defend it to anyone, but the gap between the target and the \
         commitment is the measure of how wrong it is.\n\n\
         **The target moves as the world moves.** The projection is recalculated from the \
         team's rate, so a team that is tired moves the target out, and a person who \
         leaves moves it out further. This is the most useful thing on the programme \
         screen: it is the date the work will actually be finished, given everything that \
         is true now.\n\n\
         A target the player has to defend to somebody else is a commitment, and the \
         difference between the two is the whole subject."
    ),
    topic!(
        "programme.committed",
        Section::Programmes,
        "The committed instant",
        "When the agency promised to finish. It is hard, and the state judges against it.",
        "The committed instant is the hard date: the moment the agency said, out loud, \
         that the work would be finished.\n\n\
         **A commitment is held by the state, not by the agency.** That is the distinction \
         the whole system turns on. The agency can slip its own target as often as it likes \
         and adjust the plan to match. The committed instant is a promise made to somebody \
         else, and the state keeps the score.\n\n\
         Meeting a commitment is worth +8 points of approval; missing one is worth -18. \
         The gap between those two numbers is the price of promising something \
         the agency cannot deliver.\n\n\
         **The committed instant is also the thing that can be bought back - at a price.** \
         Postponing a deadline is an order: it costs political capital (40 for a \
         political promise, 20 for a contractual one, 5 for the agency's own), it \
         costs goodwill, and it takes a week to land. **Postponing is cheaper than missing, \
         but it is not free, and the player can postpone their way to being a director \
         nobody trusts.**"
    ),
    topic!(
        "programme.slack",
        Section::Programmes,
        "Slack",
        "The room between the target and the commitment. It is the resource the player manages.",
        "Slack is the gap between the soft target and the hard commitment. It is measured in \
         time, and it is the central quantity the player is managing for the whole campaign.\n\n\
         **Slack is a budget, and it is spent the way budgets are.** It is consumed by \
         things going slowly: a project that misses its rate, a test campaign that has to \
         be repeated, a member of staff who leaves halfway through a piece of work. Each of \
         those pushes the target out, and the target moving out is slack being spent.\n\n\
         Slack can be bought back - by adding people, by starting earlier, by scoping the \
         work down - and the player buys it at a price in money and in time. A programme \
         with no slack left is not a programme that is running well; it is a programme \
         where the next thing that goes wrong is a missed commitment.\n\n\
         **This is the resource the whole strategic layer is about.** A player who is rich \
         in slack can absorb a bad month; a player who is not cannot. The forecast shows \
         it directly: a commitment with no slack left is drawn in red, because from that \
         moment the promise is only as good as the next thing that goes right.\n\n\
         Slack is also the thing the world takes away. The eight weighting contexts are all \
         ways of spending it - a tired team, a broken facility, a programme with nothing \
         left - and a programme with no slack raises the odds of an event landing on it."
    ),
    topic!(
        "programme.promise",
        Section::Programmes,
        "Promises and political commitments",
        "Promising something raises the state's approval. Breaking it costs more than it earned.",
        "A promise is a commitment the agency makes to the state, usually for something \
         visible. Making one is worth +4 points of approval. Breaking one is worth \
         -14.\n\n\
         **The asymmetry is the point.** Promising is nearly free and delivering is \
         expensive, and a state that has been promised a great deal is a state that will be \
         disappointed often. The player who promises everything gets an approval score that \
         says nothing and a political position that is hollow.\n\n\
         A promise is also a scheduling claim: it goes on the same calendar as everything \
         else, and it is resolved at a definite instant, and the resolution is scored.\n\n\
         **Approval is not the same as goodwill.** Approval is the state's opinion of the \
         work, and it is what scales the appropriation - so a promise that raises approval \
         raises the money the agency gets. Goodwill is the state's patience with the \
         director, and it is what makes the next order slower and dearer. A player who \
         keeps promises gets both; a player who only wants the appropriation can have \
         one without the other."
    ),
    topic!(
        "programme.milestone",
        Section::Programmes,
        "Milestones",
        "The waypoints on the way to an objective. They are not deadlines.",
        "A milestone is a waypoint: something the programme has to pass on the way to \
         finishing, with a date on it and a flag that says whether it has been reached.\n\n\
         **A milestone is not a commitment.** Nobody outside the agency was promised it, and \
         missing one does not cost approval or goodwill. It is the agency's own note to \
         itself about how the work is going.\n\n\
         It is still worth having, for two reasons. It is how the player says *this part \
         first, that part later* in a way the rest of the game understands, and **a \
         milestone falling due inside the month raises the odds of an event landing on the \
         programme** - the world's weighting reads it, because a programme with a \
         waypoint coming up is a programme under pressure.\n\n\
         The distinction the player should hold on to: the target is when the agency \
         expects to finish, the commitment is when it promised to finish, and a milestone \
         is a waypoint in between that nobody else knows about."
    ),
    topic!(
        "programme.order",
        Section::Programmes,
        "The order things land in",
        "Everything that happens at a definite time is on one schedule, and the order is fixed.",
        "Everything with a definite time goes on one scheduler, and the scheduler has a \
         total order: the time, then a priority, then the sequence number within that \
         instant.\n\n\
         The order of priorities matters. The month boundary is first, then commitments, then \
         orders, then missions, then projects, then attrition, and then events last.\n\n\
         **Events resolve last within an instant**, so they react to the work that has just \
         happened. A mission that lands in the same instant as a project completing sees the \
         completed project, and a world event that fires after an order has landed sees the \
         order. This is what makes the history read correctly: the record of an instant is a \
         sequence, not a set.\n\n\
         **This is also why an order has a lead time at all.** The work is scheduled for a \
         definite instant in the future, and the world is scheduled around it. A player who \
         gives an order that lands in three weeks is making a claim about the state of the \
         world in three weeks, and the game holds them to it."
    ),
    topic!(
        "programme.roles",
        Section::Programmes,
        "Roles within a programme",
        "A programme has a lead and members. The lead is what makes the support factor work.",
        "A person on a programme has a role: lead, member, or adviser. The lead is what the \
         other members work around, and **the support factor - how well the team hangs \
         together - is largely a function of having one.**\n\n\
         A programme with no lead has a poor support factor, which is one of the eight \
         multipliers, which means everything else the programme does is worth less than it \
         looks. Putting a capable manager on a programme is the cheapest performance \
         improvement in the game.\n\n\
         **The adviser is the odd one out.** An adviser does not carry the team and does \
         not add to it, but counts the same as a member for the support calculation - a \
         person who is on the team but not doing the work still keeps the team together. \
         It is the cheapest way to put somebody's name on a programme without taking them \
         off what they are really doing."
    ),
    topic!(
        "programme.audience",
        Section::Programmes,
        "Who holds the deadline",
        "The promise was made to somebody, and different somebodies are differently forgiving.",
        "Every deadline has an audience - who it was promised to - and the audience is \
         what decides what breaking it costs.\n\n\
         | Audience | Missed: approval | Missed: goodwill | Met |\n|---|---|---|---|\n\
         | the agency | nothing | nothing | +8 |\n\
         | a contractor | -9 | -4 | +8 |\n\
         | the state | **-18** | -8 | +8 |\n\
         | the public | **-18** | -8 | +8 |\n\
         | the regulator | **-18** | -8 | +8 |\n\n\
         **A deadline met is worth 8 points of approval, whoever it was to.** And an \
         internal deadline that slips costs nothing at all outside the agency, which is the \
         difference between a plan that was wrong and a promise that was broken.\n\n\
         The audience is therefore the most important thing the player chooses when they \
         set an objective. An internal objective is free to miss and is worth nothing to \
         hit; a political one is worth eight points and costs eighteen if it slips. The \
         regulator is the sharpest of them - a missed regulatory deadline is not a \
         political embarrassment, it is a serious one, and it is recorded as such.\n\n\
         Postponing costs in proportion: 40 political capital for a political deadline, \
         20 for a contractual one, 5 for the agency's own."
    ),
    topic!(
        "programme.categories",
        Section::Programmes,
        "The eight categories",
        "What a programme is for, and why the state reads the portfolio rather than one programme.",
        "**Every programme is in one of eight categories**, and the category is what the \
         state is being asked to fund:\n\n\
         | Category | What it is for |\n|---|---|\n\
         | Orbital | getting above the atmosphere and staying there |\n\
         | Lunar | going to the Moon |\n\
         | Deep space | going further |\n\
         | Earth observation | watching the homeworld |\n\
         | Military reconnaissance | watching other nations |\n\
         | Commercial launch | selling services |\n\
         | Space station | staying somewhere |\n\
         | Resource extraction | digging things up |\n\n\
         **The state reads the portfolio, not the programme.** A player with three orbital \
         programmes and one lunar programme is making a statement about the agency, and the \
         approval that statement earns is the approval of the agency rather than of any one \
         of its pieces. The category is therefore a slow, expensive decision: it is the \
         kind of thing a player sets in the first year and explains to the state for the \
         next thirty.\n\n\
         **And the category sets what the programme can lead to.** A programme is a body of \
         work, and its objectives are the dated pieces of it - and it is the objectives \
         that the state scores, the deadlines that are kept or missed, and the approval that \
         follows from them."
    ),
];
