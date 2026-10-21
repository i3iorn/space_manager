//! Every tunable number in the game.
//!
//! One file, named constants, no magic numbers anywhere else. A balance change
//! is made here, measured with the harness, and documented in `docs/BALANCE.md`.
//! See `docs/BALANCE.md` §1.

use domain::facility::FacilityType;
use domain::money::Money;
use domain::performance::{
    FULL_SCALE_BP, Modifier, ModifierKind, STAGE_CEILING_BP, STAGE_FLOOR_BP,
};
use domain::person::Role;
use domain::scale::PERCENT_UNITS;

/// The salary a new member of staff is offered, by role, in credits per Instant.
pub const STARTING_SALARY: [(Role, i64); 5] = [
    (Role::Astronaut, 1_100),
    (Role::Scientist, 1_200),
    (Role::Engineer, 1_300),
    (Role::Administrator, 900),
    (Role::Manager, 1_100),
];

/// The starting salary for a role.
#[must_use]
pub fn starting_salary(role: Role) -> Money {
    let credits = STARTING_SALARY
        .iter()
        .find(|(candidate, _)| *candidate == role)
        .map_or(1_000, |(_, credits)| *credits);
    Money::from_credits(credits)
}

/// The salary at which a hire is at the top of their band, by role.
///
/// **A hire's lead time scales with the seniority of the role being filled**,
/// so the agency cannot buy a division head in a fortnight.
pub const SENIOR_SALARY: [(Role, i64); 5] = [
    (Role::Astronaut, 2_600),
    (Role::Scientist, 2_600),
    (Role::Engineer, 2_900),
    (Role::Administrator, 2_000),
    (Role::Manager, 2_400),
];

/// The senior salary for a role.
#[must_use]
pub fn senior_salary(role: Role) -> Money {
    let credits = SENIOR_SALARY
        .iter()
        .find(|(candidate, _)| *candidate == role)
        .map_or(2_000, |(_, credits)| *credits);
    Money::from_credits(credits)
}

/// A hire takes two weeks at the entry salary, and six weeks at the top of the
/// band.
pub const HIRE_LEAD_TIME_MIN_DAYS: u64 = 14;

/// A hire takes six weeks at the top of the band.
pub const HIRE_LEAD_TIME_MAX_DAYS: u64 = 42;

/// Putting someone on something takes between one and three days.
pub const ASSIGN_LEAD_TIME_MIN_DAYS: u64 = 1;

/// Putting someone on something takes up to three days.
pub const ASSIGN_LEAD_TIME_MAX_DAYS: u64 = 3;

/// Starting a research project takes one to two weeks of set-up.
pub const START_PROJECT_LEAD_TIME_MIN_DAYS: u64 = 7;

/// Starting a research project takes up to two weeks of set-up.
pub const START_PROJECT_LEAD_TIME_MAX_DAYS: u64 = 14;

/// A test campaign takes a week to arrange.
pub const START_TEST_CAMPAIGN_LEAD_TIME_DAYS: u64 = 7;

/// Publishing a result takes a week of peer review.
pub const PUBLISH_LEAD_TIME_DAYS: u64 = 7;

/// Changing a policy takes a week of bureaucracy.
pub const SET_POLICY_LEAD_TIME_DAYS: u64 = 7;

/// A course takes three to nine months.
///
/// The middle of the range, because the course itself is free text and there
/// is nothing to scale it by yet.
pub const TRAIN_LEAD_TIME_MONTHS: u64 = 6;

/// The months between promotion rounds.
pub const PROMOTION_ROUND_MONTHS: u64 = 3;

/// A rocket takes between sixty and ninety minutes to reach a stable orbit.
pub const FLIGHT_LEAD_TIME_MINUTES: u64 = 60;

/// A rocket takes up to ninety minutes to reach a stable orbit.
pub const FLIGHT_LEAD_TIME_MAX_MINUTES: u64 = 90;

