//! Programmes, objectives, and promises.
//!
//! **The strategic layer.** A programme is what the agency has committed to
//! doing, and an objective is a dated piece of that. The gap between when the
//! agency would like to finish and when it has promised to finish is the thing
//! the player manages.

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
         much room there is."
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
         finish, and will be judged against."
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
         commitment is the measure of how wrong it is."
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
         the agency cannot deliver."
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
         where the next thing that goes wrong is a missed commitment."
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
         else, and it is resolved at a definite instant, and the resolution is scored."
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
         sequence, not a set."
    ),
    topic!(
        "programme.roles",
        Section::Programmes,
        "Roles within a programme",
        "A programme has a lead and members. The lead is what makes the support factor work.",
        "A person on a programme has a role: lead or member. The lead is what the other \
         members work around, and **the support factor - how well the team hangs together - \
         is largely a function of having one.**\n\n\
         A programme with no lead has a poor support factor, which is one of the eight \
         multipliers, which means everything else the programme does is worth less than it \
         looks. Putting a capable manager on a programme is the cheapest performance \
         improvement in the game."
    ),
    topic!(
        "programme.audience",
        Section::Programmes,
        "Who holds the deadline",
        "The promise was made to somebody, and different somebodies are differently forgiving.",
        "Every deadline has an audience - who it was promised to - and the audience is \
         what decides what breaking it costs.\n\n\
         | Audience | Missing it costs |\n|---|---|\n\
         | the agency | nothing outside the agency. The plan was wrong, and nobody was told |\n\
         | a contractor | some approval, and goodwill |\n\
         | the state | **18 points of approval**, and goodwill |\n\
         | the public | **18 points of approval**, and goodwill |\n\
         | the regulator | **18 points of approval**, and worse |\n\n\
         **A deadline met is worth 8 points of approval, whoever it was to.** And an \
         internal deadline that slips costs nothing at all outside the agency, which is the \
         difference between a plan that was wrong and a promise that was broken."
    ),
];
