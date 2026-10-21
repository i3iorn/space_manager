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