/// The goodwill the agency regains each month, in scaled units.
///
/// **Goodwill regenerates slowly** - three percent a month, up to a cap of a
/// hundred - so a director who spends it can earn it back, but not quickly.
pub const GOODWILL_REGENERATION_PER_MONTH: i64 = 3 * PERCENT_UNITS as i64;

/// The goodwill a broken promise costs.
///
/// **A broken promise costs more than a week of ordinary orders.**
pub const GOODWILL_BROKEN_PROMISE: i16 = 10;

/// The goodwill a kept promise grants.
pub const GOODWILL_KEPT_PROMISE: i16 = 1;

/// The goodwill a successful mission grants.
pub const GOODWILL_MISSION_SUCCESS: i16 = 1;

/// The cap on the goodwill, as a whole percentage.
pub const GOODWILL_CAP: i16 = 100;

/// The goodwill above which the state starts accommodating.
///
/// **Above this, approvals are fast, the state grants favours, the applicant
/// pool is deeper, and staff forgive more.**
pub const GOODWILL_ACCOMMODATING: i16 = 40;

/// The goodwill below which the state starts obstructing.
///
/// **Below this, every order takes longer to land, the costs rise, the state
/// pushes back, and the staff disengage.**
pub const GOODWILL_OBSTRUCTING: i16 = -40;

/// The goodwill below which the state begins to reconsider the agency's
/// mandate.
pub const GOODWILL_MANDATE: i16 = -80;

/// How much the goodwill changes an order's lead time, at the extremes.
///
/// **Half.** A hostile state makes every order take half again as long, and a
/// friendly one makes it take half the time.
pub const GOODWILL_LEAD_TIME_EFFECT_BP: i64 = 5_000;

/// How much the goodwill changes an order's cost, at the extremes.
///
/// **Thirty percent.** An unfriendly state charges a third more, and a friendly
/// one charges a third less.
pub const GOODWILL_COST_EFFECT_BP: i64 = 3_000;

/// How much the goodwill changes the morale a satisfying assignment holds, at
/// the extremes.
///
/// A low goodwill means the staff disengage, and it shows in their morale
/// before it shows anywhere else.
pub const GOODWILL_MORALE_EFFECT_PERCENT: i16 = 20;

/// The goodwill an order costs, by kind./// **Goodwill is the anti-spamming mechanism [D], and it is deliberately not
/// a cap.** There is no limit on how many orders the player may issue; the
/// cost of a decision is financial and political, and that is the point.
/// Hiring someone.
pub const GOODWILL_HIRE: i16 = 1;
/// Letting someone go.
pub const GOODWILL_FIRE: i16 = 4;
/// Moving someone onto something.
pub const GOODWILL_ASSIGN: i16 = 2;
/// Giving someone a step up.
pub const GOODWILL_PROMOTE: i16 = 3;
/// Putting someone through a course.
pub const GOODWILL_TRAIN: i16 = 2;
/// Starting a programme.
pub const GOODWILL_PROGRAMME: i16 = 1;
/// Committing to an objective.
pub const GOODWILL_OBJECTIVE: i16 = 1;
/// Setting a programme's budget.
pub const GOODWILL_BUDGET: i16 = 2;
/// Starting a research project.
pub const GOODWILL_PROJECT: i16 = 1;
/// Starting a test campaign.
pub const GOODWILL_TEST_CAMPAIGN: i16 = 1;
/// Building a small facility.
pub const GOODWILL_FACILITY_SMALL: i16 = 5;
/// Building a standard facility.
pub const GOODWILL_FACILITY_STANDARD: i16 = 8;
/// Building a large facility.
pub const GOODWILL_FACILITY_LARGE: i16 = 12;
/// Building an oversized facility.
pub const GOODWILL_FACILITY_OVERSIZED: i16 = 15;
/// Authorising a launch.
pub const GOODWILL_LAUNCH: i16 = 3;
/// Changing a policy.
pub const GOODWILL_POLICY: i16 = 5;
/// Publishing a result.
pub const GOODWILL_PUBLISH: i16 = 1;

