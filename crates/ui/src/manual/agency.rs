//! The people.
//!
//! **The agency is the people in it.** Everything the agency can do, it does
//! through somebody, and the numbers on the staff screen are the numbers that
//! decide whether a project finishes on time.
//!
//! The topics here are written to be *read*, not skimmed: each one says what
//! the thing is, what it does to the numbers, how it ties into the systems
//! either side of it, and what the player can do about it. A player who
//! understands these eighteen topics understands the whole game.

use crate::manual::{Section, Topic, topic};

/// The topics about the people.
pub const TOPICS: &[Topic] = &[
    topic!(
        "agency.factors",
        Section::Agency,
        "The eight factors",
        "What a team's performance is made of. Eight numbers, multiplied together.",
        "**A person's performance is eight factors multiplied together.** Not \
         averaged, not added - multiplied. That single decision is why the game is \
         about one problem at a time rather than about being good at everything.\n\n\
         Each factor runs from about -100 to +100, and each becomes a multiplier \
         through the same curve: a value of zero is exactly neutral, a value of \
         +50 is worth half as much again, and a value of +100 is worth twice as \
         much. The curve flattens as it rises, so a large bonus helps less and \
         less. **You cannot solve staffing with one input.**\n\n\
         | Factor | What it measures | Its range | What moves it |\n\
         |---|---|---|---|\n\
         | **Skill** | raw ability at the work | -50 to +50 | the person's technical attribute |\n\
         | **Role fit** | how well the work suits their specialisms | -25 to +30 | how many of their specialisms match |\n\
         | **Morale** | contentment | -50 to +50 | their morale |\n\
         | **Support** | the team around them | -30 to +25 | the team size, and whether there is a lead |\n\
         | **Tools** | the quality of the facilities | -25 to +25 | the facility's specification and condition |\n\
         | **Fatigue** | tiredness | 0 to -100 | how hard they have been working |\n\
         | **Creativity** | finding a way round the problem | -25 to +25 | their creative attribute |\n\
         | **Health** | physical and mental condition | -25 to +25 | their health |\n\n\
         **The best team in the game reads 100%.** That is the reference: \
         competent, rested people with adequate tools. Everything else is relative \
         to it, so a good team reads above 100% and a tired one reads far below. The \
         interface shows the composite and names the worst factor, because the worst \
         factor is the one to fix.\n\n\
         Two of the eight are penalties and never above neutral - fatigue, and the \
         role fit of somebody with no specialism at all. The other six can be won, \
         and winning them is expensive."
    ),
    topic!(
        "agency.binding",
        Section::Agency,
        "The binding factor",
        "The one thing holding a team back. It is named, and it is the number to fix first.",
        "**This is the most important idea in the game.** A team's performance is not \
         an average of its members and it is not a sum of their good points. It is \
         eight factors multiplied together, and the *smallest* of them decides how \
         fast the team actually works.\n\n\
         Take the reference team - competent, rested, with adequate tools - which \
         reads 100%. Now change one thing:\n\n\
         | One thing changed | The team reads |\n|---|---|\n\
         | Fatigue at 70 | **30.5%** |\n\
         | Nobody in the right field | 68.3% |\n\
         | The worst possible tools | 71.5% |\n\
         | No lead on the team | 86.4% |\n\n\
         **Fatigue at seventy costs more than every other problem in the game put \
         together.** The same team with the worst tools in the world still manages \
         seventy percent; the same team with one member worked to exhaustion manages \
         thirty. The multiplication is the point: a factor of 0.7 takes a third off \
         everything else the team has going for it.\n\n\
         That is why adding a brilliant scientist to a tired team changes nothing. \
         The product is held down by the fatigue no matter who is in the room, and \
         the interface names the factor so the player does not have to find it by \
         experiment.\n\n\
         **Fix the binding factor before anything else.** Every point of the \
         binding factor is worth more than any other improvement to the same team, \
         because it multiplies into all the rest. A player who hires a second team \
         to fix a fatigue problem has spent three times the money and got nothing."
    ),
    topic!(
        "agency.morale",
        Section::Agency,
        "Morale",
        "How content a person is. It costs performance, and a miserable person is a flight risk.",
        "Morale runs from -100 to +100 and is the player's own doing: the work they \
         are given, the hours they are kept, the promises the agency keeps, and the \
         standing the government has in the world.\n\n\
         **It drifts towards a target rather than falling.** Each month it moves \
         two points towards the target if it is below it and four points away if it \
         is above, so a satisfied agency settles at about +30 and a dissatisfied one \
         at about -25. The target moves with goodwill: a hostile state pushes it \
         down by up to twenty points, which is another way of saying that the \
         government would rather the agency were miserable.\n\n\
         Morale feeds into performance directly - it is one of the eight factors - \
         and it feeds into attrition. Below -30, a person's chance of leaving is \
         three times what it otherwise would be. At -100 the work is barely getting \
         done at all.\n\n\
         **It ties into the shift.** Working past a full load costs morale: a person \
         on a hundred and fifty percent loses seven and a half points a month, and \
         one on two hundred loses fifteen. The overtime is not only tiring, it is \
         miserable, and the agency will pay for that later in the form of people \
         leaving.\n\n\
         The two ways to raise morale are the two things a manager can actually do: \
         give people work they are satisfied with, and keep the state friendly."
    ),
    topic!(
        "agency.fatigue",
        Section::Agency,
        "Fatigue",
        "How tired a person is. It is never good news, and it never improves on its own.",
        "Fatigue runs from 0 to 100 and only ever goes up while somebody is working. \
         It is a straight rate, not a compounding one: a person on a full load \
         accrues **6 points a month**, so from nothing to seventy takes about a \
         year. An unassigned person recovers at 10 points a month, and a person on \
         light duties recovers more slowly than that but is still on the team.\n\n\
         Fatigue is a penalty and never a penalty with an upside: at zero it \
         contributes nothing, and at 100 it is at its worst. A fatigued person does \
         not merely work slowly, they work badly - mistakes, rework, the kind of \
         failure that damages a test campaign.\n\n\
         **It is the most expensive factor in the game.** At seventy, fatigue alone \
         takes a good team down to about thirty percent - more than a bad facility, a \
         badly matched team, and a missing lead put together. That is the whole \
         reason the binding factor is worth looking at first.\n\n\
         A person at their limit - morale at or below -30 and fatigue at or above \
         70 - is a quarter of the way to leaving every month, and a sixth of that \
         again if their loyalty is low. Fatigue is therefore a staffing decision \
         before it is a wellbeing one, and **the shift is the throttle that fixes \
         it** without taking the person off the team."
    ),
    topic!(
        "agency.health",
        Section::Agency,
        "Health",
        "A person's physical and mental condition. It is a floor under everything else.",
        "Health runs from 0 to 100 and moves slowly, in both directions. It is not a \
         stat the player sets; it is what happens when the agency works people too \
         hard for too long, and it recovers at two points a month when the agency \
         gives the person room.\n\n\
         Health is one of the eight performance factors, so it multiplies into the \
         team's output like any other. The difference is that health does not recover \
         on its own schedule: it needs the agency to give the person room, which \
         means fewer hours, a lighter assignment, or a spell away from the work.\n\n\
         **Below forty, a person is on light duty whatever their assignment says**, \
         and they recover at ten points a month instead of two. The agency cannot \
         order a sick person to work faster; it can only decide not to make them \
         worse.\n\n\
         Low health also pulls fatigue up, and fatigue pulls attrition up. The chain \
         runs the wrong way, and it runs faster the longer it is left alone. Health \
         is a floor under the whole agency: it is the thing that, once it goes, \
         takes months to put back."
    ),
    topic!(
        "agency.fame",
        Section::Agency,
        "Fame",
        "How well-known a person is. It decides who applies to work for you.",
        "Fame rises when a person does something the world notices - a successful \
         mission, a published result, a project that worked - and it is the reason a \
         good scientist can be attracted to a small agency.\n\n\
         Fame is not a performance factor. It does not make anybody faster. It is a \
         hiring statistic, and it is the reason the agency can be picky about who it \
         takes and the reason it sometimes cannot.\n\n\
         **This is the long game.** A mission that takes three years to prepare can \
         make the agency famous, and the fame then draws better people for the rest \
         of the campaign. Fame is one of the few things in the game that a player can \
         earn early and spend late, and it is why publishing and flying are worth \
         doing even when the immediate result is small."
    ),
    topic!(
        "agency.attributes",
        Section::Agency,
        "The four attributes",
        "Technical, creative, leadership and physical: the four kinds of ability a person has.",
        "Every person has four attributes, each from 0 to 100, and they describe \
         different axes of ability rather than a single quality.\n\n\
         Technical is the ability to do the science and the engineering correctly. \
         Creative is the ability to find a way round a problem that has no obvious \
         way round it. Leadership is the ability to direct other people and to hold \
         a programme together. Physical is the ability to do work that the body does \
         not get to opt out of - flying, field work, and long hours in uncomfortable \
         places.\n\n\
         **Technical and creative are the two that reach the performance model** - \
         technical is the skill factor and creative is the creativity factor, each \
         scaled to at most ±25. Leadership shows up where people are led rather than \
         in the arithmetic, and physical shows up in whether a person can fly a \
         mission at all.\n\n\
         The attribute that matters is the one the work needs. A brilliant scientist \
         is wasted on administration, and a brilliant administrator is wasted on a \
         research project. The role-fit factor in the performance calculation is what \
         measures that mismatch, and it is usually the reason an otherwise capable \
         team is slow.\n\n\
         A technical attribute of 82 - Okonjo, in the opening campaign - is worth a \
         skill factor of +32, and it is one of the reasons she is worth her salary."
    ),
    topic!(
        "agency.specialisms",
        Section::Agency,
        "Specialisms",
        "What a person is especially good at. They fit the work better than their raw ability suggests.",
        "A specialism is a field a person has real experience in - propulsion, life \
         support, exobiology, and so on. It is worth more than raw ability, because a \
         person who has done a thing before does it faster and does it better the \
         second time.\n\n\
         Specialisms raise the role-fit factor, which is one of the eight \
         multipliers on a team's performance. **One matching specialism is worth +20, \
         two are worth the maximum of +30, and a person with none at all is a -25 \
         penalty on every project they are put on.**\n\n\
         So a team of people with the right specialisms is worth more than a team of \
         people who are individually better at everything but who have never done \
         this particular thing. Matching a person to the work is worth more than \
         making them better at it, and it costs nothing.\n\n\
         **Training is the slow way to buy a specialism**, and it is the reason the \
         training order exists: a team short of an exobiologist can either hire one \
         at a premium or put somebody through a six-month course."
    ),
    topic!(
        "agency.roles",
        Section::Agency,
        "The five roles",
        "Scientist, engineer, astronaut, administrator, manager. Each is good at different work.",
        "The agency employs five kinds of person, and the kind decides what the person \
         is useful for.\n\n\
         A scientist studies the natural world and the agency's own technology, and is \
         the backbone of research. An engineer designs and builds, and runs the \
         facilities. An astronaut trains for and flies missions - a small population, \
         and the only role that can be hurt. An administrator keeps the agency \
         running, and is the role that makes a larger agency possible at all. A \
         manager leads people and programmes, and is what a programme runs on.\n\n\
         The role decides the salary, and the salary is a fixed, known number per role \
         and seniority. It is the one cost in the game the player cannot negotiate.\n\n\
         **The roles are not interchangeable.** A research project wants scientists; a \
         test campaign wants engineers and astronauts; a programme wants a manager. \
         Putting a manager on a project does not make the project go faster - it \
         makes the manager's salary do less work, because a manager's own output is \
         not measured in points."
    ),
    topic!(
        "agency.salary",
        Section::Agency,
        "Salary",
        "What a person costs every month. It is fixed by role and seniority, and it never goes down.",
        "Every member of staff costs a salary every month, whether or not they are \
         working on anything. The figure is a function of the role and of how senior \
         the person is, and it is not negotiable - there is no way to haggle with a \
         scientist about their pay.\n\n\
         A new scientist costs **1,200 a month**; an engineer 1,300, an astronaut \
         1,100, a manager 1,100, and an administrator 900. At the top of the band a \
         scientist costs 2,600 and an engineer 2,900. Seniority is bought with money \
         and it is worth it: a team with a senior lead has a better support factor.\n\n\
         **The unassigned are the most expensive people in the agency.** Someone on no \
         assignment is a cost and nothing else; they earn nothing, contribute nothing, \
         and still draw their salary. Every month a person sits idle, the money is \
         gone and nothing is closer to being finished.\n\n\
         Salary is why headcount is a decision. The appropriation the state gives \
         scales with the size of the agency, so growing pays for itself in the state's \
         eyes - but only if the new people are actually working."
    ),
    topic!(
        "agency.assignment",
        Section::Agency,
        "Assignments",
        "What a person is working on. Being assigned and being on the team are different things.",
        "A person is either on a division, on a programme, on a research project, \
         running a facility, on a test campaign, or unassigned. The assignment is what \
         they do all day, and it is what the agency pays them to do.\n\n\
         **Being assigned to something and being on its team list are different \
         things, and the game keeps them in step.** A person moved onto a project is \
         added to that project's team, and a person taken off it is removed. The two \
         coming apart was a bug, and the bug made every staffing decision meaningless: \
         a team that listed somebody who was not there got credit for them.\n\n\
         A person on two teams is working for both, and their performance is counted in \
         both. Spreading a scarce person thin is a legitimate strategy and a bad one, \
         and the binding factor is what tells the player which it has become.\n\n\
         **The assignment also decides the fatigue.** A person on a team accrues \
         fatigue at the ordinary rate; an unassigned person rests. Taking somebody off \
         a project is therefore the crude way to let them recover, and the shift is \
         the precise one."
    ),
    topic!(
        "agency.team",
        Section::Agency,
        "Teams and team size",
        "A project runs at the rate its team can sustain. More people is faster, up to a point.",
        "A research project advances at the rate the people on it can sustain. Their \
         performance is summed into points, and the points are added to the project.\n\n\
         **Adding people makes a project finish sooner and the target never moves.** \
         The target is a fixed 2,400 points, so a bigger team is simply a shorter \
         project: a team of four at full performance finishes in six months, a team \
         of two in twelve, a team of eight in three. That is the whole arithmetic, \
         and it is deliberately simple enough to hold in the head while making a \
         staffing decision under time pressure.\n\n\
         The catch is the coordination cost. A large team supports itself less well \
         than a small one - people in a big team are further from the lead and from \
         each other - so the support factor falls as the team grows. Past a certain \
         size, adding people adds cost and adds nothing to the rate. The support \
         factor is one of the eight, and it is the one that punishes a project that \
         was solved by throwing bodies at it.\n\n\
         **There is one thing a bigger team always fixes, though: a missing lead.** \
         A team with no lead is a -10 on the support factor, and putting anybody in \
         charge of it turns that into +5. The cheapest improvement in the game is to \
         promote somebody who is already there."
    ),
    topic!(
        "agency.shift",
        Section::Agency,
        "The shift",
        "How hard somebody works. A hundred percent is a full load, and it goes to two hundred.",
        "**The shift is the player's throttle on a person.** A hundred percent is a full \
         load. Below it the person contributes less and tires less; above it they \
         contribute more and tire much more, and the extra effort is worth less than \
         the extra fatigue it costs.\n\n\
         | Shift | Output | Fatigue | Morale |\n|---|---|---|---|\n\
         | 0% | nothing | recovers, as if unassigned | unaffected |\n\
         | 50% | half | recovers, while still on the team | unaffected |\n\
         | 100% | full | the ordinary rate | unaffected |\n\
         | 150% | 130% | one and a half times the ordinary rate | -7.5% |\n\
         | 200% | 160% | twice the ordinary rate | -15% |\n\n\
         **The old way of easing off was to unassign somebody**, and that cost the \
         agency the person, their place on the team, and their output all at once. A \
         person on light duties recovers while they are still on the team, and that is \
         the thing an unassignment could never do.\n\n\
         **Overtime is never free.** At a hundred and fifty percent the person produces \
         thirty percent more and tires fifty percent more. And because fatigue is one \
         of the eight factors, the wear is multiplied into every other one: a team \
         pushed from a hundred percent to a hundred and thirty, by a fifteen percent \
         morale loss and a fifty percent rise in fatigue, has gained very little. A \
         player who pushes a team past a full load is buying a little speed with a \
         lot of wear - and the wear is multiplied into everything else.\n\n\
         The shift moves with the left and right arrows on the staff screen, and it is \
         an order like everything else: it lands at the next hour, and it costs no \
         money and no goodwill."
    ),
    topic!(
        "agency.finding",
        Section::Agency,
        "Finding people",
        "Every list filters and sorts. The staff list sorts by who is about to leave.",
        "**An agency with three hundred people has a list that cannot be read top to \
         bottom**, and a player who has to scroll to find the one person who is about \
         to resign has a screen that does not do its job.\n\n\
         Press `/` to filter a list and `s` to change how it is sorted. **Typing into a \
         filter is a mode**: the keys go into the filter and nowhere else, so a filter \
         can contain a `q` without the game quitting. `Enter` keeps the filter; `Esc` \
         abandons it.\n\n\
         The staff list sorts by name, age, role, fatigue, **the chance the person leaves \
         this month**, and shift. The filter survives a trip to another screen and back.\n\n\
         **The risk is the number worth sorting by.** A person about to leave is a \
         person the agency can still save, and a player who cannot see the risk cannot \
         save them. It is also the number that answers a harder question: when a \
         hundred and fifty people are on the payroll, the risk column is the only \
         honest way to see whether the agency is quietly bleeding."
    ),
    topic!(
        "agency.attrition",
        Section::Agency,
        "Attrition",
        "People leave. Slowly, normally - and quickly when they are at their limit.",
        "Every member of staff has a chance of leaving every month. The ordinary rate \
         is 0.5% a month, which is 6% a year - low enough to ignore and high enough \
         that a two-hundred-person agency loses about twelve people a year without \
         anyone noticing.\n\n\
         A person at their limit - morale at or below -30 and fatigue at or above 70 - \
         leaves at 25% a month instead. That is a quarter of the team, every \
         month, and it is the single fastest way to lose a project.\n\n\
         Low morale multiplies the risk by three and low loyalty by two; high loyalty halves \
         it. So the same tired, miserable person is twice as likely to leave from a small \
         agency that cannot afford to replace them as from a large one that can.\n\n\
         **The number on the screen is the number the engine rolls against.** The staff \
         list shows each person's chance of leaving this month, and it is worth sorting \
         by it.\n\n\
         Attrition is the argument for looking at morale and fatigue before looking at \
         the budget. A quarter of the team leaving every month costs far more than the \
         salaries saved, and it costs the knowledge that went with them."
    ),
    topic!(
        "agency.hiring",
        Section::Agency,
        "Hiring",
        "You do not get the person immediately. The order takes two weeks to six weeks to land.",
        "A hire is an order, and an order takes time. The lead time is between \
         fourteen and forty-two days, and it is drawn from the campaign, not from the \
         clock - so it is the same at every dilation and the same every time the same \
         campaign is played.\n\n\
         The candidate is chosen from the people who apply, and who applies depends on the \
         agency's fame and on the salary it offers. A famous agency attracts better people \
         and can be more selective; an unknown one takes what it can get.\n\n\
         A hire also costs goodwill, and a hostile state makes hiring slower and more \
         expensive - the effect is computed when the order is issued, so the player sees the \
         delay and the price before committing to it.\n\n\
         **Hiring is a loan against the future.** The person costs a salary from the day \
         they land, and they produce nothing until they are assigned. A player who hires \
         ahead of the work is carrying a payroll with no output attached to it - which \
         is exactly what the unassigned are, and exactly what the appropriation is \
         supposed to pay for."
    ),
    topic!(
        "agency.promotion",
        Section::Agency,
        "Promotion",
        "A step up the seniority ladder, taken at a round every three months.",
        "Promotion raises a person's salary and their standing, and it is taken at a round \
         every three months rather than on the day the order is issued - the order lands and \
         waits for the round.\n\n\
         Seniority matters because a senior person is worth more and costs more, and because \
         a team with a senior lead has a better support factor than a team without one. It \
         also costs goodwill: the state likes seeing its people rise.\n\n\
         **The cheapest improvement in the game is to promote somebody already on the \
         team.** A team with no lead is a -10 on the support factor and a team with a lead \
         is a +5 - a swing of fifteen points on one of the eight multipliers, for the \
         price of a promotion and three months of waiting. Nothing else in the game is that \
         cheap."
    ),
    topic!(
        "agency.training",
        Section::Agency,
        "Training",
        "A course that takes six months and teaches a person something they did not know.",
        "Training puts a person through a course and teaches them a specialism they do not \
         have. It takes six months - a long time in a campaign that is thirty to sixty years, \
         and a short time in one that is not.\n\n\
         It is the slow way to fix a role-fit problem. Hiring somebody who already knows the \
         work is faster and more expensive; training somebody who almost knows it is slower \
         and cheaper, and the two are not equivalent, because a trained person is still a \
         person the agency has to keep.\n\n\
         **The four courses are propulsion theory, orbital mechanics, life-support systems, \
         and public speaking.** The first three are technical and the last is not: a \
         programme whose manager cannot speak in public is a programme with no approval, \
         and approval is what the appropriation is scaled by.\n\n\
         Training competes for the same people as everything else, and for six months. A \
         person in a course is not on a team, and their salary is being paid for nothing."
    ),
    topic!(
        "agency.refusals",
        Section::Agency,
        "When the agency says no",
        "The refusals, and what each of them means. A refusal is information, and information has no side effects.",
        "**A refused order changes nothing.** Not the state, not the treasury, not the \
         chronicle. The cost is charged when an order lands, and an order that was refused \
         never lands, so a refusal is never paid for.\n\n\
         | What the player asked for | What the agency says |\n|---|---|\n\
         | Somebody who has left | is no longer with the agency |\n\
         | Somebody who is already on something | is already assigned to ... |\n\
         | A programme that does not exist | There is no programme called ... |\n\
         | A facility that does not exist | There is no facility called ... |\n\
         | A vehicle that does not exist | There is no vehicle called ... |\n\
         | A capability the agency has not developed | The agency has not yet developed ... |\n\
         | A cancelled or failed programme | is Cancelled, so that cannot be done now |\n\
         | An order the engine cannot carry out | There is no way to ... yet |\n\n\
         **The check happens twice.** An order is validated when it is given and validated \
         again when it lands, because the world moves on in between: the person may have \
         left, the programme may have been cancelled, the vehicle may have been scrapped. An \
         order that can no longer be honoured is refused *at the landing*, and the player \
         is told in the report.\n\n\
         **A refusal is free information.** It costs nothing, changes nothing, and tells \
         the player that the world has moved on. The player who reads a refusal as a \
         mistake has missed that it is the only way the game can say *not any more*."
    ),
];
