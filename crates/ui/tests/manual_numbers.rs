//! The manual's numbers must be the game's numbers.
//!
//! **The prose is written by hand and the balance is written in code**, and a
//! manual that quotes a figure the simulation does not use is worse than no
//! manual at all — the player believes it, and it is wrong.
//!
//! These tests are the guard. If a balance constant moves, they fail, and the
//! failure says which sentence needs rewriting.

use sim::balance;
use ui::manual;

/// The manual's text for a topic, with the line breaks taken out.
fn text(id: &str) -> String {
    let topic = manual::find(id).expect("the topic exists");
    format!("{} {}", topic.summary, topic.body)
}

/// A number in basis points, as a whole percentage.
///
/// **One basis point is a hundredth of a percent**, so 5,000 of them is 50%.
fn percent(basis_points: i64) -> i64 {
    (basis_points + 50) / 100
}

/// A number as the manual writes it, with the thousands separators.
fn thousands(number: i64) -> String {
    let digits = number.abs().to_string();
    let mut out = String::new();
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(character);
    }
    if number < 0 { format!("-{out}") } else { out }
}

/// Whether the text mentions a number as a whole number.
///
/// **A substring match is not a match.** "12,000" contains "2,000", and a
/// guard that thinks it does has no guard in it.
fn mentions(text: &str, number: &str) -> bool {
    let bytes = text.as_bytes();
    let needle = number.as_bytes();
    text.match_indices(number).any(|(index, _)| {
        let before = index
            .checked_sub(1)
            .is_none_or(|i| !bytes[i].is_ascii_digit() && bytes[i] != b',');
        let end = index + needle.len();
        let after = end >= bytes.len() || !bytes[end].is_ascii_digit();
        before && after
    })
}

#[test]
fn the_appropriation_is_quoted_correctly() {
    let text = text("state.appropriation");
    assert!(
        mentions(&text, &thousands(balance::APPROPRIATION_PER_HEAD)),
        "the manual must quote the per-head appropriation: {}",
        balance::APPROPRIATION_PER_HEAD
    );
    assert!(
        mentions(&text, &thousands(balance::APPROPRIATION_FLOOR)),
        "the manual must quote the appropriation floor: {}",
        balance::APPROPRIATION_FLOOR
    );
}

#[test]
fn the_research_costs_are_quoted_correctly() {
    let text = text("state.net");
    assert!(
        mentions(&text, &thousands(balance::RESEARCH_COST_PER_HEAD)),
        "the manual must quote the research cost per head: {}",
        balance::RESEARCH_COST_PER_HEAD
    );
    assert!(
        mentions(&text, &thousands(balance::RESEARCH_OVERHEAD)),
        "the manual must quote the research overhead: {}",
        balance::RESEARCH_OVERHEAD
    );
    assert!(
        mentions(&text, &thousands(balance::TEST_COST_PER_HEAD)),
        "the manual must quote the test cost per head: {}",
        balance::TEST_COST_PER_HEAD
    );
}

#[test]
fn the_project_target_is_quoted_correctly() {
    // **The target never moves**, and the manual says 2,400 points.
    let target = balance::PROJECT_POINTS_TARGET / domain::scale::PERCENT_UNITS as u32;
    let text = text("research.points");
    assert!(
        mentions(&text, &thousands(i64::from(target))),
        "the manual must quote the project target: {target}"
    );
}

#[test]
fn the_lead_times_are_quoted_correctly() {
    // **A lead time is a function of the order, not a dice roll**, and the
    // manual quotes the same numbers the orders use.
    let text = text("order.hire");
    assert!(
        mentions(&text, &balance::HIRE_LEAD_TIME_MIN_DAYS.to_string())
            && mentions(&text, &balance::HIRE_LEAD_TIME_MAX_DAYS.to_string()),
        "the manual must quote the hire lead time: {} to {}",
        balance::HIRE_LEAD_TIME_MIN_DAYS,
        balance::HIRE_LEAD_TIME_MAX_DAYS
    );
}