/// The goodwill a facility costs, by its scale.
#[must_use]
pub fn facility_goodwill(scale: domain::facility::BuildScale) -> i16 {
    match scale {
        domain::facility::BuildScale::Small => GOODWILL_FACILITY_SMALL,
        domain::facility::BuildScale::Standard => GOODWILL_FACILITY_STANDARD,
        domain::facility::BuildScale::Large => GOODWILL_FACILITY_LARGE,
        domain::facility::BuildScale::Oversized => GOODWILL_FACILITY_OVERSIZED,
    }
}

/// The cost of a research project each Instant.
pub const RESEARCH_COST_PER_HEAD: i64 = 12_000;

/// The fixed overhead of a research project each Instant.
pub const RESEARCH_OVERHEAD: i64 = 6_000;

/// The cost of a test campaign each Instant, per head.
pub const TEST_COST_PER_HEAD: i64 = 7_000;

/// The fixed overhead of a test campaign each Instant.
pub const TEST_OVERHEAD: i64 = 5_000;

/// What a research project costs each Instant.
#[must_use]
pub fn research_cost_per_month(team_size: usize) -> Money {
    let heads = i64::try_from(team_size).unwrap_or(i64::MAX);
    Money::from_credits(RESEARCH_COST_PER_HEAD * heads + RESEARCH_OVERHEAD)
}

/// What a test campaign costs each Instant.
#[must_use]
pub fn test_cost_per_month(team_size: usize) -> Money {
    let heads = i64::try_from(team_size).unwrap_or(i64::MAX);
    Money::from_credits(TEST_COST_PER_HEAD * heads + TEST_OVERHEAD)
}

/// How much knowledge a completed research project is worth.
pub const KNOWLEDGE_PER_PROJECT: u32 = 120;

/// How many points a research project needs.
///
/// At full performance, a team of four completes a project in six months, and a
/// team of two in twelve. **The project does not get faster because the
/// target changed — it gets faster because more people are working on it.**
/// That is the whole point: the target is fixed, and the rate is what the
/// player's staffing decisions move.
pub const PROJECT_POINTS_TARGET: u32 = 2_400 * PERCENT_UNITS as u32;

/// How many months a research project takes at full performance, for a team of
/// [`PROJECT_FULL_TEAM`].
pub const PROJECT_MONTHS_AT_FULL_TEAM: u32 = 6;

/// The team size at which a project runs at its fastest.
pub const PROJECT_FULL_TEAM: usize = 4;

/// How many months a project takes with the given team, at full performance.
///
/// This is an estimate, and the interface shows it as one.
#[must_use]
pub fn project_duration_months(team_size: usize) -> u32 {
    let heads = u32::try_from(team_size).unwrap_or(1).max(1);
    (PROJECT_POINTS_TARGET / (heads * 100)).max(1)
}

/// How many points a test campaign needs to finish.
///
/// **The points are scaled**, like every other progress quantity in the game,
/// and the target has to be scaled with them. A target of a hundred *unscaled*
/// points against a hundred-and-twenty-eight scaled points a month would be
/// passed in the first month, and the reliability half of the central tension
/// would be free.
pub const TEST_CAMPAIGN_POINTS_TARGET: u32 = 100;

/// How many points each member of a test campaign adds in a month.
///
/// **A campaign of four takes five months**, which is the point: testing has to
/// cost time as well as money, or the player will always test and the research
/// half of the game will be pointless.
pub const TEST_CAMPAIGN_POINTS_PER_HEAD: u32 = 5;

/// How much the state's approval drifts back towards zero in a year.
///
/// **The state's opinion of an agency that has done nothing for a decade is
/// not the opinion it had when the agency was doing something.** A point a
/// year is slow enough that a kept deadline is worth more than a year of
/// drift, and steady enough that a decade of silence costs something.
pub const APPROVAL_DRIFT_PER_YEAR: i16 = PERCENT_UNITS as i16;

/// How much of the agency's prestige decays in a year.
///
/// **Prestige is slow.** It is built by significant achievement and lost a
/// point a year, which is what makes it worth spending.
pub const PRESTIGE_DECAY_PER_YEAR: i16 = PERCENT_UNITS as i16;

