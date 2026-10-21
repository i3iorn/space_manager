//! The people.
//!
//! **The agency is the people in it.** Everything the agency can do, it does
//! through somebody, and the numbers on the staff screen are the numbers that
//! decide whether a project finishes on time.

use crate::manual::{Section, Topic, topic};

/// The topics about the people.
pub const TOPICS: &[Topic] = &[
    topic!(
        "agency.morale",
        Section::Agency,
        "Morale",
        "How content a person is. It costs performance, and a miserable person is a flight risk.",
        "Morale runs from -100 to +100 and is the player's own doing: the work they are \
         given, the hours they are kept, the promises the agency keeps, and the standing \
         the government has in the world.\n\n\
         Morale feeds into performance directly - it is one of the eight factors - and it \
         feeds into attrition. Below -30, a person's chance of leaving is three times what \
         it otherwise would be. At -100 the work is barely getting done at all.\n\n\
         The state also has a morale target, and it moves with goodwill. A hostile state \
         pushes the target down by up to twenty percent, which is another way of saying \
         that the government would rather the agency were miserable."
    ),
    topic!(
        "agency.fatigue",
        Section::Agency,
        "Fatigue",
        "How tired a person is. It is never good news, and it never improves on its own.",
        "Fatigue runs from 0 to 100 and only ever goes up while someone is working. It \
         decays slowly when they are not, and the rate of decay is far slower than the rate \
         of accumulation.\n\n\
         Fatigue is a penalty and never a penalty with an upside: at zero it contributes \
         nothing, and at 100 it is at its worst. A fatigued person does not merely work \
         slowly, they work badly - mistakes, rework, the kind of failure that damages a \
         test campaign.\n\n\
         A person at their limit - morale at or below -30 and fatigue at or above 70 - is \
         a quarter of the way to leaving every month, and a sixth of that again if their \
         loyalty is low. Fatigue is therefore a staffing decision before it is a wellbeing \
         one."
    ),
    topic!(
        "agency.health",
        Section::Agency,
        "Health",
        "A person's physical and mental condition. It is a floor under everything else.",
        "Health runs from 0 to 100 and moves slowly, in both directions. It is not a stat \
         the player sets; it is what happens when the agency works people too hard for too \
         long, and it recovers when it does not.\n\n\
         Health is one of the eight performance factors, so it multiplies into the team's \
         output like any other. The difference is that health does not recover on its own \
         schedule: it needs the agency to give the person room, which means fewer hours, a \
         lighter assignment, or a spell away from the work.\n\n\
         Low health also pulls fatigue up, and fatigue pulls attrition up. The chain runs \
         the wrong way, and it runs faster the longer it is left alone."
    ),
    topic!(
        "agency.fame",
        Section::Agency,
        "Fame",
        "How well-known a person is. It decides who applies to work for you.",
        "Fame rises when a person does something the world notices - a successful \
         mission, a published result, a project that worked - and it is the reason a good \
         scientist can be attracted to a small agency.\n\n\
         Fame is not a performance factor. It does not make anybody faster. It is a hiring \
         statistic, and it is the reason the agency can be picky about who it takes and \
         the reason it sometimes cannot."
    ),
    topic!(
        "agency.attributes",
        Section::Agency,
        "The four attributes",
        "Technical, creative, leadership and physical: the four kinds of ability a person has.",
        "Every person has four attributes, each from 0 to 100, and they describe different \
         axes of ability rather than a single quality.\n\n\
         Technical is the ability to do the science and the engineering correctly. Creative \
         is the ability to find a way round a problem that has no obvious way round it. \
         Leadership is the ability to direct other people and to hold a programme together. \
         Physical is the ability to do work that the body does not get to opt out of - \
         flying, field work, and long hours in uncomfortable places.\n\n\
         The attribute that matters is the one the work needs. A brilliant scientist is \
         wasted on administration, and a brilliant administrator is wasted on a research \
         project. The role-fit factor in the performance calculation is what measures that \
         mismatch, and it is usually the reason an otherwise capable team is slow."
    ),
    topic!(
        "agency.specialisms",
        Section::Agency,
        "Specialisms",
        "What a person is especially good at. They fit the work better than their raw ability suggests.",
        "A specialism is a field a person has real experience in - propulsion, life \
         support, exobiology, and so on. It is worth more than raw ability, because a \
         person who has done a thing before does it faster and does it better the second \
         time.\n\n\
         Specialisms raise the role-fit factor, which is one of the eight multipliers on a \
         team's performance. A team of people with the right specialisms is worth more than \
         a team of people who are individually better at everything but who have never \
         done this particular thing."
    ),
    topic!(
        "agency.roles",
        Section::Agency,
        "The five roles",
        "Scientist, engineer, astronaut, administrator, manager. Each is good at different work.",
        "The agency employs five kinds of person, and the kind decides what the person is \
         useful for.\n\n\
         A scientist studies the natural world and the agency's own technology, and is the \
         backbone of research. An engineer designs and builds, and runs the facilities. An \
         astronaut trains for and flies missions - a small population, and the only role \
         that can be hurt. An administrator keeps the agency running, and is the role that \
         makes a larger agency possible at all. A manager leads people and programmes, and \
         is what a programme runs on.\n\n\
         The role decides the salary, and the salary is a fixed, known number per role and \
         seniority. It is the one cost in the game the player cannot negotiate."
    ),
    topic!(
        "agency.salary",
        Section::Agency,
        "Salary",
        "What a person costs every month. It is fixed by role and seniority, and it never goes down.",
        "Every member of staff costs a salary every month, whether or not they are \
         working on anything. The figure is a function of the role and of how senior the \
         person is, and it is not negotiable - there is no way to haggle with a scientist \
         about their pay.\n\n\
         **The unassigned are the most expensive people in the agency.** Someone on no \
         assignment is a cost and nothing else; they earn nothing, contribute nothing, and \
         still draw their salary. Every month a person sits idle, the money is gone and \
         nothing is closer to being finished.\n\n\
         Salary is why headcount is a decision. The appropriation the state gives scales \
         with the size of the agency, so growing pays for itself in the state's eyes - but \
         only if the new people are actually working."
    ),
    topic!(
        "agency.assignment",
        Section::Agency,
        "Assignments",
        "What a person is working on. Being assigned and being on the team are different things.",
        "A person is either on a division, on a programme, on a research project, running \
         a facility, on a test campaign, or unassigned. The assignment is what they do all \
         day, and it is what the agency pays them to do.\n\n\
         **Being assigned to something and being on its team list are different things, \
         and the game keeps them in step.** A person moved onto a project is added to that \
         project's team, and a person taken off it is removed. The two coming apart was a \
         bug, and the bug made every staffing decision meaningless: a team that listed \
         somebody who was not there got credit for them.\n\n\
         A person on two teams is working for both, and their performance is counted in \
         both. Spreading a scarce person thin is a legitimate strategy and a bad one, \
         and the binding factor is what tells the player which it has become."
    ),
    topic!(
        "agency.team",
        Section::Agency,
        "Teams and team size",
        "A project runs at the rate its team can sustain. More people is faster, up to a point.",
        "A research project advances at the rate the people on it can sustain. Their \
         performance is summed into points, and the points are added to the project.\n\n\
         **Adding people makes a project finish sooner and the target never moves.** The \
         target is a fixed 2,400 points, so a bigger team is simply a shorter project. \
         That is the whole arithmetic, and it is deliberately simple enough to hold in \
         the head.\n\n\
         The catch is the coordination cost. A large team supports itself less well than a \
         small one - people in a big team are further from the lead and from each other - \
         so the support factor falls as the team grows. Past a certain size, adding people \
         adds cost and adds nothing to the rate. The support factor is one of the eight, \
         and it is the one that punishes a project that was solved by throwing bodies at \
         it."
    ),
    topic!(
        "agency.binding",
        Section::Agency,
        "The binding factor",
        "The one thing holding a team back. It is named, and it is the number to fix first.",
        "**This is the most important idea in the game.** A team's performance is not an \
         average of its members and it is not a sum of their good points. It is eight \
         factors multiplied together, and the smallest of them decides how fast the team \
         actually works.\n\n\
         A team with excellent people, a fine facility, and high morale can still be slow, \
         and it will be slow for exactly one reason. That reason is the binding factor, \
         and the interface names it rather than leaving the player to find it by \
         experiment.\n\n\
         The eight factors are skill, role fit, morale, support, tools, fatigue, creativity \
         and health. Three of them are penalties and never above neutral: fatigue, and the \
         health and morale that a person in poor condition drags down. A team held back by \
         fatigue is held back by fatigue, and adding a brilliant scientist to it changes \
         nothing at all - the multiplication is the point.\n\n\
         **Fix the binding factor before anything else.** Every point of the binding factor \
         is worth more than any other improvement to the same team, because it multiplies \
         into all the rest."
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
         thirty percent more and tires fifty percent more. A player who pushes a team \
         past a full load is buying a little speed with a lot of wear - and the wear \
         is multiplied into every other factor, because fatigue is one of the eight.\n\n\
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
         save them."
    ),
    topic!(
        "agency.attrition",
        Section::Agency,
        "Attrition",
        "People leave. Slowly, normally - and quickly when they are at their limit.",
        "Every member of staff has a chance of leaving every month. The ordinary rate is \
         0.5% a month, which is 6% a year - low enough to ignore and high enough \
         that a two-hundred-person agency loses four or five people a year without anyone \
         noticing.\n\n\
         A person at their limit - morale at or below -30 and fatigue at or above 70 - \
         leaves at 25% a month instead. That is a quarter of the team, every \
         month, and it is the single fastest way to lose a project.\n\n\
         Low morale multiplies the risk by three and low loyalty by two; high loyalty halves \
         it. So the same tired, miserable person is twice as likely to leave from a small \
         agency that cannot afford to replace them as from a large one that can.\n\n\
         **The number on the screen is the number the engine rolls against.** The staff \
         list shows each person's chance of leaving this month, and it is worth sorting \
         by it."
    ),
    topic!(
        "agency.hiring",
        Section::Agency,
        "Hiring",
        "You do not get the person immediately. The order takes two weeks to six weeks to land.",
        "A hire is an order, and an order takes time. The lead time is between fourteen and \
         forty-two days, and it is drawn from the campaign, not from the clock - so it is \
         the same at every dilation and the same every time the same campaign is played.\n\n\
         The candidate is chosen from the people who apply, and who applies depends on the \
         agency's fame and on the salary it offers. A famous agency attracts better people \
         and can be more selective; an unknown one takes what it can get.\n\n\
         A hire also costs goodwill, and a hostile state makes hiring slower and more \
         expensive - the effect is computed when the order is issued, so the player sees the \
         delay and the price before committing to it."
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
         also costs goodwill: the state likes seeing its people rise."
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
         person the agency has to keep."
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
         is told in the report."
    ),
];