#[test]
fn the_goodwill_effects_are_quoted_correctly() {
    // **Goodwill is a price, not a cap**, and the effect is zero between the
    // two thresholds and maximum at the extremes.
    let text = text("state.goodwill");
    assert!(
        mentions(&text, &balance::GOODWILL_OBSTRUCTING.to_string()),
        "the manual must quote the point at which the state starts obstructing: {}",
        balance::GOODWILL_OBSTRUCTING
    );
    assert!(
        mentions(
            &text,
            &percent(balance::GOODWILL_LEAD_TIME_EFFECT_BP).to_string()
        ),
        "the manual must quote the lead-time effect: {}%",
        percent(balance::GOODWILL_LEAD_TIME_EFFECT_BP)
    );
    assert!(
        mentions(
            &text,
            &percent(balance::GOODWILL_COST_EFFECT_BP).to_string()
        ),
        "the manual must quote the cost effect: {}%",
        percent(balance::GOODWILL_COST_EFFECT_BP)
    );
}

#[test]
fn the_attrition_rates_are_quoted_correctly() {
    let text = text("agency.attrition");
    assert!(
        mentions(&text, "0.5%"),
        "the manual must quote the ordinary attrition rate"
    );
    assert!(
        mentions(&text, "25%"),
        "the manual must quote the rate at the limit"
    );
}

#[test]
fn the_salaries_are_quoted_correctly() {
    // **The salary is the one cost the player cannot negotiate**, so the
    // manual's figure has to be the figure the agency pays.
    let text = text("agency.salary");
    for role in [
        domain::person::Role::Astronaut,
        domain::person::Role::Scientist,
        domain::person::Role::Engineer,
        domain::person::Role::Administrator,
        domain::person::Role::Manager,
    ] {
        let starting = balance::starting_salary(role).credits();
        assert!(
            mentions(&text, &thousands(starting)),
            "the manual must quote the starting salary of {role:?}: {starting}"
        );
    }
}

#[test]
fn the_facility_costs_are_quoted_correctly() {
    // **A facility is the most consequential order in the game**, and the
    // manual quotes the four of them.
    let text = text("research.facility");
    for kind in [
        domain::facility::FacilityType::Laboratory,
        domain::facility::FacilityType::TestStand,
        domain::facility::FacilityType::FabricationPlant,
        domain::facility::FacilityType::LaunchComplex,
    ] {
        let cost = balance::facility_base_cost(kind).credits();
        let upkeep = balance::facility_base_upkeep(kind).credits();
        let months = u64::from(balance::facility_base_months(kind));
        assert!(
            mentions(&text, &thousands(cost)),
            "the manual must quote the cost of {kind:?}: {cost}"
        );
        assert!(
            mentions(&text, &thousands(upkeep)),
            "the manual must quote the upkeep of {kind:?}: {upkeep}"
        );
        assert!(
            mentions(&text, &months.to_string()),
            "the manual must quote the build time of {kind:?}: {months} months"
        );
    }
}

#[test]
fn the_project_duration_is_quoted_correctly() {
    // **A team of four finishes in six months**, and the manual's table has to
    // agree with the balance the engine uses.
    let text = text("research.points");
    assert!(
        mentions(&text, &balance::PROJECT_FULL_TEAM.to_string()),
        "the manual must quote the full team size: {}",
        balance::PROJECT_FULL_TEAM
    );
    assert!(
        mentions(&text, &balance::PROJECT_MONTHS_AT_FULL_TEAM.to_string()),
        "the manual must quote the months at a full team: {}",
        balance::PROJECT_MONTHS_AT_FULL_TEAM
    );
}

#[test]
fn the_fatigue_rates_are_quoted_correctly() {
    // **Six points a month on a full load, and ten a month of recovery.**
    let text = text("agency.fatigue");
    let working = balance::FATIGUE_ON_ASSIGNMENT / domain::scale::PERCENT_UNITS as u16;
    let recovery = balance::FATIGUE_RECOVERY / domain::scale::PERCENT_UNITS as u16;
    assert!(
        mentions(&text, &working.to_string()),
        "the manual must quote the working fatigue rate: {working}"
    );
    assert!(
        mentions(&text, &recovery.to_string()),
        "the manual must quote the recovery rate: {recovery}"
    );
}