/// The chance, in basis points, that a finished project yields a breakthrough.
pub const BREAKTHROUGH_CHANCE_BP: u32 = 3_500;

/// The base cost of each facility type, in credits.
pub const FACILITY_BASE_COST: [(FacilityType, i64); 4] = [
    (FacilityType::Laboratory, 350_000),
    (FacilityType::TestStand, 600_000),
    (FacilityType::FabricationPlant, 800_000),
    (FacilityType::LaunchComplex, 1_200_000),
];

/// The base monthly upkeep of each facility type, in credits.
pub const FACILITY_BASE_UPKEEP: [(FacilityType, i64); 4] = [
    (FacilityType::Laboratory, 4_000),
    (FacilityType::TestStand, 6_000),
    (FacilityType::FabricationPlant, 8_000),
    (FacilityType::LaunchComplex, 14_000),
];

/// The base build time of each facility type, in months.
pub const FACILITY_BASE_MONTHS: [(FacilityType, u32); 4] = [
    (FacilityType::Laboratory, 15),
    (FacilityType::TestStand, 21),
    (FacilityType::FabricationPlant, 24),
    (FacilityType::LaunchComplex, 30),
];

/// The base cost of a facility, in credits.
#[must_use]
pub fn facility_base_cost(kind: FacilityType) -> Money {
    let credits = FACILITY_BASE_COST
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map_or(0, |(_, credits)| *credits);
    Money::from_credits(credits)
}

/// The base monthly upkeep of a facility, in credits.
#[must_use]
pub fn facility_base_upkeep(kind: FacilityType) -> Money {
    let credits = FACILITY_BASE_UPKEEP
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map_or(0, |(_, credits)| *credits);
    Money::from_credits(credits)
}

/// The base build time of a facility, in months.
#[must_use]
pub fn facility_base_months(kind: FacilityType) -> u32 {
    FACILITY_BASE_MONTHS
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map_or(12, |(_, months)| *months)
}

/// Fatigue above this is "very tired".
pub const FATIGUE_HIGH: u16 = 70 * PERCENT_UNITS as u16;

/// Fatigue above this is "about to break".
pub const FATIGUE_SEVERE: u16 = 90 * PERCENT_UNITS as u16;

/// Loyalty below this makes a resignation likely.
pub const LOYALTY_LOW: i16 = -50 * PERCENT_UNITS as i16;

/// The approval change for a kept promise.
/// The approval change when a committed deadline is met.
pub const APPROVAL_DEADLINE_MET: i16 = 8 * PERCENT_UNITS as i16;

/// The approval change when a committed deadline is missed.
pub const APPROVAL_DEADLINE_MISSED: i16 = -18 * PERCENT_UNITS as i16;

/// The approval change when a promise is kept.
pub const APPROVAL_PROMISE_KEPT: i16 = 4 * PERCENT_UNITS as i16;

/// The approval change when a promise is broken.
///
/// **A broken promise costs more than three times what a kept one rewards.** The
/// asymmetry is the point: the state remembers failures more than it rewards
/// successes.
pub const APPROVAL_PROMISE_BROKEN: i16 = -14 * PERCENT_UNITS as i16;

/// How much fatigue a person accrues each Instant on a full assignment.
///
/// Roughly 6% a Instant, so a person who works a project for a year without a
/// break is at 72% fatigue — very slow, but not yet broken. A project that
/// takes two years will cost the agency a good engineer unless somebody rests.
pub const FATIGUE_ON_ASSIGNMENT: u16 = 6 * PERCENT_UNITS as u16;

/// How much fatigue a person accrues each Instant with nothing to do.
pub const FATIGUE_UNASSIGNED: u16 = 3 * PERCENT_UNITS as u16;

/// How much fatigue a person recovers each Instant when resting.
pub const FATIGUE_RECOVERY: u16 = 10 * PERCENT_UNITS as u16;

/// The lightest shift a person can be put on, as a percentage.
///
/// **Zero is the same as being idle**, and it is how a person is parked on a
/// team without being taken off it.
pub const SHIFT_MIN_PERCENT: u32 = 0;

/// The hardest shift a person can be put on, as a percentage.
pub const SHIFT_MAX_PERCENT: u32 = 200;

/// The step the shift moves by, as a percentage.
pub const SHIFT_STEP_PERCENT: u32 = 10;

/// How much of the effort above a full load turns into output, in basis points.
///
/// **Overtime is never free.** At a hundred and fifty percent the person
/// produces thirty percent more and tires fifty percent more, and the extra
/// fatigue is multiplied into every other factor. A player who pushes a team
/// past a full load is buying a little speed with a lot of wear.
pub const SHIFT_OVERTIME_OUTPUT_BP: i64 = 6_000;

/// How much a person working above a full load is less satisfied, in scaled
/// percent.
///
/// **Overtime is not only tiring.** It is also miserable, and the morale it
/// costs is a morale the agency will pay for later.
pub const SHIFT_OVERTIME_MORALE_PERCENT: i16 = 15;

/// How long a shift change takes to land, in hours.
///
/// **One hour**, because a shift change is a rostering decision and not a
/// project. It lands at the next point on the integration grid, which is as
/// quick as a thing can land without the order model being a lie.
pub const SHIFT_LEAD_TIME_HOURS: u64 = 1;

/// The political capital postponing a deadline costs, by audience.
///
/// **The state does not forgive a broken promise for nothing, and it forgives
/// an internal deadline most easily.** The cost is the price of asking.
pub const POSTPONE_POLITICAL_CAPITAL: i64 = 40;
/// The political capital postponing a contractual deadline costs.
pub const POSTPONE_CONTRACTUAL_CAPITAL: i64 = 20;
/// The political capital postponing an internal deadline costs.
pub const POSTPONE_INTERNAL_CAPITAL: i64 = 5;

/// The goodwill postponing a deadline costs, by audience.
pub const GOODWILL_POSTPONE_POLITICAL: i16 = -8;
/// The goodwill postponing a contractual deadline costs.
pub const GOODWILL_POSTPONE_CONTRACTUAL: i16 = -4;
/// The goodwill postponing an internal deadline costs.
pub const GOODWILL_POSTPONE_INTERNAL: i16 = -1;

/// How long a renegotiation takes, in days.
///
/// **A week**, because a renegotiation is a conversation and not a decision.
pub const POSTPONE_LEAD_TIME_DAYS: u64 = 7;

/// How much a facility's condition falls each month when it is short of staff.
///
/// **A facility nobody is looking after falls apart**, and the wear is the
/// point: the upkeep is the money and the staff are the maintenance, and
/// paying for neither is how a laboratory ends up useless.
pub const FACILITY_DECAY_PER_MONTH: u16 = 4 * PERCENT_UNITS as u16;

/// How much of the agency's industrial capacity is lost when the work exceeds
/// it, in basis points.
///
/// **Over capacity is a real constraint.** Every member of staff working at
/// once needs a place to work, and the facilities are the places.
pub const CAPACITY_OVERLOAD_TOOLS_BP: i64 = 5_000;
/// How long a design takes, in days.
///
/// **A design is a drawing**, and a drawing takes an afternoon. The cost is
/// the money and the time is the building.
pub const DESIGN_LEAD_TIME_DAYS: u64 = 3;

/// How long it takes to build a vehicle, in months, per million credits.
///
/// **A vehicle takes real time**, and the time is its cost: a cheap sounding
/// rocket is built in a month and an orbital launcher takes a year.
pub const VEHICLE_BUILD_MONTHS_PER_MILLION: i64 = 3;

/// The approval a successful launch earns.
pub const APPROVAL_LAUNCH_SUCCEEDS: i16 = 5 * PERCENT_UNITS as i16;

/// The approval a partly successful launch earns.
pub const APPROVAL_LAUNCH_PARTIAL: i16 = -4 * PERCENT_UNITS as i16;