#[test]
fn the_test_campaign_numbers_are_quoted_correctly() {
    // **A hundred points, five a month a head, and five months of four.**
    let text = text("research.test");
    assert!(
        mentions(&text, &balance::TEST_CAMPAIGN_POINTS_TARGET.to_string()),
        "the manual must quote the campaign's target: {}",
        balance::TEST_CAMPAIGN_POINTS_TARGET
    );
    assert!(
        mentions(&text, &balance::TEST_CAMPAIGN_POINTS_PER_HEAD.to_string()),
        "the manual must quote the points a head adds: {}",
        balance::TEST_CAMPAIGN_POINTS_PER_HEAD
    );
}

#[test]
fn the_approval_numbers_are_quoted_correctly() {
    // **A deadline met is worth eight and a deadline missed costs eighteen**,
    // and a promise broken costs fourteen against four for making one.
    let audience = text("programme.audience");
    assert!(
        mentions(
            &audience,
            &(balance::APPROVAL_DEADLINE_MISSED / domain::scale::PERCENT_UNITS as i16)
                .abs()
                .to_string()
        ),
        "the manual must quote the cost of a missed deadline"
    );
    let promises = text("state.promises");
    assert!(
        mentions(
            &promises,
            &(balance::APPROVAL_PROMISE_BROKEN / domain::scale::PERCENT_UNITS as i16)
                .abs()
                .to_string()
        ),
        "the manual must quote the cost of a broken promise"
    );
}

#[test]
fn the_political_capital_costs_are_quoted_correctly() {
    // **Forty for a political deadline, twenty for a contractual one, five for
    // the agency's own.**
    let text = text("programme.audience");
    for capital in [
        balance::POSTPONE_POLITICAL_CAPITAL,
        balance::POSTPONE_CONTRACTUAL_CAPITAL,
        balance::POSTPONE_INTERNAL_CAPITAL,
    ] {
        assert!(
            mentions(&text, &capital.to_string()),
            "the manual must quote the political capital: {capital}"
        );
    }
}

#[test]
fn the_facility_decay_is_quoted_correctly() {
    // **Four percent a month** for a facility nobody is working in.
    let text = text("research.condition");
    let decay = balance::FACILITY_DECAY_PER_MONTH / domain::scale::PERCENT_UNITS as u16;
    assert!(
        mentions(&text, &decay.to_string()),
        "the manual must quote the decay rate: {decay}"
    );
}

#[test]
fn the_mission_risk_is_quoted_correctly() {
    // **The base chance is ninety per cent**, and the manual's worked example
    // is built on it.
    let text = text("research.mission");
    let base = balance::MISSION_BASE_PROBABILITY_BP / 100;
    assert!(
        mentions(&text, &base.to_string()),
        "the manual must quote the base chance of success: {base}"
    );
}

#[test]
fn the_shift_numbers_are_quoted_correctly() {
    // **A hundred and fifty percent gives a hundred and thirty percent of the
    // output**, and it costs seven and a half points of morale.
    let text = text("agency.shift");
    // **A hundred and fifty percent gives a hundred and thirty percent of the
    // output** - the extra effort is worth sixty per cent of itself.
    let output = 100 + (150 - 100) * balance::SHIFT_OVERTIME_OUTPUT_BP / 10_000;
    let morale = balance::SHIFT_OVERTIME_MORALE_PERCENT;
    assert!(
        mentions(&text, &output.to_string()),
        "the manual must quote the overtime output: {output}"
    );
    assert!(
        mentions(&text, &morale.to_string()),
        "the manual must quote the overtime morale: {morale}"
    );
}

#[test]
fn the_appropriation_scale_is_quoted_correctly() {
    // **The state's approval scales the funding by up to half in either
    // direction**, and the manual says so.
    let text = text("state.appropriation");
    let scale = balance::APPROPRIATION_POLITICAL_SCALE_BP;
    assert!(
        mentions(&text, &scale.to_string()),
        "the manual must quote the political scale: {scale}"
    );
}

#[test]
fn the_dilations_are_quoted_correctly() {
    // **The nine rates, from sixty times to six hundred thousand times.**
    let text = text("world.arc.time");
    for scale in domain::time::TimeScale::ALL {
        assert!(
            mentions(
                &text,
                &thousands(scale.multiplier().try_into().unwrap_or(0))
            ),
            "the manual must quote the dilation: {}x",
            scale.multiplier()
        );
    }
}