/// The approval a failed launch costs.
pub const APPROVAL_LAUNCH_FAILS: i16 = -12 * PERCENT_UNITS as i16;

/// How likely a launch is to work, in basis points, before the vehicle's
/// reliability is taken into account.
pub const MISSION_BASE_PROBABILITY_BP: u32 = 9_000;

/// The goodwill a design costs.
///
/// **Designing is the agency choosing to spend**, and the state is pleased
/// with an agency that is building something.
pub const GOODWILL_DESIGN: i16 = 2;
/// The points an objective needs, at `256` units per point.
///
/// **An objective is a smaller thing than a research project**, and it is
/// finished by a programme's staff rather than by a project team.
pub const OBJECTIVE_POINTS_TARGET: u32 = 600 * PERCENT_UNITS as u32;
/// How fast an unhappy person's morale falls each Instant, on a scale of 0-100.
pub const MORALE_DECAY: i16 = 4 * PERCENT_UNITS as i16;

/// How fast a content person's morale rises each Instant, on a scale of 0-100.
pub const MORALE_RECOVERY: i16 = 2 * PERCENT_UNITS as i16;

/// The morale a person holds when their assignment suits them.
pub const MORALE_TARGET_SATISFIED: i16 = 30 * PERCENT_UNITS as i16;

/// The morale a person holds when their assignment does not suit them.
pub const MORALE_TARGET_UNSATISFIED: i16 = -25 * PERCENT_UNITS as i16;

/// How quickly health recovers, on a scale of 0-100.
pub const HEALTH_RECOVERY: u16 = 2 * PERCENT_UNITS as u16;

/// The health below which a person is considered unfit for duty.
pub const HEALTH_UNFIT: u16 = 40 * PERCENT_UNITS as u16;

/// The monthly state appropriation per member of staff, in credits.
///
/// **The state funds the agency in proportion to what it employs.** A fixed
/// allocation would make a small agency absurdly rich and a large one
/// bankrupt, and neither is true.
pub const APPROPRIATION_PER_HEAD: i64 = 1_900;

/// The smallest allocation the state will make, in credits per Instant.
///
/// A young agency with three scientists cannot ask the state for a budget sized
/// for a division. The floor is low enough that such an agency can afford
/// roughly one researcher, and hiring a second puts it on a clock.
pub const APPROPRIATION_FLOOR: i64 = 12_000;

/// How much political standing moves the appropriation, in basis points per
/// point of approval.
///
/// Approval of +100 multiplies the allocation by 1.5; approval of -100 halves
/// it. That is the scale `docs/BALANCE.md` §9.2 describes.
pub const APPROPRIATION_POLITICAL_SCALE_BP: i64 = 50;

/// The ordinary chance, in basis points, that a person leaves each month.
///
/// **Six percent a year**, which is the ordinary turnover of an institution. It
/// applies to everyone: a healthy agency still loses people, and the cost of
/// replacing them is part of what it means to run one.
pub const RESIGNATION_BASE_CHANCE_BP: u32 = 50;

/// The chance, in basis points, that a person who is *at their limit* leaves
/// each month.
///
/// **This is the heavy rate.** The people who are miserable **and** exhausted
/// are the ones who walk, and they walk quickly. A base rate alone would make
/// the at-the-limit case almost invisible, and it is the case the player can
/// see coming and act on.
pub const RESIGNATION_LIMIT_CHANCE_BP: u32 = 2_500;

/// The multiplier on the resignation chance when a person's morale is at the
/// floor.
pub const RESIGNATION_LOW_MORALE_MULTIPLIER: u32 = 3;

/// The multiplier on the resignation chance when a person's loyalty is at the
/// floor.
pub const RESIGNATION_LOW_LOYALTY_MULTIPLIER: u32 = 2;

/// The divisor on the resignation chance when a person's loyalty is high.
///
/// A person who is loyal to the agency is half as likely to walk.
pub const RESIGNATION_HIGH_LOYALTY_DIVISOR: u32 = 2;

/// The morale at or below which a person is miserable enough to leave.
pub const RESIGNATION_LOW_MORALE: i16 = -30 * PERCENT_UNITS as i16;

/// The fatigue at or above which a person is exhausted.
pub const RESIGNATION_HIGH_FATIGUE: u16 = 70 * PERCENT_UNITS as u16;

/// The loyalty at or above which a person is loyal to the agency.
pub const LOYALTY_HIGH: i16 = 30 * PERCENT_UNITS as i16;

/// How many days between event checks.
///
/// The base interval is a week. Most checks produce nothing - the agency is
/// usually quiet - and the interval tightens under stress.
pub const EVENT_CHECK_INTERVAL_DAYS: u64 = 7;

/// The courses a person can be sent on.
///
/// **A fixed list, because the interface has no text field.** The set is small
/// and each course teaches something the modifiers notice, so the choice
/// matters.
pub const COURSES: [&str; 4] = [
    "Advanced propulsion theory",
    "Orbital mechanics",
    "Life-support systems",
    "Public speaking",
];

/// The saturation constant of the modifier curve.
///
/// The curve is `factor = 1 + (v/100) × (1 - v/100)`, which is a parabola that
/// flattens as `v` approaches 100. The effect is that a large bonus helps less
/// and less, which is what stops the player from solving staffing with one
/// input.
pub const MODIFIER_SATURATION_DENOMINATOR: u32 = 100;

/// The composite a **good** team produces, as a product of eight basis-point
/// factors.
///
/// This is the reference the composite is divided by, so that a team of
/// competent, rested people with adequate tools reads as 100%. Everything else
/// is relative to that: a tired team reads at 50%, an exhausted one at 30%.
///
/// It is written as the product of one good team's factors rather than as a
/// bare number, so that the reference is self-explaining: change a factor's
/// curve and the reference moves with it.
///
/// The good team is: skill +13, role fit +10, morale +10, support +10, tools
/// +5, fatigue 0, creativity +13, health +15.
pub const REFERENCE_COMPOSITE: u128 = {
    let mut product: u128 = 1;
    product *= modifier_factor_bp(13) as u128;
    product *= modifier_factor_bp(10) as u128;
    product *= modifier_factor_bp(10) as u128;
    product *= modifier_factor_bp(10) as u128;
    product *= modifier_factor_bp(5) as u128;
    product *= modifier_factor_bp(0) as u128;
    product *= modifier_factor_bp(13) as u128;
    product *= modifier_factor_bp(15) as u128;
    product
};

/// The cost of a modifier at its worst, in basis points.
#[must_use]
pub fn modifier_cost_bp(value: i16) -> u32 {
    if value >= 0 {
        return 0;
    }
    let penalty = (-value) as u32;
    penalty * (FULL_SCALE_BP - penalty) / MODIFIER_SATURATION_DENOMINATOR
}

/// The factor a modifier contributes, in basis points.
///
/// Symmetric, saturating, and floored: a modifier of -100 still leaves the
/// stage at a quarter of its maximum, because a team with one exhausted member
/// is very bad rather than non-existent.
#[must_use]
pub const fn modifier_factor_bp(value: i16) -> u32 {
    let magnitude = value.unsigned_abs() as u32;
    let gain = magnitude * (FULL_SCALE_BP - magnitude) / MODIFIER_SATURATION_DENOMINATOR;
    let factor = if value >= 0 {
        FULL_SCALE_BP + gain
    } else {
        FULL_SCALE_BP.saturating_sub(gain)
    };
    if factor < STAGE_FLOOR_BP {
        STAGE_FLOOR_BP
    } else if factor > STAGE_CEILING_BP {
        STAGE_CEILING_BP
    } else {
        factor
    }
}

/// The modifier for a person's raw ability, from an attribute on 0-100.
///
/// The scale is centred: an average attribute gives no bonus, a very good one
/// gives a good deal, and a poor one is a real handicap.
#[must_use]
pub fn skill_modifier(attribute: u8) -> Modifier {
    let value = i16::from(attribute) - 50;
    Modifier::new(ModifierKind::Skill, value)
}

/// The modifier for how well a person's specialisms suit the work.
///
/// A person with the right specialism is worth having; a person with the wrong
/// one is a real drag on a project, but not a catastrophe.
#[must_use]
pub fn role_fit_modifier(matches: usize, has_any: bool) -> Modifier {
    if !has_any {
        return Modifier::new(ModifierKind::RoleFit, -25);
    }
    let value = 20 + i16::try_from(matches).unwrap_or(0) * 10;
    Modifier::new(ModifierKind::RoleFit, value.min(30))
}

/// The modifier for a person's morale.
#[must_use]
pub fn morale_modifier(morale: i16) -> Modifier {
    // The morale is a scaled percentage; the modifier is a ratio of -100..+100.
    Modifier::new(ModifierKind::Morale, morale / (2 * PERCENT_UNITS as i16))
}

/// The modifier for a person's health.
#[must_use]
pub fn health_modifier(health: u16) -> Modifier {
    let percent = i16::try_from(health / PERCENT_UNITS as u16).unwrap_or(50);
    Modifier::new(ModifierKind::Health, (percent - 50) / 2)
}

/// The modifier for a person's fatigue. Never positive.
#[must_use]
pub fn fatigue_modifier(fatigue: u16) -> Modifier {
    let percent = i16::try_from(fatigue / PERCENT_UNITS as u16).unwrap_or(100);
    Modifier::new(ModifierKind::Fatigue, -percent)
}

/// The modifier for the support a person gets from their team.
#[must_use]
pub fn support_modifier(team_size: usize, lead_present: bool) -> Modifier {
    let from_size = i16::try_from(team_size).unwrap_or(0).clamp(0, 8) * 3;
    let from_lead = if lead_present { 5 } else { -10 };
    Modifier::new(
        ModifierKind::Support,
        (from_size + from_lead).clamp(-30, 25),
    )
}

/// The modifier for the quality of a person's tools.
#[must_use]
pub fn tools_modifier(quality: u8) -> Modifier {
    Modifier::new(ModifierKind::Tools, (i16::from(quality) - 50) / 2)
}

/// The modifier for a person's creativity.
#[must_use]
pub fn creativity_modifier(creative: u8) -> Modifier {
    Modifier::new(ModifierKind::Creativity, (i16::from(creative) - 50) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_neutral_modifier_leaves_the_scale_alone() {
        assert_eq!(modifier_factor_bp(0), FULL_SCALE_BP);
    }

    #[test]
    fn the_modifier_curve_saturates() {
        // A bonus of +100 helps, but a bonus of +50 is more than half as
        // valuable: the curve flattens.
        let hundred = modifier_factor_bp(100) - FULL_SCALE_BP;
        let fifty = modifier_factor_bp(50) - FULL_SCALE_BP;
        assert!(hundred > fifty);
        assert!(fifty * 2 > hundred);
    }

    #[test]
    fn the_worst_modifier_is_floored_not_zero() {
        assert_eq!(modifier_factor_bp(-100), STAGE_FLOOR_BP);
    }

    #[test]
    fn the_curve_is_symmetric() {
        assert_eq!(
            modifier_factor_bp(60) - FULL_SCALE_BP,
            FULL_SCALE_BP - modifier_factor_bp(-60)
        );
    }

    #[test]
    fn research_scales_with_the_team() {
        // The overhead is fixed, so the *marginal* cost of one more researcher
        // is exactly the per-head cost.
        let two = research_cost_per_month(2);
        let three = research_cost_per_month(3);
        let four = research_cost_per_month(4);
        assert_eq!(four.minor() - two.minor(), 2 * RESEARCH_COST_PER_HEAD * 100);
        assert!(three > two);
    }

    #[test]
    fn testing_is_cheaper_than_research() {
        assert!(test_cost_per_month(3) < research_cost_per_month(3));
    }

    #[test]
    fn a_bigger_team_finishes_sooner() {
        assert!(project_duration_months(8) < project_duration_months(1));
    }
}
